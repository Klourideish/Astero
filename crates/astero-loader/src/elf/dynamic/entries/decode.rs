use super::super::{error::DynamicError, tags::DynamicTag};
use crate::elf::decoding::u64_at;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicEntry {
    pub index: u64,
    pub tag: DynamicTag,
    pub value: u64,
}
pub(in crate::elf::dynamic) fn decode(
    bytes: &[u8],
    index: u64,
) -> Result<DynamicEntry, DynamicError> {
    if bytes.len() < 16 {
        return Err(DynamicError::TruncatedEntry {
            index,
            remaining: bytes.len() as u64,
        });
    }
    let tag = u64_at(bytes, 0).map_err(DynamicError::Header)?;
    Ok(DynamicEntry {
        index,
        tag: DynamicTag::from_raw(i64::from_le_bytes(tag.to_le_bytes())),
        value: u64_at(bytes, 8).map_err(DynamicError::Header)?,
    })
}
