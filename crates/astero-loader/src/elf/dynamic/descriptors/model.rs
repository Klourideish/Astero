use crate::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::{DynamicFailure, DynamicLimits},
        entries::DynamicEntry,
        error::DynamicError,
        observation::{RawTable, SymbolTableDescriptor, TableDescriptor},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DescriptorLimits {
    pub dynamic: DynamicLimits,
    /// Distinct supported families attempted, including incomplete/conflicting groups. Zero is valid.
    pub max_descriptors: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorFamily {
    Strings,
    Symbols,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DescriptorValue {
    Strings(TableDescriptor),
    Symbols(SymbolTableDescriptor),
}
/// Pair provenance retains the original tag, raw value and dynamic-entry index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptorRecord {
    pub(super) fields: [DynamicEntry; 2],
    pub(super) value: DescriptorValue,
}
impl DescriptorRecord {
    pub fn fields(&self) -> &[DynamicEntry; 2] {
        &self.fields
    }
    pub fn value(&self) -> &DescriptorValue {
        &self.value
    }
    pub fn family(&self) -> DescriptorFamily {
        match self.value {
            DescriptorValue::Strings(_) => DescriptorFamily::Strings,
            DescriptorValue::Symbols(_) => DescriptorFamily::Symbols,
        }
    }
}
#[derive(Debug)]
pub enum DescriptorFailure {
    Discovery(DynamicFailure),
    Budget {
        attempted_families: u64,
        maximum: u64,
    },
    Interpretation(DynamicError),
    Allocation {
        requested_descriptors: u64,
    },
}
impl std::fmt::Display for DescriptorFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "descriptor observation: {self:?}")
    }
}
impl std::error::Error for DescriptorFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Discovery(e) => Some(e),
            Self::Interpretation(e) => Some(e),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorUnavailable {
    NoDynamicTable,
    NoSupportedDescriptors,
}
#[derive(Debug)]
pub enum DescriptorOutcome {
    Complete(Vec<DescriptorRecord>),
    Unavailable(DescriptorUnavailable),
    Failed(DescriptorFailure),
}
/// Immutable metadata evidence; raw evidence is retained even if selected pairing fails.
/// ```compile_fail
/// use astero_loader::elf::dynamic::descriptors::{DescriptorObservationReport,DescriptorOutcome,DescriptorUnavailable};
/// fn replace(r: &mut DescriptorObservationReport) { r.outcome=DescriptorOutcome::Unavailable(DescriptorUnavailable::NoDynamicTable); }
/// ```
#[derive(Debug)]
pub struct DescriptorObservationReport {
    pub(super) source: SourceArtifact,
    pub(super) limits: DescriptorLimits,
    pub(super) raw: Option<RawTable>,
    pub(super) outcome: DescriptorOutcome,
}
impl DescriptorObservationReport {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn limits(&self) -> DescriptorLimits {
        self.limits
    }
    pub fn raw(&self) -> Option<&RawTable> {
        self.raw.as_ref()
    }
    pub fn outcome(&self) -> &DescriptorOutcome {
        &self.outcome
    }
}
