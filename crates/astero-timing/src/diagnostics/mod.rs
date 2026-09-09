//! Detached bounded observations, not logging or mutable scheduler access.
use crate::{
    clock::ClockMode,
    scheduler::{Label, StopReason},
    time::{Deadline, Span, Tick},
};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub scheduled: u64,
    pub fired: u64,
    pub cancelled: u64,
    pub released: u64,
    pub rejected: u64,
    pub worker_waits: u64,
    pub saturated: bool,
}
impl Counters {
    pub(crate) fn add(value: &mut u64, saturated: &mut bool) {
        match value.checked_add(1) {
            Some(n) => *value = n,
            None => *saturated = true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingEvent {
    pub sequence: u64,
    pub label: Label,
    pub scheduled_at: Tick,
    pub deadline: Deadline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    Fired,
    Cancelled,
    Released,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalEvent {
    pub event: PendingEvent,
    pub observed_at: Option<Tick>,
    pub transition: Transition,
    pub lateness: Span,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub mode: ClockMode,
    pub now: Tick,
    pub stopped: Option<StopReason>,
    pub pending_total: usize,
    pub next_deadline: Option<Deadline>,
    pub retained: Vec<PendingEvent>,
    pub omitted: usize,
    pub counters: Counters,
    pub worker_waiting: bool,
    pub peak_pending: usize,
    pub max_lateness: Span,
    pub last_terminal: Option<TerminalEvent>,
}
