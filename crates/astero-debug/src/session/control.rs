//! Guest execution control remains unavailable.
use crate::capabilities::{Capability, Unsupported};

pub fn pause() -> Result<(), Unsupported> {
    Err(Unsupported {
        capability: Capability::ExecutionControl,
    })
}
