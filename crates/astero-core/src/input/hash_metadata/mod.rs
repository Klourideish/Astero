//! Explicit hash-evidence delegation; no session creation or symbol consumption.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::dynamic::hash::bounded::{
    HashLimits, HashMetadataFailure, HashMetadataLimits, HashMetadataOutcome, HashMetadataReport,
};
pub fn observe(source: &SourceArtifact, limits: HashMetadataLimits) -> HashMetadataReport {
    astero_loader::elf::dynamic::hash::bounded::observe(source.clone(), limits)
}
