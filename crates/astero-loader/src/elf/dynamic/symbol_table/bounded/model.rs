use super::SymbolObservationFailure;
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::dynamic::{hash::bounded::HashMetadataReport, symbol_table::SymbolFields},
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolObservationLimits {
    pub max_descriptors: u64,
    pub max_symbols: u64,
    pub max_name_lookups: u64,
    pub max_name_scan_bytes: u64,
    pub max_total_name_scan_bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolRecord {
    pub(super) fields: SymbolFields,
    pub(super) source: SourceArtifact,
    pub(super) name: Option<BoundSourceRange>,
}
impl SymbolRecord {
    pub fn fields(&self) -> &SymbolFields {
        &self.fields
    }
    pub fn name_range(&self) -> Option<BoundSourceRange> {
        self.name
    }
    pub fn name_bytes(&self) -> Option<&[u8]> {
        self.name
            .as_ref()
            .map(|r| self.source.read(r).expect("private immutable name proof"))
    }
}
#[derive(Debug)]
pub enum SymbolOutcome {
    Complete(Vec<SymbolRecord>),
    Unavailable,
    Failed(SymbolObservationFailure),
}
/// Read-only owned report with shared proof and source; no borrowed local string-table lifetime.
/// ```compile_fail
/// use astero_loader::elf::dynamic::symbol_table::bounded::{SymbolObservationReport,SymbolOutcome};
/// fn mutate(r: &mut SymbolObservationReport) { r.outcome=SymbolOutcome::Unavailable; }
/// ```
#[derive(Debug)]
pub struct SymbolObservationReport {
    pub(super) source: SourceArtifact,
    pub(super) proof: Arc<HashMetadataReport>,
    pub(super) limits: SymbolObservationLimits,
    pub(super) outcome: SymbolOutcome,
}
impl SymbolObservationReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn proof(&self) -> &Arc<HashMetadataReport> {
        &self.proof
    }
    pub fn limits(&self) -> SymbolObservationLimits {
        self.limits
    }
    pub fn outcome(&self) -> &SymbolOutcome {
        &self.outcome
    }
}
