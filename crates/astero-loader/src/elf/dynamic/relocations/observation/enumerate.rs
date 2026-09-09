use super::{RelocationObservation, RelocationTables};
use crate::elf::dynamic::{
    relocations::{
        descriptor::{RelocationFormat, TableKind},
        error::RelocationError,
    },
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
    tables: &'t RelocationTables,
    symbols: &'s SymbolTable<'e>,
    table: usize,
    index: u64,
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
        let mut count = 0u64;
        for table in &self.tables {
            if table.format() != RelocationFormat::Rela {
                return Err(RelocationError::UnsupportedRel {
                    source: table.source_range(),
                });
            }
            // At most two source-backed tables; use checked addition even for theoretical huge sources.
            count = count
                .checked_add(if table.kind() == TableKind::Dynamic {
                    self.alias
                        .map_or(table.entry_count(), |a| a.first_dynamic_index)
                } else {
                    table.entry_count()
                })
                .ok_or(RelocationError::Arithmetic {
                    kind: table.kind(),
                    index: 0,
                })?;
        }
        if count > limits.max_entries {
            return Err(RelocationError::EntryBudget {
                count,
                limit: limits.max_entries,
            });
        }
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
            tables: self,
            symbols,
            table: 0,
            index: 0,
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
        while let Some(table) = self.tables.tables.get(self.table) {
            let count = if table.kind() == TableKind::Dynamic {
                self.tables
                    .alias
                    .map_or(table.entry_count(), |a| a.first_dynamic_index)
            } else {
                table.entry_count()
            };
            if self.index >= count {
                self.table += 1;
                self.index = 0;
                continue;
            }
            let result = self.tables.read(
                table,
                self.index,
                self.symbols,
                self.per_name.min(self.remaining),
            );
            self.index += 1;
            match &result {
                Ok(o) => {
                    if let Some(name) = o.symbol.observation.name {
                        self.remaining -= name.scanned_bytes();
                    }
                }
                Err(_) => self.failed = true,
            }
            return Some(result);
        }
        None
    }
}
impl std::iter::FusedIterator for RelocationIterator<'_, '_, '_> {}
