use super::{RawRelocation, RelocationObservation, RelocationTables};
use crate::elf::{
    ElfInspection,
    dynamic::{
        self, DynamicObservation, ObservationLimits,
        relocations::{
            descriptor::{self, RelocationFormat, TableKind, TrustedRelocationExtent},
            error::RelocationError,
            plt, rela, symbol_reference,
        },
        symbol_table::SymbolTable,
    },
};
impl RelocationTables {
    pub fn new(elf: &ElfInspection, limits: ObservationLimits) -> Result<Self, RelocationError> {
        let dynamic = dynamic::observe(elf, limits).map_err(RelocationError::Dynamic)?;
        let tables = match &dynamic {
            DynamicObservation::Absent => Vec::new(),
            DynamicObservation::Present(t) => descriptor::discover(t),
        };
        let alias = plt::classify(&tables)?;
        Ok(Self {
            source: elf.artifact().source().clone(),
            tables,
            alias,
        })
    }
    /// Reuses M9 trusted descriptor and alias policy after independently bounded discovery.
    pub(in crate::elf::dynamic) fn from_dynamic(
        table: &crate::elf::dynamic::DynamicTable,
    ) -> Result<Self, RelocationError> {
        let tables = descriptor::discover(table);
        let alias = plt::classify(&tables)?;
        Ok(Self {
            source: table.source().clone(),
            tables,
            alias,
        })
    }
    pub fn read_raw(
        &self,
        extent: &TrustedRelocationExtent,
        index: u64,
    ) -> Result<RawRelocation, RelocationError> {
        let expected = self.source.identity();
        let actual = extent.source_range().source_id();
        if expected != actual {
            return Err(RelocationError::ExtentSourceMismatch { expected, actual });
        }
        if !self.tables.contains(extent) {
            return Err(RelocationError::ExtentNotOwned);
        }
        if extent.format() != RelocationFormat::Rela {
            return Err(RelocationError::UnsupportedRel {
                source: extent.source_range(),
            });
        }
        if index >= extent.entry_count() {
            return Err(RelocationError::EntryIndex {
                kind: extent.kind(),
                index,
                count: extent.entry_count(),
            });
        }
        let offset = index
            .checked_mul(24)
            .and_then(|n| extent.source_range().extent().offset.0.checked_add(n))
            .ok_or(RelocationError::Arithmetic {
                kind: extent.kind(),
                index,
            })?;
        let source = self
            .source
            .checked_range(offset, 24)
            .map_err(RelocationError::Source)?;
        let bytes = self.source.read(&source).map_err(RelocationError::Source)?;
        let record = rela::decode(bytes, index)?;
        let (mut dynamic_index, mut plt_index) = match extent.kind() {
            TableKind::Dynamic => (Some(index), None),
            TableKind::Plt => (None, Some(index)),
        };
        if let Some(alias) = self.alias {
            match extent.kind() {
                TableKind::Dynamic if index >= alias.first_dynamic_index => {
                    plt_index = Some(index - alias.first_dynamic_index)
                }
                TableKind::Plt => dynamic_index = Some(alias.first_dynamic_index + index),
                _ => {}
            }
        }
        Ok(RawRelocation {
            table_kind: extent.kind(),
            index,
            source,
            record,
            dynamic_index,
            plt_index,
        })
    }
    pub fn read<'a>(
        &self,
        extent: &TrustedRelocationExtent,
        index: u64,
        symbols: &'a SymbolTable<'_>,
        name_budget: u64,
    ) -> Result<RelocationObservation<'a>, RelocationError> {
        let raw = self.read_raw(extent, index)?;
        let symbol = symbol_reference::validate(&raw, symbols, name_budget)?;
        Ok(RelocationObservation { raw, symbol })
    }
}
