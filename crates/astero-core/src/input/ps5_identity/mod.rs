//! Explicit delegation over existing immutable linkage evidence; no session mutation.
use super::linkage_evidence::LinkageReport;
pub use astero_loader::elf::dynamic::identity::{
    IdentityLimits, IdentityOutcome, NameEvidence, Ps5IdentityEvidenceReport,
};
use std::sync::Arc;
pub fn observe(input: Arc<LinkageReport>, limits: IdentityLimits) -> Ps5IdentityEvidenceReport {
    astero_loader::elf::dynamic::identity::observe(input, limits)
}
