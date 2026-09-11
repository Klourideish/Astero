use astero_abi::layouts::entry::CallFrame;
use astero_audio::codecs::ajm::{Ajm, Error};
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::synchronization::owned::{Synchronization, Thread};
use astero_timing::{
    scheduler::{Config, TimingEngine},
    time::{Deadline, Tick},
};
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
fn setup() -> (TimingEngine, Arc<Synchronization>) {
    let (e, _) = TimingEngine::manual(Config {
        max_pending: 32,
        max_snapshot_entries: 32,
    })
    .unwrap();
    let s = Arc::new(Synchronization::new(e.scheduler(), 16, 16).unwrap());
    s.arm(Deadline::at(Tick::from_nanos(1000000))).unwrap();
    (e, s)
}
#[test]
fn ajm_context_stale_and_capacity() {
    let s = Ajm::new(1);
    let a = s.initialize().unwrap();
    assert_eq!(s.initialize(), Err(Error::Capacity));
    s.finalize(a).unwrap();
    let b = s.initialize().unwrap();
    assert_ne!(a, b);
    assert_eq!(s.finalize(a), Err(Error::InvalidContext));
}
#[test]
fn ajm_instance_requires_registered_codec() {
    let s = Ajm::new(8);
    let c = s.initialize().unwrap();
    assert_eq!(s.create(c, 1, 0), Err(Error::InvalidParameter));
    s.module(c, 1, true).unwrap();
    let i = s.create(c, 1, 123).unwrap();
    assert_eq!(s.module(c, 1, false), Err(Error::InvalidParameter));
    s.destroy(c, i).unwrap();
    s.module(c, 1, false).unwrap();
}
#[test]
fn ajm_context_identity() {
    let s = Ajm::new(8);
    let a = s.initialize().unwrap();
    let b = s.initialize().unwrap();
    s.module(a, 1, true).unwrap();
    let i = s.create(a, 1, 0).unwrap();
    assert_eq!(s.destroy(b, i), Err(Error::InvalidParameter));
    s.destroy(a, i).unwrap();
}
#[test]
fn ajm_memory_ranges_and_shutdown() {
    let s = Ajm::new(8);
    let c = s.initialize().unwrap();
    assert_eq!(s.memory(c, u64::MAX, 1, true), Err(Error::InvalidParameter));
    s.memory(c, 16, 32, true).unwrap();
    assert_eq!(s.snapshot().memories, 1);
    s.shutdown();
    assert_eq!(s.snapshot().memories, 0);
    assert_eq!(s.initialize(), Err(Error::Stopped));
}
#[test]
fn ajm_publication_rollback() {
    let s = Arc::new(Ajm::new(8));
    let r = astero_libs::media::ajm::registrations(s.clone());
    let mut m = Memory::new();
    m.fail = true;
    assert!(matches!(
        invoke(&r, 0, [0, 8, 0, 0, 0, 0], &mut m).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().contexts, 0);
}
#[test]
fn ajm_invalid_output_is_not_published() {
    let s = Arc::new(Ajm::new(8));
    let r = astero_libs::media::ajm::registrations(s.clone());
    assert!(matches!(
        invoke(&r, 0, [0, 126, 0, 0, 0, 0], &mut Memory::new()).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().contexts, 0);
}
#[test]
fn ajm_exact_identity() {
    let r = astero_libs::media::ajm::registrations(Arc::new(Ajm::new(8)));
    assert!(
        r.iter()
            .all(|x| x.key.library == b"libSceAjm" && x.key.module == b"libSceAjm")
    );
    assert_eq!(r.len(), 9);
}
#[test]
fn at9_known_config() {
    assert_eq!(
        astero_audio::codecs::atrac9::configuration([0xfe, 0x74, 0x0b, 0xf0]).unwrap(),
        [2, 48000, 256, 1024, 384]
    );
}
#[test]
fn at9_reserved_channel_refuses() {
    for channel in [6, 7] {
        let b = (0xfe700000u32 | (channel << 17)).to_be_bytes();
        assert!(astero_audio::codecs::atrac9::configuration(b).is_err());
    }
}
#[test]
fn at9_output_exact_boundary() {
    let r = astero_libs::media::ajm::registrations(Arc::new(Ajm::new(8)));
    let mut m = Memory::new();
    m.write(8, &[0xfe, 0x74, 0x0b, 0xf0]).unwrap();
    assert_eq!(
        invoke(&r, 8, [8, 108, 0, 0, 0, 0], &mut m),
        (CallResult::Returned, 0)
    );
    assert_eq!(m.read(108, 4).unwrap(), 2u32.to_le_bytes());
    let before = m.bytes.clone();
    assert!(matches!(
        invoke(&r, 8, [8, 109, 0, 0, 0, 0], &mut m).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(before, m.bytes);
}
#[test]
fn c11_mutex_lock_error_mapping() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s, Thread(1));
    let mut m = Memory::new();
    assert_eq!(
        invoke(&r, 0, [8, 0, 0, 0, 0, 0], &mut m),
        (CallResult::Returned, 0)
    );
    assert_eq!(invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).1, 0);
    assert_eq!(invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).1, 3);
    assert_eq!(invoke(&r, 2, [8, 0, 0, 0, 0, 0], &mut m).1, 0);
    assert_eq!(invoke(&r, 3, [8, 0, 0, 0, 0, 0], &mut m).1, 0);
    assert_eq!(invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).1, 4);
}
#[test]
fn c11_wrong_owner_does_not_unlock() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    let other = astero_libs::libc::c11::registrations(s, Thread(2));
    let mut m = Memory::new();
    invoke(&r, 0, [8, 0, 0, 0, 0, 0], &mut m);
    invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m);
    invoke(&other, 2, [8, 0, 0, 0, 0, 0], &mut m);
    assert_eq!(invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).1, 3);
}
#[test]
fn c11_busy_destroy_retains_identity() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    invoke(&r, 0, [8, 0, 0, 0, 0, 0], &mut m);
    let before = m.bytes.clone();
    invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m);
    invoke(&r, 3, [8, 0, 0, 0, 0, 0], &mut m);
    assert_eq!(m.bytes, before);
    assert_eq!(s.snapshot().objects.len(), 1);
}
#[test]
fn c11_condition_lifecycle() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    for op in [4, 5, 6, 8] {
        assert_eq!(
            invoke(&r, op, [8, 0, 0, 0, 0, 0], &mut m),
            (CallResult::Returned, 0)
        );
    }
    assert_eq!(s.snapshot().objects.len(), 0);
}
#[test]
fn c11_failed_write_rolls_back() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    m.fail = true;
    assert!(matches!(
        invoke(&r, 4, [8, 0, 0, 0, 0, 0], &mut m).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().objects.len(), 0);
}
#[test]
fn c11_shutdown_stops_instead_of_success() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    let mut m = Memory::new();
    invoke(&r, 0, [8, 0, 0, 0, 0, 0], &mut m);
    s.shutdown();
    assert_eq!(
        invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).0,
        CallResult::StopRequested
    );
}
#[test]
fn c11_unknown_flags_refuse() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    assert_eq!(
        invoke(&r, 0, [8, u64::MAX, 0, 0, 0, 0], &mut Memory::new()).0,
        CallResult::Unsupported
    );
    assert_eq!(s.snapshot().objects.len(), 0);
}

#[test]
fn c11_wait_releases_and_reacquires_shared_mutex() {
    let (_e, s) = setup();
    let mut m = Memory::new();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    invoke(&r, 0, [8, 0, 0, 0, 0, 0], &mut m);
    invoke(&r, 4, [16, 0, 0, 0, 0, 0], &mut m);
    let id = u64::from_le_bytes(m.read(8, 8).unwrap().try_into().unwrap());
    let cond = u64::from_le_bytes(m.read(16, 8).unwrap().try_into().unwrap());
    invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m);
    let worker = s.clone();
    let j = std::thread::spawn(move || {
        let r = astero_libs::libc::c11::registrations(worker.clone(), Thread(1));
        let outcome = invoke(&r, 7, [16, 8, 0, 0, 0, 0], &mut m);
        assert!(worker.mutex_require_owner(id, 8, Thread(1)).is_ok());
        outcome
    });
    let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while s.snapshot().waiters == 0 {
        assert!(std::time::Instant::now() < until);
        std::thread::yield_now();
    }
    s.mutex_lock(id, 8, Thread(2), true, None).unwrap();
    s.cond_signal(cond, 16, false).unwrap();
    s.mutex_unlock(id, 8, Thread(2)).unwrap();
    assert_eq!(j.join().unwrap(), (CallResult::Returned, 0));
    assert_eq!(s.snapshot().waiters, 0);
}
#[test]
fn c11_blocked_wait_shutdown_releases_ticket() {
    let (_e, s) = setup();
    let mut m = Memory::new();
    let r = astero_libs::libc::c11::registrations(s.clone(), Thread(1));
    invoke(&r, 0, [8, 0, 0, 0, 0, 0], &mut m);
    invoke(&r, 4, [16, 0, 0, 0, 0, 0], &mut m);
    invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m);
    let worker = s.clone();
    let j = std::thread::spawn(move || {
        let r = astero_libs::libc::c11::registrations(worker, Thread(1));
        invoke(&r, 7, [16, 8, 0, 0, 0, 0], &mut m)
    });
    let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while s.snapshot().waiters == 0 {
        assert!(std::time::Instant::now() < until);
        std::thread::yield_now();
    }
    s.shutdown();
    assert_eq!(j.join().unwrap().0, CallResult::StopRequested);
    assert_eq!(s.snapshot().waiters, 0);
}

#[test]
fn c11_mode_two_uses_prototype_errorcheck_policy() {
    let (_e, s) = setup();
    let r = astero_libs::libc::c11::registrations(s, Thread(1));
    let mut m = Memory::new();
    assert_eq!(
        invoke(&r, 0, [8, 2, 0, 0, 0, 0], &mut m),
        (CallResult::Returned, 0)
    );
    assert_eq!(invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).1, 0);
    assert_eq!(invoke(&r, 1, [8, 0, 0, 0, 0, 0], &mut m).1, 3);
}
