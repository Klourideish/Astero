//! PS5Rust startup scalar/memory contracts, adapted to checked scoped guest access.
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::{CallResult, ProviderKey, ProviderKind, Registration},
};
pub const MAX_OPERATION_BYTES: u64 = 1024 * 1024;
/// Source numeric identities; library context is supplied explicitly by the composition policy.
pub const EXPORTS: &[(&str, u64)] = &[
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
fn byte(m: &dyn GuestMemory, a: u64, i: u64) -> Result<u8, AccessError> {
    let v = m.read(a.checked_add(i).ok_or(AccessError::Range)?, 1)?;
    v.first().copied().ok_or(AccessError::Range)
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
        "malloc" => allocate(m, x),
        "operator new" | "operator new[]" => m.allocate(x.max(1)),
        "free" | "operator delete" | "operator delete[]" | "operator delete sized" => {
            m.free(x)?;
            Ok(0)
        }
        "calloc" => {
            let Some(n) = x.checked_mul(y) else {
                return Ok(0);
            };
            count(n)?;
            let p = allocate(m, n)?;
            if p != 0 {
                let b = vec![0; count(n)?];
                if let Err(e) = m.write(p, &b) {
                    m.free(p)?;
                    return Err(e);
                }
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
            let result = m.read(x, n).and_then(|b| m.write(p, &b));
            if let Err(e) = result {
                m.free(p)?;
                return Err(e);
            }
            m.free(x)?;
            Ok(p)
        }
        "memcpy" | "memmove" => {
            count(z)?;
            let b = m.read(y, z)?;
            m.write(x, &b)?;
            Ok(x)
        }
        "memset" => {
            let n = count(z)?;
            m.write(x, &vec![y as u8; n])?;
            Ok(x)
        }
        "memcmp" => {
            count(z)?;
            let l = m.read(x, z)?;
            let r = m.read(y, z)?;
            Ok(l.iter()
                .zip(r.iter())
                .find_map(|(a, b)| (*a != *b).then_some((*a as i32 - *b as i32) as u32 as u64))
                .unwrap_or(0))
        }
        "strlen" | "strnlen" => {
            let maximum = if name == "strnlen" {
                y
            } else {
                MAX_OPERATION_BYTES
            };
            count(maximum)?;
            for i in 0..maximum {
                if byte(m, x, i)? == 0 {
                    return Ok(i);
                }
            }
            if name == "strnlen" {
                Ok(maximum)
            } else {
                Err(AccessError::Limit)
            }
        }
        "strcmp" | "strncmp" => {
            let maximum = if name == "strncmp" {
                z
            } else {
                MAX_OPERATION_BYTES
            };
            count(maximum)?;
            for i in 0..maximum {
                let l = byte(m, x, i)?;
                let r = byte(m, y, i)?;
                if l != r {
                    return Ok((l as i32 - r as i32) as u32 as u64);
                }
                if l == 0 {
                    return Ok(0);
                }
            }
            if name == "strncmp" {
                Ok(0)
            } else {
                Err(AccessError::Limit)
            }
        }
        "strchr" | "strrchr" => {
            let mut found = 0;
            for i in 0..MAX_OPERATION_BYTES {
                let b = byte(m, x, i)?;
                if b == y as u8 {
                    found = x.checked_add(i).ok_or(AccessError::Range)?;
                    if name == "strchr" {
                        return Ok(found);
                    }
                }
                if b == 0 {
                    return Ok(found);
                }
            }
            Err(AccessError::Limit)
        }
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
                    if v == 0
                        && (name == "malloc"
                            || name == "calloc"
                            || (name == "realloc" && f.arguments[1] != 0))
                        && let Some(address) = errno
                        && let Err(e) = m.write(address, &12u32.to_le_bytes())
                    {
                        return CallResult::AccessFailure(e);
                    }
                    f.rax = v;
                    CallResult::Returned
                }
                Err(e) => CallResult::AccessFailure(e),
            })),
        })
        .collect()
}
