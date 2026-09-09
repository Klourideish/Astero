mod dynamic_fixtures;
use astero_loader::elf::{
    address_translation::TranslationError,
    dynamic::{
        ObservationLimits,
        error::DynamicError,
        hash::HashLimits,
        relocations::{
            RelocationLimits, RelocationTables,
            descriptor::{RelocationFormat, TableKind},
            error::RelocationError,
            symbol_reference::SymbolReferenceKind,
        },
        symbol_table::{SymbolError, SymbolTable},
    },
};
use dynamic_fixtures::{image, parsed, program, put32, put64};
fn dl() -> ObservationLimits {
    ObservationLimits { max_entries: 64 }
}
fn hl() -> HashLimits {
    HashLimits { max_words: 128 }
}
fn limits() -> RelocationLimits {
    RelocationLimits {
        max_entries: 64,
        max_name_scan_bytes: 16,
        max_total_name_scan_bytes: 128,
    }
}
fn fixture(extra: &[(i64, u64)], hash: bool) -> Vec<u8> {
    let mut tags = vec![(6, 0x2000), (11, 24), (5, 0x2800), (10, 6)];
    if hash {
        tags.push((4, 0x1600));
    }
    tags.extend_from_slice(extra);
    tags.push((0, 0));
    let mut b = image(&tags);
    b.resize(0x2000, 0);
    program(&mut b, 0, 1, 0x100, 0x1100, 0x1f00, 0x2000);
    put32(&mut b, 0x600, 1);
    put32(&mut b, 0x604, 3);
    b[0x1800..0x1806].copy_from_slice(b"\0name\0");
    put32(&mut b, 0x1018, 1);
    put32(&mut b, 0x1030, 1);
    b
}
fn rela_tags(count: u64) -> Vec<(i64, u64)> {
    vec![(7, 0x1800), (8, count * 24), (9, 24)]
}
fn entry(b: &mut [u8], at: usize, offset: u64, symbol: u32, kind: u32, addend: i64) {
    put64(b, at, offset);
    put64(b, at + 8, (u64::from(symbol) << 32) | u64::from(kind));
    b[at + 16..at + 24].copy_from_slice(&addend.to_le_bytes());
}
fn discover(b: Vec<u8>) -> Result<RelocationTables, RelocationError> {
    RelocationTables::new(&parsed(b), dl())
}
#[test]
fn rela_fields_references_and_provenance_are_deterministic() {
    let mut b = fixture(&rela_tags(3), true);
    entry(&mut b, 0x800, 0x4567, 0, 8, -17);
    entry(&mut b, 0x818, 0x89ab, 1, 0xffffeeee, 19);
    entry(&mut b, 0x830, u64::MAX, 2, u32::MAX, i64::MIN);
    let elf = parsed(b);
    let tables = RelocationTables::new(&elf, dl()).unwrap();
    let symbols = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let extent = &tables.extents()[0];
    assert_eq!(extent.address().0, 0x1800);
    assert_eq!(extent.entry_size(), 24);
    assert_eq!(extent.entry_count(), 3);
    assert_eq!(extent.origins().len(), 3);
    let observed = tables
        .enumerate(&symbols, limits())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        observed,
        tables
            .enumerate(&symbols, limits())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    );
    assert_eq!(observed[0].symbol.kind, SymbolReferenceKind::Null);
    assert_eq!(observed[1].symbol.kind, SymbolReferenceKind::Nonzero(1));
    assert_eq!(observed[2].symbol.kind, SymbolReferenceKind::Nonzero(2));
    assert_eq!(
        observed[2].raw.record.info,
        (2u64 << 32) | u64::from(u32::MAX)
    );
    assert_eq!(observed[0].raw.record.addend, -17);
    assert_eq!(observed[1].raw.record.addend, 19);
    assert_eq!(observed[2].raw.record.addend, i64::MIN);
    assert_eq!(observed[2].raw.record.offset, u64::MAX);
    assert_eq!(observed[1].raw.record.relocation_type(), 0xffffeeee);
    assert_eq!(
        observed[1].raw.source,
        elf.artifact().source().checked_range(0x818, 24).unwrap()
    );
    assert_eq!(observed[1].raw.dynamic_index, Some(1));
    assert_eq!(observed[1].raw.plt_index, None);
    assert!(elf.artifact().description().imports.is_empty());
    assert!(elf.artifact().description().exports.is_empty());
}
#[test]
fn minimal_null_relocation_requires_extent_and_null_symbol_observation() {
    let elf = parsed(fixture(&rela_tags(1), true));
    let tables = RelocationTables::new(&elf, dl()).unwrap();
    let symbols = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert_eq!(
        tables
            .read(&tables.extents()[0], 0, &symbols, 0)
            .unwrap()
            .symbol
            .kind,
        SymbolReferenceKind::Null
    );
    let untrusted = SymbolTable::new(&elf, dl()).unwrap();
    assert!(matches!(
        tables.read(&tables.extents()[0], 0, &untrusted, 0),
        Err(RelocationError::SymbolExtentUnavailable { symbol: 0, .. })
    ));
    let mut b = fixture(&rela_tags(1), true);
    b[0x1000] = 1;
    let elf = parsed(b);
    let symbols = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let tables = RelocationTables::new(&elf, dl()).unwrap();
    assert!(matches!(
        tables.read(&tables.extents()[0], 0, &symbols, 8),
        Err(RelocationError::SymbolObservation {
            error: SymbolError::InvalidNullSymbol,
            ..
        })
    ));
}
#[test]
fn raw_maximum_info_is_preserved_but_invalid_reference_is_rejected() {
    for symbol in [3, u32::MAX] {
        let mut b = fixture(&rela_tags(1), true);
        entry(&mut b, 0x800, 1, symbol, u32::MAX, i64::MAX);
        let elf = parsed(b);
        let tables = RelocationTables::new(&elf, dl()).unwrap();
        let symbols = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
        let t = &tables.extents()[0];
        let raw = tables.read_raw(t, 0).unwrap();
        assert_eq!(raw.record.symbol_index(), symbol);
        assert_eq!(raw.record.relocation_type(), u32::MAX);
        assert_eq!(raw.record.addend, i64::MAX);
        assert!(
            matches!(tables.read(t,0,&symbols,8),Err(RelocationError::SymbolIndex{count:3,symbol:s,..}) if s==symbol)
        );
    }
}
#[test]
fn source_mismatches_cannot_rebind_relocation_or_symbol_evidence() {
    let a = parsed(fixture(&rela_tags(1), true));
    let b = parsed(fixture(&rela_tags(1), true));
    let ta = RelocationTables::new(&a, dl()).unwrap();
    let tb = RelocationTables::new(&b, dl()).unwrap();
    let sb = SymbolTable::with_hash(&b, dl(), hl()).unwrap();
    assert!(matches!(
        ta.read_raw(&tb.extents()[0], 0),
        Err(RelocationError::ExtentSourceMismatch { .. })
    ));
    assert!(matches!(
        ta.read(&ta.extents()[0], 0, &sb, 8),
        Err(RelocationError::SymbolSourceMismatch { .. })
    ));
    assert!(matches!(
        ta.enumerate(&sb, limits()),
        Err(RelocationError::SymbolSourceMismatch { .. })
    ));
}
#[test]
fn symbol_name_failure_is_not_silently_downgraded_to_in_range() {
    let mut b = fixture(&rela_tags(1), true);
    entry(&mut b, 0x800, 0, 1, 0, 0);
    put32(&mut b, 0x1018, 100);
    let elf = parsed(b);
    let t = RelocationTables::new(&elf, dl()).unwrap();
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(matches!(
        t.read(&t.extents()[0], 0, &s, 8),
        Err(RelocationError::SymbolObservation {
            symbol: 1,
            error: SymbolError::Name { .. },
            ..
        })
    ));
}
#[test]
fn no_hash_keeps_reference_validation_unavailable() {
    let elf = parsed(fixture(&rela_tags(1), false));
    let t = RelocationTables::new(&elf, dl()).unwrap();
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(t.read_raw(&t.extents()[0], 0).is_ok());
    assert!(matches!(
        t.enumerate(&s, limits()),
        Err(RelocationError::SymbolExtentUnavailable { .. })
    ));
}
#[test]
fn incomplete_rela_descriptors_and_entry_sizes_fail_at_m5() {
    for missing in 0..3 {
        let mut tags = rela_tags(1);
        tags.remove(missing);
        assert!(matches!(
            discover(fixture(&tags, true)),
            Err(RelocationError::Dynamic(
                DynamicError::IncompleteDescriptor { .. }
            ))
        ));
    }
    for width in [0, 16, 23, 25, u64::MAX] {
        assert!(matches!(
            discover(fixture(&[(7, 0x1800), (8, 24), (9, width)], true)),
            Err(RelocationError::Dynamic(
                DynamicError::UnsupportedEntrySize { .. }
            ))
        ));
    }
}
#[test]
fn descriptor_source_boundaries_and_overflow_remain_structured() {
    for (address, ok) in [
        (0x3000 - 24, true),
        (0x3000 - 23, false),
        (0x3000, false),
        (0x4000, false),
        (u64::MAX - 7, false),
    ] {
        let r = discover(fixture(&[(7, address), (8, 24), (9, 24)], true));
        assert_eq!(r.is_ok(), ok);
        if !ok {
            assert!(matches!(
                r,
                Err(RelocationError::Dynamic(DynamicError::Translation { .. }))
            ));
        }
    }
    assert!(matches!(
        discover(fixture(&[(7, 0x3000), (8, 24), (9, 24)], true)),
        Err(RelocationError::Dynamic(DynamicError::Translation {
            error: TranslationError::ZeroFill { .. },
            ..
        }))
    ));
}
#[test]
fn plt_rela_provenance_and_rel_deferral_are_explicit() {
    let mut b = fixture(&[(23, 0x1900), (2, 24), (20, 7)], true);
    entry(&mut b, 0x900, 123, 1, 456, -7);
    let elf = parsed(b);
    let t = RelocationTables::new(&elf, dl()).unwrap();
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let o = t.read(&t.extents()[0], 0, &s, 8).unwrap();
    assert_eq!(o.raw.table_kind, TableKind::Plt);
    assert_eq!(o.raw.plt_index, Some(0));
    assert_eq!(o.raw.dynamic_index, None);
    let elf = parsed(fixture(&[(23, 0x1900), (2, 16), (20, 17)], true));
    let t = RelocationTables::new(&elf, dl()).unwrap();
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert_eq!(t.extents()[0].format(), RelocationFormat::DeferredRel);
    assert_eq!(t.extents()[0].entry_size(), 16);
    assert!(matches!(
        t.read_raw(&t.extents()[0], 0),
        Err(RelocationError::UnsupportedRel { .. })
    ));
    assert!(matches!(
        t.enumerate(&s, limits()),
        Err(RelocationError::UnsupportedRel { .. })
    ));
}
#[test]
fn malformed_plt_groups_and_formats_remain_m5_errors() {
    for tags in [
        vec![(23, 0x1900), (2, 23), (20, 7)],
        vec![(23, 0x1900), (2, 24), (20, 99)],
        vec![(23, 0x1900), (20, 7)],
    ] {
        assert!(matches!(
            discover(fixture(&tags, true)),
            Err(RelocationError::Dynamic(
                DynamicError::InvalidDescriptorSize { .. }
                    | DynamicError::UnsupportedPltRelocationKind(_)
                    | DynamicError::IncompleteDescriptor { .. }
            ))
        ));
    }
}
#[test]
fn aliases_retain_both_labels_and_canonical_enumeration_never_duplicates() {
    for first in [0, 1, 2] {
        let mut tags = rela_tags(3);
        tags.extend([(23, 0x1800 + first * 24), (2, (3 - first) * 24), (20, 7)]);
        let mut b = fixture(&tags, true);
        for i in 0..3 {
            entry(&mut b, 0x800 + i * 24, i as u64, i as u32, 9, 0);
        }
        let elf = parsed(b);
        let t = RelocationTables::new(&elf, dl()).unwrap();
        let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
        assert_eq!(t.tail_alias().unwrap().first_dynamic_index, first);
        let records = t
            .enumerate(&s, limits())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(records.len(), 3);
        for (i, o) in records.iter().enumerate() {
            assert_eq!(o.raw.record.offset, i as u64);
            assert_eq!(o.raw.dynamic_index, Some(i as u64));
            assert_eq!(
                o.raw.plt_index,
                if i as u64 >= first {
                    Some(i as u64 - first)
                } else {
                    None
                }
            );
        }
        let direct = t.read_raw(&t.extents()[0], first).unwrap();
        assert_eq!(direct.plt_index, Some(0));
    }
}
#[test]
fn conflicting_overlap_and_duplicate_tags_are_rejected() {
    for (addr, size, format) in [
        (0x1818, 24, 7),
        (0x1830, 48, 7),
        (0x1801, 48, 7),
        (0x1800, 48, 17),
    ] {
        let mut tags = rela_tags(3);
        tags.extend([(23, addr), (2, size), (20, format)]);
        assert!(matches!(
            discover(fixture(&tags, true)),
            Err(RelocationError::DescriptorConflict { .. })
        ));
    }
    for value in [0x1800, 0x1900] {
        let mut tags = rela_tags(1);
        tags.push((7, value));
        assert!(matches!(
            discover(fixture(&tags, true)),
            Err(RelocationError::Dynamic(DynamicError::DuplicateTag { .. }))
        ));
    }
}
#[test]
fn disjoint_tables_and_empty_tables_preserve_exact_counts() {
    let mut tags = rela_tags(1);
    tags.extend([(23, 0x1900), (2, 24), (20, 7)]);
    let elf = parsed(fixture(&tags, true));
    let t = RelocationTables::new(&elf, dl()).unwrap();
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(t.tail_alias().is_none());
    assert_eq!(t.enumerate(&s, limits()).unwrap().count(), 2);
    for tags in [vec![], rela_tags(0)] {
        let elf = parsed(fixture(&tags, true));
        let t = RelocationTables::new(&elf, dl()).unwrap();
        let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
        assert_eq!(t.enumerate(&s, limits()).unwrap().count(), 0);
    }
}
#[test]
fn iteration_limits_fail_stop_and_do_not_report_partial_completion() {
    let mut b = fixture(&rela_tags(3), true);
    for i in 0..3 {
        entry(&mut b, 0x800 + i * 24, 0, 1, 0, 0);
    }
    let elf = parsed(b);
    let t = RelocationTables::new(&elf, dl()).unwrap();
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(matches!(
        t.enumerate(
            &s,
            RelocationLimits {
                max_entries: 2,
                ..limits()
            }
        ),
        Err(RelocationError::EntryBudget { count: 3, limit: 2 })
    ));
    let mut iter = t
        .enumerate(
            &s,
            RelocationLimits {
                max_total_name_scan_bytes: 5,
                ..limits()
            },
        )
        .unwrap();
    assert!(iter.next().unwrap().is_ok());
    assert!(matches!(
        iter.next(),
        Some(Err(RelocationError::SymbolObservation { .. }))
    ));
    assert!(iter.next().is_none());
    assert!(iter.next().is_none());
}
#[test]
fn bounded_size_index_info_and_addend_sweeps() {
    for size in 0..=73 {
        let r = discover(fixture(&[(7, 0x1800), (8, size), (9, 24)], true));
        assert_eq!(r.is_ok(), size % 24 == 0);
    }
    let t = discover(fixture(&rela_tags(3), true)).unwrap();
    for index in [0, 1, 2, 3, 4, u64::MAX] {
        assert_eq!(t.read_raw(&t.extents()[0], index).is_ok(), index < 3);
    }
    for symbol in [0, 1, 2, 3, u32::MAX] {
        for kind in [0, 1, 0x80000000, u32::MAX] {
            for addend in [i64::MIN, -1, 0, 1, i64::MAX] {
                let mut b = fixture(&rela_tags(1), true);
                entry(&mut b, 0x800, u64::MAX, symbol, kind, addend);
                let elf = parsed(b);
                let t = RelocationTables::new(&elf, dl()).unwrap();
                let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
                let raw = t.read_raw(&t.extents()[0], 0).unwrap();
                assert_eq!(
                    (
                        raw.record.symbol_index(),
                        raw.record.relocation_type(),
                        raw.record.addend
                    ),
                    (symbol, kind, addend)
                );
                assert_eq!(t.read(&t.extents()[0], 0, &s, 16).is_ok(), symbol < 3);
            }
        }
    }
}
