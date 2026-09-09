use crate::{acquisition::Selection, inspection::RequestError, symbols};
use astero_core::input::{
    classification::{self, ClassificationLimits, SymbolClassificationReport},
    hash_metadata::HashMetadataLimits,
    symbols::SymbolObservationLimits,
};
use std::sync::Arc;
/// CLI explicitly composes prerequisites, then passes the complete report to the classifier.
pub fn classify_acquired(
    selection: &Selection,
    hash: HashMetadataLimits,
    symbol_limits: SymbolObservationLimits,
    limits: ClassificationLimits,
) -> Result<SymbolClassificationReport, RequestError> {
    let input = Arc::new(symbols::observe_acquired(selection, hash, symbol_limits)?);
    Ok(classification::classify(input, limits))
}
