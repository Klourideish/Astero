//! Exact repository-owned bytes; no external binaries or parser dependency.
use crate::{
    artifact::SourceArtifact,
    modules::{ModuleId, ModuleMetadata},
};
pub fn context() -> Option<ModuleMetadata> {
    Some(ModuleMetadata {
        id: ModuleId(4),
        name: "generated ELF".into(),
    })
}
pub fn source(bytes: Vec<u8>) -> SourceArtifact {
    SourceArtifact::new(bytes, Some("generated fixture".into())).unwrap()
}
pub fn put16(b: &mut [u8], at: usize, n: u16) {
    b[at..at + 2].copy_from_slice(&n.to_le_bytes());
}
pub fn put32(b: &mut [u8], at: usize, n: u32) {
    b[at..at + 4].copy_from_slice(&n.to_le_bytes());
}
pub fn put64(b: &mut [u8], at: usize, n: u64) {
    b[at..at + 8].copy_from_slice(&n.to_le_bytes());
}
pub fn header(count: u16) -> Vec<u8> {
    let mut b = vec![0; 64 + usize::from(count) * 56];
    b[..9].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0]);
    put16(&mut b, 16, 2);
    put16(&mut b, 18, 62);
    put32(&mut b, 20, 1);
    put64(&mut b, 24, 0x1000);
    put64(&mut b, 32, if count == 0 { 0 } else { 64 });
    put16(&mut b, 52, 64);
    put16(&mut b, 54, 56);
    put16(&mut b, 56, count);
    b
}
pub fn segment(
    b: &mut [u8],
    index: usize,
    flags: u32,
    address: u64,
    offset: u64,
    file_size: u64,
    memory_size: u64,
) {
    let at = 64 + index * 56;
    put32(b, at, 1);
    put32(b, at + 4, flags);
    put64(b, at + 8, offset);
    put64(b, at + 16, address);
    put64(b, at + 24, 0xfeed);
    put64(b, at + 32, file_size);
    put64(b, at + 40, memory_size);
    put64(b, at + 48, 16);
}
pub fn executable() -> Vec<u8> {
    let mut b = header(1);
    b.resize(0x110, 0xa5);
    segment(&mut b, 0, 5, 0x1000, 0x100, 16, 16);
    b
}
pub fn multiple() -> Vec<u8> {
    let mut b = header(3);
    b.resize(0x128, 0x5a);
    segment(&mut b, 0, 6, 0x2000, 0x120, 8, 32);
    // Entry 1 is PT_NULL, with deliberately meaningless unused values.
    put64(&mut b, 64 + 56 + 8, u64::MAX);
    segment(&mut b, 2, 5, 0x1000, 0x100, 16, 16);
    b
}
