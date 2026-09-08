use astero_core::{
    observation::{Availability, Diagnostic, ObservationError, ObserveSession},
    session::{Lifecycle, Session, SessionError},
};

#[test]
fn identity_and_unloaded_state_are_truthful() {
    let a = Session::new().unwrap();
    let b = Session::new().unwrap();
    let s = a.observer().snapshot().unwrap();
    assert_ne!(s.id, b.observer().snapshot().unwrap().id);
    assert_eq!(s.lifecycle, Lifecycle::Created);
    assert_eq!(s.loaded_target, None);
    assert_eq!(s.statistics.lifecycle_changes, 0);
    assert_eq!(s.subsystems.len(), 8);
    assert!(
        s.subsystems
            .iter()
            .all(|s| s.availability == Availability::NotImplemented)
    );
    assert_eq!(s.diagnostics, vec![Diagnostic::NoGuestLoaded]);
}

#[test]
fn host_lifecycle_is_one_way_and_guest_run_is_rejected() {
    let mut s = Session::new().unwrap();
    let view = s.observer();
    assert_eq!(s.run(), Err(SessionError::InvalidTransition));
    s.initialize().unwrap();
    let ready = view.snapshot().unwrap();
    assert_eq!(ready.lifecycle, Lifecycle::Ready);
    assert_eq!(ready.statistics.lifecycle_changes, 1);
    assert_eq!(s.initialize(), Err(SessionError::InvalidTransition));
    assert_eq!(s.run(), Err(SessionError::NoGuestLoaded));
    assert_eq!(view.snapshot().unwrap(), ready);
    s.stop().unwrap();
    let stopped = view.snapshot().unwrap();
    assert_eq!(stopped.lifecycle, Lifecycle::Stopped);
    assert_eq!(stopped.statistics.lifecycle_changes, 2);
    assert_eq!(s.stop(), Err(SessionError::InvalidTransition));
    assert_eq!(s.initialize(), Err(SessionError::InvalidTransition));
    assert_eq!(
        s.report_host_fault("late"),
        Err(SessionError::InvalidTransition)
    );
    assert_eq!(view.snapshot().unwrap(), stopped);
}

#[test]
fn host_fault_has_provenance_and_no_guest_execution() {
    let mut s = Session::new().unwrap();
    let before = s.observer().snapshot().unwrap();
    assert_eq!(s.report_host_fault("  "), Err(SessionError::EmptyFault));
    assert_eq!(before, s.observer().snapshot().unwrap());
    s.report_host_fault("test-injected host orchestration failure")
        .unwrap();
    let fault = s.observer().snapshot().unwrap();
    assert_eq!(fault.lifecycle, Lifecycle::Faulted);
    assert_eq!(
        fault.diagnostics[1],
        Diagnostic::HostFault("test-injected host orchestration failure".into())
    );
    assert_eq!(s.initialize(), Err(SessionError::InvalidTransition));
    assert_eq!(s.run(), Err(SessionError::InvalidTransition));
    s.stop().unwrap();
    assert_eq!(
        s.observer()
            .snapshot()
            .unwrap()
            .statistics
            .lifecycle_changes,
        2
    );
}

#[test]
fn detached_snapshot_mutation_cannot_change_session() {
    let mut s = Session::new().unwrap();
    let observer = s.observer();
    let old = observer.snapshot().unwrap();
    let mut copy = old.clone();
    copy.subsystems.clear();
    copy.lifecycle = Lifecycle::Running;
    assert_eq!(observer.snapshot().unwrap(), old);
    s.initialize().unwrap();
    assert_eq!(old.lifecycle, Lifecycle::Created);
    assert_eq!(observer.snapshot().unwrap().lifecycle, Lifecycle::Ready);
}

#[test]
fn observer_does_not_keep_owner_alive() {
    let s = Session::new().unwrap();
    let observer = s.observer();
    drop(s);
    assert_eq!(observer.snapshot(), Err(ObservationError::SessionClosed));
}

#[test]
fn observation_is_coherent_across_threads() {
    let mut s = Session::new().unwrap();
    let observer = s.observer();
    let reader = std::thread::spawn(move || {
        for _ in 0..1000 {
            let view = observer.snapshot().unwrap();
            let expected = match view.lifecycle {
                Lifecycle::Created => 0,
                Lifecycle::Ready => 1,
                Lifecycle::Stopped => 2,
                other => panic!("unexpected {other:?}"),
            };
            assert_eq!(view.statistics.lifecycle_changes, expected);
        }
    });
    s.initialize().unwrap();
    s.stop().unwrap();
    reader.join().unwrap();
}
