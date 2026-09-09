//! Clock source policy is separate from the scheduler. Manual advance also notifies its worker.
use crate::{
    scheduler::{Error, Shared},
    time::{Span, Tick, TimeError},
};
use std::sync::{
    Weak,
    atomic::{AtomicU64, Ordering},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockMode {
    HostMonotonic,
    Manual,
}
pub(crate) enum Source {
    Real(std::time::Instant),
    Manual(AtomicU64),
}
impl Source {
    pub(crate) fn now(&self) -> Result<Tick, TimeError> {
        match self {
            Self::Real(origin) => {
                Span::from_std(origin.elapsed()).map(|s| Tick::from_nanos(s.as_nanos()))
            }
            Self::Manual(t) => Ok(Tick::from_nanos(t.load(Ordering::Acquire))),
        }
    }
    pub(crate) fn mode(&self) -> ClockMode {
        match self {
            Self::Real(_) => ClockMode::HostMonotonic,
            Self::Manual(_) => ClockMode::Manual,
        }
    }
}
/// Weak control handle; cannot keep a scheduler worker alive after its owner is dropped.
#[derive(Clone)]
pub struct ManualClock {
    pub(crate) shared: Weak<Shared>,
}
impl ManualClock {
    pub fn advance(&self, by: Span) -> Result<Tick, Error> {
        let shared = self.shared.upgrade().ok_or(Error::Closed)?;
        let state = shared.lock();
        if state.stopped.is_some() {
            return Err(Error::Closed);
        }
        let now = shared.clock.now()?.checked_add(by)?;
        let Source::Manual(value) = &shared.clock else {
            return Err(Error::WrongClock);
        };
        value.store(now.as_nanos(), Ordering::Release);
        // Same mutex as the worker's check/wait transition prevents lost notifications.
        shared.wake.notify_all();
        drop(state);
        Ok(now)
    }
    pub fn now(&self) -> Result<Tick, Error> {
        self.shared
            .upgrade()
            .ok_or(Error::Closed)?
            .clock
            .now()
            .map_err(Error::Time)
    }
}
