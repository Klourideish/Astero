//! Exact guest timing providers. SCE errors and POSIX errno remain separate.
use astero_abi::layouts::time::Timespec;
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::{
    synchronization::owned::Thread,
    timing::{
        clock::{self, Error},
        sleep::{GuestTiming, Wake},
    },
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug)]
pub struct Export {
    pub name: &'static str,
    pub nid: u64,
    pub op: &'static str,
    pub posix: bool,
    pub libc: bool,
}
macro_rules! e {
    ($n:literal,$id:literal,$op:literal,$p:literal,$l:literal) => {
        Export {
            name: $n,
            nid: $id,
            op: $op,
            posix: $p,
            libc: $l,
        }
    };
}
pub const EXPORTS: &[Export] = &[
    e!("sceKernelUsleep", 0xD637D72D15738AC7, "us", false, false),
    e!("sceKernelSleep", 0xFD947E846EDA0C7C, "s", false, false),
    e!("sceKernelNanosleep", 0x42FB19C689AF507B, "ns", false, false),
    e!("sleep", 0xD30BB7DE1BA735D1, "s", true, false),
    e!("usleep", 0x41CB5E4706EC9D5D, "us", true, false),
    e!("nanosleep", 0xC92F14D931827B50, "ns", true, false),
    e!("_nanosleep", 0x361A6CA7176310A5, "ns", true, false),
    e!(
        "sceKernelClockGettime",
        0x4018BB1C22B4DE1C,
        "time",
        false,
        false
    ),
    e!("clock_gettime", 0x94B313F6F240724D, "time", true, false),
    e!("clock_getres", 0xB26223EDEAB3644F, "res", true, false),
    e!(
        "sceKernelGettimeofday",
        0x7A37A471A35036AD,
        "wall",
        false,
        false
    ),
    e!("gettimeofday", 0x9FCF2FC770B99D6F, "wall", true, false),
    e!(
        "sceKernelGetProcessTime",
        0xE09DAC5099AE1D94,
        "counter",
        false,
        false
    ),
    e!(
        "sceKernelGetProcessTimeCounter",
        0x7E0C6731E4CD52D6,
        "counter",
        false,
        false
    ),
    e!(
        "sceKernelGetProcessTimeCounterFrequency",
        0x04DA30C76979F3C1,
        "frequency",
        false,
        false
    ),
    e!("sceKernelReadTsc", 0xFF62115023BFFCF3, "tsc", false, false),
    e!(
        "sceKernelGetTscFrequency",
        0xD63DD2DE7FED4D6E,
        "tscfreq",
        false,
        false
    ),
    e!("clock", 0x4193FA23D659C691, "cpu", true, true),
];
pub fn key(e: &Export) -> ProviderKey {
    let l = if e.libc {
        b"libc".to_vec()
    } else {
        b"libkernel".to_vec()
    };
    ProviderKey {
        nid: e.nid,
        library: l.clone(),
        module: l,
    }
}
#[derive(Debug)]
enum Failure {
    Time(Error),
    Memory(AccessError),
}
impl From<Error> for Failure {
    fn from(e: Error) -> Self {
        Self::Time(e)
    }
}
impl From<AccessError> for Failure {
    fn from(e: AccessError) -> Self {
        Self::Memory(e)
    }
}
fn preflight(m: &dyn GuestMemory, p: u64) -> Result<(), Failure> {
    if p == 0 {
        return Err(AccessError::Range.into());
    }
    m.charge(16)?;
    m.validate(p, 16, true)?;
    Ok(())
}
fn execute(
    e: &Export,
    a: [u64; 6],
    m: &mut dyn GuestMemory,
    t: &GuestTiming,
    thread: Thread,
) -> Result<(u64, bool), Failure> {
    match e.op {
        "s" | "us" | "ns" => {
            let span = if e.op == "ns" {
                if a[0] == 0 {
                    return Err(AccessError::Range.into());
                }
                m.charge(16)?;
                let b = m.read(a[0], 16)?;
                clock::timespec(Timespec::decode(&b).map_err(|_| Error::Invalid)?)?
            } else {
                clock::units(a[0], if e.op == "us" { 1000 } else { 1_000_000_000 })?
            };
            if e.op == "ns" && a[1] != 0 {
                preflight(m, a[1])?;
            }
            let r = t.sleep(thread, span)?;
            if r.wake == Wake::Interrupted {
                if e.op == "ns" && a[1] != 0 {
                    let remaining = span.as_nanos().saturating_sub(r.elapsed_ns);
                    m.write(a[1], &Timespec::from_nanos(remaining).encode())?;
                }
                return Ok((0, true));
            }
            // Successful nanosleep leaves optional remainder unchanged; only interruption defines it.
            Ok((0, false))
        }
        "time" | "res" => {
            let n = if e.op == "res" {
                clock::quantum(a[0] as i32)?
            } else {
                t.query(a[0] as i32)?
            };
            if e.op == "res" && a[1] == 0 {
                return Ok((0, false));
            }
            preflight(m, a[1])?;
            m.write(a[1], &Timespec::from_nanos(n).encode())?;
            Ok((0, false))
        }
        "wall" => {
            if a[1] != 0 {
                return Err(Error::Unsupported.into());
            }
            let n = t.query(0)?;
            let mut b = Timespec::from_nanos(n);
            b.nanos /= 1000;
            preflight(m, a[0])?;
            m.write(a[0], &b.encode())?;
            Ok((0, false))
        }
        "counter" => Ok((t.query(4)? / 1000, false)),
        "frequency" => Ok((1_000_000, false)),
        "tsc" => Ok((clock::virtual_tsc_nanos(t.query(4)?)?, false)),
        "tscfreq" => Ok((clock::TSC_FREQUENCY, false)),
        _ => Err(Error::Unsupported.into()),
    }
}
pub fn registrations(t: Arc<GuestTiming>, thread: Thread, errno: u64) -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|e| {
            let t = t.clone();
            Registration {
                key: key(e),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(move |f, m| {
                    match execute(e, f.arguments, m, &t, thread) {
                        Ok((v, stop)) => {
                            f.rax = v;
                            if stop {
                                CallResult::StopRequested
                            } else {
                                CallResult::Returned
                            }
                        }
                        Err(Failure::Time(Error::Interrupted)) => CallResult::StopRequested,
                        Err(err) => {
                            let code = match err {
                                Failure::Time(Error::Invalid) => 22,
                                Failure::Time(Error::Unsupported) => 45,
                                Failure::Time(Error::Capacity) => 12,
                                Failure::Time(Error::Interrupted) => 4,
                                Failure::Memory(AccessError::Range) => 14,
                                Failure::Memory(e) => return CallResult::AccessFailure(e),
                            };
                            if e.posix {
                                if let Err(e) = m.write(errno, &(code as i32).to_le_bytes()) {
                                    return CallResult::AccessFailure(e);
                                }
                                f.rax = u64::MAX;
                            } else {
                                f.rax = (0x80020000u32 | code) as i32 as i64 as u64;
                            }
                            CallResult::Returned
                        }
                    }
                })),
            }
        })
        .collect()
}
