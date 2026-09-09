//! Read-only linkage report handle. None means no report has been supplied, not zero imports.
pub use astero_loader::elf::dynamic::candidates::report::{Completeness, LinkageEvidenceReport};
use std::sync::Arc;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkageSnapshot {
    report: Option<Arc<LinkageEvidenceReport>>,
}
impl LinkageSnapshot {
    pub fn report(&self) -> Option<&LinkageEvidenceReport> {
        self.report.as_deref()
    }
}
/// Pins one already collected report; never enumerates or classifies loader symbols.
pub fn inspect_linkage(report: Option<Arc<LinkageEvidenceReport>>) -> LinkageSnapshot {
    LinkageSnapshot { report }
}
