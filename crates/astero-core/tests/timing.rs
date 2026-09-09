use astero_core::session::{
    Session,
    inputs::SessionInputs,
    timing::{Config, Span, TimingEngine},
};
#[test]
fn session_timing_is_explicit_owned_and_stops_with_session() {
    let plain = Session::new().unwrap();
    assert!(plain.timing().is_none());
    let (engine, clock) = TimingEngine::manual(Config {
        max_pending: 2,
        max_snapshot_entries: 2,
    })
    .unwrap();
    let mut session = Session::with_timing(SessionInputs::default(), engine).unwrap();
    let scheduler = session.timing().unwrap();
    let ticket = scheduler
        .after(
            Span::from_nanos(5),
            astero_timing::scheduler::Label::new("future-consumer").unwrap(),
        )
        .unwrap();
    clock.advance(Span::from_nanos(5)).unwrap();
    assert!(matches!(
        ticket
            .wait_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        astero_timing::scheduler::Completion::Fired(_)
    ));
    let pending = scheduler
        .after(
            Span::from_nanos(5),
            astero_timing::scheduler::Label::new("pending").unwrap(),
        )
        .unwrap();
    session.stop().unwrap();
    assert!(matches!(
        pending
            .wait_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        astero_timing::scheduler::Completion::Stopped(_)
    ));
    assert!(session.run().is_err());
}
#[test]
fn dropping_session_releases_future_consumer_waits() {
    let (engine, _) = TimingEngine::manual(Config {
        max_pending: 1,
        max_snapshot_entries: 1,
    })
    .unwrap();
    let session = Session::with_timing(SessionInputs::default(), engine).unwrap();
    let s = session.timing().unwrap();
    let ticket = s
        .after(
            Span::from_nanos(1),
            astero_timing::scheduler::Label::new("wait").unwrap(),
        )
        .unwrap();
    drop(session);
    assert!(matches!(
        ticket
            .wait_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        astero_timing::scheduler::Completion::Stopped(_)
    ));
    assert!(s.now().is_err());
}
