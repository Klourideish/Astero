use super::ObservationLimits;
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::{
        address_translation::AddressTranslator,
        dynamic::{
            entries::{self, DynamicEntry},
            error::DynamicError,
            tags::DynamicTag,
        },
        program_headers::ProgramHeader,
    },
    metadata::VirtualAddress,
};
/// Shared raw traversal, with no descriptor pairing or pointer dereferences.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawTable {
    pub(in crate::elf::dynamic) source_range: BoundSourceRange,
    pub(in crate::elf::dynamic) program_index: usize,
    pub(in crate::elf::dynamic) entries: Vec<DynamicEntry>,
}
impl RawTable {
    pub fn source_range(&self) -> BoundSourceRange {
        self.source_range
    }
    pub fn program_index(&self) -> usize {
        self.program_index
    }
    /// Includes DT_NULL. Any trailing declared bytes remain uninterpreted.
    pub fn entries(&self) -> &[DynamicEntry] {
        &self.entries
    }
}
pub(in crate::elf::dynamic) fn observe_raw(
    source: &SourceArtifact,
    programs: &[ProgramHeader],
    limits: ObservationLimits,
) -> Result<Option<RawTable>, DynamicError> {
    let mut found = None;
    for (index, p) in programs.iter().enumerate() {
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
        return Ok(None);
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
    let range = source
        .checked_range(p.file_offset, p.file_size)
        .map_err(|error| DynamicError::Source {
            segment: index,
            error,
        })?;
    let translator =
        AddressTranslator::from_program_headers(source, programs.iter().cloned().map(Ok))
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
        observed
            .try_reserve(1)
            .map_err(|_| DynamicError::Allocation {
                entries: entry_index + 1,
            })?;
        observed.push(entry);
        if entry.tag == DynamicTag::Null {
            return Ok(Some(RawTable {
                source_range: range,
                program_index: index,
                entries: observed,
            }));
        }
    }
    Err(DynamicError::MissingTerminator {
        entries: observed.len() as u64,
    })
}
