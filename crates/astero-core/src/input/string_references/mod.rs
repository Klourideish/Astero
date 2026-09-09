//! Explicit reference-only delegation; no session state or dependency semantics.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::dynamic::string_references::{
    StringEncoding, StringLimits, StringReferenceFailure, StringReferenceLimits,
    StringReferenceObservationReport, StringReferenceOutcome, StringReferenceRecord,
};
pub fn observe(
    source: &SourceArtifact,
    limits: StringReferenceLimits,
) -> StringReferenceObservationReport {
    astero_loader::elf::dynamic::string_references::observe(source.clone(), limits)
}
