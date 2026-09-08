use super::super::{
    decoding::{read, u16_at, u32_at, u64_at},
    error::{ElfError, Structure},
    identification::{self, Identification},
};
use crate::artifact::SourceArtifact;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Elf64Header {
    pub identification: Identification,
    pub object_type: u16,
    pub machine: u16,
    pub version: u32,
    pub entry: u64,
    pub program_offset: u64,
    pub section_offset: u64,
    pub flags: u32,
    pub header_size: u16,
    pub program_entry_size: u16,
    pub program_count: u16,
    pub section_entry_size: u16,
    pub section_count: u16,
    pub section_name_index: u16,
}
pub(in crate::elf) fn decode(source: &SourceArtifact) -> Result<Elf64Header, ElfError> {
    let identification = identification::decode(source)?;
    let b = read(source, 0, 64, Structure::Header)?;
    let h = Elf64Header {
        identification,
        object_type: u16_at(b, 16)?,
        machine: u16_at(b, 18)?,
        version: u32_at(b, 20)?,
        entry: u64_at(b, 24)?,
        program_offset: u64_at(b, 32)?,
        section_offset: u64_at(b, 40)?,
        flags: u32_at(b, 48)?,
        header_size: u16_at(b, 52)?,
        program_entry_size: u16_at(b, 54)?,
        program_count: u16_at(b, 56)?,
        section_entry_size: u16_at(b, 58)?,
        section_count: u16_at(b, 60)?,
        section_name_index: u16_at(b, 62)?,
    };
    if h.version != 1 {
        return Err(ElfError::UnsupportedVersion {
            structure: Structure::Header,
            version: h.version,
        });
    }
    if h.header_size != 64 {
        return Err(ElfError::UnsupportedHeaderSize(h.header_size));
    }
    if h.program_count == 0xffff {
        return Err(ElfError::UnsupportedExtendedProgramCount);
    }
    if h.program_count == 0 {
        if h.program_offset != 0 {
            return Err(ElfError::InvalidTableOffset {
                offset: h.program_offset,
                count: 0,
            });
        }
        // With no entries, both absent and ordinary entry-size metadata are supported.
        if !matches!(h.program_entry_size, 0 | 56) {
            return Err(ElfError::UnsupportedProgramHeaderSize(h.program_entry_size));
        }
    } else {
        if h.program_entry_size != 56 {
            return Err(ElfError::UnsupportedProgramHeaderSize(h.program_entry_size));
        }
        if h.program_offset < 64 {
            return Err(ElfError::InvalidTableOffset {
                offset: h.program_offset,
                count: h.program_count,
            });
        }
        let size = u64::from(h.program_count).checked_mul(u64::from(h.program_entry_size));
        if size
            .and_then(|size| h.program_offset.checked_add(size))
            .is_none()
        {
            return Err(ElfError::TableOverflow {
                offset: h.program_offset,
                count: h.program_count,
                entry_size: h.program_entry_size,
            });
        }
        read(
            source,
            h.program_offset,
            u64::from(h.program_count) * 56,
            Structure::ProgramHeaderTable,
        )?;
    }
    Ok(h)
}
