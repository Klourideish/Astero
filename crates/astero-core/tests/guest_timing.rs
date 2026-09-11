use astero_abi::layouts::{entry::CallFrame, time::Timespec};
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_kernel::{
    synchronization::owned::Thread,
    timing::{clock::Realtime, sleep::GuestTiming},
};
use astero_libs::kernel::timing::*;
use astero_timing::scheduler::{Config, TimingEngine};
use std::sync::Arc;
struct Memory {
    b: Vec<u8>,
    ro: bool,
}
impl Memory {
    fn new() -> Self {
        Self {
            b: vec![0; 2 * 1024 * 1024],
            ro: false,
        }
    }
    fn put(&mut self, a: usize, b: &[u8]) {
        self.b[a..a + b.len()].copy_from_slice(b)
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, w: bool) -> Result<(), AccessError> {
        if a.checked_add(n).is_none_or(|e| e > self.b.len() as u64) || w && self.ro {
            Err(AccessError::Range)
        } else {
            Ok(())
        }
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.validate(a, n, false)?;
        Ok(self.b[a as usize..(a + n) as usize].to_vec())
    }
    fn read_window(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.read(a, n.min(4096 - a % 4096))
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        self.validate(a, b.len() as u64, true)?;
        self.put(a as usize, b);
        Ok(())
    }
    fn allocate(&mut self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::Allocation)
    }
    fn free(&mut self, _: u64) -> Result<(), AccessError> {
        Err(AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::InvalidAllocation)
    }
}

fn setup() -> (TimingEngine, PreparedRegistry, Memory) {
    let (e, _c) = TimingEngine::manual(Config {
        max_pending: 16,
        max_snapshot_entries: 16,
    })
    .unwrap();
    let t = Arc::new(GuestTiming::new(
        e.scheduler(),
        Realtime::Fixed(9_123_456_789),
    ));
    let r = PreparedRegistry::new(registrations(t, Thread(1), 32), 18).unwrap();
    (e, r, Memory::new())
}
fn call(r: &PreparedRegistry, m: &mut Memory, name: &str, a: [u64; 6]) -> (CallResult, u64) {
    let e = EXPORTS.iter().find(|e| e.name == name).unwrap();
    let mut f = CallFrame {
        arguments: a,
        ..Default::default()
    };
    let c = r.invoke(&key(e), &mut f, m).unwrap();
    (c, f.rax)
}
#[test]
fn exact_keys_and_coherent_exports() {
    let (_e, r, _m) = setup();
    assert_eq!(EXPORTS.len(), 18);
    for e in EXPORTS {
        assert!(r.find(&key(e)).is_ok());
        let mut k = key(e);
        k.library = b"wrong".to_vec();
        assert!(r.find(&k).is_err());
    }
}
#[test]
fn zero_sleep_aliases_succeed() {
    let (_e, r, mut m) = setup();
    m.put(128, &Timespec::from_nanos(0).encode());
    for n in ["sceKernelSleep", "sleep", "sceKernelUsleep", "usleep"] {
        assert_eq!(call(&r, &mut m, n, [0; 6]), (CallResult::Returned, 0));
    }
    for n in ["sceKernelNanosleep", "nanosleep", "_nanosleep"] {
        assert_eq!(call(&r, &mut m, n, [128, 0, 0, 0, 0, 0]).1, 0);
    }
}
#[test]
fn sce_and_posix_invalid_timespec_are_distinct() {
    let (_e, r, mut m) = setup();
    m.put(
        128,
        &Timespec {
            seconds: 0,
            nanos: 1000000000,
        }
        .encode(),
    );
    assert_eq!(
        call(&r, &mut m, "sceKernelNanosleep", [128, 0, 0, 0, 0, 0]).1 as u32,
        0x80020016
    );
    assert_eq!(
        call(&r, &mut m, "nanosleep", [128, 0, 0, 0, 0, 0]).1,
        u64::MAX
    );
    assert_eq!(u32::from_le_bytes(m.b[32..36].try_into().unwrap()), 22);
}
#[test]
fn clock_serialization_and_timeval() {
    let (_e, r, mut m) = setup();
    assert_eq!(call(&r, &mut m, "clock_gettime", [0, 128, 0, 0, 0, 0]).1, 0);
    assert_eq!(
        Timespec::decode(&m.b[128..144]).unwrap(),
        Timespec {
            seconds: 9,
            nanos: 123456000
        }
    );
    assert_eq!(call(&r, &mut m, "gettimeofday", [128, 0, 0, 0, 0, 0]).1, 0);
    assert_eq!(Timespec::decode(&m.b[128..144]).unwrap().nanos, 123456);
}
#[test]
fn res_null_and_cpu_refusal() {
    let (_e, r, mut m) = setup();
    assert_eq!(call(&r, &mut m, "clock_getres", [4, 0, 0, 0, 0, 0]).1, 0);
    assert_eq!(
        call(&r, &mut m, "clock_gettime", [15, 128, 0, 0, 0, 0]).1,
        u64::MAX
    );
    assert_eq!(u32::from_le_bytes(m.b[32..36].try_into().unwrap()), 45);
    assert_eq!(call(&r, &mut m, "clock", [0; 6]).1, u64::MAX);
}
#[test]
fn invalid_output_preserves_tail_and_reports_efault() {
    let (_e, r, mut m) = setup();
    let p = m.b.len() as u64 - 8;
    m.b[p as usize..].fill(0xa5);
    assert_eq!(
        call(&r, &mut m, "sceKernelClockGettime", [0, p, 0, 0, 0, 0]).1 as u32,
        0x8002000e
    );
    assert_eq!(&m.b[p as usize..], &[0xa5; 8]);
}
#[test]
fn sleep_conversion_overflow_is_not_wrapped() {
    let (_e, r, mut m) = setup();
    assert_eq!(
        call(&r, &mut m, "sceKernelUsleep", [u64::MAX, 0, 0, 0, 0, 0]).1 as u32,
        0x80020016
    );
}
#[test]
fn optional_remainder_preflight_and_success_preservation() {
    let (_e, r, mut m) = setup();
    m.put(128, &[0; 16]);
    m.b[256..272].fill(0xa5);
    assert_eq!(call(&r, &mut m, "nanosleep", [128, 256, 0, 0, 0, 0]).1, 0);
    assert_eq!(&m.b[256..272], &[0xa5; 16]);
    let end = m.b.len() as u64;
    assert_eq!(
        call(&r, &mut m, "nanosleep", [128, end - 1, 0, 0, 0, 0]).1,
        u64::MAX
    );
}
