use super::StringReferenceFailure;
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::dynamic::{
        dependencies::StringLimits, descriptors::DescriptorLimits, entries::DynamicEntry,
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringReferenceLimits {
    pub descriptors: DescriptorLimits,
    pub max_string_references: u64,
    pub strings: StringLimits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StringEncoding {
    Empty,
    Utf8,
    RawBytes,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringReferenceRecord {
    pub(super) entry: DynamicEntry,
    pub(super) source: SourceArtifact,
    pub(super) range: BoundSourceRange,
    pub(super) encoding: StringEncoding,
}
impl StringReferenceRecord {
    pub fn entry(&self) -> DynamicEntry {
        self.entry
    }
    pub fn source_range(&self) -> BoundSourceRange {
        self.range
    }
    pub fn encoding(&self) -> StringEncoding {
        self.encoding
    }
    /// Original bytes, excluding NUL. The immutable private token was produced by successful M6 lookup.
    pub fn as_bytes(&self) -> &[u8] {
        self.source
            .read(&self.range)
            .expect("private immutable lookup proof")
    }
    pub fn scanned_bytes(&self) -> u64 {
        self.range.extent().size + 1
    }
}
#[derive(Debug)]
pub enum StringReferenceOutcome {
    Complete(Vec<StringReferenceRecord>),
    Unavailable,
    Failed(StringReferenceFailure),
}
/// Immutable source-bound reference evidence, never a dependency or inspected-artifact description.
/// ```compile_fail
/// use astero_loader::elf::dynamic::string_references::{StringReferenceObservationReport,StringReferenceOutcome};
/// fn replace(r:&mut StringReferenceObservationReport){r.outcome=StringReferenceOutcome::Unavailable;}
/// ```
#[derive(Debug)]
pub struct StringReferenceObservationReport {
    pub(super) source: SourceArtifact,
    pub(super) limits: StringReferenceLimits,
    pub(super) outcome: StringReferenceOutcome,
}
impl StringReferenceObservationReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn limits(&self) -> StringReferenceLimits {
        self.limits
    }
    pub fn outcome(&self) -> &StringReferenceOutcome {
        &self.outcome
    }
}
