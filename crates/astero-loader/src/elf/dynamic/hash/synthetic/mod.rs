//! Small deterministic ordinary hash fixtures; invalid symbol bytes deliberately remain unread.
use crate::elf::dynamic::synthetic::{image, put32, put64};
#[derive(Clone, Copy, Debug)]
pub enum Variant {
    SysV,
    Gnu,
    Both,
    Conflict,
    None,
    LowerBound,
    Truncated,
}
pub fn image_with_hashes(variant: Variant) -> Vec<u8> {
    let mut entries = vec![(6, 0x1400), (11, 24)];
    if matches!(
        variant,
        Variant::SysV | Variant::Both | Variant::Conflict | Variant::Truncated
    ) {
        entries.push((4, 0x1300));
    }
    if matches!(
        variant,
        Variant::Gnu | Variant::Both | Variant::Conflict | Variant::LowerBound
    ) {
        entries.push((0x6ffffef5, 0x1340));
    }
    // These unrelated descriptors are intentionally unusable and must not be interpreted by M20.
    entries.extend([(1, u64::MAX), (7, u64::MAX), (-42, 99), (0, 0)]);
    let mut bytes = image(&entries);
    for (i, v) in [1, 3, 1, 0, 2, 0].into_iter().enumerate() {
        put32(&mut bytes, 0x300 + i * 4, v);
    }
    // SysV chain 1 -> 2 -> 0. GNU contiguous suffix 1..3, low-bit termination.
    for (i, v) in [1, 1, 1, 5].into_iter().enumerate() {
        put32(&mut bytes, 0x340 + i * 4, v);
    }
    put64(&mut bytes, 0x350, 0x12345678);
    put32(
        &mut bytes,
        0x358,
        if matches!(variant, Variant::LowerBound) {
            0
        } else {
            1
        },
    );
    put32(
        &mut bytes,
        0x35c,
        if matches!(variant, Variant::Conflict) {
            1
        } else {
            2
        },
    );
    put32(&mut bytes, 0x360, 3);
    bytes[0x400..0x448].fill(0xff); // Not valid null/name metadata; not read by extent validation.
    if matches!(variant, Variant::Truncated) {
        bytes.truncate(0x317);
    }
    bytes
}
