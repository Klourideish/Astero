use crate::acquisition::{Selection, State};
use astero_core::input::inspection::{self, InspectionLimits, InspectionReport};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestError {
    NotAcquired,
}
/// Only this explicit operation inspects. It never changes the acquisition selection.
pub fn inspect_acquired(
    selection: &Selection,
    limits: InspectionLimits,
) -> Result<InspectionReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => Ok(inspection::inspect(source, limits)),
        State::Ready | State::Failed(_) => Err(RequestError::NotAcquired),
    }
}
