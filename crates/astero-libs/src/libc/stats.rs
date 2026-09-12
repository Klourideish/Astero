//! Bounded version-one managed-size query. Layout corroborated by the current caller;
//! semantic mapping is experimental where PS5Rust only supplied a stub.
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
pub const NID: u64 = 0x2AE3AE0F9F21AA7E;
pub fn query(m: &mut dyn GuestMemory, p: u64) -> Result<u64, AccessError> {
    if p == 0 {
        return Err(AccessError::Range);
    }
    m.validate(p, 40, true)?;
    let header = m.read(p, 4)?;
    if header.as_slice() != 0x10028u32.to_le_bytes() {
        return Ok(22);
    }
    let s = m.heap_stats()?;
    let bytes = astero_abi::layouts::allocator::managed_size_v1([
        s.arena_bytes,
        s.arena_bytes,
        s.peak_bytes,
        s.live_bytes,
    ]);
    m.write(p, &bytes)?;
    Ok(0)
}
pub fn registration() -> Registration {
    Registration {
        key: ProviderKey {
            nid: NID,
            library: b"libc".to_vec(),
            module: b"libc".to_vec(),
        },
        kind: ProviderKind::HleImplementation,
        handler: Some(Box::new(|f, m| match query(m, f.arguments[0]) {
            Ok(v) => {
                f.rax = v;
                CallResult::Returned
            }
            Err(e) => CallResult::AccessFailure(e),
        })),
    }
}
