//! Prototype-corroborated 120-byte FreeBSD-derived stat prefix. Metadata policy is not hardware truth.
pub fn encode(directory: bool, size: u64) -> [u8; 120] {
    let mut b = [0; 120];
    let mode: u16 = if directory { 0x41ff } else { 0x81ff };
    b[8..10].copy_from_slice(&mode.to_le_bytes());
    b[10..12].copy_from_slice(&1u16.to_le_bytes());
    b[72..80].copy_from_slice(&size.to_le_bytes());
    let blocks = if directory { 128 } else { size.div_ceil(512) };
    b[80..88].copy_from_slice(&blocks.to_le_bytes());
    let block_size: u32 = if directory { 65536 } else { 512 };
    b[88..92].copy_from_slice(&block_size.to_le_bytes());
    b
}
