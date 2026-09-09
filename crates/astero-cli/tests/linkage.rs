use astero_debug::snapshots::linkage::{Completeness, inspect_linkage};
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
fn cli_uses_candidate_labels_and_byte_faithful_details() {
    let snapshot = inspect_linkage(Some(Arc::new(report(3, true, false))));
    assert_eq!(
        snapshot.report().unwrap().completeness(),
        &Completeness::Complete
    );
    let text = astero_cli::linkage::render(&snapshot, true);
    assert!(text.contains("Import candidates: 1"));
    assert!(text.contains("Export candidates: 1"));
    assert!(text.contains("Enumeration: complete"));
    assert!(text.contains("\\xff"));
    assert!(text.contains("Symbol 1:"));
    assert!(text.contains("Symbol 2:"));
    assert!(!text.contains("resolved import"));
    assert!(!astero_cli::linkage::render(&snapshot, false).contains("Symbol 1:"));
}
#[test]
fn cli_preserves_completeness_and_never_prints_partial_totals() {
    for (r, label) in [
        (report(2, true, false), "partial"),
        (report(3, false, false), "unavailable"),
        (report(3, true, true), "failed"),
    ] {
        let snapshot = inspect_linkage(Some(Arc::new(r)));
        let text = astero_cli::linkage::render(&snapshot, true);
        assert!(text.contains(&format!("Enumeration: {label}")));
        assert!(text.contains("Import candidates observed:"));
        assert!(!text.contains("Import candidates:"));
    }
    let text = astero_cli::linkage::render(&inspect_linkage(None), false);
    assert!(text.contains("unavailable"));
    assert!(!text.contains("candidates: 0"));
}
#[test]
fn executable_linkage_command_reports_absence_honestly() {
    for args in [vec!["--linkage"], vec!["--linkage", "--details"]] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_astero-cli"))
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("unavailable (no report supplied)")
        );
    }
}
