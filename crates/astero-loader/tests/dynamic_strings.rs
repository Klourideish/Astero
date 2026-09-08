mod dynamic_fixtures;
mod fixtures;
use astero_loader::{
    admission::{DescriptorKind, MetadataProblem, Rejection, admit},
    artifact::{self, DeferredRequirement},
    dependencies::{Dependency, DependencyName, DependencyNameError},
    elf::{
        ElfInspection,
        address_translation::TranslationError,
        dynamic::{
            DynamicObservation, ObservationLimits,
            dependencies::{
                DependencyObservation, DependencyObservationError as Error, StringLimits, observe,
            },
            error::DynamicError,
            string_table::{DynamicStringTable, StringTableError},
            tags::DynamicTag,
        },
    },
    load_plan::plan,
};
use dynamic_fixtures::*;
fn with_strings(bytes: &[u8], offsets: &[u64]) -> ElfInspection {
    let mut entries = vec![(5, 0x1400), (10, bytes.len() as u64)];
    entries.extend(offsets.iter().map(|&o| (1, o)));
    entries.push((0, 0));
    let mut b = image(&entries);
    b[0x400..0x400 + bytes.len()].copy_from_slice(bytes);
    parsed(b)
}
fn inspect(elf: &ElfInspection) -> Result<DependencyObservation, Error> {
    observe(
        elf,
        ObservationLimits { max_entries: 128 },
        StringLimits {
            max_scan_bytes_per_reference: 512,
            max_total_scan_bytes: 4096,
        },
    )
}
fn table(bytes: &[u8]) -> DynamicStringTable {
    inspect(&with_strings(bytes, &[]))
        .unwrap()
        .string_table()
        .unwrap()
        .clone()
}
fn names(report: &DependencyObservation) -> Vec<&[u8]> {
    report
        .artifact()
        .description()
        .dependencies
        .iter()
        .map(|d| match d {
            Dependency::Named(n) => n.as_bytes(),
            _ => panic!("fabricated module identity"),
        })
        .collect()
}
#[test]
fn bounded_views_handle_empty_suffix_and_exact_final_nul() {
    let t = table(b"\0lib\0");
    for (offset, expected) in [
        (0, &b""[..]),
        (1, &b"lib"[..]),
        (2, &b"ib"[..]),
        (3, &b"b"[..]),
        (4, &b""[..]),
    ] {
        let v = t.lookup(offset, 5).unwrap();
        assert_eq!(v.as_bytes(), expected);
        assert_eq!(v.source_range().extent().offset.0, 0x400 + offset);
        assert_eq!(v.scanned_bytes(), expected.len() as u64 + 1);
        assert_eq!(v.as_utf8().unwrap().as_bytes(), expected);
        let source = t.source().read(&v.source_range()).unwrap();
        assert_eq!(source.as_ptr(), v.as_bytes().as_ptr());
    }
    for offset in [5, 6, u64::MAX] {
        assert_eq!(
            t.lookup(offset, u64::MAX),
            Err(StringTableError::OffsetOutOfBounds {
                offset,
                table: t.source_range()
            })
        );
    }
}
#[test]
fn no_table_empty_table_and_nul_only_table_are_distinct() {
    let mut b = image(&[]);
    put32(&mut b, 120, 0);
    let r = inspect(&parsed(b)).unwrap();
    assert_eq!(r.dynamic(), &DynamicObservation::Absent);
    assert!(r.string_table().is_none());
    let r = inspect(&parsed(image(&[(0, 0)]))).unwrap();
    assert!(matches!(r.dynamic(), DynamicObservation::Present(_)));
    assert!(r.string_table().is_none());
    let empty = table(b"");
    assert!(matches!(
        empty.lookup(0, 8),
        Err(StringTableError::OffsetOutOfBounds { .. })
    ));
    let nul = table(b"\0");
    assert_eq!(nul.lookup(0, 1).unwrap().as_bytes(), b"");
}
#[test]
fn dependency_names_preserve_non_utf8_identity_without_lossy_conversion() {
    let r = inspect(&with_strings(b"\0lib\xff\0", &[1])).unwrap();
    assert_eq!(names(&r), vec![&b"lib\xff"[..]]);
    let Dependency::Named(name) = &r.artifact().description().dependencies[0] else {
        panic!()
    };
    assert!(name.as_utf8().is_err());
    let view = r.string_table().unwrap().lookup(1, 5).unwrap();
    assert!(view.as_utf8().is_err());
    assert_eq!(view.as_bytes(), name.as_bytes());
}
#[test]
fn needed_order_and_duplicates_survive_in_generic_metadata() {
    let r = inspect(&with_strings(b"\0a\0b\0a\0unused\xff\0", &[1, 3, 1, 5])).unwrap();
    assert_eq!(names(&r), vec![&b"a"[..], &b"b"[..], &b"a"[..], &b"a"[..]]);
    assert_eq!(
        r.references()
            .iter()
            .map(|v| (v.entry_index, v.string_offset))
            .collect::<Vec<_>>(),
        vec![(2, 1), (3, 3), (4, 1), (5, 5)]
    );
    assert_eq!(r.references()[0].source, r.references()[2].source);
    assert_ne!(r.references()[0].source, r.references()[3].source);
}
#[test]
fn missing_nul_never_scans_past_declared_strsz() {
    // Underlying source contains zero immediately after the declared table; it is not a terminator.
    let elf = with_strings(b"\0lib", &[1]);
    let e = inspect(&elf).unwrap_err();
    assert!(matches!(
        e,
        Error::Lookup {
            entry_index: 2,
            error: StringTableError::MissingTerminator { offset: 1, .. }
        }
    ));
    assert!(std::error::Error::source(&e).is_some());
    let valid = with_strings(b"\0lib\0", &[1]);
    assert_eq!(names(&inspect(&valid).unwrap()), vec![&b"lib"[..]]);
}
#[test]
fn empty_string_is_valid_lookup_but_invalid_dependency_declaration() {
    let elf = with_strings(b"\0", &[0]);
    assert_eq!(
        inspect(&elf),
        Err(Error::InvalidName {
            entry_index: 2,
            offset: 0,
            error: DependencyNameError::Empty
        })
    );
    assert_eq!(table(b"\0").lookup(0, 1).unwrap().as_bytes(), b"");
}
#[test]
fn generic_name_constructor_preserves_bytes_and_rejects_empty_or_interior_nul() {
    assert_eq!(DependencyName::new(vec![]), Err(DependencyNameError::Empty));
    assert_eq!(
        DependencyName::new(b"a\0b".to_vec()),
        Err(DependencyNameError::InteriorNul { index: 1 })
    );
    let name = DependencyName::new(b"../A\xff".to_vec()).unwrap();
    assert_eq!(name.as_bytes(), b"../A\xff");
    assert!(name.as_utf8().is_err()); // observation is not path normalization/search
}
#[test]
fn scan_budgets_include_terminators_and_repeated_offsets() {
    let elf = with_strings(b"\0lib\0", &[1, 1]);
    let run = |per, total| {
        observe(
            &elf,
            ObservationLimits { max_entries: 128 },
            StringLimits {
                max_scan_bytes_per_reference: per,
                max_total_scan_bytes: total,
            },
        )
    };
    assert!(matches!(
        run(3, 8),
        Err(Error::Lookup {
            entry_index: 2,
            error: StringTableError::ScanLimit { limit: 3, .. }
        })
    ));
    assert!(matches!(
        run(4, 7),
        Err(Error::Lookup {
            entry_index: 3,
            error: StringTableError::ScanLimit { limit: 3, .. }
        })
    ));
    assert_eq!(names(&run(4, 8).unwrap()), vec![&b"lib"[..], &b"lib"[..]]);
    assert!(matches!(
        table(b"\0").lookup(0, 0),
        Err(StringTableError::ScanLimit { limit: 0, .. })
    ));
}
#[test]
fn missing_descriptor_groups_fail_during_dynamic_observation() {
    for entries in [
        vec![(1, 0), (0, 0)],
        vec![(5, 0x1400), (0, 0)],
        vec![(10, 4), (0, 0)],
    ] {
        assert!(matches!(
            inspect(&parsed(image(&entries))),
            Err(Error::Dynamic(DynamicError::IncompleteDescriptor { .. }))
        ));
    }
}
#[test]
fn descriptor_translation_failures_are_not_string_errors() {
    for (address, size) in [(0x3000, 1), (0x1600, 1), (0x15ff, 2), (u64::MAX, 1)] {
        let r = inspect(&parsed(image(&[(5, address), (10, size), (0, 0)])));
        assert!(matches!(
            r,
            Err(Error::Dynamic(DynamicError::Translation {
                tag: Some(DynamicTag::StrTab),
                ..
            }))
        ));
    }
    let mut b = image(&[(5, 0x1400), (10, 4), (0, 0)]);
    b.truncate(0x500);
    assert!(matches!(
        inspect(&parsed(b)),
        Err(Error::Dynamic(DynamicError::Translation {
            error: TranslationError::Source { .. },
            ..
        }))
    ));
}
#[test]
fn needed_offsets_at_or_beyond_table_end_keep_m5_failure_boundary() {
    for offset in [4, 5, u64::MAX] {
        assert_eq!(
            inspect(&with_strings(b"\0ab\0", &[offset])),
            Err(Error::Dynamic(DynamicError::NeededOffsetOutOfBounds {
                index: 0,
                offset,
                string_size: 4
            }))
        );
    }
}
#[test]
fn conflicting_string_descriptors_are_not_silently_chosen() {
    for (tag, value, first) in [(5, 0x1500, 0), (10, 99, 1)] {
        assert_eq!(
            inspect(&parsed(image(&[
                (5, 0x1400),
                (10, 4),
                (tag, value),
                (0, 0)
            ]))),
            Err(Error::Dynamic(DynamicError::DuplicateTag {
                tag: DynamicTag::from_raw(tag),
                first,
                second: 2
            }))
        );
    }
}
#[test]
fn named_declarations_pass_through_generic_admission_and_planning_without_resolution() {
    let elf = with_strings(b"\0a\0b\0", &[1, 3, 1]);
    let report = inspect(&elf).unwrap();
    assert_eq!(
        admit(report.artifact()),
        Err(Rejection::UnsupportedRequirement(
            DeferredRequirement::UninspectedProgramSemantics
        ))
    );
    // A separate M2 synthetic fixture exercises the generic name contract; the ELF guard is not cleared.
    let mut raw = fixtures::linked();
    raw.description.dependencies = report.artifact().description().dependencies.clone();
    assert!(matches!(
        admit(&artifact::inspect(raw.clone())),
        Err(Rejection::MalformedMetadata {
            kind: DescriptorKind::Import,
            problem: MetadataProblem::UnknownDependency,
            ..
        })
    ));
    raw.description.imports.clear();
    raw.description.relocations.clear();
    let target = admit(&artifact::inspect(raw)).unwrap();
    let planned = plan(&target);
    assert_eq!(
        planned.metadata().dependencies,
        report.artifact().description().dependencies
    );
    assert_eq!(plan(&target), planned);
    assert_eq!(target.metadata().dependencies.len(), 3);
}
#[test]
fn conventional_optional_tags_remain_raw_without_assuming_names_or_search_paths() {
    let r = inspect(&parsed(image(&[(14, u64::MAX), (15, 0), (29, 0), (0, 0)]))).unwrap();
    assert!(r.string_table().is_none());
    assert!(names(&r).is_empty());
    let DynamicObservation::Present(d) = r.dynamic() else {
        panic!()
    };
    assert_eq!(d.entries()[0].tag, DynamicTag::Unknown(14));
    assert_eq!(d.entries()[1].tag, DynamicTag::Unknown(15));
    assert_eq!(d.entries()[2].tag, DynamicTag::Unknown(29));
    assert_eq!(
        r.artifact().description().module.as_ref().unwrap().name,
        "fixture"
    );
}
#[test]
fn source_views_and_owned_names_survive_original_report_lifetimes() {
    let elf = with_strings(b"\0lib\0", &[1]);
    let before = elf.clone();
    let r = inspect(&elf).unwrap();
    assert_eq!(r, inspect(&elf).unwrap());
    assert_eq!(elf, before);
    let source_id = elf.artifact().identity();
    let owned = r.artifact().description().dependencies.clone();
    drop(elf);
    drop(before);
    let t = r.string_table().unwrap();
    let view = t.lookup(1, 4).unwrap();
    assert_eq!(view.source_range().source_id(), source_id);
    assert_eq!(view.as_bytes(), b"lib");
    assert_eq!(
        r.artifact()
            .source()
            .read(&r.references()[0].source)
            .unwrap()
            .as_ptr(),
        view.as_bytes().as_ptr()
    );
    let mut detached = r.artifact().description().clone();
    detached.dependencies.clear();
    assert_eq!(names(&r), vec![&b"lib"[..]]);
    drop(r);
    let Dependency::Named(n) = &owned[0] else {
        panic!()
    };
    assert_eq!(n.as_bytes(), b"lib");
}
#[test]
fn every_offset_and_budget_matches_bounded_nul_oracle() {
    for bytes in [
        &b""[..],
        &b"\0"[..],
        &b"\0a\0"[..],
        &b"\0ab\0"[..],
        &b"\0x\xff\0"[..],
        &b"\0abc"[..],
    ] {
        let t = table(bytes);
        for offset in 0..=bytes.len() + 1 {
            for budget in 0..=bytes.len() + 1 {
                let r = t.lookup(offset as u64, budget as u64);
                if offset >= bytes.len() {
                    assert!(matches!(r, Err(StringTableError::OffsetOutOfBounds { .. })));
                    continue;
                }
                let end = bytes.len().min(offset + budget);
                let expected = bytes[offset..end].iter().position(|b| *b == 0);
                if let Some(len) = expected {
                    assert_eq!(r.unwrap().as_bytes(), &bytes[offset..offset + len]);
                } else if end < bytes.len() {
                    assert!(matches!(r, Err(StringTableError::ScanLimit { .. })));
                } else {
                    assert!(matches!(r, Err(StringTableError::MissingTerminator { .. })));
                }
            }
        }
    }
}
#[test]
fn dependency_count_sweep_preserves_each_observed_declaration() {
    for count in 0..=12 {
        let r = inspect(&with_strings(b"\0a\0", &vec![1; count])).unwrap();
        assert_eq!(names(&r), vec![&b"a"[..]; count]);
        assert_eq!(r.references().len(), count);
        for (i, reference) in r.references().iter().enumerate() {
            assert_eq!(reference.entry_index, i as u64 + 2);
            assert_eq!(reference.string_offset, 1);
        }
    }
}
#[test]
fn string_descriptor_end_boundary_sweep_preserves_source_backing() {
    for address in [0x15fe, 0x15ff, 0x1600, 0x1601] {
        for size in 0..=2 {
            let r = inspect(&parsed(image(&[(5, address), (10, size), (0, 0)])));
            if address + size <= 0x1600 {
                let r = r.unwrap();
                assert_eq!(r.string_table().unwrap().source_range().extent().size, size);
            } else {
                assert!(matches!(
                    r,
                    Err(Error::Dynamic(DynamicError::Translation { .. }))
                ));
            }
        }
    }
}
