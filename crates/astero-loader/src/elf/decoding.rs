//! Private bounded source reads and explicit little-endian scalar decoding.
use super::error::{ElfError, Structure};
use crate::artifact::SourceArtifact;
pub(super) fn read(
    source: &SourceArtifact,
    offset: u64,
    size: u64,
    structure: Structure,
) -> Result<&[u8], ElfError> {
    let range = source
        .checked_range(offset, size)
        .map_err(|error| ElfError::Source { structure, error })?;
    source
        .read(&range)
        .map_err(|error| ElfError::Source { structure, error })
}
fn field<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N], ElfError> {
    offset
        .checked_add(N)
        .and_then(|end| bytes.get(offset..end))
        .and_then(|s| s.try_into().ok())
        .ok_or(ElfError::MalformedField { offset, width: N })
}
pub(super) fn u16_at(bytes: &[u8], offset: usize) -> Result<u16, ElfError> {
    Ok(u16::from_le_bytes(field(bytes, offset)?))
}
pub(super) fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, ElfError> {
    Ok(u32::from_le_bytes(field(bytes, offset)?))
}
pub(super) fn u64_at(bytes: &[u8], offset: usize) -> Result<u64, ElfError> {
    Ok(u64::from_le_bytes(field(bytes, offset)?))
}
