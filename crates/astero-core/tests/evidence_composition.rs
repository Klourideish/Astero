use astero_core::session::inputs::Completeness;
use astero_loader::{
    artifact::SourceArtifact,
    elf::{
        dynamic::{
            ObservationLimits,
            candidates::report::{self, LinkageEvidenceReport, ReportBudget},
            hash::HashLimits,
            relocations::{RelocationLimits, RelocationTables},
            symbol_table::SymbolTable,
        },
        inspect,
    },
};
use std::sync::Arc;
// Repository-owned ELF bytes only, independent of any runtime/session state.
fn report(max_symbols: u64, trusted: bool, bad_name: bool) -> LinkageEvidenceReport {
    let mut b = vec![0u8; 0x600];
    b[..7].copy_from_slice(&[127, b'E', b'L', b'F', 2, 1, 1]);
    for (at, v) in [(16, 3u16), (18, 62), (52, 64), (54, 56), (56, 2)] {
        b[at..at + 2].copy_from_slice(&v.to_le_bytes());
    }
    for (at, v) in [
        (20, 1u32),
        (64, 1),
        (68, 6),
        (120, 2),
        (124, 6),
        (0x350, 1),
        (0x354, 3),
        (0x418, 1),
        (0x430, 1),
    ] {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    for (at, v) in [
        (32, 64u64),
        (72, 0x100),
        (80, 0x1100),
        (96, 0x500),
        (104, 0x500),
        (112, 1),
        (128, 0x200),
        (136, 0x1200),
        (152, 96),
        (160, 96),
        (168, 1),
    ] {
        b[at..at + 8].copy_from_slice(&v.to_le_bytes());
    }
    for (i, (tag, value)) in [
        (6u64, 0x1400u64),
        (11, 24),
        (5, 0x1500),
        (10, 3),
        (4, 0x1350),
        (0, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let at = 0x200 + i * 16;
        b[at..at + 8].copy_from_slice(&tag.to_le_bytes());
        b[at + 8..at + 16].copy_from_slice(&value.to_le_bytes());
    }
    b[0x41c] = 0x12;
    b[0x434] = 0x12;
    b[0x436] = 1;
    b[0x501] = 255;
    if bad_name {
        b[0x430] = 3;
    }
    let elf = inspect(
        SourceArtifact::new(b, None).unwrap(),
        Some(astero_loader::modules::ModuleMetadata {
            id: astero_loader::modules::ModuleId(7),
            name: "synthetic context".into(),
        }),
    )
    .unwrap();
    let dl = ObservationLimits { max_entries: 32 };
    let symbols = if trusted {
        SymbolTable::with_hash(&elf, dl, HashLimits { max_words: 64 }).unwrap()
    } else {
        SymbolTable::new(&elf, dl).unwrap()
    };
    let relocations = RelocationTables::new(&elf, dl).unwrap();
    report::collect(
        &elf,
        &symbols,
        &relocations,
        ReportBudget {
            max_symbols,
            max_details: 8,
            max_retained_name_bytes: 32,
            max_name_scan_bytes: 8,
            max_total_name_scan_bytes: 32,
            relocations: RelocationLimits {
                max_entries: 8,
                max_name_scan_bytes: 8,
                max_total_name_scan_bytes: 32,
            },
        },
    )
}

use astero_core::{
    observation::{ObservationError, ObserveSession},
    session::{
        Session, SessionError,
        inputs::{EvidenceTarget, InputError, SessionInputs},
    },
};
fn inputs(report: Arc<LinkageEvidenceReport>) -> SessionInputs {
    SessionInputs::new(
        Some(EvidenceTarget {
            source: report.source(),
            module: report.module(),
        }),
        Some(report),
    )
    .unwrap()
}
#[test]
fn absent_evidence_and_declared_target_without_report_are_explicit() {
    let session = Session::new().unwrap();
    let s = session.observer().snapshot().unwrap();
    assert!(s.inputs.target().is_none());
    assert!(s.inputs.linkage().is_none());
    assert!(s.loaded_target.is_none());
    let r = report(3, true, false);
    let target = EvidenceTarget {
        source: r.source(),
        module: r.module(),
    };
    let session = Session::with_inputs(SessionInputs::new(Some(target), None).unwrap()).unwrap();
    assert_eq!(
        session.observer().snapshot().unwrap().inputs.target(),
        Some(target)
    );
    assert!(
        session
            .observer()
            .snapshot()
            .unwrap()
            .inputs
            .linkage()
            .is_none()
    );
}
#[test]
fn all_completeness_states_and_owned_details_survive_composition() {
    for r in [
        report(3, true, false),
        report(1, true, false),
        report(3, false, false),
        report(3, true, true),
    ] {
        let r = Arc::new(r);
        let mut session = Session::with_inputs(inputs(Arc::clone(&r))).unwrap();
        let observer = session.observer();
        let before = observer.snapshot().unwrap();
        assert!(Arc::ptr_eq(before.inputs.linkage().unwrap(), &r));
        assert_eq!(
            before.inputs.linkage().unwrap().completeness(),
            r.completeness()
        );
        assert_eq!(before.inputs.target().unwrap().source, r.source());
        assert_eq!(before, observer.snapshot().unwrap());
        session.initialize().unwrap();
        assert_eq!(session.run(), Err(SessionError::NoGuestLoaded));
        let after = observer.snapshot().unwrap();
        assert_ne!(after.lifecycle, before.lifecycle);
        assert!(Arc::ptr_eq(after.inputs.linkage().unwrap(), &r));
        assert!(after.loaded_target.is_none());
    }
}
#[test]
fn inputs_reject_missing_target_and_source_or_module_mismatch() {
    let r = Arc::new(report(3, true, false));
    assert_eq!(
        SessionInputs::new(None, Some(Arc::clone(&r))),
        Err(InputError::TargetRequired)
    );
    let other = report(3, true, false);
    for target in [
        EvidenceTarget {
            source: other.source(),
            module: r.module(),
        },
        EvidenceTarget {
            source: r.source(),
            module: Some(astero_loader::modules::ModuleId(99)),
        },
    ] {
        assert!(matches!(
            SessionInputs::new(Some(target), Some(Arc::clone(&r))),
            Err(InputError::IdentityMismatch { .. })
        ));
    }
}
#[test]
fn observers_share_evidence_but_do_not_keep_session_alive() {
    let r = Arc::new(report(3, true, false));
    let session = Session::with_inputs(inputs(Arc::clone(&r))).unwrap();
    let observer = session.observer();
    let strong = Arc::strong_count(&r);
    let clone = observer.clone();
    assert_eq!(Arc::strong_count(&r), strong);
    let a = observer.snapshot().unwrap();
    let b = clone.snapshot().unwrap();
    assert!(Arc::ptr_eq(
        a.inputs.linkage().unwrap(),
        b.inputs.linkage().unwrap()
    ));
    drop(session);
    assert_eq!(clone.snapshot(), Err(ObservationError::SessionClosed));
    assert_eq!(
        a.inputs.linkage().unwrap().completeness(),
        &Completeness::Complete
    );
}
