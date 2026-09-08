//! Existing host lifecycle values; no guest transitions added.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Created,
    Ready,
    Running,
    Paused,
    Stopped,
    Faulted,
}
