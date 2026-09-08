//! Inspect through core's read-only contract; no session state is owned here.
use crate::capabilities::{Capability, INVENTORY, Support};
use astero_core::observation::{ObservationError, ObserveSession, SessionSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub session: SessionSnapshot,
    pub capabilities: &'static [(Capability, Support)],
}

pub fn inspect_session(source: &impl ObserveSession) -> Result<Inspection, ObservationError> {
    Ok(Inspection {
        session: source.snapshot()?,
        capabilities: INVENTORY,
    })
}
