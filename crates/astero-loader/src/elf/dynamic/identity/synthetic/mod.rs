//! Small M24 fixture: reuse M23 symbols/relocations and move only strings/dynamic records.
use crate::elf::dynamic::{
    candidates::workload::synthetic::linkage_image,
    synthetic::{program, put32, put64},
};
pub fn identity_image() -> Vec<u8> {
    let mut b = linkage_image();
    b.resize(0xa00, 0);
    b[0x600..0x6e0].copy_from_slice(&linkage_image()[0x200..0x2e0]);
    let names = b"\0H2e8t5ScQGc#B#A\0RpQJJVKTiFM#B#A\0sample\0library\0";
    b[0x900..0x900 + names.len()].copy_from_slice(names);
    put64(&mut b, 0x628, 0x1900);
    put64(&mut b, 0x638, names.len() as u64);
    put32(&mut b, 0x430, 17);
    put64(&mut b, 0x6b8, 33);
    put64(&mut b, 0x6c8, 33);
    let entries = [
        (0x61000043u64, (0x101 << 32) | 33),
        (0x61000049, (1 << 48) | (1 << 32) | 40),
        (0x6100ffff, u64::MAX),
        (0, 0),
    ];
    for (i, (tag, value)) in entries.into_iter().enumerate() {
        put64(&mut b, 0x6e0 + i * 16, tag);
        put64(&mut b, 0x6e8 + i * 16, value);
    }
    program(&mut b, 0, 1, 0x100, 0x1100, 0x900, 0x900);
    program(&mut b, 1, 2, 0x600, 0x1600, 18 * 16, 18 * 16);
    b
}
