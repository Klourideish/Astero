use super::{SourceArtifact, SourceId};
use crate::{
    dependencies::Dependency,
    exports::Export,
    imports::Import,
    metadata::{RegionObservation, VirtualAddress},
    modules::{ArtifactRole, ModuleMetadata},
    relocations::Relocation,
};
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
/// Synthetic observations must carry an actual immutable source. No bytes are parsed.
/// ```compile_fail
/// use astero_loader::artifact::InputArtifact;
/// fn spoof(input: &mut InputArtifact) { input.source_size = 0x5000; }
/// ```
/// ```compile_fail
/// use astero_loader::artifact::{InputArtifact, SourceId};
/// fn spoof(input: &mut InputArtifact, identity: SourceId) { input.identity = identity; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputArtifact {
    pub source: SourceArtifact,
    pub description: ArtifactDescription,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectedArtifact {
    input: InputArtifact,
}
impl InspectedArtifact {
    pub fn identity(&self) -> SourceId {
        self.input.source.identity()
    }
    pub fn source_size(&self) -> u64 {
        self.input.source.len()
    }
    pub fn source_label(&self) -> Option<&str> {
        self.input.source.provenance()
    }
    pub fn source(&self) -> &SourceArtifact {
        &self.input.source
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
