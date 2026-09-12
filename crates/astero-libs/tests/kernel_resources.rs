#![cfg(all(windows, target_arch = "x86_64"))]
use astero_abi::layouts::entry::CallFrame;
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::{
    objects::memory::{DIRECT_SIZE, MemoryResources},
    synchronization::{owned::Thread, semaphore::Semaphores},
};
use astero_timing::scheduler::{Config, TimingEngine};
use std::sync::Arc;
struct Memory {
    bytes: Vec<u8>,
    fail: bool,
}
impl Memory {
    fn new() -> Self {
        Self {
            bytes: vec![0; 128],
            fail: false,
        }
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, _: bool) -> Result<(), AccessError> {
        if a != 0 && a.checked_add(n).is_some_and(|e| e <= 128) {
            Ok(())
        } else {
            Err(AccessError::Range)
        }
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.validate(a, n, false)?;
        Ok(self.bytes[a as usize..(a + n) as usize].to_vec())
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        self.validate(a, b.len() as u64, true)?;
        if self.fail {
            return Err(AccessError::Range);
        }
        self.bytes[a as usize..a as usize + b.len()].copy_from_slice(b);
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
fn invoke(r: &[Registration], op: usize, a: [u64; 6], m: &mut Memory) -> (CallResult, u64) {
    let mut f = CallFrame {
        arguments: a,
        ..Default::default()
    };
    let result = r[op].handler.as_ref().unwrap()(&mut f, m);
    (result, f.rax)
}

fn setup() -> (TimingEngine, Arc<Semaphores>) {
    let (e, _) = TimingEngine::manual(Config {
        max_pending: 16,
        max_snapshot_entries: 16,
    })
    .unwrap();
    let s = Arc::new(Semaphores::new(e.scheduler(), 8, 8).unwrap());
    (e, s)
}
#[test]
fn creation_returns_status_and_publishes_u32_handle() {
    let (_e, s) = setup();
    let r = astero_libs::kernel::semaphore::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    m.bytes[40..42].copy_from_slice(b"s\0");
    assert_eq!(
        invoke(&r, 1, [8, 40, 1, 2, 3, 0], &mut m),
        (CallResult::Returned, 0)
    );
    assert_eq!(u32::from_le_bytes(m.bytes[8..12].try_into().unwrap()), 1);
    assert_eq!(s.snapshot().objects.len(), 1);
    assert_ne!(r[0].key.nid, r[1].key.nid);
    assert_eq!(r[0].key.module, b"libkernel");
}
#[test]
fn semaphore_failed_publication_rolls_back() {
    let (_e, s) = setup();
    let r = astero_libs::kernel::semaphore::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    m.bytes[40..42].copy_from_slice(b"s\0");
    m.fail = true;
    assert!(matches!(
        invoke(&r, 0, [8, 40, 0, 0, 1, 0], &mut m).0,
        CallResult::AccessFailure(_)
    ));
    assert!(s.snapshot().objects.is_empty());
}
#[test]
fn timed_poll_writes_remaining_budget() {
    let (_e, s) = setup();
    let h = s.create(b"s", 0, 1).unwrap();
    let r = astero_libs::kernel::semaphore::registrations(s, Thread(1));
    let mut m = Memory::new();
    let (result, value) = invoke(&r, 2, [h as u64, 1, 8, 0, 0, 0], &mut m);
    assert_eq!(result, CallResult::Returned);
    assert_eq!(value as u32, 0x8002003c);
    assert_eq!(&m.bytes[8..12], &[0; 4]);
}
#[test]
fn cancel_output_preflight_leaves_count_unchanged() {
    let (_e, s) = setup();
    let h = s.create(b"s", 0, 3).unwrap();
    let r = astero_libs::kernel::semaphore::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    assert!(matches!(
        invoke(&r, 5, [h as u64, 2, 127, 0, 0, 0], &mut m).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().objects[0].count, 0);
}
#[test]
fn direct_size_and_main_allocation_contract() {
    let s = Arc::new(MemoryResources::new());
    let r = astero_libs::kernel::memory::registrations(s.clone(), 100);
    let mut m = Memory::new();
    assert_eq!(
        invoke(&r, 0, [0; 6], &mut m),
        (CallResult::Returned, DIRECT_SIZE)
    );
    assert_eq!(
        invoke(&r, 2, [65536, 65536, 0, 8, 127, 127], &mut m),
        (CallResult::Returned, 0)
    );
    assert_eq!(s.snapshot().allocations.len(), 1);
    assert_eq!(&m.bytes[8..16], &[0; 8]);
}
#[test]
fn allocation_publication_failure_releases_backing() {
    let s = Arc::new(MemoryResources::new());
    let r = astero_libs::kernel::memory::registrations(s.clone(), 100);
    let mut m = Memory::new();
    m.fail = true;
    assert!(matches!(
        invoke(&r, 2, [65536, 65536, 0, 8, 0, 0], &mut m).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().active_native_resources, 0);
}
#[test]
fn direct_query_exact_bytes() {
    let s = Arc::new(MemoryResources::new());
    s.allocate(65536, DIRECT_SIZE, 65536, 65536, 3).unwrap();
    let r = astero_libs::kernel::memory::registrations(s, 100);
    let mut m = Memory::new();
    m.bytes.fill(0xaa);
    assert_eq!(
        invoke(&r, 4, [65536, 0, 16, 24, 0, 0], &mut m),
        (CallResult::Returned, 0)
    );
    assert_eq!(
        u64::from_le_bytes(m.bytes[16..24].try_into().unwrap()),
        65536
    );
    assert_eq!(
        u64::from_le_bytes(m.bytes[24..32].try_into().unwrap()),
        131072
    );
    assert_eq!(&m.bytes[32..40], &[3, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(m.bytes[40], 0xaa);
}

#[test]
fn sce_unmap_preserves_partial_and_backing() {
    let s = Arc::new(MemoryResources::new());
    let offset = s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap();
    let address = s.map(0, 65536, 3, 0, offset, 65536).unwrap();
    let r = astero_libs::kernel::memory::registrations(s.clone(), 100);
    let i = r
        .iter()
        .position(|r| r.key.nid == 0x71091EF54B8140E9)
        .unwrap();
    let mut m = Memory::new();
    assert_ne!(invoke(&r, i, [address, 32768, 0, 0, 0, 0], &mut m).1, 0);
    assert_eq!(s.snapshot().mappings.len(), 1);
    assert_eq!(invoke(&r, i, [address, 65536, 0, 0, 0, 0], &mut m).1, 0);
    assert!(s.snapshot().mappings.is_empty());
    assert_eq!(s.snapshot().unmapped_total, 1);
    assert_eq!(s.snapshot().allocations.len(), 1);
    assert_ne!(invoke(&r, i, [address, 65536, 0, 0, 0, 0], &mut m).1, 0);
    s.release(offset, 65536).unwrap();
    s.shutdown();
}
