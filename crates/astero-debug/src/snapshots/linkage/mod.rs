//! Session-bound linkage inspection, distinct from explicitly standalone tooling.
use astero_core::observation::{ObservationError, ObserveSession, SessionSnapshot};
pub use astero_core::session::inputs::{Completeness, LinkageEvidenceReport};
use std::sync::Arc;
#[derive(Clone, Debug, PartialEq, Eq)]
enum Origin {
    Session(SessionSnapshot),
    Standalone(Option<Arc<LinkageEvidenceReport>>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkageSnapshot {
    origin: Origin,
}
impl LinkageSnapshot {
    pub fn report(&self) -> Option<&LinkageEvidenceReport> {
        match &self.origin {
            Origin::Session(s) => s.inputs.linkage().map(AsRef::as_ref),
            Origin::Standalone(r) => r.as_deref(),
        }
    }
    pub fn session(&self) -> Option<&SessionSnapshot> {
        match &self.origin {
            Origin::Session(s) => Some(s),
            Origin::Standalone(_) => None,
        }
    }
}
/// One session observation captures lifecycle and immutable report identity together.
pub fn inspect_linkage(source: &impl ObserveSession) -> Result<LinkageSnapshot, ObservationError> {
    Ok(LinkageSnapshot {
        origin: Origin::Session(source.snapshot()?),
    })
}
/// Offline tool/test route: explicitly has no session identity or lifecycle claim.
pub fn inspect_standalone_report(report: Option<Arc<LinkageEvidenceReport>>) -> LinkageSnapshot {
    LinkageSnapshot {
        origin: Origin::Standalone(report),
    }
}
