use crate::{
    artifact::SourceArtifact,
    elf::{
        dynamic::{error::DynamicError, observation::RawTable},
        inspect::bounded::InspectionFailure,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicLimits {
    pub max_program_headers: u64,
    /// Attempted dynamic entries, including retained DT_NULL. Zero is valid and refuses any entry.
    pub max_dynamic_entries: u64,
}
#[derive(Debug)]
pub enum DynamicFailure {
    Headers(InspectionFailure),
    Table(DynamicError),
}
impl std::fmt::Display for DynamicFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "raw dynamic observation: {self:?}")
    }
}
impl std::error::Error for DynamicFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Headers(e) => Some(e),
            Self::Table(e) => Some(e),
        }
    }
}
#[derive(Debug)]
pub enum DynamicOutcome {
    Complete(RawTable),
    Unavailable,
    Failed(DynamicFailure),
}
/// Immutable observational input; neither an admitted target nor session state.
/// ```compile_fail
/// use astero_loader::elf::dynamic::bounded::{DynamicObservationReport, DynamicOutcome};
/// fn replace(r: &mut DynamicObservationReport) { r.outcome = DynamicOutcome::Unavailable; }
/// ```
#[derive(Debug)]
pub struct DynamicObservationReport {
    pub(super) source: SourceArtifact,
    pub(super) limits: DynamicLimits,
    pub(super) outcome: DynamicOutcome,
}
impl DynamicObservationReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn limits(&self) -> DynamicLimits {
        self.limits
    }
    pub fn outcome(&self) -> &DynamicOutcome {
        &self.outcome
    }
}
