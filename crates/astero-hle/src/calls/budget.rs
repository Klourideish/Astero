//! Shared process byte-work budget, separate from mapped-address validity and copy scratch.
use super::memory::AccessError;
use std::sync::atomic::{AtomicU64, Ordering};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessLimits {
    pub max_operation_bytes: u64,
    pub max_total_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessSnapshot {
    pub limits: AccessLimits,
    pub charged_bytes: u64,
    pub large_operations: u64,
    pub refusals: u64,
}
pub struct AccessBudget {
    limits: AccessLimits,
    bytes: AtomicU64,
    large: AtomicU64,
    refusals: AtomicU64,
}
impl AccessBudget {
    pub fn new(limits: AccessLimits) -> Self {
        Self {
            limits,
            bytes: AtomicU64::new(0),
            large: AtomicU64::new(0),
            refusals: AtomicU64::new(0),
        }
    }
    pub fn charge(&self, n: u64) -> Result<(), AccessError> {
        if n > self.limits.max_operation_bytes
            || self
                .bytes
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                    old.checked_add(n)
                        .filter(|v| *v <= self.limits.max_total_bytes)
                })
                .is_err()
        {
            self.refusals.fetch_add(1, Ordering::Relaxed);
            return Err(AccessError::Limit);
        }
        if n > 1024 * 1024 {
            self.large.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    }
    pub fn snapshot(&self) -> AccessSnapshot {
        AccessSnapshot {
            limits: self.limits,
            charged_bytes: self.bytes.load(Ordering::Acquire),
            large_operations: self.large.load(Ordering::Acquire),
            refusals: self.refusals.load(Ordering::Acquire),
        }
    }
}
