use astero_loader::{
    artifact::SourceArtifact,
    elf::{
        self,
        address_translation::TranslationError,
        dynamic::{
            self, ObservationLimits,
            bounded::{
                self, DynamicFailure, DynamicLimits, DynamicObservationReport, DynamicOutcome,
            },
            error::DynamicError,
            synthetic::*,
            tags::DynamicTag,
        },
        inspect::bounded::{InspectionLimits, InspectionOutcome, inspect},
    },
};
fn source(b: Vec<u8>) -> SourceArtifact {
    SourceArtifact::new(b, Some("M17 synthetic".into())).unwrap()
}
fn run(b: Vec<u8>, entries: u64) -> DynamicObservationReport {
    bounded::observe(
        source(b),
        DynamicLimits {
            max_program_headers: 3,
            max_dynamic_entries: entries,
        },
    )
}
fn error(r: &DynamicObservationReport) -> &DynamicError {
    match r.outcome() {
        DynamicOutcome::Failed(DynamicFailure::Table(e)) => e,
        other => panic!("{other:?}"),
    }
}
#[test]
fn raw_evidence_preserves_source_and_does_not_validate_descriptors() {
    let bytes = image(&[
        (5, u64::MAX),
        (6, u64::MAX),
        (4, u64::MAX),
        (7, u64::MAX),
        (-42, u64::MAX),
        (0, 7),
    ]);
    let s = source(bytes.clone());
    let all = s.checked_range(0, s.len()).unwrap();
    let limits = DynamicLimits {
        max_program_headers: 2,
        max_dynamic_entries: 6,
    };
    let a = bounded::observe(s.clone(), limits);
    let b = bounded::observe(s.clone(), limits);
    let (DynamicOutcome::Complete(a_table), DynamicOutcome::Complete(b_table)) =
        (a.outcome(), b.outcome())
    else {
        panic!("raw pointers must remain uninterpreted")
    };
    assert_eq!(a_table, b_table);
    assert_eq!(a.limits(), limits);
    assert_eq!(a.source().identity(), s.identity());
    assert_eq!(a.source().provenance(), s.provenance());
    assert_eq!(a_table.source_range().source_id(), s.identity());
    assert_eq!(a_table.source_range().extent().offset.0, 0x200);
    assert_eq!(a_table.entries()[4].tag, DynamicTag::Unknown(-42));
    assert_eq!(a_table.entries()[4].value, u64::MAX);
    assert_eq!(a_table.entries()[5].value, 7);
    assert_eq!(s.read(&all).unwrap(), bytes);
    let old = elf::inspect(s, None).unwrap();
    assert!(dynamic::observe(&old, ObservationLimits { max_entries: 6 }).is_err());
}
#[test]
fn separate_header_inspection_succeeds_when_dynamic_is_malformed() {
    let s = source(image(&[(1, 0)]));
    let h = inspect(
        s.clone(),
        InspectionLimits {
            max_program_headers: 2,
        },
    );
    assert!(matches!(h.outcome(), InspectionOutcome::Complete(_)));
    assert!(matches!(
        error(&bounded::observe(
            s,
            DynamicLimits {
                max_program_headers: 2,
                max_dynamic_entries: 1
            }
        )),
        DynamicError::MissingTerminator { entries: 1 }
    ));
    let r = bounded::observe(
        source(image(&[(0, 0)])),
        DynamicLimits {
            max_program_headers: 1,
            max_dynamic_entries: 1,
        },
    );
    assert!(matches!(
        r.outcome(),
        DynamicOutcome::Failed(DynamicFailure::Headers(_))
    ));
}
#[test]
fn entry_budget_sweep_includes_terminal_and_never_labels_a_prefix_complete() {
    for count in 1..=12 {
        let mut entries = vec![(-42, 123); count];
        entries[count - 1] = (0, 9);
        for budget in 0..=13 {
            let r = run(image(&entries), budget);
            if budget < count as u64 {
                assert!(matches!(error(&r),DynamicError::EntryLimit{limit} if *limit==budget));
            } else {
                let DynamicOutcome::Complete(t) = r.outcome() else {
                    panic!("{r:?}")
                };
                assert_eq!(t.entries().len(), count);
                assert_eq!(t.entries().last().unwrap().tag, DynamicTag::Null);
            }
        }
    }
}
#[test]
fn absent_multiple_empty_truncated_missing_and_trailing_policies_are_explicit() {
    let mut b = image(&[(0, 0)]);
    put32(&mut b, 120, 0);
    assert!(matches!(run(b, 0).outcome(), DynamicOutcome::Unavailable));
    let mut b = image(&[(0, 0)]);
    put16(&mut b, 56, 3);
    program(&mut b, 2, 2, 0x200, 0x1200, 16, 16);
    assert!(matches!(
        error(&run(b, 1)),
        DynamicError::MultipleTables {
            first: 1,
            second: 2
        }
    ));
    assert!(matches!(
        error(&run(image(&[]), 0)),
        DynamicError::MissingTerminator { entries: 0 }
    ));
    for size in 1..32 {
        let mut b = image(&[(-42, 1), (-43, 2)]);
        program(&mut b, 1, 2, 0x200, 0x1200, size, size);
        let r = run(b, 2);
        if size == 16 {
            assert!(matches!(
                error(&r),
                DynamicError::MissingTerminator { entries: 1 }
            ));
        } else {
            assert!(
                matches!(error(&r),DynamicError::TruncatedEntry{index,remaining} if *index==size/16 && *remaining==size%16)
            );
        }
    }
    // Odd trailing bytes after a terminal entry are deliberately not decoded.
    let mut b = image(&[(0, 5)]);
    program(&mut b, 1, 2, 0x200, 0x1200, 17, 17);
    let r = run(b, 1);
    let DynamicOutcome::Complete(t) = r.outcome() else {
        panic!("{r:?}")
    };
    assert_eq!(t.entries().len(), 1);
    assert_eq!(t.source_range().extent().size, 17);
}
#[test]
fn table_bounds_translation_and_overflow_are_structured_before_scanning() {
    for (offset, address, file, memory, expected) in [
        (0x200, 0x1200, 32, 16, "size"),
        (0x200, u64::MAX, 16, 16, "overflow"),
        (0x600, 0x1600, 16, 16, "source"),
        (0x200, 0x1600, 16, 16, "bss"),
        (0x200, 0xdead, 16, 16, "unmapped"),
        (0x201, 0x1200, 16, 16, "mismatch"),
        (0x200, 0x15ff, 16, 16, "crossing"),
    ] {
        let mut b = image(&[(0, 0)]);
        program(&mut b, 1, 2, offset, address, file, memory);
        let r = run(b, 1);
        let e = error(&r);
        assert!(
            match expected {
                "size" => matches!(e, DynamicError::InvalidTableSize { .. }),
                "overflow" => matches!(e, DynamicError::TableAddressOverflow { .. }),
                "source" => matches!(e, DynamicError::Source { .. }),
                "bss" => matches!(
                    e,
                    DynamicError::Translation {
                        error: TranslationError::ZeroFill { .. },
                        ..
                    }
                ),
                "unmapped" => matches!(
                    e,
                    DynamicError::Translation {
                        error: TranslationError::Unmapped { .. },
                        ..
                    }
                ),
                "mismatch" => matches!(e, DynamicError::TableOffsetMismatch { .. }),
                "crossing" => matches!(
                    e,
                    DynamicError::Translation {
                        error: TranslationError::CrossesSourceBoundary { .. },
                        ..
                    }
                ),
                _ => false,
            },
            "{e:?}"
        );
    }
    // Exact source boundary succeeds, one-byte truncation fails even with DT_NULL at the start.
    for len in [0x5ff, 0x600] {
        let mut b = image(&[(0, 0)]);
        program(&mut b, 1, 2, 0x5f0, 0x15f0, 16, 16);
        b.truncate(len);
        assert_eq!(
            matches!(run(b, 1).outcome(), DynamicOutcome::Complete(_)),
            len == 0x600
        );
    }
}
#[test]
fn shared_raw_traversal_preserves_existing_descriptor_observer_results() {
    let s = source(image(&[(-42, 3), (-42, 4), (0, 0)]));
    let raw = bounded::observe(
        s.clone(),
        DynamicLimits {
            max_program_headers: 2,
            max_dynamic_entries: 3,
        },
    );
    let old = dynamic::observe(
        &elf::inspect(s, None).unwrap(),
        ObservationLimits { max_entries: 3 },
    )
    .unwrap();
    let (DynamicOutcome::Complete(a), dynamic::DynamicObservation::Present(b)) =
        (raw.outcome(), old)
    else {
        panic!("expected tables")
    };
    assert_eq!(a.entries(), b.entries());
    assert_eq!(a.source_range(), b.source_range());
}
