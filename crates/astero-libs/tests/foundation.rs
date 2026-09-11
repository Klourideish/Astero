use astero_hle::calls::memory::{AccessError, GuestMemory};
use astero_libs::libc::primitives::call;
use astero_memory::allocation::heap::GuestHeap;
struct Memory {
    bytes: Vec<u8>,
    heap: GuestHeap,
}
impl Memory {
    fn new() -> Self {
        Self {
            bytes: vec![0; 65536],
            heap: GuestHeap::new(32768, 32768, 128).unwrap(),
        }
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, _: bool) -> std::result::Result<(), AccessError> {
        let end = a.checked_add(n).ok_or(AccessError::Range)?;
        self.bytes
            .get(a as usize..usize::try_from(end).map_err(|_| AccessError::Range)?)
            .map(|_| ())
            .ok_or(AccessError::Range)
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        let end = a.checked_add(n).ok_or(AccessError::Range)?;
        self.bytes
            .get(a as usize..end as usize)
            .map(|s| s.to_vec())
            .ok_or(AccessError::Range)
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        let end = a.checked_add(b.len() as u64).ok_or(AccessError::Range)?;
        self.bytes
            .get_mut(a as usize..end as usize)
            .ok_or(AccessError::Range)?
            .copy_from_slice(b);
        Ok(())
    }
    fn allocate(&mut self, n: u64) -> Result<u64, AccessError> {
        self.heap.allocate(n).map_err(|_| AccessError::Allocation)
    }
    fn free(&mut self, a: u64) -> Result<(), AccessError> {
        self.heap
            .free(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, a: u64) -> Result<u64, AccessError> {
        self.heap
            .size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
}
#[test]
fn allocation_preserves_realloc_prefix_and_failure_ownership() {
    let mut m = Memory::new();
    let p = call("malloc", [32, 0, 0, 0, 0, 0], &mut m).unwrap();
    m.write(p, b"owned").unwrap();
    let q = call("realloc", [p, 64, 0, 0, 0, 0], &mut m).unwrap();
    assert_eq!(m.read(q, 5).unwrap(), b"owned");
    assert!(m.free(p).is_err());
    assert_eq!(call("realloc", [q, 65536, 0, 0, 0, 0], &mut m).unwrap(), 0);
    assert_eq!(m.allocation_size(q).unwrap(), 64);
    call("free", [q, 0, 0, 0, 0, 0], &mut m).unwrap();
    assert_eq!(m.heap.snapshot().live, 0);
}
#[test]
fn calloc_overflow_zero_and_reuse() {
    let mut m = Memory::new();
    assert_eq!(
        call("calloc", [u64::MAX, 2, 0, 0, 0, 0], &mut m).unwrap(),
        0
    );
    let p = m.allocate(64).unwrap();
    m.write(p, &[99; 64]).unwrap();
    m.free(p).unwrap();
    assert_eq!(call("calloc", [8, 8, 0, 0, 0, 0], &mut m).unwrap(), p);
    assert_eq!(m.read(p, 64).unwrap(), [0; 64]);
    let z = call("malloc", [0; 6], &mut m).unwrap();
    assert_ne!(z, 0);
    assert_eq!(call("realloc", [z, 0, 0, 0, 0, 0], &mut m).unwrap(), 0);
    assert!(m.free(z).is_err());
}
#[test]
fn memory_overlap_comparison_and_zero_count() {
    let mut m = Memory::new();
    m.write(100, b"abcdef").unwrap();
    assert_eq!(
        call("memmove", [102, 100, 6, 0, 0, 0], &mut m).unwrap(),
        102
    );
    assert_eq!(m.read(100, 8).unwrap(), b"ababcdef");
    call("memset", [100, 255, 2, 0, 0, 0], &mut m).unwrap();
    assert_eq!(call("memcmp", [100, 102, 1, 0, 0, 0], &mut m).unwrap(), 158);
    assert_eq!(call("memcpy", [0, 0, 0, 0, 0, 0], &mut m).unwrap(), 0);
}
#[test]
fn strcmp_does_not_consume_third_argument() {
    let mut m = Memory::new();
    m.write(100, b"abc\0").unwrap();
    m.write(200, b"abd\0").unwrap();
    assert_eq!(
        call("strcmp", [100, 200, 0, 0, 0, 0], &mut m).unwrap() as u32 as i32,
        -1
    );
    assert_eq!(call("strncmp", [100, 200, 2, 0, 0, 0], &mut m).unwrap(), 0);
    assert_eq!(call("strncmp", [0, 0, 0, 0, 0, 0], &mut m).unwrap(), 0);
}
#[test]
fn byte_strings_nul_search_and_limits() {
    let mut m = Memory::new();
    m.write(100, &[255, b'a', 255, 0]).unwrap();
    assert_eq!(call("strlen", [100, 0, 0, 0, 0, 0], &mut m).unwrap(), 3);
    assert_eq!(call("strnlen", [100, 2, 0, 0, 0, 0], &mut m).unwrap(), 2);
    assert_eq!(call("strchr", [100, 255, 0, 0, 0, 0], &mut m).unwrap(), 100);
    assert_eq!(
        call("strrchr", [100, 255, 0, 0, 0, 0], &mut m).unwrap(),
        102
    );
    assert_eq!(call("strchr", [100, 0, 0, 0, 0, 0], &mut m).unwrap(), 103);
    assert!(call("memset", [100, 0, u64::MAX, 0, 0, 0], &mut m).is_err());
    assert!(call("strlen", [65536, 0, 0, 0, 0, 0], &mut m).is_err());
}
#[test]
fn exact_registry_dispatch_and_unknown_refusal() {
    use astero_hle::dispatch::prepared::*;
    let r = PreparedRegistry::new(
        astero_libs::libc::primitives::registrations(b"libc", b"libc"),
        astero_libs::libc::primitives::EXPORTS.len(),
    )
    .unwrap();
    let mut k = ProviderKey {
        nid: 0x8f856258d1c4830c,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    let mut f = astero_abi::layouts::entry::CallFrame::default();
    f.arguments[0] = 100;
    let mut m = Memory::new();
    m.write(100, b"hello\0").unwrap();
    assert_eq!(r.invoke(&k, &mut f, &mut m).unwrap(), CallResult::Returned);
    assert_eq!(f.rax, 5);
    k.module = b"unknown".to_vec();
    assert!(r.invoke(&k, &mut f, &mut m).is_err());
}
#[test]
fn environment_is_guest_resident_and_overwrite_controlled() {
    use astero_hle::dispatch::prepared::*;
    use astero_libs::libc::process::*;
    let r = PreparedRegistry::new(registrations(20, 4096), 12).unwrap();
    let key = |nid| ProviderKey {
        nid,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    let mut m = Memory::new();
    m.write(100, b"NAME\0").unwrap();
    m.write(200, b"value\0").unwrap();
    let mut f = astero_abi::layouts::entry::CallFrame {
        arguments: [100, 200, 1, 0, 0, 0],
        ..Default::default()
    };
    assert_eq!(
        r.invoke(&key(SETENV_NID), &mut f, &mut m).unwrap(),
        CallResult::Returned
    );
    r.invoke(&key(GETENV_NID), &mut f, &mut m).unwrap();
    assert!(f.rax >= 32768);
    assert_eq!(m.read(f.rax, 6).unwrap(), b"value\0");
    assert_eq!(m.heap.snapshot().live, 1);
}
#[test]
fn owned_return_landing_is_distinct_from_arbitrary_host_address() {
    use astero_hle::dispatch::prepared::*;
    let (v, s) = astero_libs::libc::startup::owned_registrations(3, vec![(100, 20)], Some(9000));
    let r = PreparedRegistry::new(v, 2).unwrap();
    let k = ProviderKey {
        nid: astero_libs::libc::startup::ATEXIT_NID,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    let mut f = astero_abi::layouts::entry::CallFrame::default();
    for (a, result) in [(9000, 0), (9001, u32::MAX as u64), (110, 0)] {
        f.arguments[0] = a;
        r.invoke_host_model(&k, &mut f).unwrap();
        assert_eq!(f.rax, result);
    }
    assert_eq!(
        s.lock().unwrap().callbacks().collect::<Vec<_>>(),
        [110, 9000]
    );
}
#[test]
fn cxa_registration_retains_argument_dso_order_and_bound() {
    use astero_hle::dispatch::prepared::*;
    use astero_libs::libc::startup::*;
    let (mut r, s) = owned_registrations(2, vec![(100, 20)], Some(9000));
    r.push(cxa_registration(s.clone(), vec![(100, 20)]));
    let r = PreparedRegistry::new(r, 3).unwrap();
    let k = ProviderKey {
        nid: CXA_ATEXIT_NID,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    let mut f = astero_abi::layouts::entry::CallFrame {
        arguments: [110, 456, 789, 0, 0, 0],
        ..Default::default()
    };
    for expected in [0, 0, u32::MAX as u64] {
        r.invoke_host_model(&k, &mut f).unwrap();
        assert_eq!(f.rax, expected);
    }
    let s = s.lock().unwrap();
    let records: Vec<_> = s.records().collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].argument, Some(456));
    assert_eq!(records[0].dso, Some(789));
}
#[test]
fn process_pointers_and_controlled_termination_are_explicit() {
    use astero_hle::dispatch::prepared::*;
    use astero_libs::libc::process::*;
    let r = PreparedRegistry::new(registrations(24, 4096), 12).unwrap();
    let mut f = astero_abi::layouts::entry::CallFrame::default();
    for (nid, expected) in [(ERROR_NID, 24), (PROCPARAM_NID, 4096)] {
        let k = ProviderKey {
            nid,
            library: b"libkernel".to_vec(),
            module: b"libkernel".to_vec(),
        };
        assert_eq!(
            r.invoke_host_model(&k, &mut f).unwrap(),
            CallResult::Returned
        );
        assert_eq!(f.rax, expected);
    }
    let k = ProviderKey {
        nid: TERMINATIONS[0].1,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    assert_eq!(
        r.invoke_host_model(&k, &mut f).unwrap(),
        CallResult::StopRequested
    );
    let object = astero_libs::libc::startup::stack_guard(8192);
    assert!(r.find(&object.key).is_err());
    assert_eq!(object.address_with_addend(7), Some(8199));
    assert_eq!(object.address_with_addend(8), None);
    assert_eq!(object.address_with_addend(-1), None);
}

#[test]
fn new_never_returns_null_success_on_exhaustion() {
    let mut m = Memory::new();
    let p = call("operator new", [48, 0, 0, 0, 0, 0], &mut m).unwrap();
    assert!(p.is_multiple_of(16));
    assert!(call("operator new", [u64::MAX, 0, 0, 0, 0, 0], &mut m).is_err());
    call("operator delete", [p, 0, 0, 0, 0, 0], &mut m).unwrap();
    assert_eq!(m.heap.snapshot().live, 0);
}
