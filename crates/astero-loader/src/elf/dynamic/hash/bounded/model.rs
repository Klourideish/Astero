use crate::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::{DynamicFailure, DynamicLimits},
        hash::{HashLimits, HashObservation, error::HashError},
        observation::RawTable,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashMetadataLimits {
    pub dynamic: DynamicLimits,
    /// Shared M8 32-bit word work budget, including repeated visits and bloom extent.
    pub hash: HashLimits,
}
#[derive(Debug)]
pub enum HashMetadataFailure {
    Discovery(DynamicFailure),
    Hash(HashError),
}
impl std::fmt::Display for HashMetadataFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "hash metadata: {self:?}")
    }
}
impl std::error::Error for HashMetadataFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Discovery(e) => Some(e),
            Self::Hash(e) => Some(e),
        }
    }
}
#[derive(Debug)]
pub enum HashMetadataOutcome {
    /// Hash metadata is validated; extent() is None for GNU lower-bound-only evidence.
    Complete(HashObservation),
    /// No supported hash descriptor (including no dynamic table).
    Unavailable,
    Failed(HashMetadataFailure),
}
/// Immutable count evidence, not symbol entries. Raw tags remain auditable on hash failure.
/// ```compile_fail
/// use astero_loader::elf::dynamic::hash::bounded::{HashMetadataReport, HashMetadataOutcome};
/// fn replace(r: &mut HashMetadataReport) { r.outcome = HashMetadataOutcome::Unavailable; }
/// ```
#[derive(Debug)]
pub struct HashMetadataReport {
    pub(super) source: SourceArtifact,
    pub(super) limits: HashMetadataLimits,
    pub(super) raw: Option<RawTable>,
    pub(super) outcome: HashMetadataOutcome,
}
impl HashMetadataReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn limits(&self) -> HashMetadataLimits {
        self.limits
    }
    pub fn raw(&self) -> Option<&RawTable> {
        self.raw.as_ref()
    }
    /// Original selected descriptor fields, including duplicates on failure; no payload reads.
    pub fn descriptor_fields(
        &self,
    ) -> impl Iterator<Item = &crate::elf::dynamic::entries::DynamicEntry> {
        use crate::elf::dynamic::tags::DynamicTag;
        self.raw.iter().flat_map(|r| r.entries()).filter(|e| {
            matches!(
                e.tag,
                DynamicTag::Hash | DynamicTag::GnuHash | DynamicTag::SymTab | DynamicTag::SymEnt
            )
        })
    }
    pub fn outcome(&self) -> &HashMetadataOutcome {
        &self.outcome
    }
}
