use crate::synchronization::owned::Thread;
use astero_timing::{
    scheduler::{Completion, Label, Scheduler, Ticket},
    time::{Deadline, Tick},
};
use std::sync::{Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    NotFound,
    Deleted,
    Busy,
    Overflow,
    Timeout,
    Cancelled,
    Interrupted,
    Capacity,
}
pub type Result<T = ()> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct Object {
    pub handle: u32,
    pub name: Vec<u8>,
    pub initial: i32,
    pub count: i32,
    pub maximum: i32,
}
struct Waiter {
    handle: u32,
    thread: Thread,
    count: i32,
    ticket: Ticket,
    result: Option<Result>,
}
struct State {
    objects: Vec<Object>,
    waiters: Vec<Waiter>,
    next: u32,
    stopped: bool,
    limit: Option<Deadline>,
    waits: u64,
    signals: u64,
    timeouts: u64,
    cancellations: u64,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub objects: Vec<Object>,
    pub waiting_threads: Vec<Thread>,
    pub waits: u64,
    pub signals: u64,
    pub timeouts: u64,
    pub cancellations: u64,
    pub stopped: bool,
}
pub struct Semaphores {
    state: Mutex<State>,
    scheduler: Scheduler,
    capacity: usize,
    max_waiters: usize,
}
impl Semaphores {
    pub fn wait_micros(
        &self,
        handle: u32,
        count: i32,
        thread: Thread,
        micros: Option<u32>,
    ) -> (Result, u32) {
        let Ok(now) = self.scheduler.now() else {
            return (Err(Error::Interrupted), 0);
        };
        let deadline = micros.map(|us| {
            Deadline::after(now, astero_timing::time::Span::from_nanos(us as u64 * 1000))
        });
        let end = match deadline.transpose() {
            Ok(d) => d,
            Err(_) => return (Err(Error::Invalid), 0),
        };
        let result = self.wait(handle, count, thread, end);
        let remaining = end
            .and_then(|d| {
                self.scheduler
                    .now()
                    .ok()
                    .map(|n| d.remaining(n).as_nanos() / 1000)
            })
            .unwrap_or(0);
        (result, remaining.min(u32::MAX as u64) as u32)
    }
    pub fn new(scheduler: Scheduler, capacity: usize, max_waiters: usize) -> Result<Self> {
        let mut objects = Vec::new();
        objects
            .try_reserve_exact(capacity)
            .map_err(|_| Error::Capacity)?;
        let mut waiters = Vec::new();
        waiters
            .try_reserve_exact(max_waiters)
            .map_err(|_| Error::Capacity)?;
        Ok(Self {
            scheduler,
            capacity,
            max_waiters,
            state: Mutex::new(State {
                objects,
                waiters,
                next: 1,
                stopped: false,
                limit: None,
                waits: 0,
                signals: 0,
                timeouts: 0,
                cancellations: 0,
            }),
        })
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    pub fn arm(&self, deadline: Deadline) -> Result {
        let mut s = self.lock();
        if !s.waiters.is_empty() {
            return Err(Error::Busy);
        }
        s.limit = Some(deadline);
        Ok(())
    }
    pub fn create(&self, name: &[u8], initial: i32, maximum: i32) -> Result<u32> {
        if name.len() > 31 || initial < 0 || maximum <= 0 || initial > maximum {
            return Err(Error::Invalid);
        }
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        }
        if s.objects.len() == self.capacity {
            return Err(Error::Capacity);
        }
        let handle = s.next;
        s.next = handle.checked_add(1).ok_or(Error::Capacity)?;
        s.objects.push(Object {
            handle,
            name: name.to_vec(),
            initial,
            count: initial,
            maximum,
        });
        Ok(handle)
    }
    fn object(s: &State, handle: u32) -> Result<usize> {
        s.objects
            .iter()
            .position(|o| o.handle == handle)
            .ok_or(Error::NotFound)
    }
    pub fn poll(&self, handle: u32, count: i32) -> Result {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        }
        let i = Self::object(&s, handle)?;
        if count <= 0 || count > s.objects[i].maximum {
            return Err(Error::Invalid);
        }
        if s.objects[i].count < count
            || s.waiters
                .iter()
                .any(|w| w.handle == handle && w.result.is_none())
        {
            return Err(Error::Busy);
        }
        s.objects[i].count -= count;
        Ok(())
    }
    /// State publication under the service lock decides signal/cancel/expiry races.
    /// FIFO is emulator policy, not a claim about firmware priority scheduling.
    pub fn wait(
        &self,
        handle: u32,
        count: i32,
        thread: Thread,
        timeout: Option<Deadline>,
    ) -> Result {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        }
        let i = Self::object(&s, handle)?;
        if count <= 0 || count > s.objects[i].maximum {
            return Err(Error::Invalid);
        }
        let now = self.scheduler.now().map_err(|_| Error::Interrupted)?;
        if s.limit.is_some_and(|d| d.is_due(now)) {
            return Err(Error::Interrupted);
        }
        s.waits = s.waits.saturating_add(1);
        if s.objects[i].count >= count
            && !s
                .waiters
                .iter()
                .any(|w| w.handle == handle && w.result.is_none())
        {
            s.objects[i].count -= count;
            return Ok(());
        }
        if timeout.is_some_and(|d| d.is_due(now)) {
            s.timeouts = s.timeouts.saturating_add(1);
            return Err(Error::Timeout);
        }
        if s.waiters.len() == self.max_waiters {
            return Err(Error::Capacity);
        }
        let end = timeout.unwrap_or(Deadline::at(Tick::from_nanos(u64::MAX)));
        let deadline = s.limit.map_or(end, |d| d.min(end));
        let ticket = self
            .scheduler
            .schedule(
                deadline,
                Label::new("kernel semaphore").expect("fixed label"),
            )
            .map_err(|_| Error::Capacity)?;
        s.waiters.push(Waiter {
            handle,
            thread,
            count,
            ticket: ticket.clone(),
            result: None,
        });
        drop(s);
        let completion = ticket.wait();
        let mut s = self.lock();
        let at = s
            .waiters
            .iter()
            .position(|w| w.ticket.sequence() == ticket.sequence())
            .expect("waiter retained until return");
        let w = s.waiters.remove(at);
        let result = w.result.unwrap_or_else(|| {
            if s.stopped
                || matches!(completion, Completion::Stopped(_))
                || s.limit.is_some_and(|d| d <= end)
            {
                Err(Error::Interrupted)
            } else {
                Err(Error::Timeout)
            }
        });
        if result == Err(Error::Timeout) {
            s.timeouts = s.timeouts.saturating_add(1)
        }
        self.grant(&mut s, handle);
        result
    }
    fn grant(&self, s: &mut State, handle: u32) {
        let Ok(i) = Self::object(s, handle) else {
            return;
        };
        for w in s
            .waiters
            .iter_mut()
            .filter(|w| w.handle == handle && w.result.is_none())
        {
            if matches!(
                w.ticket.poll(),
                Some(Completion::Fired(_) | Completion::Stopped(_))
            ) {
                continue;
            }
            if s.objects[i].count < w.count {
                break;
            }
            if matches!(
                self.scheduler.cancel(&w.ticket),
                Ok(astero_timing::scheduler::CancelOutcome::Cancelled)
            ) {
                s.objects[i].count -= w.count;
                w.result = Some(Ok(()));
            }
        }
    }
    pub fn signal(&self, handle: u32, count: i32) -> Result {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        }
        let i = Self::object(&s, handle)?;
        if count <= 0 {
            return Err(Error::Invalid);
        }
        let next = s.objects[i]
            .count
            .checked_add(count)
            .ok_or(Error::Overflow)?;
        if next > s.objects[i].maximum {
            return Err(Error::Overflow);
        }
        s.objects[i].count = next;
        s.signals = s.signals.saturating_add(1);
        self.grant(&mut s, handle);
        Ok(())
    }
    fn wake(&self, s: &mut State, handle: Option<u32>, error: Error) -> usize {
        let mut n = 0;
        for w in &mut s.waiters {
            if handle.is_none_or(|h| h == w.handle) && w.result.is_none() {
                w.result = Some(Err(error));
                let _ = self.scheduler.cancel(&w.ticket);
                n += 1;
            }
        }
        n
    }
    pub fn cancel(&self, handle: u32, count: i32) -> Result<usize> {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        }
        let i = Self::object(&s, handle)?;
        if count > s.objects[i].maximum {
            return Err(Error::Invalid);
        }
        s.objects[i].count = if count < 0 {
            s.objects[i].initial
        } else {
            count
        };
        let n = self.wake(&mut s, Some(handle), Error::Cancelled);
        s.cancellations = s.cancellations.saturating_add(n as u64);
        Ok(n)
    }
    pub fn delete(&self, handle: u32) -> Result {
        let mut s = self.lock();
        let i = Self::object(&s, handle)?;
        s.objects.remove(i);
        self.wake(&mut s, Some(handle), Error::Deleted);
        Ok(())
    }
    pub fn shutdown(&self) {
        let mut s = self.lock();
        s.stopped = true;
        self.wake(&mut s, None, Error::Interrupted);
        s.objects.clear();
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.lock();
        Snapshot {
            objects: s.objects.clone(),
            waiting_threads: s.waiters.iter().map(|w| w.thread).collect(),
            waits: s.waits,
            signals: s.signals,
            timeouts: s.timeouts,
            cancellations: s.cancellations,
            stopped: s.stopped,
        }
    }
}
impl Drop for Semaphores {
    fn drop(&mut self) {
        self.shutdown()
    }
}
