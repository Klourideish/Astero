//! Direct-memory offset and view adapters; exact libkernel identities.
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::objects::memory::{DIRECT_SIZE, Error, MemoryResources};
use std::sync::Arc;
pub const EXPORTS: &[(&str, u64, &str)] = &[
    ("sceKernelGetDirectMemorySize", 0xA4EF7A4F0CCE9B91, "size"),
    ("sceKernelAllocateDirectMemory", 0xAD35F0EB9C662C80, "alloc"),
    (
        "sceKernelAllocateMainDirectMemory",
        0x07EBDCD803B666B7,
        "main",
    ),
    (
        "sceKernelAvailableDirectMemorySize",
        0x0B47FB4C971B7DA7,
        "available",
    ),
    (
        "sceKernelDirectMemoryQuery",
        0x047A2E2D0CE1D17D,
        "directquery",
    ),
    ("sceKernelMapDirectMemory", 0x2FF4372C48C86E00, "map"),
    (
        "sceKernelReleaseDirectMemory",
        0x301B88B6F6DAEB3F,
        "release",
    ),
    (
        "sceKernelCheckedReleaseDirectMemory",
        0x8705523C29A9E6D3,
        "release",
    ),
    ("sceKernelVirtualQuery", 0xAD58D1BC72745FA7, "query"),
    ("getpagesize", 0x93E017AAEDBF7817, "page"),
    ("munmap", 0x52A0C68D7039C943, "unmap"),
    ("mprotect", 0x61039FC4BE107DE5, "protect"),
    ("sceKernelMapNamedDirectMemory", 0x35C6965317CC3484, "map"),
    ("sceKernelMapDirectMemory2", 0x0504278A8963F6D4, "map2"),
    ("sceKernelSetVirtualRangeName", 0x0C6306DC9B21AD95, "name"),
    (
        "sceKernelReserveVirtualRange",
        0xEE8C6FDCF3C2BA6A,
        "reserve",
    ),
    ("sceKernelMunmap", 0x71091EF54B8140E9, "sceunmap"),
    ("sceKernelMprotect", 0x03A66B3A6F21CB46, "sceprotect"),
    ("sceKernelMprotect", 0xBD23009B77316136, "sceprotect"),
];
fn out(m: &dyn GuestMemory, p: u64, n: u64) -> Result<(), AccessError> {
    if p == 0 {
        return Err(AccessError::Range);
    }
    m.validate(p, n, true)
}
enum Failure {
    Memory(AccessError),
    Resource(Error),
}
impl From<AccessError> for Failure {
    fn from(e: AccessError) -> Self {
        Self::Memory(e)
    }
}
impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Self::Resource(e)
    }
}
fn run(
    op: &str,
    a: [u64; 6],
    m: &mut dyn GuestMemory,
    s: &MemoryResources,
) -> Result<u64, Failure> {
    match op {
        "reserve" => {
            out(m, a[0], 8)?;
            let b = m.read(a[0], 8)?;
            let hint = u64::from_le_bytes(b.try_into().expect("eight bytes"));
            let address = s.reserve(hint, a[1], a[2] as u32, a[3])?;
            if let Err(e) = m.write(a[0], &address.to_le_bytes()) {
                let _ = s.unmap(address, a[1]);
                return Err(e.into());
            }
            Ok(0)
        }
        "size" => Ok(DIRECT_SIZE),
        "page" => Ok(4096),
        "alloc" | "main" => {
            let (start, end, size, align, kind, p) = if op == "main" {
                (0, DIRECT_SIZE, a[0], a[1], a[2], a[3])
            } else {
                (a[0], a[1], a[2], a[3], a[4], a[5])
            };
            out(m, p, 8)?;
            let offset = s.allocate(start, end, size, align, kind as i32)?;
            if let Err(e) = m.write(p, &offset.to_le_bytes()) {
                let _ = s.release(offset, size);
                return Err(e.into());
            }
            Ok(0)
        }
        "available" => {
            out(m, a[3], 8)?;
            out(m, a[4], 8)?;
            let (offset, size) = s.available(a[0], a[1], a[2])?;
            m.write(a[3], &offset.to_le_bytes())?;
            m.write(a[4], &size.to_le_bytes())?;
            Ok(0)
        }
        "map" => {
            out(m, a[0], 8)?;
            let b = m.read(a[0], 8)?;
            let hint = u64::from_le_bytes(b.try_into().expect("eight bytes"));
            let address = s.map(hint, a[1], a[2] as u32, a[3] as u32, a[4], a[5])?;
            if let Err(e) = m.write(a[0], &address.to_le_bytes()) {
                let _ = s.unmap(address, a[1]);
                return Err(e.into());
            }
            Ok(0)
        }
        "unmap" | "sceunmap" => {
            s.unmap(a[0], a[1])?;
            Ok(0)
        }
        "protect" | "sceprotect" => {
            s.protect(a[0], a[1], a[2] as u32)?;
            Ok(0)
        }
        "name" => {
            let mut name = Vec::new();
            if a[2] == 0 {
                return Err(AccessError::Range.into());
            }
            for i in 0..33 {
                let c = m.read(a[2].checked_add(i).ok_or(AccessError::Range)?, 1)?[0];
                if c == 0 {
                    s.name(a[0], a[1], &name)?;
                    return Ok(0);
                }
                name.push(c);
            }
            Err(Error::Invalid.into())
        }
        "release" => {
            s.release(a[0], a[1])?;
            Ok(0)
        }
        "directquery" => {
            if a[3] < 24 {
                return Err(Error::Invalid.into());
            }
            out(m, a[2], 24)?;
            let r = s.query_direct(a[0], a[1] & 1 != 0)?;
            let mut b = [0; 24];
            b[..8].copy_from_slice(&r.offset.to_le_bytes());
            b[8..16].copy_from_slice(&(r.offset + r.size).to_le_bytes());
            b[16..20].copy_from_slice(&r.memory_type.to_le_bytes());
            m.write(a[2], &b)?;
            Ok(0)
        }
        "query" => {
            if a[3] < 72 {
                return Err(Error::Invalid.into());
            }
            out(m, a[2], 72)?;
            let r = s.query(a[0], a[1] & 1 != 0)?;
            let mut b = [0; 72];
            b[..8].copy_from_slice(&r.address.to_le_bytes());
            b[8..16].copy_from_slice(&(r.address + r.size).to_le_bytes());
            b[16..24].copy_from_slice(&r.offset.to_le_bytes());
            b[24..28].copy_from_slice(&r.protection.to_le_bytes());
            b[28..32].copy_from_slice(&r.memory_type.to_le_bytes());
            b[32] = if r.direct { 0x12 } else { 0 };
            b[33..33 + r.name.len()].copy_from_slice(&r.name);
            m.write(a[2], &b)?;
            Ok(0)
        }
        _ => Err(Error::Unsupported.into()),
    }
}
pub fn registrations(s: Arc<MemoryResources>, errno: u64) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|(_, nid, op)| {
            let s = s.clone();
            Registration {
                key: super::semaphore::key(*nid),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    let result = if *op == "map2" {
                        if s.query_direct(f.arguments[5], false)
                            .is_ok_and(|r| r.memory_type == f.arguments[2] as i32)
                        {
                            if let Some(p) = f.stack_argument_address {
                                match m.read(p, 8) {
                                    Ok(b) => run(
                                        "map",
                                        [
                                            f.arguments[0],
                                            f.arguments[1],
                                            f.arguments[3],
                                            f.arguments[4],
                                            f.arguments[5],
                                            u64::from_le_bytes(b.try_into().expect("eight bytes")),
                                        ],
                                        m,
                                        &s,
                                    ),
                                    Err(e) => Err(Failure::Memory(e)),
                                }
                            } else {
                                Err(Failure::Resource(Error::Unsupported))
                            }
                        } else {
                            Err(Failure::Resource(Error::Unsupported))
                        }
                    } else {
                        run(op, f.arguments, m, &s)
                    };
                    match result {
                        Ok(v) => {
                            f.rax = v;
                            CallResult::Returned
                        }
                        Err(Failure::Memory(e)) => CallResult::AccessFailure(e),
                        Err(Failure::Resource(Error::Stopped)) => CallResult::StopRequested,
                        Err(Failure::Resource(e)) => {
                            let code: u32 = match e {
                                Error::Capacity | Error::Native(_) => 12,
                                Error::Busy => 16,
                                Error::NotFound => 2,
                                Error::Unsupported => 45,
                                _ => 22,
                            };
                            if matches!(*op, "unmap" | "protect") {
                                if let Err(e) = m.write(errno, &code.to_le_bytes()) {
                                    return CallResult::AccessFailure(e);
                                }
                                f.rax = u64::MAX;
                            } else {
                                f.rax = (0x80020000 | code) as i32 as i64 as u64;
                            }
                            CallResult::Returned
                        }
                    }
                })),
            }
        })
        .collect()
}
