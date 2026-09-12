#![cfg(all(windows, target_arch = "x86_64"))]
use astero_memory::mapping::{GuestAddress, windows_native::*};
#[test]
fn backing_survives_views_and_aliases_without_guest_pointers() {
    let b = NativeBacking::new(65536, 65536).unwrap();
    let o = b.observer();
    let rw = Protection {
        read: true,
        write: true,
        execute: false,
    };
    let mut a = b
        .map(
            GuestRange {
                start: GuestAddress(0x580000000),
                size: 65536,
            },
            0,
            rw,
        )
        .unwrap();
    let v = b
        .map(
            GuestRange {
                start: GuestAddress(0x580020000),
                size: 65536,
            },
            0,
            rw,
        )
        .unwrap();
    a.write(GuestAddress(0x580000000), &[1, 2, 3]).unwrap();
    assert_eq!(v.read(GuestAddress(0x580020000), 3).unwrap(), [1, 2, 3]);
    a.release().unwrap();
    drop(a);
    drop(v);
    let r = b
        .map(
            GuestRange {
                start: GuestAddress(0x580000000),
                size: 65536,
            },
            0,
            Protection {
                read: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.read(GuestAddress(0x580000000), 3).unwrap(), [1, 2, 3]);
    assert!(r.write(GuestAddress(0x580000000), &[4]).is_err());
    drop(b);
    assert_eq!(o.active_reservations(), 1);
    drop(r);
    assert_eq!(o.active_reservations(), 0);
    assert_eq!(o.release_error(), 0);
}
#[test]
fn view_refuses_invalid_geometry_and_rwx() {
    let b = NativeBacking::new(65536, 65536).unwrap();
    let r = GuestRange {
        start: GuestAddress(0x590000000),
        size: 65536,
    };
    assert!(b.map(r, 4096, Protection::default()).is_err());
    assert!(
        b.map(
            r,
            0,
            Protection {
                read: true,
                write: true,
                execute: true
            }
        )
        .is_err()
    );
    assert!(NativeBacking::new(65536, 32768).is_err());
    let rx = b
        .map(
            r,
            0,
            Protection {
                read: true,
                write: false,
                execute: true,
            },
        )
        .unwrap();
    assert!(rx.snapshot().protections_verified);
    assert!(rx.snapshot().instruction_cache_flushed);
}
