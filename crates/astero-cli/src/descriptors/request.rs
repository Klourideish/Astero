use crate::{
    acquisition::{Selection, State},
    inspection::RequestError,
};
use astero_core::input::descriptors::{self, DescriptorLimits, DescriptorObservationReport};
/// Explicit request only; acquired bytes and session state are never changed.
pub fn observe_acquired(
    selection: &Selection,
    limits: DescriptorLimits,
) -> Result<DescriptorObservationReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => Ok(descriptors::observe(source, limits)),
        State::Ready | State::Failed(_) => Err(RequestError::NotAcquired),
    }
}
