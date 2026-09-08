use super::TranslationError;
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::ElfInspection,
    metadata::VirtualAddress,
};
struct Mapping {
    index: usize,
    start: u64,
    memory_end: u64,
    file_end: u64,
    source: BoundSourceRange,
}
/// Validates load extents once, then translates against the immutable ELF-declared address model.
/// Does not grant admission or certify alignment/permissions. Indices refer to program headers.
pub struct AddressTranslator<'a> {
    source: &'a SourceArtifact,
    mappings: Vec<Mapping>,
}
impl<'a> AddressTranslator<'a> {
    pub fn new(elf: &'a ElfInspection) -> Result<Self, TranslationError> {
        let source = elf.artifact().source();
        let mut mappings = Vec::new();
        for (index, p) in elf.program_headers().enumerate() {
            let p = p.map_err(TranslationError::Header)?;
            if p.kind != 1 {
                continue;
            }
            if p.file_size > p.memory_size {
                return Err(TranslationError::InvalidLoadSize {
                    segment: index,
                    file_size: p.file_size,
                    memory_size: p.memory_size,
                });
            }
            let memory_end = p.virtual_address.checked_add(p.memory_size).ok_or(
                TranslationError::LoadRangeOverflow {
                    segment: index,
                    address: p.virtual_address,
                    size: p.memory_size,
                },
            )?;
            let range = source
                .checked_range(p.file_offset, p.file_size)
                .map_err(|error| TranslationError::Source {
                    segment: index,
                    error,
                })?;
            mappings.push(Mapping {
                index,
                start: p.virtual_address,
                memory_end,
                file_end: p.virtual_address + p.file_size,
                source: range,
            });
        }
        Ok(Self { source, mappings })
    }
    /// Half-open positive ranges must stay within one unambiguous PT_LOAD and its file prefix.
    /// Empty requests use closed endpoints; shared endpoints are conservatively ambiguous.
    pub fn translate(
        &self,
        address: VirtualAddress,
        size: u64,
    ) -> Result<BoundSourceRange, TranslationError> {
        let address = address.0;
        let end = address
            .checked_add(size)
            .ok_or(TranslationError::RequestOverflow { address, size })?;
        let mut owner: Option<&Mapping> = None;
        for m in &self.mappings {
            if address >= m.start
                && (address < m.memory_end || (size == 0 && address == m.memory_end))
            {
                if let Some(first) = owner {
                    return Err(TranslationError::Ambiguous {
                        first: first.index,
                        second: m.index,
                        address,
                        size,
                    });
                }
                owner = Some(m);
            }
        }
        let m = owner.ok_or(TranslationError::Unmapped { address, size })?;
        // Detect a conflicting mapping starting partway through the requested range too.
        for other in &self.mappings {
            if other.index != m.index
                && size != 0
                && other.start < other.memory_end
                && other.start < end.min(m.memory_end)
                && address.max(m.start) < other.memory_end
            {
                return Err(TranslationError::Ambiguous {
                    first: m.index,
                    second: other.index,
                    address,
                    size,
                });
            }
        }
        if end > m.memory_end {
            return Err(TranslationError::CrossesMapping {
                segment: m.index,
                address,
                size,
            });
        }
        if address > m.file_end || (size != 0 && address == m.file_end) {
            return Err(TranslationError::ZeroFill {
                segment: m.index,
                address,
                size,
            });
        }
        if end > m.file_end {
            return Err(TranslationError::CrossesSourceBoundary {
                segment: m.index,
                address,
                size,
            });
        }
        // Both sums are bounded by the fully checked mapping source prefix.
        let offset = m.source.extent().offset.0 + (address - m.start);
        self.source
            .checked_range(offset, size)
            .map_err(|error| TranslationError::Source {
                segment: m.index,
                error,
            })
    }
}
