use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        self,
        bounded::DynamicLimits,
        descriptors::{DescriptorLimits, DescriptorOutcome, DescriptorValue},
        string_references::{
            self, StringEncoding, StringLimits, StringReferenceFailure, StringReferenceLimits,
            StringReferenceOutcome,
        },
        string_table::{DynamicStringTable, StringTableError},
        synthetic::*,
    },
};
fn limits(references: u64, per: u64, total: u64) -> StringReferenceLimits {
    StringReferenceLimits {
        descriptors: DescriptorLimits {
            dynamic: DynamicLimits {
                max_program_headers: 2,
                max_dynamic_entries: 32,
            },
            max_descriptors: 2,
        },
        max_string_references: references,
        strings: StringLimits {
            max_scan_bytes_per_reference: per,
            max_total_scan_bytes: total,
        },
    }
}
fn fixture(payload: &[u8], offsets: &[u64]) -> Vec<u8> {
    let mut entries = vec![
        (5, 0x1400),
        (10, payload.len() as u64),
        (14, u64::MAX),
        (4, u64::MAX),
        (7, u64::MAX),
    ];
    entries.extend(offsets.iter().map(|&o| (1, o)));
    entries.push((0, 0));
    let mut b = image(&entries);
    b[0x400..0x400 + payload.len()].copy_from_slice(payload);
    b
}
fn source(b: Vec<u8>) -> SourceArtifact {
    SourceArtifact::new(b, Some("M19 synthetic".into())).unwrap()
}
#[test]
fn bytes_identity_order_duplicates_and_unreferenced_data_are_preserved() {
    let bytes = fixture(b"\0a\0\xff\0unreferenced-unterminated", &[1, 3, 0, 1]);
    let s = source(bytes.clone());
    let all = s.checked_range(0, s.len()).unwrap();
    let l = limits(4, 2, 7);
    let a = string_references::observe(s.clone(), l);
    let b = string_references::observe(s.clone(), l);
    let (StringReferenceOutcome::Complete(ar), StringReferenceOutcome::Complete(br)) =
        (a.outcome(), b.outcome())
    else {
        panic!("only referenced strings")
    };
    assert_eq!(ar, br);
    assert_eq!(a.limits(), l);
    assert_eq!(ar.len(), 4);
    assert_eq!(
        ar.iter().map(|r| r.entry().value).collect::<Vec<_>>(),
        vec![1, 3, 0, 1]
    );
    assert_eq!(
        ar.iter().map(|r| r.entry().index).collect::<Vec<_>>(),
        vec![5, 6, 7, 8]
    );
    assert_eq!(ar[0].as_bytes(), b"a");
    assert_eq!(ar[1].as_bytes(), [255]);
    assert_eq!(ar[1].encoding(), StringEncoding::RawBytes);
    assert_eq!(ar[2].encoding(), StringEncoding::Empty);
    assert!(ar[2].as_bytes().is_empty());
    assert_eq!(ar[0].encoding(), StringEncoding::Utf8);
    assert_eq!(ar[0].source_range(), ar[3].source_range());
    assert_ne!(ar[0].entry().index, ar[3].entry().index);
    assert_eq!(ar.iter().map(|r| r.scanned_bytes()).sum::<u64>(), 7);
    assert_eq!(a.source().identity(), s.identity());
    assert_eq!(a.source().provenance(), s.provenance());
    assert_eq!(ar[0].source_range().source_id(), s.identity());
    assert_eq!(s.read(&all).unwrap(), bytes);
}
#[test]
fn prerequisite_absence_conflicts_and_unknown_references_are_distinct() {
    let r = string_references::observe(source(fixture(b"no-nul", &[])), limits(0, 0, 0));
    assert!(matches!(r.outcome(), StringReferenceOutcome::Unavailable));
    let r = string_references::observe(source(image(&[(1, 0), (0, 0)])), limits(1, 1, 1));
    assert!(matches!(
        r.outcome(),
        StringReferenceOutcome::Failed(StringReferenceFailure::TableUnavailable { .. })
    ));
    for entries in [
        vec![(5, 0x1400), (1, 0), (0, 0)],
        vec![(5, 0x1400), (5, 0x1401), (10, 4), (1, 0), (0, 0)],
    ] {
        let r = string_references::observe(source(image(&entries)), limits(1, 4, 4));
        let StringReferenceOutcome::Failed(StringReferenceFailure::Prerequisite(p)) = r.outcome()
        else {
            panic!("prerequisite")
        };
        assert!(matches!(p.outcome(), DescriptorOutcome::Failed(_)));
        assert!(p.raw().is_some());
    }
    let mut b = image(&[(0, 0)]);
    put32(&mut b, 120, 0);
    assert!(matches!(
        string_references::observe(source(b), limits(0, 0, 0)).outcome(),
        StringReferenceOutcome::Unavailable
    ));
}
#[test]
fn reference_and_scan_budgets_charge_duplicates_empty_and_nul() {
    for maximum in 0..=3 {
        let r = string_references::observe(source(fixture(b"a\0", &[0, 0])), limits(maximum, 2, 4));
        assert_eq!(
            matches!(r.outcome(), StringReferenceOutcome::Complete(_)),
            maximum >= 2
        );
        if maximum < 2 {
            assert!(matches!(
                r.outcome(),
                StringReferenceOutcome::Failed(StringReferenceFailure::ReferenceBudget {
                    observed_references: 2,
                    ..
                })
            ));
        }
    }
    for per in 0..=3 {
        for total in 0..=5 {
            let r =
                string_references::observe(source(fixture(b"a\0", &[0, 0])), limits(2, per, total));
            assert_eq!(
                matches!(r.outcome(), StringReferenceOutcome::Complete(_)),
                per >= 2 && total >= 4
            );
        }
    }
    let r = string_references::observe(source(fixture(b"a\0", &[0, 0])), limits(2, 2, 3));
    assert!(matches!(
        r.outcome(),
        StringReferenceOutcome::Failed(StringReferenceFailure::Lookup {
            completed_references: 1,
            effective_scan_limit: 1,
            remaining_total_scan_bytes: 1,
            error: StringTableError::ScanLimit { .. },
            ..
        })
    ));
    for per in 0..=1 {
        let r = string_references::observe(source(fixture(b"\0", &[0])), limits(1, per, 1));
        assert_eq!(
            matches!(r.outcome(), StringReferenceOutcome::Complete(_)),
            per == 1
        );
    }
}
#[test]
fn lookup_sweep_matches_m6_bounds_without_truncated_success() {
    for payload in [
        &b"\0"[..],
        &b"abc\0"[..],
        &b"a\0\xff\0tail"[..],
        &b"abc"[..],
    ] {
        for offset in 0..=payload.len() + 1 {
            for budget in 0..=payload.len() + 1 {
                let r = string_references::observe(
                    source(fixture(payload, &[offset as u64])),
                    limits(1, budget as u64, budget as u64),
                );
                if offset >= payload.len() {
                    assert!(matches!(
                        r.outcome(),
                        StringReferenceOutcome::Failed(StringReferenceFailure::Lookup {
                            error: StringTableError::OffsetOutOfBounds { .. },
                            ..
                        })
                    ));
                    continue;
                }
                let available = &payload[offset..];
                let scanned = &available[..available.len().min(budget)];
                if let Some(end) = scanned.iter().position(|&b| b == 0) {
                    let StringReferenceOutcome::Complete(records) = r.outcome() else {
                        panic!("{r:?}")
                    };
                    assert_eq!(records[0].as_bytes(), &scanned[..end]);
                } else if budget < available.len() {
                    assert!(matches!(
                        r.outcome(),
                        StringReferenceOutcome::Failed(StringReferenceFailure::Lookup {
                            error: StringTableError::ScanLimit { .. },
                            ..
                        })
                    ));
                } else {
                    assert!(matches!(
                        r.outcome(),
                        StringReferenceOutcome::Failed(StringReferenceFailure::Lookup {
                            error: StringTableError::MissingTerminator { .. },
                            ..
                        })
                    ));
                }
            }
        }
    }
}
#[test]
fn m18_descriptor_remains_payload_free_and_m6_bridge_checks_identity() {
    let s = source(fixture(b"abc", &[0]));
    let d = dynamic::descriptors::observe(s.clone(), limits(1, 3, 3).descriptors);
    let DescriptorOutcome::Complete(records) = d.outcome() else {
        panic!()
    };
    let record = records
        .iter()
        .find(|r| matches!(r.value(), DescriptorValue::Strings(_)))
        .unwrap();
    let table = DynamicStringTable::from_descriptor(&s, record)
        .unwrap()
        .unwrap();
    assert!(matches!(
        table.lookup(0, 3),
        Err(StringTableError::MissingTerminator { .. })
    ));
    let other = source(fixture(b"abc", &[0]));
    assert!(matches!(
        DynamicStringTable::from_descriptor(&other, record),
        Err(StringTableError::Source(_))
    ));
    let r = string_references::observe(s, limits(1, 3, 3));
    assert!(matches!(
        r.outcome(),
        StringReferenceOutcome::Failed(StringReferenceFailure::Lookup {
            error: StringTableError::MissingTerminator { .. },
            ..
        })
    ));
}
