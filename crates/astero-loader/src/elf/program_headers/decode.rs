use super::super::{
    decoding::{read, u32_at, u64_at},
    error::{ElfError, Structure},
    header::Elf64Header,
};
use crate::artifact::SourceArtifact;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramHeader {
    pub kind: u32,
    pub flags: u32,
    pub file_offset: u64,
    pub virtual_address: u64,
    pub physical_address: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub alignment: u64,
}
pub struct ProgramHeaders<'a> {
    source: &'a SourceArtifact,
    header: &'a Elf64Header,
    next: u16,
}
impl<'a> ProgramHeaders<'a> {
    // Only the ELF inspector can provide this already-validated header/source pair.
    pub(in crate::elf) fn new(source: &'a SourceArtifact, header: &'a Elf64Header) -> Self {
        Self {
            source,
            header,
            next: 0,
        }
    }
}
impl Iterator for ProgramHeaders<'_> {
    type Item = Result<ProgramHeader, ElfError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.header.program_count {
            return None;
        }
        let index = self.next;
        self.next += 1;
        // Whole-table proof established by header decoding; count is at most 65534.
        let offset = self.header.program_offset + u64::from(index) * 56;
        Some(decode(self.source, offset, index))
    }
}
fn decode(source: &SourceArtifact, offset: u64, index: u16) -> Result<ProgramHeader, ElfError> {
    let b = read(source, offset, 56, Structure::ProgramHeader { index })?;
    Ok(ProgramHeader {
        kind: u32_at(b, 0)?,
        flags: u32_at(b, 4)?,
        file_offset: u64_at(b, 8)?,
        virtual_address: u64_at(b, 16)?,
        physical_address: u64_at(b, 24)?,
        file_size: u64_at(b, 32)?,
        memory_size: u64_at(b, 40)?,
        alignment: u64_at(b, 48)?,
    })
}
