use crate::{
    artifact::{Architecture, ArtifactFamily, DeferredRequirement},
    modules::ArtifactRole,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RangeDomain {
    Virtual,
    Source,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataField {
    Module,
    ModuleName,
    Regions,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataProblem {
    DuplicateDependency,
    SelfDependency,
    EmptySymbol,
    UnknownDependency,
    DuplicateImport,
    DuplicateExport,
    InvalidExportRange,
    NonExecutableExport,
    InvalidRelocationRange,
    UnknownImport,
    InvalidLocalTarget,
    ConflictingRelocations,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescriptorKind {
    Dependency,
    Import,
    Export,
    Relocation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rejection {
    UnsupportedFormat(ArtifactFamily),
    UnsupportedArchitecture(Architecture),
    UnsupportedRole(ArtifactRole),
    MissingMetadata(MetadataField),
    UnsupportedRequirement(DeferredRequirement),
    EmptyRegion {
        region: usize,
    },
    RangeOverflow {
        region: usize,
        domain: RangeDomain,
    },
    InvalidSize {
        region: usize,
        file_size: u64,
        memory_size: u64,
    },
    SourceOutOfBounds {
        region: usize,
        end: u64,
        source_size: u64,
    },
    InvalidAlignment {
        region: usize,
        alignment: u64,
        address: u64,
    },
    OverlappingRegions {
        first: usize,
        second: usize,
    },
    InvalidEntryPoint {
        address: Option<u64>,
    },
    MalformedMetadata {
        kind: DescriptorKind,
        index: usize,
        problem: MetadataProblem,
    },
    UnsupportedRelocation {
        index: usize,
    },
}
impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "target admission rejected: {self:?}")
    }
}
impl std::error::Error for Rejection {}
