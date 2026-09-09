mod dynamic_fixtures;
use astero_loader::elf::dynamic::{
    ObservationLimits,
    candidates::{
        self, CandidateLimits,
        classification::{Classification as C, Reason as R},
        error::CandidateError,
        evidence::{NameState, RelocationUse},
    },
    hash::HashLimits,
    relocations::{RelocationLimits, RelocationTables, error::RelocationError},
    symbol_table::{Binding, EnumerationLimits, SymbolError, SymbolTable, SymbolType, Visibility},
};
use dynamic_fixtures::{image, parsed, program, put16, put32, put64};
fn dl() -> ObservationLimits {
    ObservationLimits { max_entries: 64 }
}
fn hl() -> HashLimits {
    HashLimits { max_words: 256 }
}
fn limits() -> CandidateLimits {
    CandidateLimits {
        symbols: EnumerationLimits {
            max_symbols: 64,
            max_name_scan_bytes: 32,
            max_total_name_scan_bytes: 1024,
        },
        relocations: RelocationLimits {
            max_entries: 64,
            max_name_scan_bytes: 32,
            max_total_name_scan_bytes: 1024,
        },
    }
}
fn fixture(count: u32, ordinary: &[u32], plt: &[u32]) -> Vec<u8> {
    let mut tags = vec![(6, 0x2000), (11, 24), (5, 0x2800), (10, 9), (4, 0x1600)];
    if !ordinary.is_empty() {
        tags.extend([(7, 0x1800), (8, ordinary.len() as u64 * 24), (9, 24)]);
    }
    if !plt.is_empty() {
        tags.extend([(23, 0x1c00), (2, plt.len() as u64 * 24), (20, 7)]);
    }
    tags.push((0, 0));
    let mut b = image(&tags);
    b.resize(0x2000, 0);
    program(&mut b, 0, 1, 0x100, 0x1100, 0x1f00, 0x2000);
    put32(&mut b, 0x600, 1);
    put32(&mut b, 0x604, count);
    b[0x1800..0x1809].copy_from_slice(b"\0name\0\xff\0\0");
    for (at, entries) in [(0x800, ordinary), (0xc00, plt)] {
        for (i, &symbol) in entries.iter().enumerate() {
            put64(&mut b, at + 24 * i, 0x1234 + i as u64);
            put64(
                &mut b,
                at + 24 * i + 8,
                (u64::from(symbol) << 32) | 0xffffeeee,
            );
            b[at + 24 * i + 16..at + 24 * i + 24].copy_from_slice(&(-17i64).to_le_bytes());
        }
    }
    b
}
fn symbol(b: &mut [u8], i: usize, name: u32, info: u8, other: u8, section: u16) {
    let at = 0x1000 + 24 * i;
    put32(b, at, name);
    b[at + 4] = info;
    b[at + 5] = other;
    put16(b, at + 6, section);
    put64(b, at + 8, 0x123456);
    put64(b, at + 16, 37);
}
#[test]
fn imports_preserve_both_relocation_classes_duplicates_and_null() {
    let mut b = fixture(3, &[0, 1, 1], &[1]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    symbol(&mut b, 2, 1, 0x21, 0, 0);
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let rows = candidates::enumerate(&s, &r, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows[0].classification(), C::Null);
    assert!(rows[0].import().is_none());
    assert_eq!(rows[0].evidence().relocations().len(), 1);
    assert_eq!(rows[1].classification(), C::ImportCandidate);
    assert!(rows[1].import().is_some());
    assert!(rows[1].export().is_none());
    assert_eq!(rows[1].evidence().relocation_use(), RelocationUse::Both);
    assert_eq!(rows[1].evidence().relocations().len(), 3);
    for raw in rows[1].evidence().relocations() {
        assert_eq!(raw.record.addend, -17);
        assert_eq!(raw.record.relocation_type(), 0xffffeeee);
        assert_eq!(raw.source.source_id(), elf.artifact().source().identity());
    }
    assert_eq!(
        rows[2].evidence().relocation_use(),
        RelocationUse::NoneObserved
    );
    assert_eq!(
        rows[1].evidence().symbol().name,
        rows[2].evidence().symbol().name
    );
    assert_ne!(rows[1].evidence().identity(), rows[2].evidence().identity());
    assert_eq!(
        rows[1].import().unwrap().evidence().identity().symbol_index,
        1
    );
    assert_eq!(rows[1].evidence().extent(), s.extent().unwrap());
    assert_eq!(
        rows,
        candidates::enumerate(&s, &r, limits())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    );
    assert!(elf.artifact().description().imports.is_empty());
    assert!(elf.artifact().description().exports.is_empty());
}
#[test]
fn names_remain_absent_empty_utf8_or_raw_without_normalization() {
    let mut b = fixture(5, &[], &[]);
    for (i, name) in [0, 8, 1, 6].into_iter().enumerate() {
        symbol(&mut b, i + 1, name, 0x12, 0, 0);
    }
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let rows = candidates::enumerate(&s, &r, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for (row, state) in rows[1..].iter().zip([
        NameState::Absent,
        NameState::Empty,
        NameState::Utf8,
        NameState::Bytes,
    ]) {
        assert_eq!(row.classification(), C::ImportCandidate);
        assert_eq!(row.evidence().name_state(), state);
    }
    let name = rows[4].evidence().symbol().name.unwrap();
    assert_eq!(name.as_bytes(), &[255]);
    assert!(name.as_utf8().is_err());
    assert_eq!(
        name.source_range().source_id(),
        elf.artifact().source().identity()
    );
}
#[test]
fn exports_require_external_attributes_and_nonempty_names() {
    let cases = [
        (1, 0x12, 0, 1, C::ExportCandidate),
        (1, 0x21, 3, 1, C::ExportCandidate),
        (1, 0x02, 0, 1, C::InternalDefined(R::LocalBinding)),
        (1, 0x12, 2, 1, C::InternalDefined(R::NonExternalVisibility)),
        (1, 0x12, 1, 1, C::InternalDefined(R::NonExternalVisibility)),
        (0, 0x12, 0, 1, C::Unclassified(R::MissingName)),
        (8, 0x12, 0, 1, C::Unclassified(R::EmptyName)),
        (6, 0x12, 0, 1, C::ExportCandidate),
        (1, 0x12, 0, 0xfff1, C::Absolute),
        (1, 0x12, 0, 0xfff2, C::Common),
        (1, 0x12, 0, 0xffff, C::Unclassified(R::SpecialSection)),
        (1, 0x12, 0, 0xff10, C::Unclassified(R::SpecialSection)),
    ];
    let mut b = fixture(cases.len() as u32 + 1, &[], &[]);
    for (i, &(n, info, v, section, _)) in cases.iter().enumerate() {
        symbol(&mut b, i + 1, n, info, v, section);
    }
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let rows = candidates::enumerate(&s, &r, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for (row, case) in rows[1..].iter().zip(cases) {
        assert_eq!(row.classification(), case.4);
        assert_eq!(row.export().is_some(), case.4 == C::ExportCandidate);
    }
    assert_eq!(
        rows[1].export().unwrap().evidence().symbol().value,
        0x123456
    );
    assert_ne!(rows[1].evidence().identity(), rows[3].evidence().identity());
}
#[test]
fn uncertain_attributes_are_explicit_and_raw_values_survive() {
    let cases = [
        (0x02, 0, 0, C::UndefinedUnclassified(R::LocalBinding)),
        (
            0x12,
            3,
            0,
            C::UndefinedUnclassified(R::NonExternalVisibility),
        ),
        (0xa2, 0, 0, C::UndefinedUnclassified(R::UnknownAttributes)),
        (0x1e, 0, 0, C::UndefinedUnclassified(R::UnsupportedType)),
        (0x12, 7, 1, C::Unclassified(R::UnknownAttributes)),
        (0x12, 128, 1, C::Unclassified(R::UnknownAttributes)),
        (0x02, 3, 1, C::Unclassified(R::ConflictingAttributes)),
    ];
    let mut b = fixture(cases.len() as u32 + 1, &[], &[]);
    for (i, &(info, v, section, _)) in cases.iter().enumerate() {
        symbol(&mut b, i + 1, 1, info, v, section);
    }
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let rows = candidates::enumerate(&s, &r, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    for (row, (info, v, _, expected)) in rows[1..].iter().zip(cases) {
        assert_eq!(row.classification(), expected);
        assert_eq!(row.evidence().symbol().info, info);
        assert_eq!(row.evidence().symbol().other, v);
    }
    assert_eq!(rows[3].evidence().symbol().binding, Binding::Unknown(10));
    assert_eq!(
        rows[4].evidence().symbol().symbol_type,
        SymbolType::Unknown(14)
    );
    assert_eq!(
        rows[5].evidence().symbol().visibility,
        Visibility::Unknown(7)
    );
}
#[test]
fn only_trusted_membership_and_matching_sources_enable_candidates() {
    let mut b = fixture(2, &[], &[]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    symbol(&mut b, 2, 1, 0x12, 0, 0);
    let elf = parsed(b.clone());
    let other = parsed(b);
    let untrusted = SymbolTable::new(&elf, dl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    assert!(matches!(
        candidates::enumerate(&untrusted, &r, limits()),
        Err(CandidateError::Symbol(SymbolError::CountUnavailable))
    ));
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(s.read_candidate(2, 32).is_ok());
    let rows = candidates::enumerate(&s, &r, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows.last().unwrap().evidence().identity().symbol_index, 1);
    let wrong = RelocationTables::new(&other, dl()).unwrap();
    assert!(matches!(
        candidates::enumerate(&s, &wrong, limits()),
        Err(CandidateError::SourceMismatch { .. })
    ));
}
#[test]
fn invalid_names_fail_stop_and_never_become_candidates() {
    let mut b = fixture(4, &[], &[]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    symbol(&mut b, 2, 9, 0x12, 0, 0);
    symbol(&mut b, 3, 1, 0x12, 0, 0);
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let mut it = candidates::enumerate(&s, &r, limits()).unwrap();
    assert!(it.next().unwrap().is_ok());
    assert!(it.next().unwrap().is_ok());
    assert!(matches!(
        it.next().unwrap(),
        Err(CandidateError::Symbol(SymbolError::Name { index: 2, .. }))
    ));
    assert!(it.next().is_none());
    assert!(it.next().is_none());
}
#[test]
fn relocation_reference_failures_are_not_skipped() {
    let mut b = fixture(2, &[2], &[]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    assert!(matches!(
        candidates::enumerate(&s, &r, limits()),
        Err(CandidateError::Relocation(_))
    ));
}
#[test]
fn budgets_are_errors_not_successful_truncated_enumerations() {
    let mut b = fixture(3, &[1, 1], &[]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    symbol(&mut b, 2, 1, 0x12, 0, 0);
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let mut l = limits();
    l.symbols.max_symbols = 2;
    assert!(matches!(
        candidates::enumerate(&s, &r, l),
        Err(CandidateError::Symbol(SymbolError::EnumerationLimit { .. }))
    ));
    l = limits();
    l.relocations.max_entries = 1;
    assert!(matches!(
        candidates::enumerate(&s, &r, l),
        Err(CandidateError::Relocation(
            RelocationError::EntryBudget { .. }
        ))
    ));
    l = limits();
    l.relocations.max_total_name_scan_bytes = 5;
    assert!(matches!(
        candidates::enumerate(&s, &r, l),
        Err(CandidateError::Relocation(_))
    ));
    l = limits();
    l.symbols.max_total_name_scan_bytes = 5;
    let mut it = candidates::enumerate(&s, &r, l).unwrap();
    assert!(it.next().unwrap().is_ok());
    assert!(it.next().unwrap().is_ok());
    assert!(it.next().unwrap().is_err());
    assert!(it.next().is_none());
}
#[test]
fn bounded_attribute_and_relocation_sweep_preserves_determinism() {
    // 192 independent small images, each with four name states: bounded, not exhaustive fuzzing.
    for info in [0x02, 0x10, 0x21, 0x16, 0xa2, 0x1e] {
        for visibility in [0, 2, 3, 7] {
            for section in [0, 1] {
                for use_kind in 0..4 {
                    let ordinary = if use_kind & 1 != 0 {
                        vec![1, 2, 3, 4]
                    } else {
                        vec![]
                    };
                    let plt = if use_kind & 2 != 0 {
                        vec![1, 2, 3, 4]
                    } else {
                        vec![]
                    };
                    let mut b = fixture(5, &ordinary, &plt);
                    for (i, name) in [0, 8, 1, 6].into_iter().enumerate() {
                        symbol(&mut b, i + 1, name, info, visibility, section);
                    }
                    let elf = parsed(b);
                    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
                    let r = RelocationTables::new(&elf, dl()).unwrap();
                    let collect = || {
                        candidates::enumerate(&s, &r, limits())
                            .unwrap()
                            .collect::<Result<Vec<_>, _>>()
                            .unwrap()
                    };
                    let rows = collect();
                    assert_eq!(rows, collect());
                    for row in &rows[1..] {
                        assert_eq!(row.evidence().symbol().info, info);
                        assert_eq!(row.evidence().symbol().other, visibility);
                        assert_eq!(
                            row.evidence().relocation_use(),
                            [
                                RelocationUse::NoneObserved,
                                RelocationUse::Ordinary,
                                RelocationUse::Plt,
                                RelocationUse::Both
                            ][use_kind]
                        );
                        if info == 0xa2 || info == 0x1e || visibility == 7 {
                            assert!(row.import().is_none() && row.export().is_none());
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn aliased_relocation_record_is_associated_once_with_both_labels() {
    let mut b = fixture(2, &[1], &[1]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    // Point DT_JMPREL at the complete ordinary RELA range, preserving M9 alias policy.
    put64(&mut b, 0x200 + 8 * 16 + 8, 0x1800);
    let elf = parsed(b);
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let rows = candidates::enumerate(&s, &r, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let e = rows[1].evidence();
    assert_eq!(e.relocation_use(), RelocationUse::Both);
    assert_eq!(e.relocations().len(), 1);
    assert_eq!(e.relocations()[0].dynamic_index, Some(0));
    assert_eq!(e.relocations()[0].plt_index, Some(0));
}
