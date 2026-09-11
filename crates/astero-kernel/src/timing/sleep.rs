//! Runtime-owned sleep tickets; no extra worker, polling, global owner or guest pointers.
use super::clock::{Error, Realtime, Result};
use crate::synchronization::owned::Thread;
use astero_timing::{
    scheduler::{Completion, Label, Scheduler, Ticket},
    time::{Deadline, Span},
};
use std::sync::{Mutex, MutexGuard};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    Completed,
    Interrupted,
}
#[derive(Clone, Debug)]
pub struct Observation {
    pub thread: Thread,
    pub sequence: u64,
    pub requested_ns: u64,
    pub deadline_ns: u64,
    pub elapsed_ns: u64,
    pub lateness_ns: u64,
    pub wake: Wake,
}
struct Pending {
    thread: Thread,
    ticket: Ticket,
}
struct State {
    pending: Vec<Pending>,
    stopped: bool,
    limit: Option<Deadline>,
    records: Vec<Observation>,
    completed: u64,
    interrupted: u64,
    queries: u64,
    clocks: Vec<(i32, u64)>,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub pending: usize,
    pub stopped: bool,
    pub completed: u64,
    pub interrupted: u64,
    pub queries: u64,
    pub clocks: Vec<(i32, u64)>,
    pub sleeping: Vec<(Thread, u64)>,
    pub records: Vec<Observation>,
}
pub struct GuestTiming {
    state: Mutex<State>,
    pub scheduler: Scheduler,
    pub wall: Realtime,
}
impl GuestTiming {
    pub fn new(scheduler: Scheduler, wall: Realtime) -> Self {
        Self {
            scheduler,
            wall,
            state: Mutex::new(State {
                pending: vec![],
                stopped: false,
                limit: None,
                records: vec![],
                completed: 0,
                interrupted: 0,
                queries: 0,
                clocks: vec![],
            }),
        }
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    pub fn arm(&self, d: Deadline) {
        self.lock().limit = Some(d)
    }
    pub fn query(&self, id: i32) -> Result<u64> {
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Interrupted);
        };
        let n = super::clock::query(&self.scheduler, self.wall, id)?;
        s.queries = s.queries.saturating_add(1);
        if s.clocks.len() < 64 {
            s.clocks.push((id, n));
        }
        Ok(n)
    }
    pub fn sleep(&self, thread: Thread, span: Span) -> Result<Observation> {
        let mut s = self.lock();
        let now = self.scheduler.now().map_err(|_| Error::Interrupted)?;
        if s.stopped || s.limit.is_some_and(|d| d.is_due(now)) {
            return Err(Error::Interrupted);
        }
        if s.pending.len() >= 64 {
            return Err(Error::Capacity);
        }
        let end = Deadline::after(now, span).map_err(|_| Error::Invalid)?;
        let deadline = s.limit.map_or(end, |d| d.min(end));
        s.pending.try_reserve(1).map_err(|_| Error::Capacity)?;
        let ticket = self
            .scheduler
            .schedule(deadline, Label::new("guest sleep").expect("constant"))
            .map_err(|_| Error::Capacity)?;
        s.pending.push(Pending {
            thread,
            ticket: ticket.clone(),
        });
        drop(s);
        let completion = ticket.wait();
        let finished = self.scheduler.now().unwrap_or(now);
        let mut s = self.lock();
        s.pending
            .retain(|p| p.ticket.sequence() != ticket.sequence());
        let success = !s.stopped
            && matches!(completion, Completion::Fired(_))
            && deadline == end
            && end.is_due(finished);
        let wake = if success {
            s.completed = s.completed.saturating_add(1);
            Wake::Completed
        } else {
            s.interrupted = s.interrupted.saturating_add(1);
            Wake::Interrupted
        };
        let record = Observation {
            thread,
            sequence: ticket.sequence(),
            requested_ns: span.as_nanos(),
            deadline_ns: end.tick().as_nanos(),
            elapsed_ns: finished.as_nanos().saturating_sub(now.as_nanos()),
            lateness_ns: finished.as_nanos().saturating_sub(end.tick().as_nanos()),
            wake,
        };
        if s.records.len() < 64 {
            s.records.push(record.clone());
        }
        Ok(record)
    }
    pub fn cancel_thread(&self, t: Thread) {
        let s = self.lock();
        for p in &s.pending {
            if p.thread == t {
                let _ = self.scheduler.cancel(&p.ticket);
            }
        }
    }
    pub fn shutdown(&self) {
        let mut s = self.lock();
        s.stopped = true;
        for p in &s.pending {
            let _ = self.scheduler.cancel(&p.ticket);
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.lock();
        Snapshot {
            pending: s.pending.len(),
            stopped: s.stopped,
            completed: s.completed,
            interrupted: s.interrupted,
            queries: s.queries,
            clocks: s.clocks.clone(),
            sleeping: s
                .pending
                .iter()
                .map(|p| (p.thread, p.ticket.sequence()))
                .collect(),
            records: s.records.clone(),
        }
    }
}
impl Drop for GuestTiming {
    fn drop(&mut self) {
        self.shutdown()
    }
}
