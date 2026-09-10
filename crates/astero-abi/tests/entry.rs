use astero_abi::layouts::entry::*;
#[test]
fn process_record_has_exact_little_endian_layout() {
    let b = process_arguments(0x102030405);
    assert_eq!(&b[..8], &[1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        u64::from_le_bytes(b[8..16].try_into().unwrap()),
        0x102030405
    );
    assert!(b[16..].iter().all(|v| *v == 0));
}
#[test]
fn context_has_no_inherited_host_values() {
    let c = InitialContext::planned(0x1234, 0x2008, 0x2040, 0x3000);
    assert_eq!(c.gpr[5], 0x2040);
    assert!(c.gpr.iter().enumerate().all(|(i, v)| i == 5 || *v == 0));
    assert_eq!(c.rflags & 0x400, 0);
    assert_eq!(c.mxcsr, 0x1f80);
    assert_eq!(c.x87_control, 0x37f);
    assert!(c.xmm.iter().flatten().all(|v| *v == 0));
    assert!(c.preserve_host_gs);
}
