use astero_debug::snapshots::linkage::{Completeness, inspect_standalone_report};
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
    let elf = inspect(SourceArtifact::new(b, None).unwrap(), None).unwrap();
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

#[test]
fn snapshot_pins_the_same_owned_report_without_reclassification() {
    let report = Arc::new(report(3, true, false));
    let pointer = Arc::as_ptr(&report);
    let snapshot = inspect_standalone_report(Some(Arc::clone(&report)));
    assert!(std::ptr::eq(snapshot.report().unwrap(), pointer));
    assert_eq!(
        snapshot.report().unwrap().completeness(),
        &Completeness::Complete
    );
    assert_eq!(snapshot.report().unwrap().counts().imports, 1);
    drop(report);
    assert_eq!(
        snapshot.report().unwrap().details()[1].name,
        Some(vec![255])
    );
}
#[test]
fn snapshot_preserves_partial_unavailable_failed_and_absent_states() {
    assert!(inspect_standalone_report(None).report().is_none());
    for r in [
        report(1, true, false),
        report(3, false, false),
        report(3, true, true),
    ] {
        let expected = r.completeness().clone();
        let snapshot = inspect_standalone_report(Some(Arc::new(r)));
        assert_eq!(snapshot.report().unwrap().completeness(), &expected);
        assert!(!matches!(expected, Completeness::Complete));
    }
}

#[test]
fn session_linkage_inspection_captures_lifecycle_and_report_together() {
    use astero_core::{
        observation::{ObservationError, ObserveSession},
        session::{
            Session,
            inputs::{EvidenceTarget, SessionInputs},
        },
    };
    use astero_debug::snapshots::linkage::inspect_linkage;
    for r in [
        report(3, true, false),
        report(1, true, false),
        report(3, false, false),
        report(3, true, true),
    ] {
        let r = Arc::new(r);
        let target = EvidenceTarget {
            source: r.source(),
            module: r.module(),
        };
        let mut session =
            Session::with_inputs(SessionInputs::new(Some(target), Some(Arc::clone(&r))).unwrap())
                .unwrap();
        session.initialize().unwrap();
        let observer = session.observer();
        let s = inspect_linkage(&observer).unwrap();
        assert_eq!(s.session().unwrap(), &observer.snapshot().unwrap());
        assert!(std::ptr::eq(s.report().unwrap(), Arc::as_ptr(&r)));
        assert_eq!(s.report().unwrap().completeness(), r.completeness());
        assert!(inspect_standalone_report(Some(r)).session().is_none());
        drop(session);
        assert_eq!(
            inspect_linkage(&observer),
            Err(ObservationError::SessionClosed)
        );
    }
}
