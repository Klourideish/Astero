use astero_core::{
    observation::ObserveSession,
    session::{
        Session,
        inputs::{
            EvidenceOrigin,
            synthetic::{SyntheticCase, session_with_case},
        },
    },
};
use astero_gui::model::linkage::{LinkageView, display_name};
use std::sync::Arc;
#[test]
fn no_evidence_is_unavailable_and_no_guest_is_loaded() {
    let session = Session::new().unwrap();
    let snapshot = session.observer().snapshot().unwrap();
    let view = LinkageView::new(&snapshot);
    assert!(view.report().is_none());
    assert!(view.status().contains("unavailable"));
    assert_eq!(view.loaded_state(), "No guest loaded");
    assert!(view.details().is_empty());
    assert!(view.selected(0).is_none());
    assert!(!view.provenance().starts_with("Synthetic"));
}
#[test]
fn all_report_states_preserve_session_provenance_and_counts() {
    for (case, label) in [
        (SyntheticCase::Complete, "Enumeration: Complete"),
        (SyntheticCase::Partial, "Enumeration: Partial"),
        (SyntheticCase::Unavailable, "unavailable"),
        (SyntheticCase::Failed, "failed"),
    ] {
        let session = session_with_case(case, 8).unwrap();
        let snapshot = session.observer().snapshot().unwrap();
        let view = LinkageView::new(&snapshot);
        assert!(view.status().contains(label));
        assert_eq!(view.provenance(), "Synthetic linkage evidence");
        assert_eq!(snapshot.inputs.origin(), EvidenceOrigin::Synthetic);
        assert_eq!(view.loaded_state(), "No guest loaded");
        assert!(std::ptr::eq(
            view.report().unwrap(),
            Arc::as_ptr(snapshot.inputs.linkage().unwrap())
        ));
        assert!(std::ptr::eq(
            view.counts().unwrap(),
            snapshot.inputs.linkage().unwrap().counts()
        ));
    }
}
#[test]
fn retained_selection_keeps_duplicates_unknowns_and_relocation_labels() {
    let session = session_with_case(SyntheticCase::Complete, 8).unwrap();
    let snapshot = session.observer().snapshot().unwrap();
    let view = LinkageView::new(&snapshot);
    let c = view.counts().unwrap();
    assert_eq!(
        (c.observed, c.imports, c.exports, c.internal, c.unclassified),
        (8, 1, 1, 1, 4)
    );
    assert_eq!(
        (
            c.relocations.unique,
            c.relocations.ordinary,
            c.relocations.plt
        ),
        (2, 1, 1)
    );
    let first = view.selected(1).unwrap();
    let second = view.selected(2).unwrap();
    assert_eq!(first.name, second.name);
    assert_ne!(first.index, second.index);
    assert_eq!((first.relocations.ordinary, first.relocations.plt), (1, 1));
    assert!(std::ptr::eq(first, &view.report().unwrap().details()[1]));
    assert!(view.selected(999).is_none());
    assert_eq!(view.details(), view.report().unwrap().details());
    assert!(format!("{:?}", view.selected(5).unwrap().binding).contains("Unknown"));
}
#[test]
fn name_formats_preserve_absent_empty_utf8_and_raw_bytes() {
    let session = session_with_case(SyntheticCase::Complete, 8).unwrap();
    let snapshot = session.observer().snapshot().unwrap();
    let view = LinkageView::new(&snapshot);
    assert_eq!(display_name(view.selected(0).unwrap()), "<absent name>");
    assert_eq!(display_name(view.selected(4).unwrap()), "<empty name>");
    assert_eq!(display_name(view.selected(1).unwrap()), "<non-UTF8: FF>");
    assert!(display_name(view.selected(5).unwrap()).contains("same"));
    let raw = view.selected(1).unwrap().name.clone();
    let a = display_name(view.selected(1).unwrap());
    let b = display_name(view.selected(1).unwrap());
    assert_eq!(a, b);
    assert_eq!(view.selected(1).unwrap().name, raw);
}
#[test]
fn partial_details_do_not_turn_observed_counts_into_totals() {
    let session = session_with_case(SyntheticCase::Partial, 8).unwrap();
    let snapshot = session.observer().snapshot().unwrap();
    let view = LinkageView::new(&snapshot);
    assert_eq!(view.details().len(), 2);
    assert_eq!(view.counts().unwrap().imports, 1);
    assert!(view.status().contains("budget reached"));
    assert!(view.status().contains("remaining symbols Some(6)"));
    assert!(view.selected(2).is_none());
}
