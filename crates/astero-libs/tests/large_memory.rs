use astero_hle::calls::{budget::*, memory::*};
use astero_libs::libc::primitives::call;
use astero_memory::allocation::heap::GuestHeap;
struct Memory {
    bytes: Vec<u8>,
    fail_at: Option<u64>,
    denied: Option<(u64, u64, bool)>,
    heap: GuestHeap,
    budget: AccessBudget,
}
impl Memory {
    fn new() -> Self {
        Self {
            bytes: vec![0; 8 * 1024 * 1024],
            fail_at: None,
            denied: None,
            heap: GuestHeap::new(4 * 1024 * 1024, 4 * 1024 * 1024, 128).unwrap(),
            budget: AccessBudget::new(AccessLimits {
                max_operation_bytes: MAX_OPERATION_BYTES,
                max_total_bytes: 512 * 1024 * 1024,
            }),
        }
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, w: bool) -> Result<(), AccessError> {
        let end = a.checked_add(n).ok_or(AccessError::Range)?;
        if n == 0 {
            return Ok(());
        }
        if end > self.bytes.len() as u64 {
            return Err(AccessError::Range);
        }
        if let Some((lo, hi, ro)) = self.denied
            && a < hi
            && end > lo
            && (!ro || w)
        {
            return Err(AccessError::Range);
        }
        Ok(())
    }
    fn charge(&self, n: u64) -> Result<(), AccessError> {
        self.budget.charge(n)
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        assert!(n <= COPY_CHUNK_BYTES);
        self.validate(a, n, false)?;
        Ok(self.bytes[a as usize..(a + n) as usize].to_vec())
    }
    fn read_window(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.read(a, n.min(4096 - a % 4096))
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        assert!(b.len() as u64 <= COPY_CHUNK_BYTES);
        if self.fail_at.is_some_and(|at| a >= at) {
            return Err(AccessError::HostCopy {
                operation: "write",
                address: a,
                size: b.len() as u64,
                code: 299,
            });
        }
        self.validate(a, b.len() as u64, true)?;
        self.bytes[a as usize..a as usize + b.len()].copy_from_slice(b);
        Ok(())
    }
    fn allocate(&mut self, n: u64) -> Result<u64, AccessError> {
        self.heap.allocate(n).map_err(|_| AccessError::Allocation)
    }
    fn allocate_aligned(&mut self, n: u64, a: u64) -> Result<u64, AccessError> {
        self.heap
            .allocate_aligned(n, a)
            .map_err(|_| AccessError::Allocation)
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
    fn usable_size(&self, a: u64) -> Result<u64, AccessError> {
        self.heap
            .usable_size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
}
#[test]
fn exact_and_observed_large_fill_use_bounded_scratch() {
    for n in [1024 * 1024, 1024 * 1024 + 1, 1_352_000] {
        let mut m = Memory::new();
        assert_eq!(call("memset", [17, 0xab, n, 0, 0, 0], &mut m).unwrap(), 17);
        assert!(m.bytes[17..17 + n as usize].iter().all(|b| *b == 0xab));
        assert_eq!(m.bytes[16], 0);
        assert_eq!(m.bytes[17 + n as usize], 0);
    }
}
#[test]
fn invalid_tail_and_readonly_preflight_leave_destination_unchanged() {
    for ro in [false, true] {
        let mut m = Memory::new();
        m.denied = Some((1_300_000, 1_400_000, ro));
        assert_eq!(
            call("memset", [0, 42, 1_352_000, 0, 0, 0], &mut m),
            Err(AccessError::Range)
        );
        assert!(m.bytes.iter().all(|b| *b == 0));
    }
}
#[test]
fn overflow_and_budget_refuse_before_mutation() {
    let mut m = Memory::new();
    assert_eq!(
        call("memset", [u64::MAX - 1, 1, 8, 0, 0, 0], &mut m),
        Err(AccessError::Range)
    );
    assert_eq!(
        call("memset", [0, 1, MAX_OPERATION_BYTES + 1, 0, 0, 0], &mut m),
        Err(AccessError::Limit)
    );
    assert!(m.bytes.iter().all(|b| *b == 0));
}
#[test]
fn large_copy_compare_and_search() {
    let mut m = Memory::new();
    for (i, b) in m.bytes[..1_352_000].iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }
    call("memcpy", [2_000_000, 0, 1_352_000, 0, 0, 0], &mut m).unwrap();
    assert_eq!(
        call("memcmp", [0, 2_000_000, 1_352_000, 0, 0, 0], &mut m).unwrap(),
        0
    );
    m.bytes[3_300_000] = 255;
    assert_eq!(
        call("memchr", [2_000_000, 255, 1_352_000, 0, 0, 0], &mut m).unwrap(),
        3_300_000
    );
}
#[test]
fn memmove_both_directions_cross_chunks_and_memcpy_rejects_overlap() {
    for (dst, src) in [(117, 16), (16, 117)] {
        let mut m = Memory::new();
        for (i, b) in m.bytes[..1_600_000].iter_mut().enumerate() {
            *b = (i % 251) as u8;
        }
        let mut expected = m.bytes.clone();
        expected.copy_within(src..src + 1_352_000, dst);
        assert_eq!(
            call(
                "memcpy",
                [dst as u64, src as u64, 1_352_000, 0, 0, 0],
                &mut m
            ),
            Err(AccessError::Overlap)
        );
        call(
            "memmove",
            [dst as u64, src as u64, 1_352_000, 0, 0, 0],
            &mut m,
        )
        .unwrap();
        assert_eq!(m.bytes, expected);
    }
}
#[test]
fn page_end_nul_does_not_require_next_page() {
    let mut m = Memory::new();
    m.bytes[4093..4096].copy_from_slice(b"ab\0");
    m.denied = Some((4096, 8192, false));
    assert_eq!(call("strlen", [4093, 0, 0, 0, 0, 0], &mut m).unwrap(), 2);
    assert_eq!(call("strchr", [4093, 0, 0, 0, 0, 0], &mut m).unwrap(), 4095);
}
#[test]
fn copy_concat_padding_and_destination_tail_failure() {
    let mut m = Memory::new();
    m.bytes[100..104].copy_from_slice(b"abc\0");
    call("strcpy", [1000, 100, 0, 0, 0, 0], &mut m).unwrap();
    call("strncat", [1000, 100, 2, 0, 0, 0], &mut m).unwrap();
    assert_eq!(&m.bytes[1000..1006], b"abcab\0");
    call("strncpy", [2000, 100, 6, 0, 0, 0], &mut m).unwrap();
    assert_eq!(&m.bytes[2000..2006], b"abc\0\0\0");
    m.denied = Some((3002, 4000, true));
    assert!(call("strcpy", [3000, 100, 0, 0, 0, 0], &mut m).is_err());
    assert_eq!(&m.bytes[3000..3002], &[0, 0]);
}
#[test]
fn case_span_and_linear_substring_contracts() {
    let mut m = Memory::new();
    m.bytes[100..106].copy_from_slice(b"Ababa\0");
    m.bytes[200..206].copy_from_slice(b"abABA\0");
    m.bytes[300..303].copy_from_slice(b"ba\0");
    assert_eq!(
        call("strcasecmp", [100, 200, 0, 0, 0, 0], &mut m).unwrap(),
        0
    );
    assert_eq!(call("strstr", [100, 300, 0, 0, 0, 0], &mut m).unwrap(), 101);
    assert_eq!(call("strcspn", [100, 300, 0, 0, 0, 0], &mut m).unwrap(), 1);
    assert_eq!(call("strspn", [200, 300, 0, 0, 0, 0], &mut m).unwrap(), 2);
}
#[test]
fn tokenizer_state_is_caller_owned_and_dup_is_heap_owned() {
    let mut m = Memory::new();
    m.bytes[100..107].copy_from_slice(b",aa,bb\0");
    m.bytes[200..202].copy_from_slice(b",\0");
    assert_eq!(
        call("strtok_r", [100, 200, 300, 0, 0, 0], &mut m).unwrap(),
        101
    );
    assert_eq!(
        call("strtok_r", [0, 200, 300, 0, 0, 0], &mut m).unwrap(),
        104
    );
    assert_eq!(call("strtok_r", [0, 200, 300, 0, 0, 0], &mut m).unwrap(), 0);
    let p = call("strdup", [101, 0, 0, 0, 0, 0], &mut m).unwrap();
    assert_eq!(m.read(p, 3).unwrap(), b"aa\0");
    assert_eq!(m.heap.size(p).unwrap(), 3);
}
#[test]
fn aligned_allocation_preserves_output_on_failure_and_owns_usable_bytes() {
    let mut m = Memory::new();
    m.bytes[8..16].copy_from_slice(&99u64.to_le_bytes());
    assert_eq!(
        call("posix_memalign", [8, 3, 32, 0, 0, 0], &mut m).unwrap(),
        22
    );
    assert_eq!(m.read(8, 8).unwrap(), 99u64.to_le_bytes());
    assert_eq!(
        call("posix_memalign", [8, 4096, 17, 0, 0, 0], &mut m).unwrap(),
        0
    );
    let p = u64::from_le_bytes(m.read(8, 8).unwrap().try_into().unwrap());
    assert_eq!(p % 4096, 0);
    assert_eq!(
        call("malloc_usable_size", [p, 0, 0, 0, 0, 0], &mut m).unwrap(),
        32
    );
    m.free(p).unwrap();
    assert_eq!(
        call("aligned_alloc", [32, 33, 0, 0, 0, 0], &mut m).unwrap(),
        0
    );
}
#[test]
fn process_budget_is_atomic_and_refusal_preserves_remaining() {
    use std::sync::Arc;
    let b = Arc::new(AccessBudget::new(AccessLimits {
        max_operation_bytes: 10,
        max_total_bytes: 100,
    }));
    let hs: Vec<_> = (0..20)
        .map(|_| {
            let b = b.clone();
            std::thread::spawn(move || b.charge(10).is_ok())
        })
        .collect();
    assert_eq!(
        hs.into_iter()
            .filter_map(|h| h.join().unwrap().then_some(()))
            .count(),
        10
    );
    assert_eq!(b.snapshot().charged_bytes, 100);
    assert!(b.charge(1).is_err());
}

#[test]
fn puts_retains_raw_bytes_and_refuses_output_capacity_atomically() {
    use astero_hle::dispatch::prepared::*;
    use astero_kernel::process::output::Output;
    use std::sync::{Arc, Mutex};
    let sink = Arc::new(Mutex::new(Output::new(4)));
    let reg = astero_libs::libc::output::registration(sink.clone());
    let key = reg.key.clone();
    let registry = PreparedRegistry::new(vec![reg], 1).unwrap();
    let mut m = Memory::new();
    m.bytes[100..103].copy_from_slice(&[255, b'a', 0]);
    let mut f = astero_abi::layouts::entry::CallFrame::default();
    f.arguments[0] = 100;
    assert_eq!(
        registry.invoke(&key, &mut f, &mut m).unwrap(),
        CallResult::Returned
    );
    assert_eq!(f.rax, 0);
    assert_eq!(sink.lock().unwrap().bytes(), &[255, b'a', b'\n']);
    assert_eq!(
        registry.invoke(&key, &mut f, &mut m).unwrap(),
        CallResult::AccessFailure(AccessError::Limit)
    );
    assert_eq!(sink.lock().unwrap().bytes().len(), 3);
}

#[test]
fn checked_string_family_preserves_constraint_clearing_and_count_semantics() {
    let mut m = Memory::new();
    m.bytes[100..104].copy_from_slice(b"abc\0");
    assert_eq!(
        call("strcpy_s", [1000, 4, 100, 0, 0, 0], &mut m).unwrap(),
        0
    );
    assert_eq!(&m.bytes[1000..1004], b"abc\0");
    assert_eq!(
        call("strcat_s", [1000, 4, 100, 0, 0, 0], &mut m).unwrap(),
        34
    );
    assert_eq!(m.bytes[1000], 0);
    assert_eq!(
        call("strncpy_s", [1000, 3, 100, 2, 0, 0], &mut m).unwrap(),
        0
    );
    assert_eq!(&m.bytes[1000..1003], b"ab\0");
    assert_eq!(
        call("strncat_s", [1000, 6, 100, 2, 0, 0], &mut m).unwrap(),
        0
    );
    assert_eq!(&m.bytes[1000..1005], b"abab\0");
    assert_eq!(call("strcpy_s", [1000, 8, 0, 0, 0, 0], &mut m).unwrap(), 22);
    assert_eq!(m.bytes[1000], 0);
}
#[test]
fn checked_large_memory_family_and_explicit_error_effects() {
    let mut m = Memory::new();
    assert_eq!(
        call("memset_s", [100, 1_352_000, 77, 1_352_000, 0, 0], &mut m).unwrap(),
        0
    );
    assert_eq!(
        call(
            "memcpy_s",
            [2_000_000, 1_352_000, 100, 1_352_000, 0, 0],
            &mut m
        )
        .unwrap(),
        0
    );
    assert!(m.bytes[2_000_000..3_352_000].iter().all(|b| *b == 77));
    assert_eq!(
        call("memset_s", [100, 32, 99, 33, 0, 0], &mut m).unwrap(),
        34
    );
    assert!(m.bytes[100..132].iter().all(|b| *b == 0));
    assert_eq!(call("memcpy_s", [100, 16, 0, 8, 0, 0], &mut m).unwrap(), 22);
}

#[test]
fn aligned_registry_errors_set_only_own_thread_errno() {
    use astero_hle::dispatch::prepared::*;
    let regs = astero_libs::libc::primitives::registrations_with_errno(b"libc", b"libc", Some(32));
    let reg = PreparedRegistry::new(regs, 64).unwrap();
    let key = ProviderKey {
        nid: 0xd81b6483c936e198,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    };
    let mut m = Memory::new();
    let mut f = astero_abi::layouts::entry::CallFrame {
        arguments: [3, 32, 0, 0, 0, 0],
        ..Default::default()
    };
    assert_eq!(
        reg.invoke(&key, &mut f, &mut m).unwrap(),
        CallResult::Returned
    );
    assert_eq!(m.read(32, 4).unwrap(), 22u32.to_le_bytes());
    assert_eq!(m.read(36, 4).unwrap(), 0u32.to_le_bytes());
    f.arguments = [16, 8 * 1024 * 1024, 0, 0, 0, 0];
    assert_eq!(
        reg.invoke(&key, &mut f, &mut m).unwrap(),
        CallResult::Returned
    );
    assert_eq!(f.rax, 0);
    assert_eq!(m.read(32, 4).unwrap(), 12u32.to_le_bytes());
}

#[test]
fn post_preflight_host_failure_reports_failure_without_claiming_rollback() {
    let mut m = Memory::new();
    m.fail_at = Some(COPY_CHUNK_BYTES);
    assert_eq!(
        call("memset", [0, 42, 2 * COPY_CHUNK_BYTES, 0, 0, 0], &mut m),
        Err(AccessError::HostCopy {
            operation: "write",
            address: COPY_CHUNK_BYTES,
            size: COPY_CHUNK_BYTES,
            code: 299
        })
    );
    assert!(
        m.bytes[..COPY_CHUNK_BYTES as usize]
            .iter()
            .all(|b| *b == 42)
    );
    assert!(m.bytes[COPY_CHUNK_BYTES as usize..].iter().all(|b| *b == 0));
}
