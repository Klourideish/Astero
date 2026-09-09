use super::{LinkageFailure as Failure, NeededName};
use crate::elf::dynamic::{
    DynamicTable,
    string_table::DynamicStringTable,
    symbol_table::bounded::{SymbolObservationLimits, SymbolRecord},
    tags::DynamicTag,
};
pub(super) fn collect(
    table: &DynamicTable,
    symbols: &[SymbolRecord],
    limits: SymbolObservationLimits,
) -> Result<Vec<NeededName>, Failure> {
    let mut attempts = symbols.iter().filter(|s| s.name_bytes().is_some()).count() as u64;
    let spent: u64 = symbols
        .iter()
        .filter_map(|s| s.name_bytes())
        .map(|n| n.len() as u64 + 1)
        .sum();
    let mut remaining = limits.max_total_name_scan_bytes - spent;
    let strings = DynamicStringTable::from_dynamic(table).map_err(|error| Failure::Name {
        dynamic_index: 0,
        error,
    })?;
    let mut result = Vec::new();
    for entry in table
        .entries()
        .iter()
        .filter(|e| e.tag == DynamicTag::Needed)
    {
        attempts += 1;
        if attempts > limits.max_name_lookups {
            return Err(Failure::NameLookupBudget {
                attempted: attempts,
                maximum: limits.max_name_lookups,
            });
        }
        // M5 already requires STRTAB for NEEDED; keep a structured defensive failure.
        let strings = strings.as_ref().ok_or(Failure::Dynamic(
            crate::elf::dynamic::error::DynamicError::IncompleteDescriptor {
                present: DynamicTag::Needed,
                missing: DynamicTag::StrTab,
            },
        ))?;
        let name = strings
            .lookup(entry.value, limits.max_name_scan_bytes.min(remaining))
            .map_err(|error| Failure::Name {
                dynamic_index: entry.index,
                error,
            })?;
        remaining -= name.scanned_bytes();
        result.try_reserve(1).map_err(|_| Failure::Allocation {
            records: result.len() as u64 + 1,
        })?;
        result.push(NeededName {
            dynamic_index: entry.index,
            offset: entry.value,
            source: name.source_range(),
        });
    }
    Ok(result)
}
