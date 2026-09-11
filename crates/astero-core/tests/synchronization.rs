use astero_abi::layouts::entry::CallFrame;
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::synchronization::owned::*;
use astero_timing::{
    scheduler::{Config, TimingEngine},
    time::{Deadline, Tick},
};
use std::sync::Arc;
struct Memory([u8; 128]);
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, _: bool) -> std::result::Result<(), AccessError> {
        let end = a.checked_add(n).ok_or(AccessError::Range)?;
        self.0
            .get(a as usize..usize::try_from(end).map_err(|_| AccessError::Range)?)
            .map(|_| ())
            .ok_or(AccessError::Range)
    }
    fn read(&self, a: u64, n: u64) -> std::result::Result<Vec<u8>, AccessError> {
        self.0
            .get(a as usize..a.checked_add(n).ok_or(AccessError::Range)? as usize)
            .map(|s| s.to_vec())
            .ok_or(AccessError::Range)
    }
    fn write(&mut self, a: u64, b: &[u8]) -> std::result::Result<(), AccessError> {
        self.0
            .get_mut(a as usize..a as usize + b.len())
            .ok_or(AccessError::Range)?
            .copy_from_slice(b);
        Ok(())
    }
    fn allocate(&mut self, _: u64) -> std::result::Result<u64, AccessError> {
        Err(AccessError::Allocation)
    }
    fn free(&mut self, _: u64) -> std::result::Result<(), AccessError> {
        Err(AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, _: u64) -> std::result::Result<u64, AccessError> {
        Err(AccessError::InvalidAllocation)
    }
}
fn setup() -> (TimingEngine, Arc<Synchronization>, PreparedRegistry, Memory) {
    let e = TimingEngine::real(Config {
        max_pending: 16,
        max_snapshot_entries: 16,
    })
    .unwrap();
    let s = Arc::new(Synchronization::new(e.scheduler(), 16, 16).unwrap());
    s.arm(Deadline::at(Tick::from_nanos(5_000_000_000)))
        .unwrap();
    let r = PreparedRegistry::new(
        astero_libs::pthread::exports::registrations(s.clone(), Thread(1)),
        256,
    )
    .unwrap();
    (e, s, r, Memory([0; 128]))
}
fn call(r: &PreparedRegistry, m: &mut Memory, name: &str, a: [u64; 6]) -> u64 {
    let e = astero_libs::pthread::exports::EXPORTS
        .iter()
        .find(|e| e.name == name)
        .unwrap();
    let k = ProviderKey {
        nid: e.nid,
        library: b"libkernel".to_vec(),
        module: b"libkernel".to_vec(),
    };
    let mut f = CallFrame {
        arguments: a,
        ..Default::default()
    };
    assert_eq!(r.invoke(&k, &mut f, m), Ok(CallResult::Returned));
    f.rax
}
#[test]
fn exact_rwlock_registration_guest_handle_lifecycle() {
    let (_e, s, r, mut m) = setup();
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_RWLOCK_INIT", [8, 0, 0, 0, 0, 0]),
        0
    );
    assert_ne!(m.read(8, 8).unwrap(), [0; 8]);
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_RWLOCK_RDLOCK", [8, 0, 0, 0, 0, 0]),
        0
    );
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_RWLOCK_DESTROY", [8, 0, 0, 0, 0, 0]),
        0x80020010
    );
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_RWLOCK_UNLOCK", [8, 0, 0, 0, 0, 0]),
        0
    );
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_RWLOCK_DESTROY", [8, 0, 0, 0, 0, 0]),
        0
    );
    assert_eq!(m.read(8, 8).unwrap(), [0; 8]);
    assert_eq!(s.snapshot().objects.len(), 0);
    assert_eq!(
        call(&r, &mut m, "PTHREAD_RWLOCK_DESTROY", [8, 0, 0, 0, 0, 0]),
        22
    );
}
#[test]
fn attributes_validate_values_and_drive_recursive_mutex() {
    let (_e, _s, r, mut m) = setup();
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_MUTEXATTR_INIT", [8, 0, 0, 0, 0, 0]),
        0
    );
    assert_eq!(
        call(
            &r,
            &mut m,
            "SCE_PTHREAD_MUTEXATTR_SETTYPE",
            [8, 99, 0, 0, 0, 0]
        ),
        0x80020016
    );
    assert_eq!(
        call(
            &r,
            &mut m,
            "SCE_PTHREAD_MUTEXATTR_SETTYPE",
            [8, 2, 0, 0, 0, 0]
        ),
        0
    );
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_MUTEX_INIT", [16, 8, 0, 0, 0, 0]),
        0
    );
    for _ in 0..2 {
        assert_eq!(
            call(&r, &mut m, "SCE_PTHREAD_MUTEX_LOCK", [16, 0, 0, 0, 0, 0]),
            0
        );
    }
    assert_eq!(
        call(
            &r,
            &mut m,
            "SCE_PTHREAD_MUTEXATTR_SETPROTOCOL",
            [8, 1, 0, 0, 0, 0]
        ),
        0x80020016
    );
}
#[test]
fn invalid_slot_rolls_back_and_zero_is_not_fake_lazy_object() {
    let (_e, s, r, mut m) = setup();
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_COND_INIT", [127, 0, 0, 0, 0, 0]),
        0x80020016
    );
    assert!(s.snapshot().objects.is_empty());
    assert_eq!(
        call(&r, &mut m, "SCE_PTHREAD_MUTEX_LOCK", [8, 0, 0, 0, 0, 0]),
        0x80020016
    );
}
#[test]
fn wrong_provider_context_remains_missing() {
    let (_e, _s, r, _m) = setup();
    let k = ProviderKey {
        nid: 0xe942c06b47eae230,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    assert!(matches!(r.find(&k), Err(RegistryError::Missing)));
}
