/// Caller-assigned identity within a synthetic dependency graph; not a guest handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleId(pub u64);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleMetadata {
    pub id: ModuleId,
    pub name: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactRole {
    Executable,
    Module,
    Unknown,
}
/// Roles admitted by the M2 policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetRole {
    Executable,
    Module,
}
