//! Exact kernel semaphore identities and checked 32-bit handle/timeout contracts.
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::synchronization::{
    owned::Thread,
    semaphore::{Error, Semaphores},
};

use std::sync::Arc;
pub const EXPORTS: &[(&str, u64, &str)] = &[
    ("sceKernelCreateSema", 0x840B48A18B097561, "create"),
    ("sceKernelCreateSema", 0xD7CF31E7B258A748, "create"),
    ("sceKernelWaitSema", 0x6716B45614154EC9, "wait"),
    ("sceKernelSignalSema", 0xE1CCE9A47062AE2C, "signal"),
    ("sceKernelPollSema", 0xD76C0E1E4F32C1BD, "poll"),
    ("sceKernelCancelSema", 0xE03334E94D813446, "cancel"),
    ("sceKernelDeleteSema", 0x47526F9FC6D2096F, "delete"),
];
pub fn key(nid: u64) -> ProviderKey {
    ProviderKey {
        nid,
        library: b"libkernel".to_vec(),
        module: b"libkernel".to_vec(),
    }
}
fn name(m: &dyn GuestMemory, p: u64) -> Result<Vec<u8>, AccessError> {
    if p == 0 {
        return Err(AccessError::Range);
    }
    let mut b = vec![];
    for i in 0..32 {
        let v = m.read(p.checked_add(i).ok_or(AccessError::Range)?, 1)?[0];
        if v == 0 {
            return Ok(b);
        }
        b.push(v);
    }
    Err(AccessError::Limit)
}
fn output(m: &dyn GuestMemory, p: u64) -> Result<(), AccessError> {
    if p == 0 {
        return Err(AccessError::Range);
    }
    m.validate(p, 4, true)
}
fn execute(
    op: &str,
    a: [u64; 6],
    m: &mut dyn GuestMemory,
    s: &Semaphores,
    thread: Thread,
) -> Result<Result<(), Error>, AccessError> {
    let h = a[0] as u32;
    Ok(match op {
        "create" => {
            output(m, a[0])?;
            let n = name(m, a[1])?;
            if a[2] > 1 || a[5] != 0 {
                Err(Error::Invalid)
            } else {
                match s.create(&n, a[3] as i32, a[4] as i32) {
                    Ok(h) => {
                        if let Err(e) = m.write(a[0], &h.to_le_bytes()) {
                            let _ = s.delete(h);
                            return Err(e);
                        }
                        Ok(())
                    }
                    Err(e) => Err(e),
                }
            }
        }
        "poll" => s.poll(h, a[1] as i32),
        "signal" => s.signal(h, a[1] as i32),
        "delete" => s.delete(h),
        "cancel" => {
            if a[2] != 0 {
                output(m, a[2])?;
            }
            match s.cancel(h, a[1] as i32) {
                Ok(n) => {
                    if a[2] != 0 {
                        m.write(a[2], &(n as u32).to_le_bytes())?;
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        "wait" => {
            let micros = if a[2] == 0 {
                None
            } else {
                output(m, a[2])?;
                let b = m.read(a[2], 4)?;
                Some(u32::from_le_bytes(b.try_into().expect("four bytes")))
            };
            let (r, left) = s.wait_micros(h, a[1] as i32, thread, micros);
            if micros.is_some() {
                m.write(a[2], &left.to_le_bytes())?;
            }
            r
        }
        _ => Err(Error::Invalid),
    })
}
pub fn registrations(s: Arc<Semaphores>, thread: Thread) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|(_, nid, op)| {
            let s = s.clone();
            Registration {
                key: key(*nid),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    match execute(op, f.arguments, m, &s, thread) {
                        Err(e) => CallResult::AccessFailure(e),
                        Ok(Err(Error::Interrupted)) => CallResult::StopRequested,
                        Ok(r) => {
                            let code: u32 = match r {
                                Ok(()) => 0,
                                Err(Error::NotFound) => 0x80020002,
                                Err(Error::Deleted) => 0x8002000d,
                                Err(Error::Busy) => 0x80020010,
                                Err(Error::Timeout) => 0x8002003c,
                                Err(Error::Cancelled) => 0x8002004f,
                                Err(Error::Capacity) => 0x8002000c,
                                Err(Error::Overflow) => 0x80020054,
                                Err(_) => 0x80020016,
                            };
                            f.rax = code as i32 as i64 as u64;
                            CallResult::Returned
                        }
                    }
                })),
            }
        })
        .collect()
}
