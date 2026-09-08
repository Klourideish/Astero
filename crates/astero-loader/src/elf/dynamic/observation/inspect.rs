use super::{
    descriptors,
    model::{DynamicObservation, DynamicTable, ObservationLimits},
};
use crate::elf::{
    ElfInspection,
    address_translation::AddressTranslator,
    dynamic::{entries, error::DynamicError, tags::DynamicTag},
};
use crate::metadata::VirtualAddress;
/// Observes one bounded table. Leaves M4's admission requirements unchanged, even on success.
pub fn observe(
    elf: &ElfInspection,
    limits: ObservationLimits,
) -> Result<DynamicObservation, DynamicError> {
    let mut found = None;
    for (index, p) in elf.program_headers().enumerate() {
        let p = p.map_err(DynamicError::Header)?;
        if p.kind == 2 {
            if let Some((first, _)) = found {
                return Err(DynamicError::MultipleTables {
                    first,
                    second: index,
                });
            }
            found = Some((index, p));
        }
    }
    let Some((index, p)) = found else {
        return Ok(DynamicObservation::Absent);
    };
    if p.file_size > p.memory_size {
        return Err(DynamicError::InvalidTableSize {
            segment: index,
            file_size: p.file_size,
            memory_size: p.memory_size,
        });
    }
    p.virtual_address
        .checked_add(p.memory_size)
        .ok_or(DynamicError::TableAddressOverflow {
            segment: index,
            address: p.virtual_address,
            size: p.memory_size,
        })?;
    let source = elf.artifact().source();
    let range = source
        .checked_range(p.file_offset, p.file_size)
        .map_err(|error| DynamicError::Source {
            segment: index,
            error,
        })?;
    let translator = AddressTranslator::new(elf)
        .map_err(|error| DynamicError::Translation { tag: None, error })?;
    let translated = translator
        .translate(VirtualAddress(p.virtual_address), p.file_size)
        .map_err(|error| DynamicError::Translation { tag: None, error })?;
    if translated != range {
        return Err(DynamicError::TableOffsetMismatch {
            segment: index,
            declared: p.file_offset,
            translated: translated.extent().offset.0,
        });
    }
    let bytes = source.read(&range).map_err(|error| DynamicError::Source {
        segment: index,
        error,
    })?;
    let mut observed = Vec::new();
    for (entry_index, chunk) in bytes.chunks(16).enumerate() {
        let entry_index = entry_index as u64;
        if entry_index >= limits.max_entries {
            return Err(DynamicError::EntryLimit {
                limit: limits.max_entries,
            });
        }
        let entry = entries::decode(chunk, entry_index)?;
        observed.push(entry);
        if entry.tag == DynamicTag::Null {
            let descriptors = descriptors::collect(&observed, &translator)?;
            return Ok(DynamicObservation::Present(Box::new(DynamicTable {
                source: source.clone(),
                source_range: range,
                program_index: index,
                entries: observed,
                descriptors,
            })));
        }
    }
    Err(DynamicError::MissingTerminator {
        entries: observed.len() as u64,
    })
}
