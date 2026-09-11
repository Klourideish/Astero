use astero_abi::layouts::entry::CallFrame;
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_kernel::{process::guards::Guards, synchronization::owned::Thread};
use astero_libs::runtime::guards::*;
use astero_timing::scheduler::{Config, TimingEngine};
use std::sync::Arc;
struct Memory {
    b: Vec<u8>,
    ro: bool,
}
impl Memory {
    fn new() -> Self {
        Self {
            b: vec![0; 256],
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

fn setup() -> (TimingEngine, Arc<Guards>) {
    let e = TimingEngine::real(Config {
        max_pending: 64,
        max_snapshot_entries: 64,
    })
    .unwrap();
    let g = Arc::new(Guards::new(e.scheduler(), 2).unwrap());
    g.arm(
        astero_timing::time::Deadline::after(
            e.scheduler().now().unwrap(),
            astero_timing::time::Span::from_millis(5000).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    (e, g)
}
fn call(g: &Arc<Guards>, t: u64, op: usize, a: u64, m: &mut Memory) -> (CallResult, u64) {
    let r = PreparedRegistry::new(registrations(g.clone(), Thread(t)), EXPORTS.len()).unwrap();
    let mut f = CallFrame {
        arguments: [a, 0, 0, 0, 0, 0],
        rax: 99,
        ..Default::default()
    };
    let key = ProviderKey {
        nid: EXPORTS[op].1,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    (r.invoke(&key, &mut f, m).unwrap(), f.rax)
}
#[test]
fn once_completion_preserves_reserved_bytes() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    m.b[9..16].fill(0xa5);
    assert_eq!(call(&g, 1, 0, 8, &mut m), (CallResult::Returned, 1));
    assert_eq!(m.b[8], 0);
    assert_eq!(call(&g, 1, 1, 8, &mut m).0, CallResult::Returned);
    assert_eq!(m.b[8], 1);
    assert_eq!(&m.b[9..16], &[0xa5; 7]);
    assert_eq!(call(&g, 2, 0, 8, &mut m), (CallResult::Returned, 0));
}
#[test]
fn abort_allows_retry() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    assert_eq!(call(&g, 1, 0, 8, &mut m).1, 1);
    assert_eq!(call(&g, 1, 2, 8, &mut m).0, CallResult::Returned);
    assert_eq!(m.b[8], 0);
    assert_eq!(call(&g, 2, 0, 8, &mut m).1, 1);
}
#[test]
fn wrong_owner_and_recursive_acquire_refuse() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    call(&g, 1, 0, 8, &mut m);
    assert_eq!(call(&g, 2, 1, 8, &mut m).0, CallResult::Unsupported);
    assert_eq!(m.b[8], 0);
    assert_eq!(call(&g, 1, 0, 8, &mut m).0, CallResult::StopRequested);
}
#[test]
fn invalid_and_readonly_ranges_do_not_create_guard() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    for a in [0, 1, 256, u64::MAX] {
        assert_ne!(call(&g, 1, 0, a, &mut m).0, CallResult::Returned);
    }
    m.ro = true;
    assert_ne!(call(&g, 1, 0, 8, &mut m).0, CallResult::Returned);
    assert_eq!(g.snapshot().created, 0);
}
#[test]
fn capacity_and_exact_identity() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    for a in [8, 16] {
        call(&g, 1, 0, a, &mut m);
        call(&g, 1, 2, a, &mut m);
    }
    assert_eq!(call(&g, 1, 0, 24, &mut m).0, CallResult::StopRequested);
    let r = PreparedRegistry::new(registrations(g, Thread(1)), EXPORTS.len()).unwrap();
    assert!(
        r.find(&ProviderKey {
            nid: EXPORTS[0].1,
            library: b"other".to_vec(),
            module: b"libc".to_vec()
        })
        .is_err()
    );
}
#[test]
fn competing_thread_blocks_until_release_and_shutdown_interrupts() {
    let (_e, g) = setup();
    g.acquire(8, Thread(1)).unwrap();
    let other = g.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    let h = std::thread::spawn(move || {
        let r = other.acquire(8, Thread(2));
        tx.send(r).unwrap();
    });
    assert!(
        rx.recv_timeout(std::time::Duration::from_millis(20))
            .is_err()
    );
    g.release(8, Thread(1)).unwrap();
    assert_eq!(
        rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap(),
        Ok(())
    );
    h.join().unwrap();
    let other = g.clone();
    let h = std::thread::spawn(move || other.acquire(8, Thread(3)));
    g.shutdown();
    assert_eq!(
        h.join().unwrap(),
        Err(astero_kernel::synchronization::owned::Error::Interrupted)
    );
    assert_eq!(g.snapshot().waiters, 0);
}
#[test]
fn manual_deadline_interrupts_contended_guard() {
    let (e, clock) = TimingEngine::manual(Config {
        max_pending: 64,
        max_snapshot_entries: 64,
    })
    .unwrap();
    let g = Arc::new(Guards::new(e.scheduler(), 2).unwrap());
    g.arm(
        astero_timing::time::Deadline::after(
            e.scheduler().now().unwrap(),
            astero_timing::time::Span::from_millis(1).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    g.acquire(8, Thread(1)).unwrap();
    let other = g.clone();
    let h = std::thread::spawn(move || other.acquire(8, Thread(2)));
    clock
        .advance(astero_timing::time::Span::from_millis(1).unwrap())
        .unwrap();
    assert!(h.join().unwrap().is_err());
    assert_eq!(g.snapshot().waiters, 0);
}

#[test]
fn pure_virtual_stops_without_guest_memory() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    m.ro = true;
    assert_eq!(call(&g, 1, 3, 0, &mut m).0, CallResult::StopRequested);
    assert_eq!(g.snapshot().created, 0);
}
#[test]
fn release_without_acquire_never_publishes() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    assert_eq!(call(&g, 1, 1, 8, &mut m).0, CallResult::Unsupported);
    assert_eq!(m.b[8], 0);
}
#[test]
fn exact_end_of_mapping_guard_is_accepted() {
    let (_e, g) = setup();
    let mut m = Memory::new();
    assert_eq!(call(&g, 1, 0, 248, &mut m).1, 1);
    assert_eq!(call(&g, 1, 1, 248, &mut m).0, CallResult::Returned);
    assert_eq!(m.b[248], 1);
}
