use crate::{
    artifact::SourceArtifact,
    elf::{ElfError, header::Elf64Header, program_headers::ProgramHeader},
};
use std::collections::TryReserveError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InspectionLimits {
    /// Maximum decoded/retained program headers, including non-load types. Zero allows header-only input.
    pub max_program_headers: u64,
}
#[derive(Debug)]
pub enum InspectionFailure {
    Header(ElfError),
    HeaderBudget {
        declared: u16,
        maximum: u64,
    },
    Allocation {
        requested_headers: u16,
        source: TryReserveError,
    },
    ProgramHeader {
        index: u16,
        error: ElfError,
    },
}
impl std::fmt::Display for InspectionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "structural inspection: {self:?}")
    }
}
impl std::error::Error for InspectionFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Header(e) | Self::ProgramHeader { error: e, .. } => Some(e),
            Self::Allocation { source, .. } => Some(source),
            Self::HeaderBudget { .. } => None,
        }
    }
}
/// Raw structural observations, not a validated target or loaded module.
#[derive(Debug, PartialEq, Eq)]
pub struct HeaderEvidence {
    pub(super) header: Elf64Header,
    pub(super) programs: Vec<ProgramHeader>,
}
impl HeaderEvidence {
    pub fn header(&self) -> &Elf64Header {
        &self.header
    }
    pub fn program_headers(&self) -> &[ProgramHeader] {
        &self.programs
    }
}
#[derive(Debug)]
pub enum InspectionOutcome {
    Complete(HeaderEvidence),
    Failed(InspectionFailure),
}
/// Immutable source-bound evidence, including failure; no caller-writable report fields.
/// ```compile_fail
/// use astero_loader::elf::inspect::bounded::{InspectionReport, InspectionOutcome};
/// fn replace(report: &mut InspectionReport, outcome: InspectionOutcome) { report.outcome = outcome; }
/// ```
#[derive(Debug)]
pub struct InspectionReport {
    pub(super) source: SourceArtifact,
    pub(super) limits: InspectionLimits,
    pub(super) outcome: InspectionOutcome,
}
impl InspectionReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn limits(&self) -> InspectionLimits {
        self.limits
    }
    pub fn outcome(&self) -> &InspectionOutcome {
        &self.outcome
    }
}
