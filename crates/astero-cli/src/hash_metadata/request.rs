use crate::{
    acquisition::{Selection, State},
    inspection::RequestError,
};
use astero_core::input::hash_metadata::{self, HashMetadataLimits, HashMetadataReport};
/// Only this explicit operation requests hash observation; selection is unchanged.
pub fn observe_acquired(
    selection: &Selection,
    limits: HashMetadataLimits,
) -> Result<HashMetadataReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => Ok(hash_metadata::observe(source, limits)),
        State::Ready | State::Failed(_) => Err(RequestError::NotAcquired),
    }
}
