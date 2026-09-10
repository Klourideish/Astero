#![cfg(all(windows, target_arch = "x86_64"))]
use astero_kernel::execution::host::{Bridge, BridgeError, SyntheticProbe};
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
#[test]
fn windows_register_stack_fs_and_fault_paths() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    b.validate().unwrap();
    assert!(b.validated());
    for (p, code) in [
        (SyntheticProbe::IllegalInstruction, 0xc000001d),
        (SyntheticProbe::AccessViolation, 0xc0000005),
    ] {
        let r = b.synthetic(p, &mut |_, _| false).unwrap();
        assert_eq!(r.exception, code);
        assert_ne!(r.rip, 0);
        assert_eq!(r.registers[4], r.rsp);
        assert!(r.host_fs_restored && r.host_gs_preserved);
    }
}
#[test]
fn import_ordinal_return_lanes_and_unresolved_stop() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    let r = b
        .synthetic(SyntheticProbe::ImportOrdinal(37), &mut |i, c| {
            assert_eq!(i, 37);
            assert_eq!(c.arguments, [7, 9, 0, 0, 0, 0]);
            c.rax = 123;
            true
        })
        .unwrap();
    assert_eq!(r.value, 123);
    assert_eq!(r.reason, 0);
    assert_eq!(
        b.synthetic(SyntheticProbe::Import, &mut |_, _| false)
            .unwrap()
            .reason,
        2
    );
}
#[test]
fn panicking_provider_returns_control_and_adapter_remains_usable() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    assert_eq!(
        b.synthetic(SyntheticProbe::Import, &mut |_, _| panic!(
            "synthetic panic"
        ))
        .unwrap()
        .reason,
        2
    );
    assert_eq!(
        b.synthetic(SyntheticProbe::Return, &mut |_, _| false)
            .unwrap()
            .value,
        16
    );
}
#[test]
fn exclusive_adapter_and_deterministic_handler_teardown() {
    let _guard = SERIAL.lock().unwrap();
    let b = Bridge::new().unwrap();
    assert!(matches!(Bridge::new(), Err(BridgeError::Busy)));
    drop(b);
    let mut next = Bridge::new().unwrap();
    next.validate().unwrap();
}
#[test]
fn range_metadata_is_checked_without_granting_transfer() {
    let _guard = SERIAL.lock().unwrap();
    let mut b = Bridge::new().unwrap();
    assert!(b.prepare_ranges(&[(u64::MAX, 2)]).is_err());
    b.prepare_ranges(&[(0x1000, 4096)]).unwrap();
    assert_eq!(b.prepared_ranges(), 1);
    assert!(!b.validated());
}
