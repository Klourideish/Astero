use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::dynamic::relocations::{
        descriptor::{TableKind, TrustedRelocationExtent},
        plt::TailAlias,
        rela::RelaRecord,
        symbol_reference::SymbolReference,
    },
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelocationTables {
    pub(super) source: SourceArtifact,
    pub(super) tables: Vec<TrustedRelocationExtent>,
    pub(super) alias: Option<TailAlias>,
}
/// Raw observation only: this value does not assert a validated symbol reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawRelocation {
    pub table_kind: TableKind,
    pub index: u64,
    pub source: BoundSourceRange,
    pub record: RelaRecord,
    pub dynamic_index: Option<u64>,
    pub plt_index: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelocationObservation<'a> {
    pub raw: RawRelocation,
    pub symbol: SymbolReference<'a>,
}
impl RelocationTables {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn extents(&self) -> &[TrustedRelocationExtent] {
        &self.tables
    }
    pub fn tail_alias(&self) -> Option<TailAlias> {
        self.alias
    }
}
