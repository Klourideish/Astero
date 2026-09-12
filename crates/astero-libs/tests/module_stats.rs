use astero_hle::calls::memory::*;
use astero_libs::libc::stats::query;
use astero_memory::allocation::heap::GuestHeap;
struct Memory {
    b: Vec<u8>,
    heap: GuestHeap,
}
impl Memory {
    fn new() -> Self {
        let mut b = vec![0xaa; 128];
        b[8..12].copy_from_slice(&0x10028u32.to_le_bytes());
        Self {
            b,
            heap: GuestHeap::new(4096, 4096, 32).unwrap(),
        }
    }
    fn field(&self, o: usize) -> u64 {
        u64::from_le_bytes(self.b[8 + o..16 + o].try_into().unwrap())
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, _: bool) -> Result<(), AccessError> {
        if a > 0 && a.checked_add(n).is_some_and(|e| e <= self.b.len() as u64) {
            Ok(())
        } else {
            Err(AccessError::Range)
        }
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.validate(a, n, false)?;
        Ok(self.b[a as usize..(a + n) as usize].to_vec())
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        self.validate(a, b.len() as u64, true)?;
        self.b[a as usize..a as usize + b.len()].copy_from_slice(b);
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
    fn heap_stats(&self) -> Result<HeapStats, AccessError> {
        let s = self.heap.snapshot();
        Ok(HeapStats {
            arena_bytes: s.arena_bytes,
            live_bytes: s.live_bytes,
            peak_bytes: s.peak_bytes,
            live: s.live,
            peak_live: s.peak_live,
        })
    }
}
#[test]
fn empty_arena_stats() {
    let mut m = Memory::new();
    assert_eq!(query(&mut m, 8).unwrap(), 0);
    assert_eq!(m.field(16), 4096);
    assert_eq!(m.field(32), 0);
}
#[test]
fn allocated_freed_and_peak() {
    let mut m = Memory::new();
    let p = m.allocate(17).unwrap();
    query(&mut m, 8).unwrap();
    assert_eq!(m.field(32), 32);
    m.free(p).unwrap();
    query(&mut m, 8).unwrap();
    assert_eq!(m.field(24), 32);
    assert_eq!(m.field(32), 0);
}
#[test]
fn no_partial_invalid_output() {
    let mut m = Memory::new();
    let b = m.b.clone();
    assert!(query(&mut m, 100).is_err());
    assert_eq!(m.b, b);
}
#[test]
fn unknown_version_unchanged() {
    let mut m = Memory::new();
    m.b[8] = 0;
    let b = m.b.clone();
    assert_eq!(query(&mut m, 8).unwrap(), 22);
    assert_eq!(m.b, b);
}
#[test]
fn boundary_and_reserved_bytes() {
    let mut m = Memory::new();
    m.b.truncate(48);
    query(&mut m, 8).unwrap();
    assert_eq!(&m.b[12..16], &[0; 4]);
    assert_eq!(&m.b[..8], &[0xaa; 8]);
}
#[test]
fn null_refused() {
    assert!(query(&mut Memory::new(), 0).is_err());
}
#[test]
fn repeated_query_deterministic() {
    let mut m = Memory::new();
    query(&mut m, 8).unwrap();
    let b = m.b.clone();
    query(&mut m, 8).unwrap();
    assert_eq!(m.b, b);
}
#[test]
fn concurrent_coherent_queries() {
    let m = std::sync::Arc::new(std::sync::Mutex::new(Memory::new()));
    let ts: Vec<_> = (0..4)
        .map(|_| {
            let m = m.clone();
            std::thread::spawn(move || {
                for _ in 0..20 {
                    let mut m = m.lock().unwrap();
                    let a = m.allocate(32).unwrap();
                    query(&mut *m, 8).unwrap();
                    assert!(m.field(16) >= m.field(32));
                    m.free(a).unwrap();
                }
            })
        })
        .collect();
    for t in ts {
        t.join().unwrap();
    }
    assert_eq!(m.lock().unwrap().heap.snapshot().live, 0);
}

#[test]
fn sysmodule_unknown_and_invalid_ids_return_errors() {
    let owner = std::sync::Arc::new(astero_hle::providers::modules::Modules::new(8));
    let r = astero_libs::sysmodule::registrations(owner.clone());
    let mut m = Memory::new();
    let mut f = astero_abi::layouts::entry::CallFrame::default();
    f.arguments[0] = 267;
    assert_eq!(r.len(), 7);
    for i in [0, 3] {
        r[i].handler.as_ref().unwrap()(&mut f, &mut m);
        assert_eq!(f.rax, 0x80020002);
    }
    f.arguments[0] = 65536;
    r[0].handler.as_ref().unwrap()(&mut f, &mut m);
    assert_eq!(f.rax, 0x80020003);
    assert!(owner.snapshot().records.is_empty());
}
#[test]
fn internal_arg_output_is_preflighted() {
    let owner = std::sync::Arc::new(astero_hle::providers::modules::Modules::new(8));
    let r = astero_libs::sysmodule::registrations(owner.clone());
    let mut m = Memory::new();
    let before = m.b.clone();
    let mut f = astero_abi::layouts::entry::CallFrame {
        arguments: [1, 0, 0, 0, 127, 0],
        ..Default::default()
    };
    r[4].handler.as_ref().unwrap()(&mut f, &mut m);
    assert_eq!(f.rax, 0x80020003);
    assert_eq!(m.b, before);
    assert_eq!(owner.snapshot().loads, 0);
}
