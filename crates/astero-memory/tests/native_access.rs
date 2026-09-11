#![cfg(all(windows, target_arch = "x86_64"))]
use astero_memory::{
    access::native,
    mapping::{GuestAddress, windows_native::*},
};
fn image(a: u64, n: usize, w: bool) -> NativeImage {
    let b = vec![0; n];
    let (r, _) = realize(
        &[NativeRegion {
            range: GuestRange {
                start: GuestAddress(a),
                size: n as u64,
            },
            bytes: &b,
            protection: Protection {
                read: true,
                write: w,
                execute: false,
            },
        }],
        NativeLimits {
            max_reserved_bytes: n as u64,
            max_committed_bytes: n as u64,
        },
    );
    r.unwrap()
}
#[test]
fn adjacent_owned_mappings_copy_across_boundary() {
    let a = image(0x760000000, 65536, true);
    let b = image(0x760010000, 65536, true);
    let owners = [&a, &b];
    let bytes = vec![77; 8192];
    native::write(&owners, 0x76000f000, &bytes).unwrap();
    assert_eq!(native::read(&owners, 0x76000f000, 8192).unwrap(), bytes);
    let observers = [a.observer(), b.observer()];
    drop(a);
    drop(b);
    assert!(observers.iter().all(|o| o.active_reservations() == 0));
}
#[test]
fn gap_and_readonly_tail_preflight_do_not_write_prefix() {
    let a = image(0x761000000, 65536, true);
    let b = image(0x761010000, 65536, false);
    assert!(native::write(&[&a, &b], 0x76100f000, &[9; 8192]).is_err());
    assert_eq!(
        a.read(GuestAddress(0x76100f000), 4096).unwrap(),
        vec![0; 4096]
    );
    assert!(native::validate(&[&a], 0x76100f000, 8192, false).is_err());
    assert!(native::validate(&[&a], u64::MAX, 2, false).is_err());
}
#[test]
fn noaccess_page_rejects_range() {
    let b = vec![0; 8192];
    let (r, _) = realize(
        &[
            NativeRegion {
                range: GuestRange {
                    start: GuestAddress(0x762000000),
                    size: 4096,
                },
                bytes: &b[..4096],
                protection: Protection {
                    read: true,
                    write: true,
                    execute: false,
                },
            },
            NativeRegion {
                range: GuestRange {
                    start: GuestAddress(0x762001000),
                    size: 4096,
                },
                bytes: &b[4096..],
                protection: Protection::default(),
            },
        ],
        NativeLimits {
            max_reserved_bytes: 65536,
            max_committed_bytes: 8192,
        },
    );
    let r = r.unwrap();
    assert!(native::validate(&[&r], 0x762000000, 8192, false).is_err());
}
