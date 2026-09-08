use astero_core::{
    observation::{ObservationError, ObserveSession},
    session::{Lifecycle, Session},
};
use astero_gui::view_model::SessionView;

#[test]
fn gui_reads_the_same_report_as_debugger_and_observer() {
    let mut session = Session::new().unwrap();
    let source = session.observer();
    let view = SessionView::new(source.clone());
    session.initialize().unwrap();
    assert_eq!(
        view.inspect().unwrap(),
        astero_debug::inspection::inspect_session(&source).unwrap()
    );
    assert_eq!(view.inspect().unwrap().session, source.snapshot().unwrap());
    session.stop().unwrap();
    assert_eq!(
        view.inspect().unwrap().session.lifecycle,
        Lifecycle::Stopped
    );
}

#[test]
fn gui_does_not_own_session_lifetime() {
    let session = Session::new().unwrap();
    let view = SessionView::new(session.observer());
    drop(session);
    assert_eq!(view.inspect(), Err(ObservationError::SessionClosed));
}
