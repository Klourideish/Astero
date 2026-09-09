use crate::elf::dynamic::{
    relocations::{error::RelocationError, observation::RawRelocation},
    symbol_table::{DynamicSymbolObservation, SymbolTable},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolReferenceKind {
    Null,
    Nonzero(u32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolReference<'a> {
    pub kind: SymbolReferenceKind,
    pub observation: DynamicSymbolObservation<'a>,
}
pub(in crate::elf::dynamic::relocations) fn validate<'a>(
    raw: &RawRelocation,
    symbols: &'a SymbolTable<'_>,
    name_budget: u64,
) -> Result<SymbolReference<'a>, RelocationError> {
    let symbol = raw.record.symbol_index();
    let kind = raw.table_kind;
    let index = raw.index;
    let extent = symbols
        .extent()
        .ok_or(RelocationError::SymbolExtentUnavailable {
            kind,
            index,
            symbol,
        })?;
    let expected = raw.source.source_id();
    let actual = extent.source_range().source_id();
    if expected != actual {
        return Err(RelocationError::SymbolSourceMismatch { expected, actual });
    }
    let count = extent.symbol_count();
    if u64::from(symbol) >= count {
        return Err(RelocationError::SymbolIndex {
            kind,
            index,
            symbol,
            count,
        });
    }
    let observation = symbols
        .read_candidate(u64::from(symbol), name_budget)
        .map_err(|error| RelocationError::SymbolObservation {
            kind,
            index,
            symbol,
            error,
        })?;
    Ok(SymbolReference {
        kind: if symbol == 0 {
            SymbolReferenceKind::Null
        } else {
            SymbolReferenceKind::Nonzero(symbol)
        },
        observation,
    })
}
