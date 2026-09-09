use crate::{
    acquisition::{Selection, State},
    inspection::RequestError,
};
use astero_core::input::string_references::{
    self, StringReferenceLimits, StringReferenceObservationReport,
};
/// Only this explicit operation requests string lookup; selection is unchanged.
pub fn observe_acquired(
    selection: &Selection,
    limits: StringReferenceLimits,
) -> Result<StringReferenceObservationReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => Ok(string_references::observe(source, limits)),
        State::Ready | State::Failed(_) => Err(RequestError::NotAcquired),
    }
}
