use astero_memory::mapping::{GuestAddress, windows_native::*};
fn protection(r: bool, w: bool, x: bool) -> Protection {
    Protection {
        read: r,
        write: w,
        execute: x,
    }
}
fn region(address: u64, bytes: &[u8], p: Protection) -> NativeRegion<'_> {
    NativeRegion {
        range: GuestRange {
            start: GuestAddress(address),
            size: bytes.len() as u64,
        },
        bytes,
        protection: p,
    }
}
fn limits() -> NativeLimits {
    NativeLimits {
        max_reserved_bytes: 0x40000,
        max_committed_bytes: 0x40000,
    }
}
fn geometry() -> Geometry {
    Geometry {
        page_size: 4096,
        allocation_granularity: 65536,
    }
}
#[test]
fn shared_page_union_and_padding_are_explicit() {
    let b = [0; 16];
    let r = [
        region(0x10010, &b, protection(true, false, false)),
        region(0x10020, &b, protection(false, true, false)),
    ];
    let l = plan_layout(&r, geometry(), limits()).unwrap();
    assert_eq!(l.pages().len(), 1);
    assert!(l.pages()[0].widened);
    assert_eq!(l.pages()[0].protection, protection(true, true, false));
    assert_eq!(l.envelope().start.0, 0x10000);
}
#[test]
fn no_accidental_writable_executable_union() {
    let b = [0; 8];
    let r = [
        region(0x10000, &b, protection(false, true, false)),
        region(0x10008, &b, protection(false, false, true)),
    ];
    assert!(matches!(
        plan_layout(&r, geometry(), limits()),
        Err(NativeError::WritableExecutable { .. })
    ));
}
#[test]
fn overlap_and_size_mismatch_fail_before_reservation() {
    let b = [0; 8];
    let r = [
        region(0x10000, &b, protection(true, false, false)),
        region(0x10004, &b, Protection::default()),
    ];
    assert!(matches!(
        plan_layout(&r, geometry(), limits()),
        Err(NativeError::Overlap { .. })
    ));
    let mut r = region(0x10000, &b, Protection::default());
    r.range.size = 7;
    assert!(matches!(
        plan_layout(&[r], geometry(), limits()),
        Err(NativeError::SourceSize { .. })
    ));
}
#[test]
fn overflow_and_bad_geometry_refuse() {
    let b = [0; 8];
    assert!(matches!(
        plan_layout(
            &[region(u64::MAX - 1, &b, Protection::default())],
            geometry(),
            limits()
        ),
        Err(NativeError::Overflow)
    ));
    assert!(matches!(
        plan_layout(
            &[region(0x10000, &b, Protection::default())],
            Geometry {
                page_size: 3,
                allocation_granularity: 65536
            },
            limits()
        ),
        Err(NativeError::Geometry)
    ));
}
#[test]
fn reservation_and_commit_limits_have_exact_boundaries() {
    let b = [0; 4096];
    let r = [region(0x10000, &b, Protection::default())];
    let exact = NativeLimits {
        max_reserved_bytes: 4096,
        max_committed_bytes: 4096,
    };
    assert!(plan_layout(&r, geometry(), exact).is_ok());
    assert!(
        plan_layout(
            &r,
            geometry(),
            NativeLimits {
                max_committed_bytes: 4095,
                ..exact
            }
        )
        .is_err()
    );
    assert!(
        plan_layout(
            &r,
            geometry(),
            NativeLimits {
                max_reserved_bytes: 4095,
                ..exact
            }
        )
        .is_err()
    );
}
#[test]
fn sparse_holes_do_not_commit_pages() {
    let b = [0; 16];
    let r = [
        region(0x10000, &b, Protection::default()),
        region(0x20000, &b, Protection::default()),
    ];
    let l = plan_layout(&r, geometry(), limits()).unwrap();
    assert_eq!(l.pages().len(), 2);
    assert_eq!(l.envelope().size, 0x11000);
}
#[cfg(all(windows, target_arch = "x86_64"))]
mod windows {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    fn address() -> u64 {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        0x500000000
            + (std::process::id() as u64) * 0x100000
            + NEXT.fetch_add(1, Ordering::SeqCst) * 0x40000
    }
    #[test]
    fn rx_byte_readback_and_cache_flush_without_execution() {
        let b = [0xc3; 4096];
        let a = address();
        let (r, o) = realize(&[region(a, &b, protection(true, false, true))], limits());
        let mut image = r.unwrap();
        assert_eq!(image.snapshot().host_envelope.start.0, a);
        assert!(
            image.snapshot().protections_verified && image.snapshot().instruction_cache_flushed
        );
        assert_eq!(image.read(GuestAddress(a), b.len() as u64).unwrap(), b);
        image.release().unwrap();
        image.release().unwrap();
        assert_eq!(o.active_reservations(), 0);
        assert_eq!(o.release_error(), 0);
        assert!(!image.snapshot().active);
    }
    #[test]
    fn rw_zero_fill_and_drop_cleanup() {
        let b = [0; 4096];
        let a = address();
        let (r, o) = realize(&[region(a, &b, protection(true, true, false))], limits());
        let image = r.unwrap();
        assert_eq!(image.read(GuestAddress(a), 4096).unwrap(), b);
        drop(image);
        assert_eq!(o.active_reservations(), 0);
    }
    #[test]
    fn occupied_range_refuses_without_disturbing_owner() {
        let b = [42; 4096];
        let a = address();
        let (r, o) = realize(&[region(a, &b, protection(true, false, false))], limits());
        let image = r.unwrap();
        let (second, other) = realize(&[region(a, &b, protection(true, false, false))], limits());
        assert!(matches!(
            second,
            Err(NativeError::Os {
                operation: "reserve_exact_or_collision",
                ..
            })
        ));
        assert_eq!(other.active_reservations(), 0);
        assert_eq!(image.read(GuestAddress(a), 1).unwrap(), [42]);
        drop(image);
        assert_eq!(o.active_reservations(), 0);
    }
    #[test]
    fn noaccess_executeonly_and_holes_never_read_unsafely() {
        let b = [0; 32];
        let a = address();
        let (r, o) = realize(
            &[
                region(a, &b, Protection::default()),
                region(a + 0x2000, &b, protection(false, false, true)),
            ],
            limits(),
        );
        let image = r.unwrap();
        for at in [a, a + 4096, a + 0x2000] {
            assert!(matches!(
                image.read(GuestAddress(at), 1),
                Err(NativeError::Unreadable)
            ));
        }
        drop(image);
        assert_eq!(o.active_reservations(), 0);
    }
    #[test]
    fn two_owners_and_page_query_are_independent() {
        let b = [7; 4096];
        let a = address();
        let (first, one) = realize(&[region(a, &b, protection(true, false, false))], limits());
        let (second, two) = realize(
            &[region(a + 65536, &b, protection(true, true, false))],
            limits(),
        );
        drop(first.unwrap());
        assert_eq!(one.active_reservations(), 0);
        let second = second.unwrap();
        assert_eq!(two.active_reservations(), 1);
        assert!(second.snapshot().protections_verified);
        drop(second);
        assert_eq!(two.active_reservations(), 0);
    }
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn relro_seal_is_exact_and_os_verified() {
    use astero_memory::mapping::{GuestAddress, windows_native::*};
    let base = 0x1700000000 + (std::process::id() as u64) * 0x100000;
    let bytes = [7; 8192];
    let (image, o) = realize(
        &[NativeRegion {
            range: GuestRange {
                start: GuestAddress(base),
                size: 8192,
            },
            bytes: &bytes,
            protection: Protection {
                read: true,
                write: true,
                execute: false,
            },
        }],
        NativeLimits {
            max_reserved_bytes: 8192,
            max_committed_bytes: 8192,
        },
    );
    let mut image = image.unwrap();
    assert!(
        image
            .seal_read_only(GuestRange {
                start: GuestAddress(base + 1),
                size: 4096
            })
            .is_err()
    );
    image
        .seal_read_only(GuestRange {
            start: GuestAddress(base),
            size: 4096,
        })
        .unwrap();
    assert!(!image.pages()[0].protection.write);
    assert!(image.pages()[1].protection.write);
    assert_eq!(image.read(GuestAddress(base), 8).unwrap(), [7; 8]);
    image
        .seal_read_only(GuestRange {
            start: GuestAddress(base),
            size: 4096,
        })
        .unwrap();
    image.release().unwrap();
    assert_eq!(o.active_reservations(), 0);
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn owned_native_writes_refuse_rx_and_cross_boundary_before_mutation() {
    let b = vec![0u8; 4096];
    let base = 0x2700000000;
    let (image, _) = realize(
        &[
            region(base, &b, protection(true, true, false)),
            region(base + 4096, &b, protection(true, false, true)),
        ],
        limits(),
    );
    let mut image = image.unwrap();
    image.write(GuestAddress(base + 8), &[1, 2, 3]).unwrap();
    assert_eq!(image.read(GuestAddress(base + 8), 3).unwrap(), [1, 2, 3]);
    assert!(image.write(GuestAddress(base + 4095), &[9, 9]).is_err());
    assert_eq!(image.read(GuestAddress(base + 4095), 1).unwrap(), [0]);
    assert!(image.write(GuestAddress(base + 4096), &[1]).is_err());
    image.release().unwrap();
    assert!(image.write(GuestAddress(base), &[]).is_err());
}
