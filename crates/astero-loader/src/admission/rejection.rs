use crate::{
    artifact::{Architecture, ArtifactFamily, DeferredRequirement, SourceError},
    modules::ArtifactRole,
};
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
    VirtualRangeOverflow {
        region: usize,
    },
    InvalidSize {
        region: usize,
        file_size: u64,
        memory_size: u64,
    },
    SourceRange {
        region: usize,
        error: SourceError,
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
impl std::error::Error for Rejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SourceRange { error, .. } => Some(error),
            _ => None,
        }
    }
}
