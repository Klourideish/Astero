//! PS5Rust startup scalar/memory contracts, adapted to checked scoped guest access.
pub use astero_hle::calls::memory::MAX_OPERATION_BYTES;
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::{CallResult, ProviderKey, ProviderKind, Registration},
};
/// Source numeric identities; library context is supplied explicitly by the composition policy.
pub const EXPORTS: &[(&str, u64)] = &[
    ("strcpy", 0x9226525c859df6f8),
    ("strncpy", 0xeac256896491baa9),
    ("strcat", 0x2ece2dcf38629aa4),
    ("strncat", 0x907838e6a3c2e9fd),
    ("strstr", 0xbe28b014c68d6a60),
    ("strcasecmp", 0x015ea2a4235ae11c),
    ("strncasecmp", 0xa57bdb0df721bba9),
    ("strdup", 0x83bcf3ccb0d81b0d),
    ("strspn", 0xfe453a6c1e0cffe9),
    ("strcspn", 0xab417ac92feb0a6b),
    ("strtok_r", 0x7a7a8f18b7e654d5),
    ("memcpy_s", 0x3452ecf9d44918d8),
    ("memset_s", 0x87c1b0a8f15bbba2),
    ("strcpy_s", 0xe576b600234409da),
    ("strncpy_s", 0x60dccd909cd8a848),
    ("strcat_s", 0x2be81c9c51492957),
    ("strncat_s", 0x342e0c481f814508),
    ("memalign", 0x5237f72b332f4662),
    ("aligned_alloc", 0xd81b6483c936e198),
    ("posix_memalign", 0x7154a4f72f1445b7),
    ("malloc_usable_size", 0x3437127dc619442f),
    ("operator delete aligned", 0x6d9c7e1454a59143),
    ("operator new", 0x7c99e9b955416ca9),
    ("operator new[]", 0x85d9b461f31aed34),
    ("operator delete", 0xcfe3fec429d62c19),
    ("operator delete[]", 0x30b5a5f7448558d1),
    ("operator delete sized", 0x9580f3055139999b),
    ("malloc", 0x8105fee060d08e93),
    ("free", 0xb4886caa3d2ab051),
    ("calloc", 0xd97e5a8058cac4c7),
    ("realloc", 0x63b689d6ec9d3cca),
    ("memcpy", 0x437541c425e1507b),
    ("memmove", 0xf8fe854461f82df0),
    ("memset", 0xf334c5bc120020df),
    ("memchr", 0xf2ef253f3504abe5),
    ("memcmp", 0x0df8af3c0ae1b9c8),
    ("strlen", 0x8f856258d1c4830c),
    ("strnlen", 0xe6336e6f0e2f9400),
    ("strcmp", 0x3af6f675224e02e1),
    ("strncmp", 0x69eb328eb1d55b2e),
    ("strchr", 0xa1be71016e259ffd),
    ("strrchr", 0xf720d63311057495),
];
fn count(n: u64) -> Result<usize, AccessError> {
    if n > MAX_OPERATION_BYTES {
        Err(AccessError::Limit)
    } else {
        usize::try_from(n).map_err(|_| AccessError::Limit)
    }
}
fn allocate(m: &mut dyn GuestMemory, n: u64) -> Result<u64, AccessError> {
    match m.allocate(n) {
        Ok(p) => Ok(p),
        Err(AccessError::Allocation) => Ok(0),
        Err(e) => Err(e),
    }
}
pub fn call(name: &str, a: [u64; 6], m: &mut dyn GuestMemory) -> Result<u64, AccessError> {
    let [x, y, z, ..] = a;
    match name {
        "memcpy_s" | "memset_s" | "strcpy_s" | "strncpy_s" | "strcat_s" | "strncat_s" => {
            super::checked::call(name, a, m)
        }
        "malloc_usable_size" => {
            if x == 0 {
                Ok(0)
            } else {
                m.usable_size(x)
            }
        }
        "memalign" | "aligned_alloc" => {
            if x == 0 || !x.is_power_of_two() || (name == "aligned_alloc" && !y.is_multiple_of(x)) {
                return Ok(0);
            }
            match m.allocate_aligned(y, x) {
                Ok(p) => Ok(p),
                Err(AccessError::Allocation) => Ok(0),
                Err(e) => Err(e),
            }
        }
        "posix_memalign" => {
            if y < 8 || !y.is_power_of_two() {
                return Ok(22);
            }
            if x == 0 || m.validate(x, 8, true).is_err() {
                return Ok(14);
            }
            let p = match m.allocate_aligned(z, y) {
                Ok(p) => p,
                Err(AccessError::Allocation) => return Ok(12),
                Err(e) => return Err(e),
            };
            if let Err(e) = m.write(x, &p.to_le_bytes()) {
                m.free(p)?;
                return Err(e);
            }
            Ok(0)
        }
        "malloc" => allocate(m, x),
        "operator new" | "operator new[]" => m.allocate(x.max(1)),
        "operator delete aligned"
        | "free"
        | "operator delete"
        | "operator delete[]"
        | "operator delete sized" => {
            m.free(x)?;
            Ok(0)
        }
        "calloc" => {
            let Some(n) = x.checked_mul(y) else {
                return Ok(0);
            };
            count(n)?;
            let p = allocate(m, n)?;
            if p != 0
                && let Err(e) = super::bulk::fill(m, p, 0, n)
            {
                m.free(p)?;
                return Err(e);
            }
            Ok(p)
        }
        "realloc" => {
            if x == 0 {
                return allocate(m, y);
            }
            let old = m.allocation_size(x)?;
            if y == 0 {
                m.free(x)?;
                return Ok(0);
            }
            let n = old.min(y);
            count(n)?;
            let p = allocate(m, y)?;
            if p == 0 {
                return Ok(0);
            }
            let result = super::bulk::copy(m, p, x, n, false);
            if let Err(e) = result {
                m.free(p)?;
                return Err(e);
            }
            m.free(x)?;
            Ok(p)
        }
        "memcpy" | "memmove" => {
            super::bulk::copy(m, x, y, z, name == "memmove")?;
            Ok(x)
        }
        "memset" => {
            super::bulk::fill(m, x, y as u8, z)?;
            Ok(x)
        }
        "memcmp" => super::bulk::compare(m, x, y, z),
        "memchr" => super::bulk::find(m, x, y as u8, z),
        "strlen" | "strnlen" | "strcmp" | "strncmp" | "strchr" | "strrchr" | "strcpy"
        | "strncpy" | "strcat" | "strncat" | "strstr" | "strcasecmp" | "strncasecmp" | "strdup"
        | "strtok_r" | "strspn" | "strcspn" => super::strings::call(name, a, m),
        _ => Err(AccessError::Range),
    }
}
pub fn registrations(library: &[u8], module: &[u8]) -> Vec<Registration> {
    registrations_with_errno(library, module, None)
}
pub fn registrations_with_errno(
    library: &[u8],
    module: &[u8],
    errno: Option<u64>,
) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|&(name, nid)| Registration {
            key: ProviderKey {
                nid,
                library: library.to_vec(),
                module: module.to_vec(),
            },
            kind: ProviderKind::HleImplementation,
            handler: Some(Box::new(move |f, m| match call(name, f.arguments, m) {
                Ok(v) => {
                    let allocation_failure = v == 0
                        && (matches!(
                            name,
                            "malloc" | "calloc" | "strdup" | "memalign" | "aligned_alloc"
                        ) || (name == "realloc" && f.arguments[1] != 0));
                    if allocation_failure && let Some(address) = errno {
                        let invalid_alignment = matches!(name, "memalign" | "aligned_alloc")
                            && (f.arguments[0] == 0
                                || !f.arguments[0].is_power_of_two()
                                || (name == "aligned_alloc"
                                    && !f.arguments[1].is_multiple_of(f.arguments[0])));
                        let code = if invalid_alignment { 22u32 } else { 12u32 };
                        if let Err(e) = m.write(address, &code.to_le_bytes()) {
                            return CallResult::AccessFailure(e);
                        }
                    }
                    f.rax = v;
                    CallResult::Returned
                }
                Err(e) => CallResult::AccessFailure(e),
            })),
        })
        .collect()
}
