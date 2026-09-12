#![cfg(all(windows, target_arch = "x86_64"))]
use astero_core::{
    input::entry::observability::{Health, HostPresentation, Snapshot},
    session::Session,
};
use astero_video::presentation::*;
#[test]
fn no_default_presenter() {
    let s = Session::new().unwrap();
    assert!(s.presentation().is_none());
}
#[test]
fn attach_detach_and_drop() {
    let mut s = Session::new().unwrap();
    let a = Endpoint::new("headless");
    s.attach_presentation(a.clone());
    s.detach_presentation();
    assert_eq!(a.snapshot().state, State::Closed);
    let b = Endpoint::new("headless");
    s.attach_presentation(b.clone());
    s.stop().unwrap();
    assert_eq!(b.snapshot().state, State::Closed);
    let mut s = Session::new().unwrap();
    let c = Endpoint::new("headless");
    s.attach_presentation(c.clone());
    drop(s);
    assert_eq!(c.snapshot().state, State::Closed);
}
#[test]
fn readiness_is_not_guest_videoout() {
    let h = Headless::default();
    let mut s = Snapshot::preparing("synthetic".into());
    s.host_presentation = HostPresentation::from(h.snapshot());
    assert_eq!(s.host_presentation.state, "Ready");
    h.present(synthetic(1)).unwrap();
    h.drain().unwrap();
    s.host_presentation = h.snapshot().into();
    assert_eq!(s.host_presentation.state, "Active");
    assert_eq!(s.subsystems["VideoOut"].state, Health::NotReached);
    assert_eq!(s.subsystems["GPU/AGC"].state, Health::NotReached);
    let v = s.json().unwrap();
    assert!(v.contains("host_presentation"));
}

#[test]
fn live_observer_composes_without_owning_presenter() {
    let o =
        astero_core::input::entry::observability::Observer::new("synthetic".into(), None).unwrap();
    let h = Headless::default();
    o.observe_presentation(Some(&h.endpoint()));
    assert_eq!(o.snapshot().host_presentation.state, "Ready");
    h.present(synthetic(1)).unwrap();
    h.drain().unwrap();
    assert_eq!(o.snapshot().host_presentation.presented, 1);
    assert_eq!(
        o.snapshot().subsystems["VideoOut"].state,
        Health::NotReached
    );
    drop(h);
    assert_eq!(o.snapshot().host_presentation.state, "Unavailable");
}
