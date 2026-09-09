//! Structured relocation, symbol-reference and descriptor failures.
use super::descriptor::TableKind;
use crate::{
    artifact::{BoundSourceRange, SourceError, SourceId},
    elf::{
        ElfError,
        dynamic::{error::DynamicError, symbol_table::SymbolError},
    },
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelocationError {
    Dynamic(DynamicError),
    DescriptorConflict {
        ordinary: BoundSourceRange,
        plt: BoundSourceRange,
    },
    ExtentSourceMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    ExtentNotOwned,
    UnsupportedRel {
        source: BoundSourceRange,
    },
    EntryIndex {
        kind: TableKind,
        index: u64,
        count: u64,
    },
    Arithmetic {
        kind: TableKind,
        index: u64,
    },
    Source(SourceError),
    TruncatedEntry {
        index: u64,
        actual: u64,
    },
    Decode(ElfError),
    SymbolExtentUnavailable {
        kind: TableKind,
        index: u64,
        symbol: u32,
    },
    SymbolSourceMismatch {
        expected: SourceId,
        actual: SourceId,
    },
    SymbolIndex {
        kind: TableKind,
        index: u64,
        symbol: u32,
        count: u64,
    },
    SymbolObservation {
        kind: TableKind,
        index: u64,
        symbol: u32,
        error: SymbolError,
    },
    EntryBudget {
        count: u64,
        limit: u64,
    },
}
impl std::fmt::Display for RelocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RelocationError {}
