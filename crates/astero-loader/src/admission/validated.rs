use crate::{
    artifact::{Architecture, ArtifactFamily, ArtifactId},
    dependencies::Dependency,
    exports::Export,
    imports::Import,
    metadata::{AddressRange, Permissions, SourceRange, VirtualAddress},
    modules::{ModuleMetadata, TargetRole},
    relocations::Relocation,
};
/// Normalized, format-independent facts. Public detached values do not grant admission authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TargetMetadata {
    pub identity: ArtifactId,
    pub family: ArtifactFamily,
    pub source_size: u64,
    pub source_label: Option<String>,
    pub architecture: Architecture,
    pub module: ModuleMetadata,
    pub role: TargetRole,
    pub entry_point: Option<VirtualAddress>,
    pub dependencies: Vec<Dependency>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub relocations: Vec<Relocation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedRegion {
    pub range: AddressRange,
    pub source: SourceRange,
    pub alignment: u64,
    pub permissions: Permissions,
}
/// Only admission can construct this capability; no mutable interior view is exposed.
/// ```compile_fail
/// use astero_loader::admission::ValidatedTarget;
/// fn alter(target: &mut ValidatedTarget) { target.metadata().source_size = 0; }
/// ```
/// ```compile_fail
/// use astero_loader::admission::ValidatedTarget;
/// let forged = ValidatedTarget { metadata: unreachable!(), regions: Vec::new() };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedTarget {
    pub(super) metadata: TargetMetadata,
    pub(super) regions: Vec<ValidatedRegion>,
}
impl ValidatedTarget {
    pub fn metadata(&self) -> &TargetMetadata {
        &self.metadata
    }
    pub fn regions(&self) -> &[ValidatedRegion] {
        &self.regions
    }
}
