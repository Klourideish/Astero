//! Explicit immutable observation delegation; no parsing or runtime state.
use super::symbols::SymbolObservationReport;
pub use astero_loader::elf::dynamic::candidates::structural::{
    CandidateRole, ClassificationFailure, ClassificationLimits, ClassificationOutcome,
    ClassificationRecord, SymbolClassificationReport,
};
use std::sync::Arc;
pub fn classify(
    input: Arc<SymbolObservationReport>,
    limits: ClassificationLimits,
) -> SymbolClassificationReport {
    astero_loader::elf::dynamic::candidates::structural::classify(input, limits)
}
