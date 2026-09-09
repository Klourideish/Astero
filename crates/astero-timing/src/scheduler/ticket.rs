use crate::{
    scheduler::{Shared, StopReason},
    time::{Deadline, Span, Tick},
};
use std::sync::{Arc, Condvar, Mutex, Weak};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expiry {
    pub sequence: u64,
    pub dispatch_order: u64,
    pub scheduled_at: Tick,
    pub deadline: Deadline,
    pub fired_at: Tick,
    pub lateness: Span,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Completion {
    Fired(Expiry),
    Cancelled,
    Stopped(StopReason),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancelOutcome {
    Cancelled,
    AlreadyCancelled,
    AlreadyFired,
    Stopped(StopReason),
}
pub(super) struct Signal {
    state: Mutex<Option<Completion>>,
    wake: Condvar,
}
impl Signal {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(None),
            wake: Condvar::new(),
        }
    }
    pub fn complete(&self, value: Completion) {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        debug_assert!(s.is_none());
        *s = Some(value);
        self.wake.notify_all();
    }
    pub fn poll(&self) -> Option<Completion> {
        *self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
}
/// Non-reused sequence and originating engine; clones share one terminal outcome.
#[derive(Clone)]
pub struct Ticket {
    pub(super) shared: Weak<Shared>,
    pub(super) signal: Arc<Signal>,
    pub(super) sequence: u64,
}
impl Ticket {
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
    pub fn poll(&self) -> Option<Completion> {
        self.signal.poll()
    }
    pub fn wait(&self) -> Completion {
        let mut s = self.signal.state.lock().unwrap_or_else(|p| p.into_inner());
        while s.is_none() {
            s = self.signal.wake.wait(s).unwrap_or_else(|p| p.into_inner());
        }
        s.expect("terminal predicate")
    }
    /// Host-only safety timeout for adapters/tests, not a second scheduled guest deadline.
    pub fn wait_timeout(&self, timeout: std::time::Duration) -> Option<Completion> {
        let s = self.signal.state.lock().unwrap_or_else(|p| p.into_inner());
        let (s, _) = self
            .signal
            .wake
            .wait_timeout_while(s, timeout, |s| s.is_none())
            .unwrap_or_else(|p| p.into_inner());
        *s
    }
}
