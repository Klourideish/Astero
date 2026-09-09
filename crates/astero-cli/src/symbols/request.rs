use crate::{
    acquisition::{Selection, State},
    inspection::RequestError,
};
use astero_core::input::{
    hash_metadata::{self, HashMetadataLimits},
    symbols::{self, SymbolObservationLimits, SymbolObservationReport},
};
use std::sync::Arc;
/// This command explicitly requests a proof, then supplies it to the independent symbol consumer.
pub fn observe_acquired(
    selection: &Selection,
    hash_limits: HashMetadataLimits,
    limits: SymbolObservationLimits,
) -> Result<SymbolObservationReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => {
            let proof = Arc::new(hash_metadata::observe(source, hash_limits));
            Ok(symbols::observe(source, proof, limits))
        }
        State::Ready | State::Failed(_) => Err(RequestError::NotAcquired),
    }
}
