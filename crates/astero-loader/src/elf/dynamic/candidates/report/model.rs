use super::Completeness;
use crate::{
    artifact::{BoundSourceRange, SourceId},
    elf::dynamic::{
        candidates::{classification::Classification, evidence::NameState},
        hash::extent::TrustedSymbolExtent,
        relocations::RelocationLimits,
        symbol_table::{Binding, Section, SymbolType, Visibility},
    },
    modules::{ArtifactRole, ModuleId},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReportBudget {
    pub max_symbols: u64,
    pub max_details: u64,
    pub max_retained_name_bytes: u64,
    pub max_name_scan_bytes: u64,
    pub max_total_name_scan_bytes: u64,
    pub relocations: RelocationLimits,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelocationCounts {
    pub unique: u64,
    pub ordinary: u64,
    pub plt: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReportCounts {
    pub observed: u64,
    pub imports: u64,
    pub exports: u64,
    pub internal: u64,
    pub unclassified: u64,
    pub undefined_unclassified: u64,
    pub null: u64,
    pub absolute: u64,
    pub common: u64,
    pub unnamed: u64,
    pub empty_names: u64,
    pub non_utf8_names: u64,
    pub unknown_binding: u64,
    pub unknown_type: u64,
    pub unknown_visibility: u64,
    pub relocations: RelocationCounts,
}
/// Detached detail values; report access is read-only. Name bytes are exact, never lossy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateDetail {
    pub index: u64,
    pub source: BoundSourceRange,
    pub classification: Classification,
    pub name_state: NameState,
    pub name: Option<Vec<u8>>,
    pub name_source: Option<BoundSourceRange>,
    pub name_offset: u32,
    pub info: u8,
    pub other: u8,
    pub binding: Binding,
    pub symbol_type: SymbolType,
    pub visibility: Visibility,
    pub section: Section,
    pub value: u64,
    pub size: u64,
    pub relocations: RelocationCounts,
}
/// One owned immutable collection pass; tokens retain provenance, not runtime handles.
/// ```compile_fail
/// use astero_loader::elf::dynamic::candidates::report::LinkageEvidenceReport;
/// fn change(report: &mut LinkageEvidenceReport) { report.counts.imports = 999; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkageEvidenceReport {
    pub(super) source: SourceId,
    pub(super) module: Option<ModuleId>,
    pub(super) role: ArtifactRole,
    pub(super) extent: Option<TrustedSymbolExtent>,
    pub(super) completeness: Completeness,
    pub(super) counts: ReportCounts,
    pub(super) details: Vec<CandidateDetail>,
}
impl LinkageEvidenceReport {
    pub fn source(&self) -> SourceId {
        self.source
    }
    /// Caller-provided inspection context; not an ELF-derived runtime module identity.
    pub fn module(&self) -> Option<ModuleId> {
        self.module
    }
    pub fn role(&self) -> ArtifactRole {
        self.role
    }
    pub fn extent(&self) -> Option<&TrustedSymbolExtent> {
        self.extent.as_ref()
    }
    pub fn completeness(&self) -> &Completeness {
        &self.completeness
    }
    pub fn counts(&self) -> &ReportCounts {
        &self.counts
    }
    pub fn details(&self) -> &[CandidateDetail] {
        &self.details
    }
}
