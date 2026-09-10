use astero_memory::allocation::heap::*;
#[test]
fn bounded_heap_exact_capacity_coalescence_and_invalid_free() {
    let mut h = GuestHeap::new(4096, 64, 2).unwrap();
    let a = h.allocate(17).unwrap();
    let b = h.allocate(32).unwrap();
    assert_eq!(a, 4096);
    assert_eq!(b, 4128);
    assert_eq!(h.allocate(1), Err(HeapError::Capacity));
    assert_eq!(h.free(a + 1), Err(HeapError::InvalidFree));
    h.free(a).unwrap();
    h.free(b).unwrap();
    assert_eq!(h.allocate(64).unwrap(), 4096);
}
#[test]
fn zero_null_overflow_and_identity() {
    assert!(GuestHeap::new(u64::MAX - 15, 32, 1).is_err());
    let mut h = GuestHeap::new(4096, 64, 2).unwrap();
    h.free(0).unwrap();
    let a = h.allocate(0).unwrap();
    assert_eq!(h.size(a).unwrap(), 0);
    assert!(h.allocate(u64::MAX).is_err());
    assert_eq!(h.snapshot().live, 1);
    h.free(a).unwrap();
    assert!(h.free(a).is_err());
}
