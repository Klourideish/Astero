use crate::{
    dependencies::Dependency,
    exports::Export,
    imports::Import,
    metadata::{RegionObservation, VirtualAddress},
    modules::{ArtifactRole, ModuleMetadata},
    relocations::Relocation,
};
/// Stable caller-assigned identity of a synthetic artifact version. Not a content hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArtifactId(pub u128);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactFamily {
    Synthetic,
    Elf,
    SelfFormat,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeferredRequirement {
    ThreadLocalStorage,
    InitializationCallbacks,
    DynamicPlacement,
}
/// Observations are untrusted loader-local values, not admission results.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactDescription {
    pub family: ArtifactFamily,
    pub architecture: Architecture,
    pub role: ArtifactRole,
    pub module: Option<ModuleMetadata>,
    pub entry_point: Option<VirtualAddress>,
    pub regions: Vec<RegionObservation>,
    pub dependencies: Vec<Dependency>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub relocations: Vec<Relocation>,
    pub requirements: Vec<DeferredRequirement>,
}
/// Metadata-only synthetic input. No files are opened and no bytes are parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputArtifact {
    pub identity: ArtifactId,
    pub source_size: u64,
    pub source_label: Option<String>,
    pub description: ArtifactDescription,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectedArtifact {
    input: InputArtifact,
}
impl InspectedArtifact {
    pub fn identity(&self) -> ArtifactId {
        self.input.identity
    }
    pub fn source_size(&self) -> u64 {
        self.input.source_size
    }
    pub fn source_label(&self) -> Option<&str> {
        self.input.source_label.as_deref()
    }
    pub fn description(&self) -> &ArtifactDescription {
        &self.input.description
    }
}
/// The synthetic inspector preserves supplied observations, even unsupported or invalid ones.
/// A syntactically inspectable target is not necessarily accepted for loading or execution.
/// Future format-specific inspectors must adapt into this same description boundary.
pub fn inspect(input: InputArtifact) -> InspectedArtifact {
    InspectedArtifact { input }
}
