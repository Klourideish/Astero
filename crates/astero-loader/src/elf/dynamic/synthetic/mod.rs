//! Shared generated M5 byte layout; no real binary input or parser implementation.
pub fn put16(b: &mut [u8], at: usize, value: u16) {
    b[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub fn put32(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub fn put64(b: &mut [u8], at: usize, value: u64) {
    b[at..at + 8].copy_from_slice(&value.to_le_bytes());
}
pub fn program(
    b: &mut [u8],
    index: usize,
    kind: u32,
    offset: u64,
    address: u64,
    file: u64,
    memory: u64,
) {
    let at = 64 + 56 * index;
    put32(b, at, kind);
    put32(b, at + 4, 6);
    put64(b, at + 8, offset);
    put64(b, at + 16, address);
    put64(b, at + 32, file);
    put64(b, at + 40, memory);
    put64(b, at + 48, 1);
}
pub fn image(entries: &[(i64, u64)]) -> Vec<u8> {
    let mut b = vec![0; 0x600];
    b[..9].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0]);
    put16(&mut b, 16, 3);
    put16(&mut b, 18, 62);
    put32(&mut b, 20, 1);
    put64(&mut b, 32, 64);
    put16(&mut b, 52, 64);
    put16(&mut b, 54, 56);
    put16(&mut b, 56, 2);
    program(&mut b, 0, 1, 0x100, 0x1100, 0x500, 0x600);
    let size = entries.len() as u64 * 16;
    program(&mut b, 1, 2, 0x200, 0x1200, size, size);
    for (i, &(tag, value)) in entries.iter().enumerate() {
        let at = 0x200 + 16 * i;
        b[at..at + 8].copy_from_slice(&tag.to_le_bytes());
        put64(&mut b, at + 8, value);
    }
    b
}
