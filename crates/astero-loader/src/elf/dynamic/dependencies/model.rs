use crate::{
    artifact::{BoundSourceRange, InspectedArtifact},
    elf::dynamic::{DynamicObservation, string_table::DynamicStringTable},
};
/// Explicit lookup-work limits, counting terminators and repeated offsets; no hidden name-size cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringLimits {
    pub max_scan_bytes_per_reference: u64,
    pub max_total_scan_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeededReference {
    pub entry_index: u64,
    pub string_offset: u64,
    pub source: BoundSourceRange,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyObservation {
    pub(super) artifact: InspectedArtifact,
    pub(super) dynamic: DynamicObservation,
    pub(super) string_table: Option<DynamicStringTable>,
    pub(super) references: Vec<NeededReference>,
}
impl DependencyObservation {
    pub fn artifact(&self) -> &InspectedArtifact {
        &self.artifact
    }
    pub fn dynamic(&self) -> &DynamicObservation {
        &self.dynamic
    }
    pub fn string_table(&self) -> Option<&DynamicStringTable> {
        self.string_table.as_ref()
    }
    pub fn references(&self) -> &[NeededReference] {
        &self.references
    }
}
