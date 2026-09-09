//! M23 synthetic capability fixture, reusing M21; no real input.
use crate::elf::dynamic::{
    symbol_table::synthetic::{Variant, image_with_symbols},
    synthetic::{image, put64},
};
pub fn linkage_image() -> Vec<u8> {
    let old = image_with_symbols(Variant::SysV);
    let mut b = image(&[
        (6, 0x1400),
        (11, 24),
        (5, 0x1500),
        (10, 10),
        (4, 0x1300),
        (7, 0x1540),
        (8, 72),
        (9, 24),
        (23, 0x1588),
        (2, 24),
        (20, 7),
        (1, 1),
        (1, 1),
        (-42, u64::MAX),
        (0, 0),
    ]);
    b[0x300..].copy_from_slice(&old[0x300..]);
    for (i, (symbol, kind, addend)) in [
        (0, 8, -16i64),
        (1, 0xffffeeee, 19),
        (2, 1, i64::MIN),
        (1, 7, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let at = 0x540 + i * 24;
        put64(&mut b, at, 0x9000 + i as u64 * 8);
        put64(&mut b, at + 8, (symbol << 32) | kind);
        put64(&mut b, at + 16, addend as u64);
    }
    b
}
