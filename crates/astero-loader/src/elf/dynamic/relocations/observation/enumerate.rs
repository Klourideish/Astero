use super::{RelocationObservation, RelocationTables};
use crate::elf::dynamic::{
    relocations::{descriptor::TableKind, error::RelocationError},
    symbol_table::SymbolTable,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelocationLimits {
    pub max_entries: u64,
    pub max_name_scan_bytes: u64,
    pub max_total_name_scan_bytes: u64,
}
/// Canonical ordinary-prefix then PLT enumeration: alias records are emitted once with both labels.
pub struct RelocationIterator<'t, 's, 'e> {
    raw: super::RawRelocationIterator<'t>,
    symbols: &'s SymbolTable<'e>,
    failed: bool,
    per_name: u64,
    remaining: u64,
}
impl RelocationTables {
    pub fn enumerate<'t, 's, 'e>(
        &'t self,
        symbols: &'s SymbolTable<'e>,
        limits: RelocationLimits,
    ) -> Result<RelocationIterator<'t, 's, 'e>, RelocationError> {
        let raw = self.enumerate_raw(limits.max_entries)?;
        // Require same-source trusted symbol evidence even for empty enumeration; no silent validation bypass.
        let extent = symbols
            .extent()
            .ok_or(RelocationError::SymbolExtentUnavailable {
                kind: TableKind::Dynamic,
                index: 0,
                symbol: 0,
            })?;
        if extent.source_range().source_id() != self.source.identity() {
            return Err(RelocationError::SymbolSourceMismatch {
                expected: self.source.identity(),
                actual: extent.source_range().source_id(),
            });
        }
        Ok(RelocationIterator {
            raw,
            symbols,
            failed: false,
            per_name: limits.max_name_scan_bytes,
            remaining: limits.max_total_name_scan_bytes,
        })
    }
}
impl<'s> Iterator for RelocationIterator<'_, 's, '_> {
    type Item = Result<RelocationObservation<'s>, RelocationError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let raw = match self.raw.next()? {
            Ok(raw) => raw,
            Err(e) => {
                self.failed = true;
                return Some(Err(e));
            }
        };
        let result = super::super::symbol_reference::validate(
            &raw,
            self.symbols,
            self.per_name.min(self.remaining),
        )
        .map(|symbol| RelocationObservation { raw, symbol });
        match &result {
            Ok(o) => {
                if let Some(name) = o.symbol.observation.name {
                    self.remaining -= name.scanned_bytes();
                }
            }
            Err(_) => self.failed = true,
        }
        Some(result)
    }
}
impl std::iter::FusedIterator for RelocationIterator<'_, '_, '_> {}
