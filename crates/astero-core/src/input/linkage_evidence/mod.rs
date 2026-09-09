//! Loader-owned capability delegation; no session mutation.
use super::acquisition::SourceArtifact;
pub use astero_loader::elf::dynamic::candidates::workload::{
    LinkageLimits, LinkageOutcome, LinkageReport, ReferenceRole,
};
pub fn observe(source: SourceArtifact, limits: LinkageLimits) -> LinkageReport {
    astero_loader::elf::dynamic::candidates::workload::observe(source, limits)
}
