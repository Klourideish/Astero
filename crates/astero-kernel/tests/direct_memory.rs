#![cfg(all(windows, target_arch = "x86_64"))]
use astero_kernel::objects::memory::*;
use astero_memory::mapping::GuestAddress;
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[test]
fn reserve_then_map_type_twelve() {
    let _l = SERIAL.lock().unwrap();
    let s = MemoryResources::new();
    let v = s.reserve(0, 1048576, 0, 262144).unwrap();
    let p = s.allocate(0, DIRECT_SIZE, 1048576, 0, 12).unwrap();
    assert_eq!(s.map(v, 1048576, 0xf2, 16, p, 0).unwrap(), v);
    assert_eq!(s.query(v, false).unwrap().memory_type, 12);
    s.with_regions(|r| {
        r[0].write(GuestAddress(v + 1048576 - 32), &[1; 32])
            .unwrap()
    });
    s.shutdown();
    assert_eq!(s.snapshot().active_native_resources, 0);
}
#[test]
fn virtual_reservation_is_inaccessible_and_released() {
    let _l = SERIAL.lock().unwrap();
    let s = MemoryResources::new();
    let v = s.reserve(0, 1048576, 0, 262144).unwrap();
    assert_eq!(v % 262144, 0);
    assert!(!s.query(v, false).unwrap().direct);
    s.with_regions(|r| {
        assert_eq!(r[0].snapshot().committed_bytes, 0);
        assert!(r[0].read(GuestAddress(v), 1).is_err());
    });
    s.unmap(v, 1048576).unwrap();
    assert_eq!(s.snapshot().active_native_resources, 0);
}
#[test]
fn allocation_search_and_reuse() {
    let s = MemoryResources::new();
    let a = s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap();
    let b = s.allocate(0, DIRECT_SIZE, 65536, 65536, 3).unwrap();
    assert_eq!((a, b), (0, 65536));
    s.release(a, 65536).unwrap();
    assert_eq!(s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap(), 0);
    assert_eq!(s.query_direct(b, false).unwrap().memory_type, 3);
}
#[test]
fn invalid_ranges_and_budget() {
    let s = MemoryResources::new();
    assert!(s.allocate(0, DIRECT_SIZE, 65536, 123, 0).is_err());
    assert!(s.allocate(0, DIRECT_SIZE, 0, 65536, 0).is_err());
    assert!(
        s.allocate(0, DIRECT_SIZE, COMMIT_LIMIT + 16384, 65536, 0)
            .is_err()
    );
    assert!(
        s.allocate(DIRECT_SIZE - 16384, DIRECT_SIZE, 65536, 65536, 0)
            .is_err()
    );
    assert!(s.snapshot().allocations.is_empty());
}
#[test]
fn mapped_release_retains_backing_until_last_view() {
    let _l = SERIAL.lock().unwrap();
    let s = MemoryResources::new();
    let a = s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap();
    let v = s.map(0, 65536, 3, 0, a, 65536).unwrap();
    s.release(a, 65536).unwrap();
    assert!(s.query_direct(a, false).is_err());
    assert!(s.release(a, 65536).is_err());
    assert!(s.map(0, 65536, 3, 0, a, 65536).is_err());
    assert_eq!(s.query(v, false).unwrap().offset, a);
    s.unmap(v, 65536).unwrap();
    assert!(s.snapshot().allocations.is_empty());
    assert_eq!(s.snapshot().active_native_resources, 0);
}
#[test]
fn view_protection_name_query_and_unmap() {
    let _l = SERIAL.lock().unwrap();
    let s = MemoryResources::new();
    let a = s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap();
    let v = s.map(0, 65536, 3, 0, a, 65536).unwrap();
    s.name(v, 65536, b"resource").unwrap();
    s.with_regions(|r| r[0].write(GuestAddress(v), &[7]).unwrap());
    s.protect(v, 65536, 1).unwrap();
    assert_eq!(s.query(v + 10, false).unwrap().protection, 1);
    s.with_regions(|r| {
        assert_eq!(r[0].read(GuestAddress(v), 1).unwrap(), [7]);
        assert!(r[0].write(GuestAddress(v), &[8]).is_err())
    });
    s.unmap(v, 65536).unwrap();
    assert!(s.unmap(v, 65536).is_err());
    let v = s.map(0, 65536, 3, 0, a, 65536).unwrap();
    s.with_regions(|r| assert_eq!(r[0].read(GuestAddress(v), 1).unwrap(), [7]));
    s.shutdown();
    assert_eq!(s.snapshot().active_native_resources, 0);
    assert!(s.snapshot().release_errors.is_empty());
}
#[test]
fn overlap_and_partial_unmap_refuse() {
    let _l = SERIAL.lock().unwrap();
    let s = MemoryResources::new();
    let a = s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap();
    let v = s.map(0, 65536, 3, 0, a, 65536).unwrap();
    assert!(s.map(v, 65536, 3, 16, a, 65536).is_err());
    assert!(s.unmap(v, 16384).is_err());
    assert_eq!(s.snapshot().mappings.len(), 1);
    s.shutdown();
}
#[test]
fn available_tracks_allocations() {
    let s = MemoryResources::new();
    assert_eq!(s.available(0, 65536, 16384).unwrap(), (0, 65536));
    s.allocate(0, 65536, 16384, 16384, 0).unwrap();
    assert_eq!(s.available(0, 65536, 16384).unwrap(), (16384, 49152));
}
#[test]
fn concurrent_allocations_are_disjoint() {
    let s = std::sync::Arc::new(MemoryResources::new());
    let mut ts = vec![];
    for _ in 0..8 {
        let s = s.clone();
        ts.push(std::thread::spawn(move || {
            s.allocate(0, DIRECT_SIZE, 65536, 65536, 0).unwrap()
        }));
    }
    let mut offsets: Vec<_> = ts.into_iter().map(|t| t.join().unwrap()).collect();
    offsets.sort();
    offsets.dedup();
    assert_eq!(offsets.len(), 8);
    s.shutdown();
    assert_eq!(s.snapshot().active_native_resources, 0);
}

#[test]
fn failed_replacement_restores_uncommitted_reservation() {
    let _l = SERIAL.lock().unwrap();
    let s = MemoryResources::new();
    let v = s.reserve(0, 65536, 0, 65536).unwrap();
    let p = s.allocate(0, DIRECT_SIZE, 131072, 0, 0).unwrap();
    // Guest-aligned, but not a Windows section-view offset: refuse and restore.
    assert!(s.map(v, 65536, 3, 16, p + 16384, 0).is_err());
    assert!(!s.query(v, false).unwrap().direct);
    s.with_regions(|r| assert_eq!(r[0].snapshot().committed_bytes, 0));
    s.shutdown();
    assert_eq!(s.snapshot().active_native_resources, 0);
    assert!(s.snapshot().release_errors.is_empty());
}
