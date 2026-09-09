//! Small generated symbol fixtures, using the existing hash and ELF fixture layouts.
use crate::elf::dynamic::{
    hash::synthetic::{self, Variant as HashVariant},
    synthetic::{image, put16, put32, put64},
};
#[derive(Clone, Copy, Debug)]
pub enum Variant {
    SysV,
    Gnu,
    Both,
    Raw,
    Empty,
    Unnamed,
    Unknown,
    BadName,
    LowerBound,
    None,
}
pub fn image_with_symbols(variant: Variant) -> Vec<u8> {
    let hash = match variant {
        Variant::Gnu => HashVariant::Gnu,
        Variant::Both => HashVariant::Both,
        Variant::LowerBound => HashVariant::LowerBound,
        Variant::None => HashVariant::None,
        _ => HashVariant::SysV,
    };
    let old = synthetic::image_with_hashes(hash);
    let mut entries = vec![(6, 0x1400), (11, 24), (5, 0x1500), (10, 10)];
    if matches!(hash, HashVariant::SysV | HashVariant::Both) {
        entries.push((4, 0x1300));
    }
    if matches!(
        hash,
        HashVariant::Gnu | HashVariant::Both | HashVariant::LowerBound
    ) {
        entries.push((0x6ffffef5, 0x1340));
    }
    entries.extend([(-42, u64::MAX), (0, 0)]);
    let mut b = image(&entries);
    b[0x300..].copy_from_slice(&old[0x300..]);
    b[0x400..0x448].fill(0);
    b[0x500..0x50a].copy_from_slice(b"\0alpha\0\0\xff\0");
    put32(&mut b, 0x418, 1);
    b[0x41c] = 0x12;
    put32(
        &mut b,
        0x430,
        match variant {
            Variant::Raw => 8,
            Variant::Empty => 7,
            Variant::Unnamed => 0,
            Variant::BadName => 10,
            _ => 1,
        },
    );
    b[0x434] = 0x21;
    put16(&mut b, 0x436, 1);
    put64(&mut b, 0x438, 0x1234);
    put64(&mut b, 0x440, 8);
    if matches!(variant, Variant::Unknown) {
        b[0x434] = 0xef;
        b[0x435] = 0xfd;
        put16(&mut b, 0x436, 0xff20);
        put64(&mut b, 0x438, u64::MAX);
    }
    b
}
