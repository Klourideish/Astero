//! Owned values detached from session mutation. No guest observations are invented.
use crate::session::{Lifecycle, SessionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedTarget {
    pub display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    NotImplemented,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubsystemStatus {
    pub name: &'static str,
    pub availability: Availability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Statistics {
    /// Successful host lifecycle changes only; not guest execution work.
    pub lifecycle_changes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Diagnostic {
    NoGuestLoaded,
    HostFault(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSnapshot {
    pub id: SessionId,
    pub lifecycle: Lifecycle,
    pub loaded_target: Option<LoadedTarget>,
    pub statistics: Statistics,
    pub subsystems: Vec<SubsystemStatus>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationError {
    SessionClosed,
    StateUnavailable,
}

/// Read-only application boundary, independent of any debugger or frontend.
pub trait ObserveSession {
    fn snapshot(&self) -> Result<SessionSnapshot, ObservationError>;
}
