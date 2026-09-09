use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::dynamic::{
        bounded::DynamicFailure,
        candidates::structural::SymbolClassificationReport,
        error::DynamicError,
        hash::bounded::HashMetadataLimits,
        relocations::{error::RelocationError, observation::RawRelocation},
        string_table::StringTableError,
        symbol_table::bounded::SymbolObservationLimits,
    },
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LinkageLimits {
    pub hash: HashMetadataLimits,
    pub symbols: SymbolObservationLimits,
    pub max_relocations: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceRole {
    StructuralOnly,
    ExternalReferenceCandidate,
    AmbiguousReference,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolUse {
    pub symbol_index: u64,
    pub references: u64,
    pub ordinary: u64,
    pub plt: u64,
    pub role: ReferenceRole,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NeededName {
    pub dynamic_index: u64,
    pub offset: u64,
    pub source: BoundSourceRange,
}
#[derive(Debug)]
pub enum LinkageFailure {
    Symbols,
    UnsupportedRelocationDescriptor {
        dynamic_index: u64,
        tag: i64,
    },
    Discovery(DynamicFailure),
    Dynamic(DynamicError),
    Relocation(RelocationError),
    SymbolIndex {
        raw: Box<RawRelocation>,
        count: u64,
    },
    Name {
        dynamic_index: u64,
        error: StringTableError,
    },
    NameLookupBudget {
        attempted: u64,
        maximum: u64,
    },
    Allocation {
        records: u64,
    },
}
impl std::fmt::Display for LinkageFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "linkage evidence: {self:?}")
    }
}
impl std::error::Error for LinkageFailure {}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LinkageCounts {
    pub symbols: u64,
    pub relocations: u64,
    pub symbol_associated: u64,
    pub null_references: u64,
    pub external_candidates: u64,
    pub definitions: u64,
    pub ambiguous_references: u64,
}
#[derive(Debug)]
pub struct LinkageEvidence {
    pub(super) counts: LinkageCounts,
    pub(super) relocations: Vec<RawRelocation>,
    pub(super) uses: Vec<SymbolUse>,
    pub(super) needed: Vec<NeededName>,
}
impl LinkageEvidence {
    pub fn counts(&self) -> LinkageCounts {
        self.counts
    }
    pub fn relocations(&self) -> &[RawRelocation] {
        &self.relocations
    }
    pub fn symbol_uses(&self) -> &[SymbolUse] {
        &self.uses
    }
    pub fn needed(&self) -> &[NeededName] {
        &self.needed
    }
}
#[derive(Debug)]
pub enum LinkageOutcome {
    Complete(LinkageEvidence),
    Unavailable,
    Failed(LinkageFailure),
}
/// Shared source and prerequisite proof; no public constructor or mutable report view.
/// ```compile_fail
/// use astero_loader::elf::dynamic::candidates::workload::{LinkageReport,LinkageOutcome};
/// fn alter(r:&mut LinkageReport){r.outcome=LinkageOutcome::Unavailable;}
/// ```
#[derive(Debug)]
pub struct LinkageReport {
    pub(super) source: SourceArtifact,
    pub(super) limits: LinkageLimits,
    pub(super) symbols: Arc<SymbolClassificationReport>,
    pub(super) outcome: LinkageOutcome,
}
impl LinkageReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn limits(&self) -> LinkageLimits {
        self.limits
    }
    pub fn symbols(&self) -> &Arc<SymbolClassificationReport> {
        &self.symbols
    }
    pub fn outcome(&self) -> &LinkageOutcome {
        &self.outcome
    }
}
