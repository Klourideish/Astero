use astero_hle::dispatch::prepared::{PreparedRegistry, ProviderKey};
use astero_libs::libc::startup::*;
fn key(nid: u64) -> ProviderKey {
    ProviderKey {
        nid,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    }
}
#[test]
fn startup_registration_is_exact_and_bounded() {
    let (r, s) = registrations(2, vec![(0x1000, 4096)]);
    let registry = PreparedRegistry::new(r, 2).unwrap();
    assert_eq!(registry.len(), 2);
    let mut wrong = key(INIT_ENV_NID);
    wrong.library = b"libkernel".to_vec();
    assert!(registry.find(&wrong).is_err());
    assert_eq!(s.borrow().init_env_calls, 0);
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn startup_handlers_are_callable_through_native_synthetic_boundary() {
    use astero_kernel::execution::host::{Bridge, SyntheticProbe};
    let (r, s) = registrations(2, vec![(7, 1)]);
    let registry = PreparedRegistry::new(r, 2).unwrap();
    let mut b = Bridge::new().unwrap();
    for nid in [INIT_ENV_NID, ATEXIT_NID, ATEXIT_NID] {
        let result = b
            .synthetic(SyntheticProbe::Import, &mut |_, f| {
                registry.invoke_host_model(&key(nid), f).is_ok()
            })
            .unwrap();
        assert_eq!(result.value, 0);
    }
    assert_eq!(s.borrow().init_env_calls, 1);
    assert_eq!(s.borrow().callbacks().collect::<Vec<_>>(), [7, 7]);
    let result = b
        .synthetic(SyntheticProbe::Import, &mut |_, f| {
            registry.invoke_host_model(&key(ATEXIT_NID), f).is_ok()
        })
        .unwrap();
    assert_eq!(result.value, u32::MAX as u64);
    let result = b
        .synthetic(SyntheticProbe::Import, &mut |_, f| {
            registry.invoke_host_model(&key(0), f).is_ok()
        })
        .unwrap();
    assert_eq!(result.reason, 2);
}
