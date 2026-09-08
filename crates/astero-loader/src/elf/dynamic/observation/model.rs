use super::super::entries::DynamicEntry;
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    metadata::VirtualAddress,
};
/// Explicit caller resource budget, including the terminating DT_NULL. No hidden PS5 limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationLimits {
    pub max_entries: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DynamicObservation {
    Absent,
    Present(Box<DynamicTable>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicTable {
    pub(super) source: SourceArtifact,
    pub(super) source_range: BoundSourceRange,
    pub(super) program_index: usize,
    pub(super) entries: Vec<DynamicEntry>,
    pub(super) descriptors: DynamicDescriptors,
}
impl DynamicTable {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn source_range(&self) -> BoundSourceRange {
        self.source_range
    }
    pub fn program_index(&self) -> usize {
        self.program_index
    }
    /// Includes the first DT_NULL. Remaining declared table bytes are not interpreted.
    pub fn entries(&self) -> &[DynamicEntry] {
        &self.entries
    }
    pub fn descriptors(&self) -> &DynamicDescriptors {
        &self.descriptors
    }
}
/// Detached descriptor values do not grant linking or full table-semantic validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableDescriptor {
    pub address: VirtualAddress,
    pub size: u64,
    pub entry_size: Option<u64>,
    pub source: BoundSourceRange,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolTableDescriptor {
    pub address: VirtualAddress,
    pub entry_size: u64,
    /// Only one entry's backing is proven; no symbol count or whole-table extent is inferred.
    pub first_entry: BoundSourceRange,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PltRelocationKind {
    Rel,
    Rela,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PltDescriptor {
    pub kind: PltRelocationKind,
    pub table: TableDescriptor,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DynamicDescriptors {
    pub strings: Option<TableDescriptor>,
    pub symbols: Option<SymbolTableDescriptor>,
    pub rela: Option<TableDescriptor>,
    pub plt: Option<PltDescriptor>,
    pub needed_offsets: Vec<u64>,
    /// Addresses are observed only, not dereferenced, validated as code or invoked.
    pub init: Option<VirtualAddress>,
    pub fini: Option<VirtualAddress>,
    pub init_array: Option<TableDescriptor>,
    pub fini_array: Option<TableDescriptor>,
}
