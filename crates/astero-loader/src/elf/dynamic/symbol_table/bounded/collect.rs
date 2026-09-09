use super::{
    SymbolObservationFailure as Failure, SymbolObservationLimits, SymbolObservationReport,
    SymbolOutcome, SymbolRecord,
};
use crate::{
    artifact::SourceArtifact,
    elf::dynamic::{
        descriptors::{self, DescriptorLimits, DescriptorOutcome, DescriptorValue},
        hash::bounded::{HashMetadataOutcome, HashMetadataReport},
        string_table::DynamicStringTable,
        symbol_table::{SymbolError, decode},
    },
};
use std::sync::Arc;
/// Explicitly consumes existing evidence; does not acquire bytes or derive hash evidence itself.
pub fn observe(
    source: SourceArtifact,
    proof: Arc<HashMetadataReport>,
    limits: SymbolObservationLimits,
) -> SymbolObservationReport {
    let outcome = match collect(&source, &proof, limits) {
        Ok(Some(records)) => SymbolOutcome::Complete(records),
        Ok(None) => SymbolOutcome::Unavailable,
        Err(e) => SymbolOutcome::Failed(e),
    };
    SymbolObservationReport {
        source,
        proof,
        limits,
        outcome,
    }
}
fn collect(
    source: &SourceArtifact,
    proof: &HashMetadataReport,
    limits: SymbolObservationLimits,
) -> Result<Option<Vec<SymbolRecord>>, Failure> {
    if source.identity() != proof.source().identity() {
        return Err(Failure::SourceMismatch {
            source: source.identity(),
            proof: proof.source().identity(),
        });
    }
    let hash = match proof.outcome() {
        HashMetadataOutcome::Complete(h) => h,
        HashMetadataOutcome::Unavailable => return Ok(None),
        HashMetadataOutcome::Failed(_) => return Err(Failure::EvidenceFailed),
    };
    let Some(extent) = hash.extent() else {
        return Ok(None);
    };
    let count = extent.symbol_count();
    if count > limits.max_symbols {
        return Err(Failure::EntryBudget {
            count,
            maximum: limits.max_symbols,
        });
    }
    let full = extent.source_range();
    source.read(&full).map_err(Failure::Source)?;
    if count.checked_mul(24) != Some(full.extent().size) {
        return Err(Failure::ExtentMismatch);
    }
    // Same immutable source, same explicit discovery limits; bounded M18 pairing is reused.
    let descriptors = descriptors::observe(
        source.clone(),
        DescriptorLimits {
            dynamic: proof.limits().dynamic,
            max_descriptors: limits.max_descriptors,
        },
    );
    let DescriptorOutcome::Complete(records) = descriptors.outcome() else {
        return Err(Failure::Descriptors(Box::new(descriptors)));
    };
    let symbols = records
        .iter()
        .find_map(|r| {
            if let DescriptorValue::Symbols(s) = r.value() {
                Some(s)
            } else {
                None
            }
        })
        .ok_or(Failure::ExtentMismatch)?;
    if symbols.entry_size != 24
        || symbols.first_entry.source_id() != source.identity()
        || symbols.first_entry.extent().offset != full.extent().offset
    {
        return Err(Failure::ExtentMismatch);
    }
    let strings = records
        .iter()
        .find(|r| matches!(r.value(), DescriptorValue::Strings(_)))
        .map(|r| DynamicStringTable::from_descriptor(source, r))
        .transpose()
        .map_err(|e| Failure::Symbol {
            index: 0,
            completed: 0,
            lookups: 0,
            remaining_scan_bytes: limits.max_total_name_scan_bytes,
            error: SymbolError::Name { index: 0, error: e },
        })?
        .flatten();
    let mut output = Vec::new();
    let capacity = usize::try_from(count).map_err(|_| Failure::Allocation { count })?;
    output
        .try_reserve_exact(capacity)
        .map_err(|_| Failure::Allocation { count })?;
    let (mut lookups, mut remaining) = (0u64, limits.max_total_name_scan_bytes);
    for index in 0..count {
        let at = index
            .checked_mul(24)
            .and_then(|v| full.extent().offset.0.checked_add(v))
            .ok_or(Failure::ExtentMismatch)?;
        let range = source.checked_range(at, 24).map_err(Failure::Source)?;
        let fields = decode::decode(source, range, index).map_err(|error| Failure::Symbol {
            index,
            completed: index,
            lookups,
            remaining_scan_bytes: remaining,
            error,
        })?;
        let name = if fields.name_offset == 0 {
            None
        } else {
            if lookups >= limits.max_name_lookups {
                return Err(Failure::NameLookupBudget {
                    index,
                    offset: fields.name_offset,
                    attempted: lookups + 1,
                    maximum: limits.max_name_lookups,
                });
            }
            lookups += 1;
            let result = strings
                .as_ref()
                .ok_or(SymbolError::StringsUnavailable {
                    index,
                    offset: fields.name_offset,
                })
                .and_then(|t| {
                    t.lookup(
                        u64::from(fields.name_offset),
                        remaining.min(limits.max_name_scan_bytes),
                    )
                    .map_err(|error| SymbolError::Name { index, error })
                });
            let view = result.map_err(|error| Failure::Symbol {
                index,
                completed: index,
                lookups,
                remaining_scan_bytes: remaining,
                error,
            })?;
            remaining -= view.scanned_bytes();
            Some(view.source_range())
        };
        output.push(SymbolRecord {
            fields,
            source: source.clone(),
            name,
        });
    }
    Ok(Some(output))
}
