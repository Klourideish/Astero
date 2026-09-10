//! Initial-thread/process startup adapters. No host environment or host pointers escape.
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::process::environment::Environment;
use std::{cell::RefCell, rc::Rc};
pub const ERROR_NID: u64 = 0xf41703ca43e6a352;
pub const PROCPARAM_NID: u64 = 0xf79f6aadaccf22b8;
pub const GETENV_NID: u64 = 0xb266d0ba47f16093;
pub const SETENV_NID: u64 = 0x3386186d215f27c8;
pub const TERMINATIONS: &[(&str, u64)] = &[
    ("exit", 0xb8c7a2d56f6ec8da),
    ("_exit", 0xe99f37b18585940f),
    ("abort", 0x2f54814e40be0afc),
    ("__stack_chk_fail", 0x3aede22f569bbe78),
];
fn string(m: &dyn GuestMemory, address: u64) -> Result<Vec<u8>, AccessError> {
    let mut v = Vec::new();
    v.try_reserve_exact(8192)
        .map_err(|_| AccessError::Allocation)?;
    for i in 0..8192 {
        let b = m.read(address.checked_add(i).ok_or(AccessError::Range)?, 1)?;
        let b = *b.first().ok_or(AccessError::Range)?;
        if b == 0 {
            return Ok(v);
        }
        v.push(b)
    }
    Err(AccessError::Limit)
}
pub fn registrations(errno: u64, procparam: u64) -> Vec<Registration> {
    let key = |nid, library: &[u8]| ProviderKey {
        nid,
        library: library.to_vec(),
        module: library.to_vec(),
    };
    let env = Rc::new(RefCell::new(Environment::new()));
    let read = env.clone();
    let mut entries = vec![
        Registration {
            key: key(ERROR_NID, b"libkernel"),
            kind: ProviderKind::HleImplementation,
            handler: Some(Box::new(move |f, _| {
                f.rax = errno;
                CallResult::Returned
            })),
        },
        Registration {
            key: key(PROCPARAM_NID, b"libkernel"),
            kind: ProviderKind::HleImplementation,
            handler: Some(Box::new(move |f, _| {
                f.rax = procparam;
                CallResult::Returned
            })),
        },
        Registration {
            key: key(GETENV_NID, b"libc"),
            kind: ProviderKind::HleImplementation,
            handler: Some(Box::new(move |f, m| {
                let Ok(name) = string(m, f.arguments[0]) else {
                    return CallResult::Unsupported;
                };
                f.rax = read.borrow().get(&name).unwrap_or(0);
                CallResult::Returned
            })),
        },
        Registration {
            key: key(SETENV_NID, b"libc"),
            kind: ProviderKind::HleImplementation,
            handler: Some(Box::new(move |f, m| {
                let (Ok(name), Ok(mut value)) =
                    (string(m, f.arguments[0]), string(m, f.arguments[1]))
                else {
                    return CallResult::Unsupported;
                };
                let mut env = env.borrow_mut();
                if name.is_empty() || name.contains(&b'=') {
                    if m.write(errno, &22u32.to_le_bytes()).is_err() {
                        return CallResult::Unsupported;
                    }
                    f.rax = u32::MAX as u64;
                    return CallResult::Returned;
                }
                if env.get(&name).is_some() && f.arguments[2] == 0 {
                    f.rax = 0;
                    return CallResult::Returned;
                }
                if !env.can_insert(&name) {
                    return CallResult::Unsupported;
                }
                value.push(0);
                let Ok(p) = m.allocate(value.len() as u64) else {
                    if let Err(e) = m.write(errno, &12u32.to_le_bytes()) {
                        return CallResult::AccessFailure(e);
                    }
                    f.rax = u32::MAX as u64;
                    return CallResult::Returned;
                };
                if m.write(p, &value).is_err() {
                    let _ = m.free(p);
                    return CallResult::Unsupported;
                }
                match env.put(name, p) {
                    Ok(old) => {
                        if let Some(old) = old
                            && m.free(old).is_err()
                        {
                            return CallResult::Unsupported;
                        }
                        f.rax = 0;
                        CallResult::Returned
                    }
                    Err(_) => {
                        let _ = m.free(p);
                        CallResult::Unsupported
                    }
                }
            })),
        },
    ];
    for &(_, nid) in TERMINATIONS {
        for library in [b"libc".as_slice(), b"libkernel".as_slice()] {
            entries.push(Registration {
                key: key(nid, library),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(|f, _| {
                    f.rax = f.arguments[0];
                    CallResult::StopRequested
                })),
            })
        }
    }
    entries
}
