//! Runtime-owned opaque objects and bounded M25-ticket waits. No guest memory or host pointers.
use astero_timing::{
    scheduler::{Label, Scheduler, Ticket},
    time::Deadline,
};
use std::sync::{Mutex, MutexGuard};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thread(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Mutex,
    Rwlock,
    Cond,
    MutexAttr,
    RwAttr,
    CondAttr,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Permission,
    Deadlock,
    Busy,
    Invalid,
    Capacity,
    Timeout,
    Interrupted,
}
pub type Result<T = ()> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct Object {
    pub id: u64,
    pub address: u64,
    pub kind: Kind,
    pub value: i32,
    pub owner: Option<Thread>,
    pub depth: u32,
    pub readers: Vec<(Thread, u32)>,
    pub generation: u64,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub objects: Vec<Object>,
    pub waiters: usize,
    pub created: u64,
    pub destroyed: u64,
    pub waits: u64,
    pub wakes: u64,
    pub timeouts: u64,
    pub stopped: bool,
}
pub(super) struct Waiter {
    pub ticket: Ticket,
    pub object: u64,
    pub mutex: Option<u64>,
    pub signaled: bool,
}
pub(super) struct State {
    pub objects: Vec<Object>,
    pub waiters: Vec<Waiter>,
    next: u64,
    pub stopped: bool,
    pub limit: Option<Deadline>,
    created: u64,
    destroyed: u64,
    pub waits: u64,
    pub wakes: u64,
    pub timeouts: u64,
}
pub struct Synchronization {
    pub(super) state: Mutex<State>,
    pub(super) scheduler: Scheduler,
    capacity: usize,
    pub(super) max_waiters: usize,
}
impl Synchronization {
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
            state: Mutex::new(State {
                objects,
                waiters,
                next: 1,
                stopped: false,
                limit: None,
                created: 0,
                destroyed: 0,
                waits: 0,
                wakes: 0,
                timeouts: 0,
            }),
            scheduler,
            capacity,
            max_waiters,
        })
    }
    pub(super) fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    /// Explicit compatibility adapter: host realtime at call, or this timing engine's monotonic epoch.
    pub fn absolute(&self, seconds: i64, nanos: i64, clock: i32) -> Result<Deadline> {
        crate::timing::clock::absolute(
            &self.scheduler,
            crate::timing::clock::Realtime::HostUnix,
            astero_abi::layouts::time::Timespec { seconds, nanos },
            clock,
        )
        .map_err(|e| {
            if e == crate::timing::clock::Error::Interrupted {
                Error::Interrupted
            } else {
                Error::Invalid
            }
        })
    }

    pub fn arm(&self, deadline: Deadline) -> Result {
        let mut s = self.lock();
        if !s.waiters.is_empty() {
            return Err(Error::Busy);
        }
        s.limit = Some(deadline);
        Ok(())
    }
    pub fn create(&self, address: u64, kind: Kind, value: i32) -> Result<u64> {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        }
        if address == 0
            || s.objects
                .iter()
                .any(|o| o.address == address && (kind != Kind::Mutex || o.kind != Kind::Mutex))
        {
            return Err(Error::Invalid);
        }
        if s.objects.len() == self.capacity {
            return Err(Error::Capacity);
        }
        let id = s.next;
        s.next = id.checked_add(1).ok_or(Error::Capacity)?;
        let mut readers = Vec::new();
        readers
            .try_reserve_exact(self.max_waiters)
            .map_err(|_| Error::Capacity)?;
        s.objects.push(Object {
            id,
            address,
            kind,
            value,
            owner: None,
            depth: 0,
            readers,
            generation: 0,
        });
        s.created += 1;
        Ok(id)
    }
    pub(super) fn index(s: &State, id: u64, address: u64, kind: Kind) -> Result<usize> {
        s.objects
            .iter()
            .position(|o| {
                o.id == id
                    && address != 0
                    && (kind == Kind::Mutex || o.address == address)
                    && o.kind == kind
            })
            .ok_or(Error::Invalid)
    }
    pub fn validate(&self, id: u64, address: u64, kind: Kind) -> Result {
        Self::index(&self.lock(), id, address, kind).map(|_| ())
    }
    pub fn value(&self, id: u64, address: u64, kind: Kind) -> Result<i32> {
        let s = self.lock();
        Ok(s.objects[Self::index(&s, id, address, kind)?].value)
    }
    pub fn set_value(&self, id: u64, address: u64, kind: Kind, value: i32) -> Result {
        let mut s = self.lock();
        let i = Self::index(&s, id, address, kind)?;
        s.objects[i].value = value;
        Ok(())
    }
    pub fn destroy(&self, id: u64, address: u64, kind: Kind) -> Result {
        let mut s = self.lock();
        let i = Self::index(&s, id, address, kind)?;
        let o = &s.objects[i];
        if o.owner.is_some()
            || !o.readers.is_empty()
            || s.waiters
                .iter()
                .any(|w| w.object == id || w.mutex == Some(id))
        {
            return Err(Error::Busy);
        }
        s.objects.remove(i);
        s.destroyed += 1;
        Ok(())
    }
    pub(super) fn wake(&self, s: &mut State, id: u64, one: bool) {
        for w in &mut s.waiters {
            if w.object == id && !w.signaled {
                w.signaled = true;
                let _ = self.scheduler.cancel(&w.ticket);
                s.wakes += 1;
                if one {
                    break;
                }
            }
        }
    }
    pub(super) fn ticket(
        &self,
        s: &mut State,
        id: u64,
        mutex: Option<u64>,
        deadline: Option<Deadline>,
    ) -> Result<Ticket> {
        if s.stopped {
            return Err(Error::Interrupted);
        }
        if s.waiters.len() == self.max_waiters {
            return Err(Error::Capacity);
        }
        let limit = s.limit.ok_or(Error::Invalid)?;
        let due = deadline.map_or(limit, |d| d.min(limit));
        let t = self
            .scheduler
            .schedule(due, Label::new("pthread wait").map_err(|_| Error::Invalid)?)
            .map_err(|_| Error::Capacity)?;
        s.waiters.push(Waiter {
            ticket: t.clone(),
            object: id,
            mutex,
            signaled: false,
        });
        s.waits += 1;
        Ok(t)
    }
    pub(super) fn expired(&self, s: &State, deadline: Option<Deadline>) -> Result {
        if s.stopped {
            return Err(Error::Interrupted);
        }
        let now = self.scheduler.now().map_err(|_| Error::Interrupted)?;
        if s.limit.is_some_and(|d| d.is_due(now)) {
            return Err(Error::Interrupted);
        }
        if deadline.is_some_and(|d| d.is_due(now)) {
            return Err(Error::Timeout);
        }
        Ok(())
    }
    pub(super) fn remove_waiter(&self, s: &mut State, t: &Ticket) {
        s.waiters.retain(|w| w.ticket.sequence() != t.sequence());
        let _ = self.scheduler.cancel(t);
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.lock();
        Snapshot {
            objects: s.objects.clone(),
            waiters: s.waiters.len(),
            created: s.created,
            destroyed: s.destroyed,
            waits: s.waits,
            wakes: s.wakes,
            timeouts: s.timeouts,
            stopped: s.stopped,
        }
    }
    pub fn shutdown(&self) {
        let mut s = self.lock();
        s.stopped = true;
        for w in &s.waiters {
            let _ = self.scheduler.cancel(&w.ticket);
        }
    }
}
impl Drop for Synchronization {
    fn drop(&mut self) {
        self.shutdown();
    }
}
