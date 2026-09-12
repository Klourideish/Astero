#![cfg(all(windows, target_arch = "x86_64"))]
use astero_core::input::entry::observability::*;
#[test]
fn preparation_never_claims_activity_or_cleanup() {
    let o = Observer::new("artifact".into(), None).unwrap();
    let s = o.snapshot();
    assert!(s.subsystems.values().all(|s| s.state == Health::NotReached));
    assert!(s.teardown.is_none());
    assert!(s.sha256.is_none());
    assert_eq!(s.hle_calls, 0);
}
#[test]
fn schema_is_stable_typed_and_preserves_64_bit_addresses() {
    let mut s = Snapshot::preparing("synthetic".into());
    s.entry_rip = Some(address(u64::MAX));
    let j = s.json().unwrap();
    assert!(j.contains("0xffffffffffffffff"));
    assert!(j.contains("\"schema_version\": 1"));
    let decoded: Snapshot = serde_json::from_str(&j).unwrap();
    assert_eq!(decoded.json().unwrap(), j);
}
#[test]
fn observer_is_bounded_and_shared_without_authority() {
    assert!(Observer::new("x".repeat(4097), None).is_err());
    assert!(Observer::new("x".into(), Some(1)).is_err());
    let o = Observer::new("x".into(), Some(20)).unwrap();
    let other = o.clone();
    assert_eq!(
        std::thread::spawn(move || other.snapshot().runtime_state)
            .join()
            .unwrap(),
        "Preparing"
    );
}
#[test]
fn failed_factory_does_not_claim_execution_or_teardown() {
    let o = Observer::new("fixture".into(), None).unwrap();
    assert!(
        astero_core::input::entry::execute_first_entry_observed(
            || Err("fixture refusal".into()),
            20,
            o.clone()
        )
        .is_err()
    );
    let s = o.snapshot();
    assert_eq!(s.runtime_state, "Failed");
    assert!(s.teardown.is_none());
    assert!(s.stop.is_none());
}

#[test]
fn panicking_factory_reports_failure_without_fabricated_cleanup() {
    let o = Observer::new("fixture".into(), None).unwrap();
    assert!(
        astero_core::input::entry::execute_first_entry_observed(
            || panic!("test preparation failure"),
            20,
            o.clone()
        )
        .is_err()
    );
    assert_eq!(o.snapshot().runtime_state, "Failed");
    assert!(o.snapshot().teardown.is_none());
}

#[test]
fn kernel_resources_round_trip_and_older_reports_default_to_zero() {
    let mut s = Snapshot::preparing("synthetic".into());
    s.kernel_resources.direct_bytes = 1048576;
    s.kernel_resources.semaphores = 4;
    let mut value = serde_json::to_value(&s).unwrap();
    let decoded: Snapshot = serde_json::from_value(value.clone()).unwrap();
    assert_eq!(decoded.kernel_resources.direct_bytes, 1048576);
    assert_eq!(decoded.kernel_resources.semaphores, 4);
    value.as_object_mut().unwrap().remove("kernel_resources");
    let older: Snapshot = serde_json::from_value(value).unwrap();
    assert_eq!(older.kernel_resources.direct_bytes, 0);
    assert_eq!(older.kernel_resources.semaphores, 0);
}
