//! Explicit delegation only; does not acquire input or attach evidence to a session.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::inspect::bounded::{
    HeaderEvidence, InspectionFailure, InspectionLimits, InspectionOutcome, InspectionReport,
};
pub fn inspect(source: &SourceArtifact, limits: InspectionLimits) -> InspectionReport {
    astero_loader::elf::inspect::bounded::inspect(source.clone(), limits)
}
