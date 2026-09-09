//! Explicit metadata-only delegation; no acquisition, payload traversal or session mutation.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::dynamic::descriptors::{
    DescriptorFailure, DescriptorFamily, DescriptorLimits, DescriptorObservationReport,
    DescriptorOutcome, DescriptorRecord, DescriptorUnavailable, DescriptorValue,
};
pub fn observe(source: &SourceArtifact, limits: DescriptorLimits) -> DescriptorObservationReport {
    astero_loader::elf::dynamic::descriptors::observe(source.clone(), limits)
}
