//! Existing host lifecycle counters; not guest performance measurements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Statistics {
    /// Successful host lifecycle changes only; not guest execution work.
    pub lifecycle_changes: u64,
}
