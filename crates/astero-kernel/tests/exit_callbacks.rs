use astero_kernel::process::exit_callbacks::*;
#[test]
fn callback_order_duplicates_and_no_invocation() {
    let mut c = ExitCallbacks::new(3);
    for a in [17, 18, 17] {
        c.register(a).unwrap();
    }
    assert_eq!(c.pending().collect::<Vec<_>>(), [17, 18, 17]);
    assert_eq!(c.len(), 3);
}
#[test]
fn callback_refusal_preserves_prior_records() {
    let mut c = ExitCallbacks::new(1);
    assert_eq!(c.register(0), Err(CallbackError::Null));
    c.register(8).unwrap();
    assert_eq!(c.register(9), Err(CallbackError::Capacity));
    assert_eq!(c.pending().collect::<Vec<_>>(), [8]);
}
#[test]
fn zero_callback_capacity_is_explicit() {
    let mut c = ExitCallbacks::new(0);
    assert_eq!(c.register(1), Err(CallbackError::Capacity));
    assert!(c.is_empty());
}
