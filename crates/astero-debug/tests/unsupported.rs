use astero_core::{
    observation::{ObservationError, ObserveSession},
    session::Session,
};
use astero_debug::{
    capabilities::{Capability, INVENTORY, Support, Unsupported, require_support},
    control, inspection,
};

#[test]
fn inspection_uses_the_common_snapshot() {
    let mut session = Session::new().unwrap();
    let source = session.observer();
    session.initialize().unwrap();
    let report = inspection::inspect_session(&source).unwrap();
    assert_eq!(report.session, source.snapshot().unwrap());
    assert_eq!(report.capabilities, INVENTORY);
    session.stop().unwrap();
    assert_ne!(
        report.session,
        inspection::inspect_session(&source).unwrap().session
    );
}

#[test]
fn inspection_rejects_closed_session() {
    let session = Session::new().unwrap();
    let source = session.observer();
    drop(session);
    assert_eq!(
        inspection::inspect_session(&source),
        Err(ObservationError::SessionClosed)
    );
}

#[test]
fn all_guest_capabilities_remain_explicitly_unsupported() {
    assert_eq!(INVENTORY.len(), 13);
    assert_eq!(
        INVENTORY
            .iter()
            .filter(|(_, support)| *support == Support::Implemented)
            .count(),
        2
    );
    for &(capability, support) in INVENTORY {
        assert_eq!(
            require_support(capability),
            if support == Support::Implemented {
                Ok(())
            } else {
                Err(Unsupported { capability })
            }
        );
    }
    assert_eq!(
        control::pause(),
        Err(Unsupported {
            capability: Capability::ExecutionControl
        })
    );
}
