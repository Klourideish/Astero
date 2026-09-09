use super::{RawRelocation, RelocationTables};
use crate::elf::dynamic::relocations::{
    descriptor::{RelocationFormat, TableKind},
    error::RelocationError,
};
/// Canonical raw records only. No validated symbol-reference claim.
pub struct RawRelocationIterator<'a> {
    tables: &'a RelocationTables,
    table: usize,
    index: u64,
    failed: bool,
}
impl RelocationTables {
    pub fn enumerate_raw(
        &self,
        maximum: u64,
    ) -> Result<RawRelocationIterator<'_>, RelocationError> {
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
        if count > maximum {
            return Err(RelocationError::EntryBudget {
                count,
                limit: maximum,
            });
        }
        Ok(RawRelocationIterator {
            tables: self,
            table: 0,
            index: 0,
            failed: false,
        })
    }
}
impl Iterator for RawRelocationIterator<'_> {
    type Item = Result<RawRelocation, RelocationError>;
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
            let result = self.tables.read_raw(table, self.index);
            self.index += 1;
            self.failed = result.is_err();
            return Some(result);
        }
        None
    }
}
impl std::iter::FusedIterator for RawRelocationIterator<'_> {}
