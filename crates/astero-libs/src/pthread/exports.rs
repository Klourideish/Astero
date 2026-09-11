//! Exact libkernel identities, scoped guest copies and SCE/POSIX error adapters.
use astero_hle::{calls::memory::GuestMemory, dispatch::prepared::*};
use astero_kernel::synchronization::owned::{Error, Kind, Result, Synchronization, Thread};
use std::sync::Arc;
#[derive(Clone, Copy)]
pub struct Export {
    pub name: &'static str,
    pub nid: u64,
    pub kind: Kind,
    pub operation: &'static str,
    pub sce: bool,
}
pub const EXPORTS: &[Export] = &[
    Export {
        name: "SCE_PTHREAD_MUTEX_INIT",
        nid: 0x726a3544862f6bda,
        kind: Kind::Mutex,
        operation: "INIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEX_DESTROY",
        nid: 0xd8e7f47fede68611,
        kind: Kind::Mutex,
        operation: "DESTROY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEX_LOCK",
        nid: 0xf542b5bcb6507ede,
        kind: Kind::Mutex,
        operation: "LOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEX_TRYLOCK",
        nid: 0xba9a15af330715e1,
        kind: Kind::Mutex,
        operation: "TRYLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEX_UNLOCK",
        nid: 0xb67dd5943d211bad,
        kind: Kind::Mutex,
        operation: "UNLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEX_TIMEDLOCK",
        nid: 0x21a7c8d8fc5c3e74,
        kind: Kind::Mutex,
        operation: "TIMEDLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_INIT",
        nid: 0xe942c06b47eae230,
        kind: Kind::Rwlock,
        operation: "INIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_DESTROY",
        nid: 0x041fa46f4f1397d0,
        kind: Kind::Rwlock,
        operation: "DESTROY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_RDLOCK",
        nid: 0x3b1f62d1cecbe70d,
        kind: Kind::Rwlock,
        operation: "RDLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_TRYRDLOCK",
        nid: 0x5c3de60dec9b0a79,
        kind: Kind::Rwlock,
        operation: "TRYRDLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_WRLOCK",
        nid: 0x9aa74da2bac1fa02,
        kind: Kind::Rwlock,
        operation: "WRLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_TRYWRLOCK",
        nid: 0x6c81e86424e89ac2,
        kind: Kind::Rwlock,
        operation: "TRYWRLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_UNLOCK",
        nid: 0xf8bf7c3c86c6b6d9,
        kind: Kind::Rwlock,
        operation: "UNLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_TIMEDRDLOCK",
        nid: 0x88fb594562028eb3,
        kind: Kind::Rwlock,
        operation: "TIMEDRDLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCK_TIMEDWRLOCK",
        nid: 0x69d87fffa9c8a939,
        kind: Kind::Rwlock,
        operation: "TIMEDWRLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_COND_INIT",
        nid: 0xd936fddaaba9ae5d,
        kind: Kind::Cond,
        operation: "INIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_COND_DESTROY",
        nid: 0x83e3d977686269c8,
        kind: Kind::Cond,
        operation: "DESTROY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_COND_WAIT",
        nid: 0x58a0172785c13d0e,
        kind: Kind::Cond,
        operation: "WAIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_COND_TIMEDWAIT",
        nid: 0x06632363199ec35c,
        kind: Kind::Cond,
        operation: "TIMEDWAIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_COND_SIGNAL",
        nid: 0x90387f35fc6032d1,
        kind: Kind::Cond,
        operation: "SIGNAL",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_COND_BROADCAST",
        nid: 0x246823ed4beb97e0,
        kind: Kind::Cond,
        operation: "BROADCAST",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_INIT",
        nid: 0x17c6d41f0006dbce,
        kind: Kind::MutexAttr,
        operation: "INIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_DESTROY",
        nid: 0xb2658492d8b2c86d,
        kind: Kind::MutexAttr,
        operation: "DESTROY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_SETTYPE",
        nid: 0x88ca7c42913e5cee,
        kind: Kind::MutexAttr,
        operation: "SETTYPE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_GETTYPE",
        nid: 0x82ab84841ad2da2c,
        kind: Kind::MutexAttr,
        operation: "GETTYPE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_SETPROTOCOL",
        nid: 0xd451af5348bdb1a4,
        kind: Kind::MutexAttr,
        operation: "SETPROTOCOL",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_GETPROTOCOL",
        nid: 0x1a84e615eba2fa14,
        kind: Kind::MutexAttr,
        operation: "GETPROTOCOL",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_SETPSHARED",
        nid: 0x9b12b1f5bc571762,
        kind: Kind::MutexAttr,
        operation: "SETPSHARED",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_MUTEXATTR_GETPSHARED",
        nid: 0x968b04b9b1dceb87,
        kind: Kind::MutexAttr,
        operation: "GETPSHARED",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCKATTR_INIT",
        nid: 0xc8e7c683f2356482,
        kind: Kind::RwAttr,
        operation: "INIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCKATTR_DESTROY",
        nid: 0x8b689f6777d2d9fa,
        kind: Kind::RwAttr,
        operation: "DESTROY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCKATTR_SETTYPE",
        nid: 0x87f3a27e2a2e05df,
        kind: Kind::RwAttr,
        operation: "SETTYPE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCKATTR_GETTYPE",
        nid: 0x2b296cd42845cab7,
        kind: Kind::RwAttr,
        operation: "GETTYPE",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCKATTR_SETPSHARED",
        nid: 0xfd9bd01f5f23d747,
        kind: Kind::RwAttr,
        operation: "SETPSHARED",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_RWLOCKATTR_GETPSHARED",
        nid: 0x2dc3990471aa6c59,
        kind: Kind::RwAttr,
        operation: "GETPSHARED",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_CONDATTR_INIT",
        nid: 0x9b9ff66ec35fbfbb,
        kind: Kind::CondAttr,
        operation: "INIT",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_CONDATTR_DESTROY",
        nid: 0xc1a3dcc58891dd60,
        kind: Kind::CondAttr,
        operation: "DESTROY",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_CONDATTR_SETCLOCK",
        nid: 0x73f6f18f4dbb733b,
        kind: Kind::CondAttr,
        operation: "SETCLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_CONDATTR_GETCLOCK",
        nid: 0xeaa33790ee52dcea,
        kind: Kind::CondAttr,
        operation: "GETCLOCK",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_CONDATTR_SETPSHARED",
        nid: 0xeb131ec3dfab6702,
        kind: Kind::CondAttr,
        operation: "SETPSHARED",
        sce: true,
    },
    Export {
        name: "SCE_PTHREAD_CONDATTR_GETPSHARED",
        nid: 0x0e7fc34568bdb79e,
        kind: Kind::CondAttr,
        operation: "GETPSHARED",
        sce: true,
    },
    Export {
        name: "PTHREAD_MUTEX_INIT",
        nid: 0xb6d1cd7d4faa0c15,
        kind: Kind::Mutex,
        operation: "INIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEX_DESTROY",
        nid: 0x96d09f686af62461,
        kind: Kind::Mutex,
        operation: "DESTROY",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEX_LOCK",
        nid: 0xec7d224ce7224cba,
        kind: Kind::Mutex,
        operation: "LOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEX_TRYLOCK",
        nid: 0x2bf8d785bb76827e,
        kind: Kind::Mutex,
        operation: "TRYLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEX_UNLOCK",
        nid: 0xd99f8fa58e826898,
        kind: Kind::Mutex,
        operation: "UNLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEX_TIMEDLOCK",
        nid: 0x228f7e9d329766d0,
        kind: Kind::Mutex,
        operation: "TIMEDLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_INIT",
        nid: 0xcad4142cdfe784be,
        kind: Kind::Rwlock,
        operation: "INIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_DESTROY",
        nid: 0xd78ef56a33f3c61d,
        kind: Kind::Rwlock,
        operation: "DESTROY",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_RDLOCK",
        nid: 0x8868ecaf5580b48d,
        kind: Kind::Rwlock,
        operation: "RDLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_TRYRDLOCK",
        nid: 0x485c5330e7ee0a41,
        kind: Kind::Rwlock,
        operation: "TRYRDLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_WRLOCK",
        nid: 0xb08951bd0aac3766,
        kind: Kind::Rwlock,
        operation: "WRLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_TRYWRLOCK",
        nid: 0x5e15879fa3f947b5,
        kind: Kind::Rwlock,
        operation: "TRYWRLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_UNLOCK",
        nid: 0x12098ba3a11682ca,
        kind: Kind::Rwlock,
        operation: "UNLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_TIMEDRDLOCK",
        nid: 0x95bf259d8a3fa3b9,
        kind: Kind::Rwlock,
        operation: "TIMEDRDLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCK_TIMEDWRLOCK",
        nid: 0xf73925cc097d0863,
        kind: Kind::Rwlock,
        operation: "TIMEDWRLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_COND_INIT",
        nid: 0xd13c959383122edd,
        kind: Kind::Cond,
        operation: "INIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_COND_DESTROY",
        nid: 0x4575ea8b80ad17cc,
        kind: Kind::Cond,
        operation: "DESTROY",
        sce: false,
    },
    Export {
        name: "PTHREAD_COND_WAIT",
        nid: 0x3a9f130466392878,
        kind: Kind::Cond,
        operation: "WAIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_COND_TIMEDWAIT",
        nid: 0xdbb6c08222663a1d,
        kind: Kind::Cond,
        operation: "TIMEDWAIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_COND_SIGNAL",
        nid: 0xd8c3b2fab51fba14,
        kind: Kind::Cond,
        operation: "SIGNAL",
        sce: false,
    },
    Export {
        name: "PTHREAD_COND_BROADCAST",
        nid: 0x9a4c767d584d32c8,
        kind: Kind::Cond,
        operation: "BROADCAST",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_INIT",
        nid: 0x7501d612c26da04e,
        kind: Kind::MutexAttr,
        operation: "INIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_DESTROY",
        nid: 0x1c5ee52b8eb1ce36,
        kind: Kind::MutexAttr,
        operation: "DESTROY",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_SETTYPE",
        nid: 0x9839a030e19552a8,
        kind: Kind::MutexAttr,
        operation: "SETTYPE",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_GETTYPE",
        nid: 0x19916523b461b90a,
        kind: Kind::MutexAttr,
        operation: "GETTYPE",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_SETPROTOCOL",
        nid: 0xe6dc4a7dc3140289,
        kind: Kind::MutexAttr,
        operation: "SETPROTOCOL",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_GETPROTOCOL",
        nid: 0xc83696c54139d2cd,
        kind: Kind::MutexAttr,
        operation: "GETPROTOCOL",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_SETPSHARED",
        nid: 0x117bf7ced1aab433,
        kind: Kind::MutexAttr,
        operation: "SETPSHARED",
        sce: false,
    },
    Export {
        name: "PTHREAD_MUTEXATTR_GETPSHARED",
        nid: 0x3e62ff4f0294cd72,
        kind: Kind::MutexAttr,
        operation: "GETPSHARED",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCKATTR_INIT",
        nid: 0xc4579bb00e18b052,
        kind: Kind::RwAttr,
        operation: "INIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCKATTR_DESTROY",
        nid: 0xaac7668178ea4a09,
        kind: Kind::RwAttr,
        operation: "DESTROY",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCKATTR_SETTYPE_NP",
        nid: 0xf0db8e1e24ebd55c,
        kind: Kind::RwAttr,
        operation: "SETTYPE",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCKATTR_GETTYPE_NP",
        nid: 0x97e6c6e5fb189218,
        kind: Kind::RwAttr,
        operation: "GETTYPE",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCKATTR_SETPSHARED",
        nid: 0x3ae2a0fa44430fb5,
        kind: Kind::RwAttr,
        operation: "SETPSHARED",
        sce: false,
    },
    Export {
        name: "PTHREAD_RWLOCKATTR_GETPSHARED",
        nid: 0x56a10cb82bffa876,
        kind: Kind::RwAttr,
        operation: "GETPSHARED",
        sce: false,
    },
    Export {
        name: "PTHREAD_CONDATTR_INIT",
        nid: 0x98aa13c74dc74560,
        kind: Kind::CondAttr,
        operation: "INIT",
        sce: false,
    },
    Export {
        name: "PTHREAD_CONDATTR_DESTROY",
        nid: 0x74972e4159fafc8c,
        kind: Kind::CondAttr,
        operation: "DESTROY",
        sce: false,
    },
    Export {
        name: "PTHREAD_CONDATTR_SETCLOCK",
        nid: 0x123965680a803d9a,
        kind: Kind::CondAttr,
        operation: "SETCLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_CONDATTR_GETCLOCK",
        nid: 0x7130d8c5350d3e13,
        kind: Kind::CondAttr,
        operation: "GETCLOCK",
        sce: false,
    },
    Export {
        name: "PTHREAD_CONDATTR_SETPSHARED",
        nid: 0xdc1a4ff39d21053e,
        kind: Kind::CondAttr,
        operation: "SETPSHARED",
        sce: false,
    },
    Export {
        name: "PTHREAD_CONDATTR_GETPSHARED",
        nid: 0x874a94a92b8e982f,
        kind: Kind::CondAttr,
        operation: "GETPSHARED",
        sce: false,
    },
];
fn read64(m: &dyn GuestMemory, a: u64) -> Result<u64> {
    let b = m.read(a, 8).map_err(|_| Error::Invalid)?;
    Ok(u64::from_le_bytes(
        b.try_into().map_err(|_| Error::Invalid)?,
    ))
}
fn write64(m: &mut dyn GuestMemory, a: u64, v: u64) -> Result {
    m.write(a, &v.to_le_bytes()).map_err(|_| Error::Invalid)
}
fn result(e: Error, sce: bool) -> u64 {
    let n = match e {
        Error::Permission => 1,
        Error::Deadlock => 11,
        Error::Busy => 16,
        Error::Invalid => 22,
        Error::Capacity => 35,
        Error::Timeout => 60,
        Error::Interrupted => 4,
    };
    if sce { 0x80020000 | n } else { n }
}
fn dispatch(
    s: &Synchronization,
    t: Thread,
    e: Export,
    a: [u64; 6],
    m: &mut dyn GuestMemory,
) -> Result {
    let attr = matches!(e.kind, Kind::MutexAttr | Kind::RwAttr | Kind::CondAttr);
    if e.operation == "INIT" {
        read64(m, a[0])?;
        let mut value = match e.kind {
            Kind::Mutex | Kind::MutexAttr | Kind::Rwlock | Kind::RwAttr => 1,
            _ => 0,
        };
        if !attr && a[1] != 0 {
            let k = match e.kind {
                Kind::Mutex => Kind::MutexAttr,
                Kind::Rwlock => Kind::RwAttr,
                _ => Kind::CondAttr,
            };
            value = s.value(read64(m, a[1])?, a[1], k)?;
        }
        let id = s.create(a[0], e.kind, value)?;
        if let Err(err) = write64(m, a[0], id) {
            let _ = s.destroy(id, a[0], e.kind);
            return Err(err);
        }
        return Ok(());
    }
    let id = read64(m, a[0])?;
    s.validate(id, a[0], e.kind)?;
    if e.operation == "DESTROY" {
        write64(m, a[0], id)?;
        s.destroy(id, a[0], e.kind)?;
        return write64(m, a[0], 0);
    }
    if attr {
        let value = s.value(id, a[0], e.kind)?;
        return match e.operation {
            "SETTYPE" => {
                let v = i32::try_from(a[1]).map_err(|_| Error::Invalid)?;
                let valid = if e.kind == Kind::MutexAttr {
                    (0..=4).contains(&v)
                } else {
                    v == 1
                };
                if !valid {
                    return Err(Error::Invalid);
                }
                s.set_value(id, a[0], e.kind, if v == 0 { 1 } else { v })
            }
            "SETCLOCK" => {
                if !matches!(a[1], 0 | 4) {
                    return Err(Error::Invalid);
                }
                s.set_value(id, a[0], e.kind, a[1] as i32)
            }
            "SETPROTOCOL" | "SETPSHARED" => {
                if a[1] == 0 {
                    Ok(())
                } else {
                    Err(Error::Invalid)
                }
            }
            "GETTYPE" | "GETCLOCK" => m
                .write(a[1], &value.to_le_bytes())
                .map_err(|_| Error::Invalid),
            "GETPROTOCOL" | "GETPSHARED" => m
                .write(a[1], &0i32.to_le_bytes())
                .map_err(|_| Error::Invalid),
            _ => Err(Error::Invalid),
        };
    }
    let deadline = if e.operation.starts_with("TIMED") {
        let p = if e.kind == Kind::Cond { a[2] } else { a[1] };
        let clock = if e.kind == Kind::Cond {
            s.value(id, a[0], e.kind)?
        } else {
            0
        };
        Some(s.absolute(
            read64(m, p)? as i64,
            read64(m, p.checked_add(8).ok_or(Error::Invalid)?)? as i64,
            clock,
        )?)
    } else {
        None
    };
    match e.kind {
        Kind::Mutex => {
            if e.operation == "UNLOCK" {
                s.mutex_unlock(id, a[0], t)
            } else {
                s.mutex_lock(id, a[0], t, e.operation == "TRYLOCK", deadline)
            }
        }
        Kind::Rwlock => {
            if e.operation == "UNLOCK" {
                s.rw_unlock(id, a[0], t)
            } else {
                s.rw_lock(
                    id,
                    a[0],
                    t,
                    e.operation.contains("WRLOCK"),
                    e.operation.starts_with("TRY"),
                    deadline,
                )
            }
        }
        Kind::Cond => match e.operation {
            "SIGNAL" | "BROADCAST" => s.cond_signal(id, a[0], e.operation == "BROADCAST"),
            _ => s.cond_wait((id, a[0]), (read64(m, a[1])?, a[1]), t, deadline),
        },
        _ => Err(Error::Invalid),
    }
}
pub fn registrations(service: Arc<Synchronization>, thread: Thread) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|&e| {
            let s = service.clone();
            Registration {
                key: ProviderKey {
                    nid: e.nid,
                    library: b"libkernel".to_vec(),
                    module: b"libkernel".to_vec(),
                },
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    match dispatch(&s, thread, e, f.arguments, m) {
                        Ok(()) => {
                            f.rax = 0;
                            CallResult::Returned
                        }
                        Err(Error::Interrupted) => CallResult::StopRequested,
                        Err(err) => {
                            f.rax = result(err, e.sce);
                            CallResult::Returned
                        }
                    }
                })),
            }
        })
        .collect()
}
