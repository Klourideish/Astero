use super::codec::EncodingError;
use crate::{
    artifact::BoundSourceRange,
    elf::dynamic::{
        bounded::DynamicFailure, candidates::workload::LinkageReport, error::DynamicError,
        string_table::StringTableError,
    },
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IdentityLimits {
    pub max_identity_records: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorKind {
    Module,
    NeededModule,
    ExportLibrary,
    ImportLibrary,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataEvidence {
    ExperimentalPacking,
    Uninterpreted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdentityDescriptor {
    pub dynamic_index: u64,
    pub tag: i64,
    pub raw: u64,
    pub kind: DescriptorKind,
    pub evidence: MetadataEvidence,
    pub id: Option<u16>,
    pub version_bits: Option<u16>,
    pub name: Option<BoundSourceRange>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextEvidence {
    Absent,
    Missing {
        id: u16,
    },
    /// Matching packed fields under the documented hypothesis, not provider identity proof.
    Hypothesis {
        id: u16,
        dynamic_index: u64,
        declarations: u64,
    },
    Conflict {
        id: u16,
        declarations: u64,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameEvidence {
    Unnamed,
    PlainOrRaw,
    Candidate(EncodingError),
    /// Canonical byte decoding corroborated by legacy/decrypted/firmware vectors; no function meaning.
    Encoded {
        nid: u64,
        library: ContextEvidence,
        module: ContextEvidence,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedSymbol {
    pub symbol_index: u64,
    pub name: Option<BoundSourceRange>,
    pub evidence: NameEvidence,
}
#[derive(Debug)]
pub enum IdentityFailure {
    LinkagePrerequisite,
    Budget {
        count: u64,
        maximum: u64,
    },
    Allocation {
        records: u64,
    },
    Discovery(DynamicFailure),
    Dynamic(DynamicError),
    String {
        dynamic_index: u64,
        error: StringTableError,
    },
    StringsUnavailable {
        dynamic_index: u64,
    },
    NameLookupBudget {
        attempted: u64,
        maximum: u64,
    },
}
impl std::fmt::Display for IdentityFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PS5 identity: {self:?}")
    }
}
impl std::error::Error for IdentityFailure {}
#[derive(Debug)]
pub struct IdentityEvidence {
    pub(super) descriptors: Vec<IdentityDescriptor>,
    pub(super) symbols: Vec<EncodedSymbol>,
}
impl IdentityEvidence {
    pub fn descriptors(&self) -> &[IdentityDescriptor] {
        &self.descriptors
    }
    pub fn symbols(&self) -> &[EncodedSymbol] {
        &self.symbols
    }
}
#[derive(Debug)]
pub enum IdentityOutcome {
    Complete(IdentityEvidence),
    Unavailable,
    Failed(IdentityFailure),
}
/// Immutable same-source evidence. Attachment does not prove a provider or runtime identity.
#[derive(Debug)]
pub struct Ps5IdentityEvidenceReport {
    pub(super) linkage: Arc<LinkageReport>,
    pub(super) limits: IdentityLimits,
    pub(super) outcome: IdentityOutcome,
}
impl Ps5IdentityEvidenceReport {
    pub fn linkage(&self) -> &Arc<LinkageReport> {
        &self.linkage
    }
    pub fn limits(&self) -> IdentityLimits {
        self.limits
    }
    pub fn outcome(&self) -> &IdentityOutcome {
        &self.outcome
    }
}
