use astero_memory::mapping::*;
#[test]
fn overlap_bounds_and_exact_capacity() {
    let mut m = OwnedAddressSpace::new(8);
    m.map_zeroed(GuestAddress(0x1000), 8).unwrap();
    assert!(matches!(
        m.map_zeroed(GuestAddress(0x1000), 1),
        Err(MemoryError::Capacity { .. })
    ));
    assert!(m.write(GuestAddress(0x1007), &[1, 2]).is_err());
    assert_eq!(m.read(GuestAddress(0x1000), 8).unwrap(), [0; 8]);
    assert!(m.read(GuestAddress(u64::MAX), 2).is_err());
}
#[test]
fn overlap_and_sparse_holes() {
    let mut m = OwnedAddressSpace::new(32);
    m.map_zeroed(GuestAddress(1), 8).unwrap();
    assert!(matches!(
        m.map_zeroed(GuestAddress(8), 8),
        Err(MemoryError::Overlap)
    ));
    m.map_zeroed(GuestAddress(16), 8).unwrap();
    assert!(m.read(GuestAddress(8), 9).is_err());
}
#[test]
fn finalized_backend_rejects_writes_and_maps() {
    let mut m = OwnedAddressSpace::new(32);
    m.map_zeroed(GuestAddress(1), 8).unwrap();
    m.write(GuestAddress(2), &[3, 4]).unwrap();
    m.finalize();
    assert_eq!(m.write(GuestAddress(2), &[5]), Err(MemoryError::Finalized));
    assert_eq!(
        m.map_zeroed(GuestAddress(20), 8),
        Err(MemoryError::Finalized)
    );
    assert_eq!(m.read(GuestAddress(2), 2).unwrap(), [3, 4]);
}
#[test]
fn observer_is_per_image_and_survives_drop() {
    let mut a = OwnedAddressSpace::new(8);
    let b = OwnedAddressSpace::new(8);
    let o = a.observer();
    a.map_zeroed(GuestAddress(0), 8).unwrap();
    assert_eq!(b.observer().active_mappings(), 0);
    drop(a);
    assert_eq!(o.active_mappings(), 0);
}
