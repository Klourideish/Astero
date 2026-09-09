use crate::elf::{decoding::u64_at, dynamic::relocations::error::RelocationError};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelaRecord {
    pub offset: u64,
    pub info: u64,
    pub addend: i64,
}
impl RelaRecord {
    pub fn symbol_index(&self) -> u32 {
        (self.info >> 32) as u32
    }
    pub fn relocation_type(&self) -> u32 {
        self.info as u32
    }
}
pub(in crate::elf::dynamic::relocations) fn decode(
    bytes: &[u8],
    index: u64,
) -> Result<RelaRecord, RelocationError> {
    if bytes.len() != 24 {
        return Err(RelocationError::TruncatedEntry {
            index,
            actual: bytes.len() as u64,
        });
    }
    let offset = u64_at(bytes, 0).map_err(RelocationError::Decode)?;
    let info = u64_at(bytes, 8).map_err(RelocationError::Decode)?;
    let addend = i64::from_le_bytes(
        u64_at(bytes, 16)
            .map_err(RelocationError::Decode)?
            .to_le_bytes(),
    );
    Ok(RelaRecord {
        offset,
        info,
        addend,
    })
}
