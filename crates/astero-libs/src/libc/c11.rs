//! Dinkumware C11 wrappers over the runtime-owned pthread synchronization service.
//! PS5Rust c11_sync and exports establish pointer-sized slots and wrapper return mapping.
use astero_hle::{calls::memory::GuestMemory, dispatch::prepared::*};
use astero_kernel::synchronization::owned::{Error, Kind, Synchronization, Thread};
use std::sync::Arc;

pub const EXPORTS: &[(&str, u64)] = &[
    ("_Mtx_init", 0x61A1DCDC64BBCBB8),
    ("_Mtx_lock", 0x892E1A59B5289E5D),
    ("_Mtx_unlock", 0x813B974303FDAEBB),
    ("_Mtx_destroy", 0xE4B7F9D63BE88534),
    ("_Cnd_init", 0x4AB799C9B4915A95),
    ("_Cnd_signal", 0xD2EBAA811CFDA9FA),
    ("_Cnd_broadcast", 0x56C3F775A2609950),
    ("_Cnd_wait", 0xBC46AA13FEC86587),
    ("_Cnd_destroy", 0xEF230581C4BC10F0),
];

pub fn registrations(sync: Arc<Synchronization>, thread: Thread) -> Vec<Registration> {
    EXPORTS
        .iter()
        .enumerate()
        .map(|(op, (_, nid))| {
            let sync = sync.clone();
            Registration {
                key: ProviderKey {
                    nid: *nid,
                    library: b"libc".to_vec(),
                    module: b"libc".to_vec(),
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    match call(&sync, thread, op, f.arguments, m) {
                        Ok(v) => {
                            f.rax = v;
                            CallResult::Returned
                        }
                        Err(e) => e,
                    }
                })),
            }
        })
        .collect()
}

fn read(m: &mut dyn GuestMemory, a: u64) -> Result<u64, CallResult> {
    let b = m.read(a, 8).map_err(CallResult::AccessFailure)?;
    Ok(u64::from_le_bytes(
        b.try_into().map_err(|_| CallResult::Unsupported)?,
    ))
}

fn call(
    s: &Synchronization,
    t: Thread,
    op: usize,
    a: [u64; 6],
    m: &mut dyn GuestMemory,
) -> Result<u64, CallResult> {
    let kind = if op < 4 { Kind::Mutex } else { Kind::Cond };
    if op == 0 || op == 4 {
        // PS5Rust uses an error-check mutex regardless of flags. Retain that bounded
        // policy for default and observed mode 2; timed/recursive expansion is not claimed.
        if op == 0 && !matches!(a[1], 0 | 2) {
            return Err(CallResult::Unsupported);
        }
        m.validate(a[0], 8, true)
            .map_err(CallResult::AccessFailure)?;
        let id = match s.create(a[0], kind, 1) {
            Ok(id) => id,
            Err(Error::Interrupted) => return Err(CallResult::StopRequested),
            Err(_) => return Ok(2),
        };
        if let Err(e) = m.write(a[0], &id.to_le_bytes()) {
            let _ = s.destroy(id, a[0], kind);
            return Err(CallResult::AccessFailure(e));
        }
        return Ok(0);
    }
    let id = read(m, a[0])?;
    let result = match op {
        1 => s.mutex_lock(id, a[0], t, false, None),
        2 => s.mutex_unlock(id, a[0], t),
        3 | 8 => {
            m.validate(a[0], 8, true)
                .map_err(CallResult::AccessFailure)?;
            let r = s.destroy(id, a[0], kind);
            if r.is_ok() {
                m.write(a[0], &0u64.to_le_bytes())
                    .map_err(CallResult::AccessFailure)?;
            }
            r
        }
        5 | 6 => s.cond_signal(id, a[0], op == 6),
        7 => s.cond_wait((id, a[0]), (read(m, a[1])?, a[1]), t, None),
        _ => return Err(CallResult::Unsupported),
    };
    if result == Err(Error::Interrupted) {
        return Err(CallResult::StopRequested);
    }
    // These shipped wrappers discard pthread return values, not ownership checks.
    if op != 1 {
        return Ok(0);
    }
    Ok(match result {
        Ok(()) => 0,
        Err(Error::Deadlock | Error::Busy) => 3,
        Err(_) => 4,
    })
}
