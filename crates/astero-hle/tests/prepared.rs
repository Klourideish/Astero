use astero_abi::layouts::entry::CallFrame;
use astero_hle::dispatch::prepared::*;
fn key() -> ProviderKey {
    ProviderKey {
        nid: 7,
        library: b"synthetic".to_vec(),
        module: b"test".to_vec(),
    }
}
fn handler(f: &mut CallFrame, _: &mut dyn astero_hle::calls::memory::GuestMemory) -> CallResult {
    f.rax = f.arguments[0] + f.stack_arguments[0];
    f.arguments.fill(999);
    f.xmm0 = [42; 16];
    CallResult::Returned
}
fn registration() -> Registration {
    Registration {
        key: key(),
        kind: ProviderKind::SyntheticHostTest,
        handler: Some(Box::new(handler)),
    }
}
#[test]
fn exact_context_matching_and_duplicate_ambiguity() {
    let r = PreparedRegistry::new(vec![registration()], 1).unwrap();
    let mut k = key();
    k.library = b"wrong".to_vec();
    assert!(matches!(r.find(&k), Err(RegistryError::Missing)));
    let r = PreparedRegistry::new(vec![registration(), registration()], 2).unwrap();
    assert!(matches!(r.find(&key()), Err(RegistryError::Ambiguous)));
}
#[test]
fn host_model_publishes_returns_only() {
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let captured = calls.clone();
    let mut e = registration();
    e.handler = Some(Box::new(move |f, m| {
        captured.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        handler(f, m)
    }));
    let r = PreparedRegistry::new(vec![e], 1).unwrap();
    let mut f = CallFrame::default();
    f.arguments[0] = 5;
    f.stack_arguments[0] = 6;
    let before = f.clone();
    assert_eq!(
        r.invoke_host_model(&key(), &mut f),
        Ok(CallResult::Returned)
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(f.rax, 11);
    assert_eq!(f.xmm0, [42; 16]);
    assert_eq!(f.arguments, before.arguments);
}
#[test]
fn declarations_cannot_be_invoked_or_mislabelled() {
    let mut e = registration();
    e.kind = ProviderKind::ArtifactDeclaration;
    assert!(matches!(
        PreparedRegistry::new(vec![e], 1),
        Err(RegistryError::InvalidHandler)
    ));
    let mut e = registration();
    e.kind = ProviderKind::ArtifactDeclaration;
    e.handler = None;
    let r = PreparedRegistry::new(vec![e], 1).unwrap();
    assert_eq!(
        r.invoke_host_model(&key(), &mut CallFrame::default()),
        Err(RegistryError::NoHostHandler)
    );
    assert!(matches!(
        PreparedRegistry::new(vec![registration()], 0),
        Err(RegistryError::Capacity)
    ));
}
#[test]
fn panic_never_publishes_partial_frame() {
    fn panic_handler(
        f: &mut CallFrame,
        _: &mut dyn astero_hle::calls::memory::GuestMemory,
    ) -> CallResult {
        f.rax = 8;
        panic!("synthetic")
    }
    let mut e = registration();
    e.handler = Some(Box::new(panic_handler));
    let r = PreparedRegistry::new(vec![e], 1).unwrap();
    let mut f = CallFrame::default();
    assert_eq!(
        r.invoke_host_model(&key(), &mut f),
        Err(RegistryError::HandlerPanicked)
    );
    assert_eq!(f, CallFrame::default());
}
