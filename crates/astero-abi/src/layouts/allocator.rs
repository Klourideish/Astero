//! Experimental managed-size version 1, corroborated by the M46 caller.
//! Four size lanes: peak arena, current arena, peak used, current used.
pub const MANAGED_SIZE_V1_HEADER: u32 = 0x10028;
pub const MANAGED_SIZE_V1_BYTES: usize = 40;
pub fn managed_size_v1(lanes: [u64; 4]) -> [u8; MANAGED_SIZE_V1_BYTES] {
    let mut b = [0; MANAGED_SIZE_V1_BYTES];
    b[..4].copy_from_slice(&MANAGED_SIZE_V1_HEADER.to_le_bytes());
    for (i, v) in lanes.into_iter().enumerate() {
        b[8 + i * 8..16 + i * 8].copy_from_slice(&v.to_le_bytes());
    }
    b
}
