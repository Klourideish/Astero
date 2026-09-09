//! Explicit proof consumption only; no session mutation or classification.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::dynamic::symbol_table::bounded::{
    HashMetadataReport, SymbolObservationFailure, SymbolObservationLimits, SymbolObservationReport,
    SymbolOutcome, SymbolRecord,
};
use std::sync::Arc;
pub fn observe(
    source: &SourceArtifact,
    proof: Arc<HashMetadataReport>,
    limits: SymbolObservationLimits,
) -> SymbolObservationReport {
    astero_loader::elf::dynamic::symbol_table::bounded::observe(source.clone(), proof, limits)
}
