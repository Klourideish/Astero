//! Fixed generated in-memory ELF demo input only; no files or PS5 runtime target.

use crate::{
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntheticCase {
    Complete,
    Partial,
    Unavailable,
    Failed,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyntheticError {
    Source(crate::artifact::SourceError),
    Elf(crate::elf::ElfError),
    Symbol(crate::elf::dynamic::symbol_table::SymbolError),
    Relocation(crate::elf::dynamic::relocations::error::RelocationError),
}
pub fn report(
    case: SyntheticCase,
    max_symbols: u64,
) -> Result<super::LinkageEvidenceReport, SyntheticError> {
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
        (0x354, 8),
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
        (152, 192),
        (160, 192),
        (168, 1),
    ] {
        b[at..at + 8].copy_from_slice(&v.to_le_bytes());
    }
    for (i, (tag, value)) in [
        (6u64, 0x1400u64),
        (11, 24),
        (5, 0x1500),
        (10, 9),
        (4, 0x1350),
        (7, 0x1520),
        (8, 24),
        (9, 24),
        (23, 0x1538),
        (2, 24),
        (20, 7),
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
    b[0x500..0x509].copy_from_slice(b"\0\xff\0same\0\0");
    for (index, name, info, other, section) in [
        (3usize, 0u32, 2u8, 0u8, 1u16),
        (4, 8, 0x12, 0, 1),
        (5, 3, 0xa2, 7, 0),
        (6, 3, 0x1f, 0, 0),
        (7, 0, 0x12, 0, 1),
    ] {
        let at = 0x400 + 24 * index;
        b[at..at + 4].copy_from_slice(&name.to_le_bytes());
        b[at + 4] = info;
        b[at + 5] = other;
        b[at + 6..at + 8].copy_from_slice(&section.to_le_bytes());
    }
    for at in [0x520, 0x538] {
        b[at..at + 8].copy_from_slice(&0x1234u64.to_le_bytes());
        b[at + 8..at + 16].copy_from_slice(&((1u64 << 32) | 0xffffeeee).to_le_bytes());
        b[at + 16..at + 24].copy_from_slice(&(-17i64).to_le_bytes());
    }
    if case == SyntheticCase::Failed {
        b[0x400 + 7 * 24..0x400 + 7 * 24 + 4].copy_from_slice(&9u32.to_le_bytes());
    }
    let elf = inspect(
        SourceArtifact::new(b, Some("Astero synthetic linkage demo".into()))
            .map_err(SyntheticError::Source)?,
        None,
    )
    .map_err(SyntheticError::Elf)?;
    let dl = ObservationLimits { max_entries: 32 };
    let symbols = if case == SyntheticCase::Unavailable {
        SymbolTable::new(&elf, dl)
    } else {
        SymbolTable::with_hash(&elf, dl, HashLimits { max_words: 64 })
    }
    .map_err(SyntheticError::Symbol)?;
    let relocations = RelocationTables::new(&elf, dl).map_err(SyntheticError::Relocation)?;
    Ok(report::collect(
        &elf,
        &symbols,
        &relocations,
        ReportBudget {
            max_symbols: if case == SyntheticCase::Partial {
                max_symbols.min(2)
            } else {
                max_symbols
            },
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
    ))
}
