use astero_loader::{
    artifact::SourceArtifact,
    elf::{
        self,
        address_translation::TranslationError,
        dynamic::{
            self, ObservationLimits,
            bounded::{DynamicLimits, DynamicOutcome},
            descriptors::{
                self, DescriptorFailure, DescriptorLimits, DescriptorObservationReport,
                DescriptorOutcome, DescriptorUnavailable, DescriptorValue,
            },
            error::DynamicError,
            synthetic::*,
            tags::DynamicTag,
        },
    },
};
fn limits(max_descriptors: u64) -> DescriptorLimits {
    DescriptorLimits {
        dynamic: DynamicLimits {
            max_program_headers: 2,
            max_dynamic_entries: 32,
        },
        max_descriptors,
    }
}
fn source(b: Vec<u8>) -> SourceArtifact {
    SourceArtifact::new(b, Some("M18 synthetic".into())).unwrap()
}
fn run(entries: &[(i64, u64)], max: u64) -> DescriptorObservationReport {
    descriptors::observe(source(image(entries)), limits(max))
}
fn error(r: &DescriptorObservationReport) -> &DynamicError {
    match r.outcome() {
        DescriptorOutcome::Failed(DescriptorFailure::Interpretation(e)) => e,
        other => panic!("{other:?}"),
    }
}
#[test]
fn valid_pairs_are_source_bound_auditable_and_do_not_traverse_payloads() {
    let mut bytes = image(&[
        (6, 0x1420),
        (11, 24),
        (5, 0x1400),
        (10, 16),
        (4, u64::MAX),
        (7, u64::MAX),
        (-42, 99),
        (0, 0),
    ]);
    bytes[0x400..0x440].fill(0xff);
    let s = source(bytes.clone());
    let all = s.checked_range(0, s.len()).unwrap();
    let a = descriptors::observe(s.clone(), limits(2));
    let b = descriptors::observe(s.clone(), limits(2));
    let (DescriptorOutcome::Complete(ar), DescriptorOutcome::Complete(br)) =
        (a.outcome(), b.outcome())
    else {
        panic!("payloads not interpreted")
    };
    assert_eq!(ar, br);
    assert_eq!(ar.len(), 2);
    assert_eq!(a.limits(), limits(2));
    assert_eq!(a.source().identity(), s.identity());
    assert_eq!(a.source().provenance(), s.provenance());
    assert_eq!(s.read(&all).unwrap(), bytes);
    assert_eq!(a.raw(), b.raw());
    assert_eq!(a.raw().unwrap().entries()[6].tag, DynamicTag::Unknown(-42));
    assert_eq!(ar[0].fields()[0].index, 2);
    assert_eq!(ar[0].fields()[0].value, 0x1400);
    let DescriptorValue::Strings(d) = ar[0].value() else {
        panic!("fixed family order")
    };
    assert_eq!(d.source.source_id(), s.identity());
    assert_eq!(d.source.extent().offset.0, 0x400);
    assert_eq!(d.size, 16);
    let DescriptorValue::Symbols(d) = ar[1].value() else {
        panic!("symbols")
    };
    assert_eq!(d.first_entry.extent().size, 24);
    assert_eq!(d.first_entry.extent().offset.0, 0x420);
}
#[test]
fn budget_counts_families_including_incomplete_groups_not_unknown_entries() {
    for mask in 0..4 {
        let mut entries = vec![(-42, 0), (4, u64::MAX), (7, u64::MAX)];
        if mask & 1 != 0 {
            entries.extend([(5, 0x1400), (10, 16)]);
        }
        if mask & 2 != 0 {
            entries.extend([(6, 0x1420), (11, 24)]);
        }
        entries.push((0, 0));
        let count = u64::from(mask & 1 != 0) + u64::from(mask & 2 != 0);
        for maximum in 0..=3 {
            let r = run(&entries, maximum);
            if count > maximum {
                assert!(
                    matches!(r.outcome(),DescriptorOutcome::Failed(DescriptorFailure::Budget{attempted_families,maximum:m})if *attempted_families==count && *m==maximum)
                );
            } else if count == 0 {
                assert!(matches!(
                    r.outcome(),
                    DescriptorOutcome::Unavailable(DescriptorUnavailable::NoSupportedDescriptors)
                ));
            } else {
                assert!(
                    matches!(r.outcome(),DescriptorOutcome::Complete(v)if v.len() as u64==count)
                );
            }
        }
    }
    let r = run(&[(5, 1), (5, 2), (0, 0)], 0);
    assert!(matches!(
        r.outcome(),
        DescriptorOutcome::Failed(DescriptorFailure::Budget {
            attempted_families: 1,
            maximum: 0
        })
    ));
    assert_eq!(r.raw().unwrap().entries()[1].value, 2);
}
#[test]
fn duplicate_companion_and_entry_size_errors_preserve_raw_evidence() {
    for tag in [5, 10, 6, 11] {
        for second in [1, 2] {
            let r = run(&[(tag, 1), (tag, second), (0, 0)], 2);
            assert!(matches!(
                error(&r),
                DynamicError::DuplicateTag {
                    first: 0,
                    second: 1,
                    ..
                }
            ));
            assert_eq!(r.raw().unwrap().entries()[1].value, second);
        }
    }
    for (tag, missing) in [
        (5, DynamicTag::StrSz),
        (10, DynamicTag::StrTab),
        (6, DynamicTag::SymEnt),
        (11, DynamicTag::SymTab),
    ] {
        let r = run(&[(tag, 0x1400), (0, 0)], 2);
        assert!(matches!(error(&r),DynamicError::IncompleteDescriptor{missing:m,..}if *m==missing));
    }
    for width in [0, 1, 16, 23, 25, u64::MAX] {
        let r = run(&[(6, 0x1400), (11, width), (0, 0)], 1);
        assert!(
            matches!(error(&r),DynamicError::UnsupportedEntrySize{observed,expected:24,..}if *observed==width)
        );
    }
}
#[test]
fn checked_extents_zero_size_bss_and_alias_policy_are_explicit() {
    for address in [0x1400, 0x1600] {
        let r = run(&[(5, address), (10, 0), (0, 0)], 1);
        let DescriptorOutcome::Complete(v) = r.outcome() else {
            panic!("zero extent")
        };
        let DescriptorValue::Strings(d) = v[0].value() else {
            panic!()
        };
        assert_eq!(d.source.extent().size, 0);
    }
    for (address, size, reason) in [
        (0x1600, 1, "bss"),
        (0xdead, 16, "unmapped"),
        (u64::MAX, 16, "overflow"),
        (0x15ff, 2, "crossing"),
    ] {
        let r = run(&[(5, address), (10, size), (0, 0)], 1);
        let DynamicError::Translation { error: e, .. } = error(&r) else {
            panic!("translation")
        };
        assert!(match reason {
            "bss" => matches!(e, TranslationError::ZeroFill { .. }),
            "unmapped" => matches!(e, TranslationError::Unmapped { .. }),
            "overflow" => matches!(e, TranslationError::RequestOverflow { .. }),
            "crossing" => matches!(e, TranslationError::CrossesSourceBoundary { .. }),
            _ => false,
        });
        assert_eq!(r.raw().unwrap().entries()[0].value, address);
    }
    for offset in 0..=8 {
        for size in 0..=8 {
            let r = run(&[(5, 0x15f8 + offset), (10, size), (0, 0)], 1);
            assert_eq!(
                matches!(r.outcome(), DescriptorOutcome::Complete(_)),
                offset + size <= 8
            );
        }
    }
    for address in [0x15e8, 0x15e9] {
        assert_eq!(
            matches!(
                run(&[(6, address), (11, 24), (0, 0)], 1).outcome(),
                DescriptorOutcome::Complete(_)
            ),
            address == 0x15e8
        );
    }
    // M5 does not interpret cross-family aliasing; retain both independently proven records.
    assert!(
        matches!(run(&[(5,0x1400),(10,32),(6,0x1400),(11,24),(0,0)],2).outcome(),DescriptorOutcome::Complete(v)if v.len()==2)
    );
}
#[test]
fn raw_request_remains_independent_and_discovery_failures_stay_distinct() {
    let s = source(image(&[(5, u64::MAX), (0, 0)]));
    assert!(matches!(
        dynamic::bounded::observe(s.clone(), limits(1).dynamic).outcome(),
        DynamicOutcome::Complete(_)
    ));
    assert!(matches!(
        descriptors::observe(s, limits(1)).outcome(),
        DescriptorOutcome::Failed(DescriptorFailure::Interpretation(_))
    ));
    let mut b = image(&[(0, 0)]);
    put32(&mut b, 120, 0);
    assert!(matches!(
        descriptors::observe(source(b), limits(0)).outcome(),
        DescriptorOutcome::Unavailable(DescriptorUnavailable::NoDynamicTable)
    ));
    let mut l = limits(2);
    l.dynamic.max_dynamic_entries = 0;
    assert!(matches!(
        descriptors::observe(source(image(&[(0, 0)])), l).outcome(),
        DescriptorOutcome::Failed(DescriptorFailure::Discovery(_))
    ));
    assert!(matches!(
        descriptors::observe(source(vec![1, 2, 3]), limits(2)).outcome(),
        DescriptorOutcome::Failed(DescriptorFailure::Discovery(_))
    ));
}
#[test]
fn selected_pair_semantics_match_existing_authoritative_descriptor_collector() {
    let s = source(image(&[
        (5, 0x1400),
        (10, 16),
        (6, 0x1420),
        (11, 24),
        (0, 0),
    ]));
    let old = dynamic::observe(
        &elf::inspect(s.clone(), None).unwrap(),
        ObservationLimits { max_entries: 5 },
    )
    .unwrap();
    let dynamic::DynamicObservation::Present(old) = old else {
        panic!()
    };
    let new = descriptors::observe(s, limits(2));
    let DescriptorOutcome::Complete(records) = new.outcome() else {
        panic!()
    };
    assert!(
        matches!(records[0].value(),DescriptorValue::Strings(v)if Some(v)==old.descriptors().strings.as_ref())
    );
    assert!(
        matches!(records[1].value(),DescriptorValue::Symbols(v)if Some(v)==old.descriptors().symbols.as_ref())
    );
}
