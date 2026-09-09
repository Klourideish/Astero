//! Explicit raw dynamic observation delegation. No acquisition, session, or linkage operation.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::dynamic::bounded::{
    DynamicFailure, DynamicLimits, DynamicObservationReport, DynamicOutcome,
};
pub fn observe(source: &SourceArtifact, limits: DynamicLimits) -> DynamicObservationReport {
    astero_loader::elf::dynamic::bounded::observe(source.clone(), limits)
}
