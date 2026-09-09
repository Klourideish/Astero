use crate::{
    acquisition::{Selection, State},
    inspection::RequestError,
};
use astero_core::input::dynamic::{self, DynamicLimits, DynamicObservationReport};
/// Only this explicit call observes dynamic entries; selection and session state are unchanged.
pub fn observe_acquired(
    selection: &Selection,
    limits: DynamicLimits,
) -> Result<DynamicObservationReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => Ok(dynamic::observe(source, limits)),
        State::Ready | State::Failed(_) => Err(RequestError::NotAcquired),
    }
}
