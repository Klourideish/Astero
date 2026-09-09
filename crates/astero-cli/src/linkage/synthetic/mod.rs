//! Fixed generated in-memory ELF demo input only; no files or PS5 runtime target.

use astero_loader::{
    artifact::SourceArtifact,
    elf::{
        dynamic::{
            ObservationLimits,
            candidates::report::{self, ReportBudget},
            hash::HashLimits,
            relocations::{RelocationLimits, RelocationTables},
            symbol_table::SymbolTable,
        },
        inspect,
    },
};
use std::sync::Arc;
pub fn session(max_symbols: u64) -> Result<astero_core::session::Session, String> {
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
    let elf = inspect(
        SourceArtifact::new(b, Some("Astero synthetic linkage demo".into()))
            .map_err(|e| format!("Synthetic fixture: {e:?}"))?,
        None,
    )
    .map_err(|e| format!("Synthetic fixture: {e:?}"))?;
    let dl = ObservationLimits { max_entries: 32 };
    let symbols = SymbolTable::with_hash(&elf, dl, HashLimits { max_words: 64 })
        .map_err(|e| format!("Synthetic symbol evidence: {e:?}"))?;
    let relocations =
        RelocationTables::new(&elf, dl).map_err(|e| format!("Synthetic fixture: {e:?}"))?;
    let report = Arc::new(report::collect(
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
    ));
    let target = astero_core::session::inputs::EvidenceTarget {
        source: report.source(),
        module: report.module(),
    };
    let inputs = astero_core::session::inputs::SessionInputs::new(Some(target), Some(report))
        .map_err(|e| format!("Synthetic composition: {e:?}"))?;
    astero_core::session::Session::with_inputs(inputs).map_err(|e| format!("Create session: {e:?}"))
}
