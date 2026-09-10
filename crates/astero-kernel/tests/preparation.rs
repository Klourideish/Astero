use astero_kernel::execution::preparation::{boundary::*, layout::*};
use astero_memory::mapping::windows_native::Geometry;
fn geometry() -> Geometry {
    Geometry {
        page_size: 4096,
        allocation_granularity: 65536,
    }
}
fn limits() -> RuntimeLimits {
    RuntimeLimits {
        stack_base: 0x220000000,
        stack_bytes: 8192,
        tls_base: 0x230000000,
        max_runtime_bytes: 16384,
    }
}
#[test]
fn stack_alignment_guard_and_budget_boundaries() {
    let p = plan(limits(), geometry(), 0, 0, 0).unwrap();
    assert_eq!(p.rsp % 16, 8);
    assert_eq!(p.params % 16, 0);
    assert_eq!(p.stack.start.0, p.guard.start.0 + 4096);
    assert_eq!(p.reserved_bytes, 16384);
    let mut l = limits();
    l.max_runtime_bytes = 16383;
    assert!(matches!(
        plan(l, geometry(), 0, 0, 0),
        Err(PreparationError::Budget { .. })
    ));
}
#[test]
fn tls_negative_offsets_and_checked_arithmetic() {
    let p = plan(
        RuntimeLimits {
            max_runtime_bytes: 32768,
            ..limits()
        },
        geometry(),
        5,
        5000,
        4096,
    )
    .unwrap();
    assert_eq!(p.thread_pointer - p.tls.start.0, 8192);
    assert!(matches!(
        plan(limits(), geometry(), 6, 5, 1),
        Err(PreparationError::TlsShape)
    ));
    assert!(plan(limits(), geometry(), 0, u64::MAX, 16).is_err());
    assert!(
        plan(
            RuntimeLimits {
                stack_base: u64::MAX - 65535,
                stack_bytes: 65536,
                ..limits()
            },
            geometry(),
            0,
            0,
            0
        )
        .is_err()
    );
}
#[test]
fn layout_rejects_overlap_and_invalid_geometry() {
    assert!(matches!(
        plan(
            RuntimeLimits {
                tls_base: limits().stack_base,
                ..limits()
            },
            geometry(),
            0,
            0,
            0
        ),
        Err(PreparationError::Overlap)
    ));
    assert!(
        plan(
            limits(),
            Geometry {
                page_size: 3,
                ..geometry()
            },
            0,
            0,
            0
        )
        .is_err()
    );
}
#[test]
fn recovery_model_is_first_stop_wins_and_never_armed() {
    let mut b = RecoveryBoundary::default();
    assert!(!b.native_installed());
    assert!(b.record_stop(StopReason::Fault {
        code: 0xc0000005,
        rip: 7
    }));
    assert!(!b.record_stop(StopReason::Returned));
    b.release();
    b.release();
    assert_eq!(b.state(), BoundaryState::Released);
    assert!(!b.record_stop(StopReason::Requested));
}
#[test]
fn import_encoding_is_far_indirect_and_refuses_sign_extension() {
    let b = import_stub(2, 0x700000000000).unwrap();
    assert_eq!(&b[..7], &[0x68, 2, 0, 0, 0, 0xff, 0x25]);
    assert_eq!(
        u64::from_le_bytes(b[11..19].try_into().unwrap()),
        0x700000000000
    );
    assert!(import_stub(u32::MAX, 1).is_none());
    assert!(import_stub(1, 0).is_none());
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn native_thread_storage_template_guard_and_cleanup() {
    use astero_kernel::execution::preparation::storage::ThreadStorage;
    let bias = (std::process::id() as u64) * 0x100000;
    let l = RuntimeLimits {
        stack_base: 0x300000000 + bias,
        tls_base: 0x400000000 + bias,
        max_runtime_bytes: 16384,
        ..limits()
    };
    let p = plan(l, geometry(), 3, 16, 16).unwrap();
    let (mut s, o) = ThreadStorage::build(p.clone(), &[1, 2, 3]).unwrap();
    assert_eq!(
        s.read_tls(p.tls.start.0, 16).unwrap(),
        [1, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(
        s.read_tls(p.thread_pointer, 8).unwrap(),
        p.thread_pointer.to_le_bytes()
    );
    assert!(s.read_stack(p.guard.start.0, 1).is_err());
    assert_eq!(
        s.read_stack(p.params, 32).unwrap(),
        astero_kernel::execution::preparation::process_arguments(p.argv0)
    );
    assert_eq!(s.read_stack(p.rsp, 8).unwrap(), [0; 8]);
    s.release().unwrap();
    s.release().unwrap();
    assert!(o.iter().all(|o| o.active_reservations() == 0));
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn forged_layout_refused_before_slicing() {
    use astero_kernel::execution::preparation::storage::ThreadStorage;
    let mut p = plan(limits(), geometry(), 0, 0, 0).unwrap();
    p.params = 0;
    assert!(ThreadStorage::build(p, &[]).is_err());
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn tls_collision_rolls_back_stack_before_retry() {
    use astero_kernel::execution::preparation::storage::ThreadStorage;
    use astero_memory::mapping::windows_native::*;
    let bias = std::process::id() as u64 * 0x100000;
    let p = plan(
        RuntimeLimits {
            stack_base: 0x1800000000 + bias,
            tls_base: 0x1900000000 + bias,
            ..limits()
        },
        geometry(),
        0,
        0,
        0,
    )
    .unwrap();
    let bytes = [0; 4096];
    let (held, o) = realize(
        &[NativeRegion {
            range: p.tls,
            bytes: &bytes,
            protection: Protection {
                read: true,
                write: true,
                execute: false,
            },
        }],
        NativeLimits {
            max_reserved_bytes: 4096,
            max_committed_bytes: 4096,
        },
    );
    let mut held = held.unwrap();
    assert!(ThreadStorage::build(p.clone(), &[]).is_err());
    assert_eq!(o.active_reservations(), 1);
    held.release().unwrap();
    let (s, observers) = ThreadStorage::build(p, &[]).unwrap();
    drop(s);
    assert!(observers.iter().all(|o| o.active_reservations() == 0));
}
