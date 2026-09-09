use super::ticket::Signal;
use crate::{
    clock::Source,
    diagnostics::{Counters, PendingEvent, TerminalEvent},
    time::{Span, TimeError},
};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub max_pending: usize,
    pub max_snapshot_entries: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    Shutdown,
    ClockOverflow,
    WorkerPanicked,
}
#[derive(Debug)]
pub enum Error {
    Time(TimeError),
    Closed,
    Capacity { maximum: usize },
    Allocation { entries: usize },
    LabelTooLong { bytes: usize, maximum: usize },
    SequenceExhausted,
    ForeignTicket,
    WrongClock,
    WorkerStart(std::io::Error),
    WorkerPanicked,
}
impl From<TimeError> for Error {
    fn from(e: TimeError) -> Self {
        Self::Time(e)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "timing: {self:?}")
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Time(e) => Some(e),
            Self::WorkerStart(e) => Some(e),
            _ => None,
        }
    }
}
/// Fixed 48-byte UTF-8 label; refusal instead of silent truncation or unbounded allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Label {
    bytes: [u8; 48],
    len: u8,
}
impl Label {
    pub fn new(text: &str) -> Result<Self, Error> {
        if text.len() > 48 {
            return Err(Error::LabelTooLong {
                bytes: text.len(),
                maximum: 48,
            });
        }
        let mut bytes = [0; 48];
        bytes[..text.len()].copy_from_slice(text.as_bytes());
        Ok(Self {
            bytes,
            len: text.len() as u8,
        })
    }
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len as usize]).expect("constructed UTF-8 label")
    }
}
pub(super) struct Event {
    pub details: PendingEvent,
    pub signal: Arc<Signal>,
}
pub(crate) struct State {
    pub(super) events: Vec<Event>,
    pub next_sequence: u64,
    pub counters: Counters,
    pub stopped: Option<StopReason>,
    pub worker_waiting: bool,
    pub peak_pending: usize,
    pub max_lateness: Span,
    pub last_terminal: Option<TerminalEvent>,
}
pub(crate) struct Shared {
    pub(super) state: Mutex<State>,
    pub(crate) wake: Condvar,
    pub(crate) clock: Source,
    pub(super) config: Config,
}
impl Shared {
    pub(crate) fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
}
