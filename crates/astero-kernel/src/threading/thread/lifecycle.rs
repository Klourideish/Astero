//! One bounded table per runtime. Detached guest identity never detaches a host JoinHandle.
use super::attributes::Attributes;
pub use crate::synchronization::owned::Thread;
use astero_timing::{
    scheduler::{Label, Scheduler, Ticket},
    time::Deadline,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::JoinHandle;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    NoSuchThread,
    Deadlock,
    Busy,
    Capacity,
    Interrupted,
    Unsupported,
    Fault,
}
pub type Result<T = ()> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Created,
    Running,
    Exited,
    Joined,
    Reclaimed,
    Failed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThreadEnd {
    Returned,
    PthreadExit,
    Interrupted,
    ProviderStop,
    NativeFault,
    BridgeFailure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub value: u64,
    pub reason: ThreadEnd,
}
#[derive(Clone, Debug)]
pub struct Record {
    pub layout: Option<crate::execution::preparation::layout::ThreadLayout>,
    pub id: Thread,
    pub state: State,
    pub attributes: Attributes,
    pub name: Vec<u8>,
    pub start: u64,
    pub argument: u64,
    pub outcome: Option<Outcome>,
    pub host: Option<String>,
}
struct Owned {
    record: Record,
    handle: Option<JoinHandle<()>>,
    waiter: Option<Ticket>,
}
pub struct ThreadTable {
    records: Mutex<Vec<Owned>>,
    scheduler: Scheduler,
    deadline: Mutex<Option<Deadline>>,
    stopped: AtomicBool,
    capacity: usize,
    peak: std::sync::atomic::AtomicUsize,
}
impl ThreadTable {
    pub fn new(scheduler: Scheduler, capacity: usize) -> Self {
        Self {
            peak: std::sync::atomic::AtomicUsize::new(1),
            records: Mutex::new(vec![Owned {
                record: Record {
                    layout: None,
                    id: Thread(1),
                    state: State::Running,
                    attributes: Attributes::default(),
                    name: b"main".to_vec(),
                    start: 0,
                    argument: 0,
                    outcome: None,
                    host: None,
                },
                handle: None,
                waiter: None,
            }]),
            scheduler,
            deadline: Mutex::new(None),
            stopped: AtomicBool::new(false),
            capacity,
        }
    }
    pub fn arm(&self, deadline: Deadline) {
        *self.deadline.lock().unwrap_or_else(|p| p.into_inner()) = Some(deadline);
    }
    pub fn initialize_main(
        &self,
        start: u64,
        argument: u64,
        layout: crate::execution::preparation::layout::ThreadLayout,
    ) {
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        let r = &mut s[0].record;
        r.start = start;
        r.argument = argument;
        r.attributes.stack_size = layout.stack.size;
        r.layout = Some(layout);
        r.host = Some(format!("{:?}", std::thread::current().id()));
    }
    pub fn complete_main(&self, outcome: Outcome) {
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        s[0].record.outcome = Some(outcome);
        s[0].record.state = State::Exited;
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
            || self
                .deadline
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .is_some_and(|d| self.scheduler.now().map_or(true, |now| d.is_due(now)))
    }
    pub fn remaining_millis(&self) -> Result<u64> {
        let d = self
            .deadline
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .ok_or(Error::Interrupted)?;
        let now = self.scheduler.now().map_err(|_| Error::Interrupted)?;
        if self.stopped() || d.is_due(now) {
            return Err(Error::Interrupted);
        }
        Ok((d
            .tick()
            .as_nanos()
            .saturating_sub(now.as_nanos())
            .div_ceil(1_000_000))
        .clamp(1, 500))
    }
    pub fn reserve(
        &self,
        attr: Attributes,
        start: u64,
        argument: u64,
        name: Vec<u8>,
    ) -> Result<Thread> {
        attr.validate()?;
        if self.stopped() {
            return Err(Error::Interrupted);
        }
        if start == 0 || name.len() > 31 {
            return Err(Error::Invalid);
        }
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        if s.len() > self.capacity {
            return Err(Error::Capacity);
        }
        s.try_reserve(1).map_err(|_| Error::Capacity)?;
        let id = Thread(s.len() as u64 + 1); // Thread(1) is the M33 initial identity; never reused.
        s.push(Owned {
            record: Record {
                layout: None,
                id,
                state: State::Created,
                attributes: attr,
                name,
                start,
                argument,
                outcome: None,
                host: None,
            },
            handle: None,
            waiter: None,
        });
        self.peak.fetch_max(
            s.iter()
                .filter(|o| matches!(o.record.state, State::Created | State::Running))
                .count(),
            Ordering::Relaxed,
        );
        Ok(id)
    }
    /// Spawn remains behind a publication gate. Failed guest-handle copy joins the unstarted host.
    pub fn launch<F, P>(self: &Arc<Self>, id: Thread, job: F, publish: P) -> Result
    where
        F: FnOnce() -> Outcome + Send + 'static,
        P: FnOnce() -> Result,
    {
        let (send, recv) = mpsc::sync_channel(1);
        let weak = Arc::downgrade(self);
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        let o = s
            .iter_mut()
            .find(|o| o.record.id == id && o.record.state == State::Created && o.handle.is_none())
            .ok_or(Error::Invalid)?;
        if self.stopped() {
            o.record.state = State::Failed;
            return Err(Error::Interrupted);
        }
        let handle = match std::thread::Builder::new()
            .name(format!("astero-guest-{}", id.0))
            .spawn(move || {
                if recv.recv() != Ok(true) {
                    return;
                }
                let Some(table) = weak.upgrade() else {
                    return;
                };
                {
                    let mut s = table.records.lock().unwrap_or_else(|p| p.into_inner());
                    let o = s.iter_mut().find(|o| o.record.id == id).unwrap();
                    o.record.state = State::Running;
                    o.record.host = Some(format!("{:?}", std::thread::current().id()));
                }
                drop(table);
                let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job))
                    .unwrap_or(Outcome {
                        value: 0,
                        reason: ThreadEnd::BridgeFailure,
                    });
                let Some(table) = weak.upgrade() else {
                    return;
                };
                let mut s = table.records.lock().unwrap_or_else(|p| p.into_inner());
                let o = s.iter_mut().find(|o| o.record.id == id).unwrap();
                o.record.outcome = Some(outcome);
                o.record.state = State::Exited;
                if let Some(t) = &o.waiter {
                    let _ = table.scheduler.cancel(t);
                }
            }) {
            Ok(h) => h,
            Err(_) => {
                o.record.state = State::Failed;
                return Err(Error::Capacity);
            }
        };
        if let Err(e) = publish() {
            o.record.state = State::Failed;
            drop(s);
            drop(send);
            let _ = handle.join();
            return Err(e);
        }
        o.handle = Some(handle);
        send.send(true).map_err(|_| Error::Fault)?;
        Ok(())
    }
    pub fn abandon(&self, id: Thread) {
        if let Some(o) = self
            .records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter_mut()
            .find(|o| o.record.id == id && o.record.state == State::Created && o.handle.is_none())
        {
            o.record.state = State::Failed;
        }
    }
    pub fn join(&self, caller: Thread, target: Thread) -> Result<u64> {
        if caller == target {
            return Err(Error::Deadlock);
        }
        if target == Thread(1) {
            return Err(Error::Unsupported);
        }
        if self.stopped() {
            return Err(Error::Interrupted);
        }
        let deadline = self
            .deadline
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .ok_or(Error::Interrupted)?;
        let ticket;
        {
            let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
            let o = s
                .iter_mut()
                .find(|o| o.record.id == target)
                .ok_or(Error::NoSuchThread)?;
            if o.record.attributes.detached
                || matches!(
                    o.record.state,
                    State::Joined | State::Reclaimed | State::Failed
                )
            {
                return Err(Error::Invalid);
            }
            if o.waiter.is_some() {
                return Err(Error::Busy);
            }
            ticket = self
                .scheduler
                .schedule(deadline, Label::new("pthread join").unwrap())
                .map_err(|_| Error::Capacity)?;
            o.waiter = Some(ticket.clone());
            if o.record.state == State::Exited {
                let _ = self.scheduler.cancel(&ticket);
            }
        }
        ticket.wait();
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        let o = s.iter_mut().find(|o| o.record.id == target).unwrap();
        o.waiter = None;
        let _ = self.scheduler.cancel(&ticket);
        if o.record.state != State::Exited {
            return Err(Error::Interrupted);
        }
        let result = o.record.outcome.clone().ok_or(Error::Fault)?;
        o.record.state = State::Joined;
        let handle = o.handle.take();
        drop(s);
        if let Some(h) = handle {
            h.join().map_err(|_| Error::Fault)?;
        }
        match result.reason {
            ThreadEnd::Returned | ThreadEnd::PthreadExit => Ok(result.value),
            ThreadEnd::Interrupted => Err(Error::Interrupted),
            _ => Err(Error::Fault),
        }
    }
    pub fn detach(&self, id: Thread) -> Result {
        if id == Thread(1) {
            return Err(Error::Unsupported);
        }
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        let o = s
            .iter_mut()
            .find(|o| o.record.id == id)
            .ok_or(Error::NoSuchThread)?;
        if o.record.attributes.detached
            || o.waiter.is_some()
            || matches!(
                o.record.state,
                State::Joined | State::Reclaimed | State::Failed
            )
        {
            return Err(Error::Invalid);
        }
        o.record.attributes.detached = true;
        Ok(())
    }
    pub fn rename(&self, id: Thread, name: Vec<u8>) -> Result {
        if name.len() > 31 {
            return Err(Error::Invalid);
        }
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        let o = s
            .iter_mut()
            .find(|o| {
                o.record.id == id
                    && !matches!(
                        o.record.state,
                        State::Joined | State::Reclaimed | State::Failed
                    )
            })
            .ok_or(Error::NoSuchThread)?;
        o.record.name = name;
        Ok(())
    }
    pub fn set_storage(
        &self,
        id: Thread,
        layout: crate::execution::preparation::layout::ThreadLayout,
    ) -> Result {
        let mut s = self.records.lock().unwrap_or_else(|p| p.into_inner());
        let o = s
            .iter_mut()
            .find(|o| o.record.id == id)
            .ok_or(Error::NoSuchThread)?;
        if o.record.layout.is_some() {
            return Err(Error::Busy);
        }
        o.record.layout = Some(layout);
        Ok(())
    }
    pub fn peak_threads(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }
    pub fn snapshot(&self) -> Vec<Record> {
        self.records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter()
            .map(|o| o.record.clone())
            .collect()
    }
    pub fn request_stop(&self) {
        self.stopped.store(true, Ordering::Release);
        for o in self
            .records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter()
        {
            if let Some(t) = &o.waiter {
                let _ = self.scheduler.cancel(t);
            }
        }
    }
    /// Owner must stop native execution and synchronization waits before this final host join.
    pub fn reap_all(&self) {
        self.request_stop();
        let handles: Vec<_> = self
            .records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter_mut()
            .filter_map(|o| o.handle.take())
            .collect();
        for h in handles {
            let _ = h.join();
        }
        for o in self
            .records
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .iter_mut()
        {
            if o.record.state == State::Exited && o.record.attributes.detached {
                o.record.state = State::Reclaimed;
            }
        }
    }
}
