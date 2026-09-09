use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::DynamicLimits,
        candidates::workload::{self, LinkageLimits, LinkageOutcome},
        hash::{HashLimits, bounded::HashMetadataLimits},
        identity::{self, codec::*, *},
        symbol_table::bounded::SymbolObservationLimits,
        synthetic::{put32, put64},
    },
};
use std::sync::Arc;
fn limits() -> LinkageLimits {
    LinkageLimits {
        hash: HashMetadataLimits {
            dynamic: DynamicLimits {
                max_program_headers: 2,
                max_dynamic_entries: 32,
            },
            hash: HashLimits { max_words: 64 },
        },
        symbols: SymbolObservationLimits {
            max_descriptors: 2,
            max_symbols: 3,
            max_name_lookups: 16,
            max_name_scan_bytes: 64,
            max_total_name_scan_bytes: 256,
        },
        max_relocations: 4,
    }
}
fn report(b: Vec<u8>, maximum: u64) -> Ps5IdentityEvidenceReport {
    run(b, limits(), maximum)
}
fn run(b: Vec<u8>, l: LinkageLimits, maximum: u64) -> Ps5IdentityEvidenceReport {
    identity::observe(
        Arc::new(workload::observe(
            SourceArtifact::new(b, Some("M24 synthetic".into())).unwrap(),
            l,
        )),
        IdentityLimits {
            max_identity_records: maximum,
        },
    )
}
fn evidence(r: &Ps5IdentityEvidenceReport) -> &IdentityEvidence {
    let IdentityOutcome::Complete(e) = r.outcome() else {
        panic!("{:?}", r.outcome())
    };
    e
}
#[test]
fn known_decrypted_and_approved_firmware_codec_vectors() {
    for (s, n) in [
        ("H2e8t5ScQGc", 0x1f67bcb7949c4067),
        ("P330P3dFF68", 0x3f7df43f774517af),
        ("-ZR+hG7aDHw", 0xfd947e846eda0c7c),
        ("RpQJJVKTiFM", 0x4694092552938853),
    ] {
        assert_eq!(decode_nid(s.as_bytes()), Ok(n));
        assert_eq!(&encode_nid(n), s.as_bytes());
    }
}
#[test]
fn canonical_roundtrips_and_aliases_are_not_confirmed() {
    for n in [0, 1, 63, 64, 65535, u64::MAX, 1 << 63] {
        let mut s = encode_nid(n);
        assert_eq!(decode_nid(&s), Ok(n));
        s[10] = b'B';
        assert_eq!(decode_nid(&s), Err(EncodingError::NonCanonicalPadding));
    }
    assert!(matches!(
        decode_nid(b"///////////"),
        Err(EncodingError::Alphabet { .. })
    ));
    assert_eq!(decode_nid(b"short"), Err(EncodingError::Length));
    assert_eq!(decode_context(b"BA"), Ok(64));
    assert_eq!(decode_context(b"A"), Ok(0));
    assert!(decode_context(b"AA").is_err());
    assert!(decode_context(b"---").is_err());
}
#[test]
fn records_preserve_provenance_raw_values_and_hypothesis_context() {
    let r = report(synthetic::identity_image(), 6);
    let e = evidence(&r);
    assert_eq!(e.descriptors().len(), 3);
    assert_eq!(e.symbols().len(), 3);
    let d = &e.descriptors()[0];
    assert_eq!(
        (d.id, d.version_bits, d.evidence),
        (Some(0), Some(257), MetadataEvidence::ExperimentalPacking)
    );
    assert_eq!(
        r.linkage().source().read(&d.name.unwrap()).unwrap(),
        b"sample"
    );
    assert_eq!(e.descriptors()[2].raw, u64::MAX);
    assert_eq!(e.descriptors()[2].evidence, MetadataEvidence::Uninterpreted);
    assert!(matches!(
        e.symbols()[1].evidence,
        NameEvidence::Encoded {
            nid: 0x1f67bcb7949c4067,
            library: ContextEvidence::Hypothesis { id: 1, .. },
            module: ContextEvidence::Hypothesis { id: 0, .. }
        }
    ));
}
#[test]
fn budget_refuses_before_successful_prefix() {
    for n in [0, 1, 5] {
        assert!(
            matches!(report(synthetic::identity_image(),n).outcome(),IdentityOutcome::Failed(IdentityFailure::Budget{count:6,maximum}) if *maximum==n)
        );
    }
    assert_eq!(
        evidence(&report(synthetic::identity_image(), 6))
            .symbols()
            .len(),
        3
    );
}
#[test]
fn aggregate_name_budgets_include_prior_work_and_metadata() {
    let mut l = limits();
    l.symbols.max_name_lookups = 4;
    assert!(matches!(
        run(synthetic::identity_image(), l, 6).outcome(),
        IdentityOutcome::Failed(IdentityFailure::NameLookupBudget { attempted: 5, .. })
    ));
    l.symbols.max_name_lookups = 6;
    l.symbols.max_total_name_scan_bytes = 60;
    assert!(matches!(
        run(synthetic::identity_image(), l, 6).outcome(),
        IdentityOutcome::Failed(IdentityFailure::String { .. })
    ));
    l.symbols.max_total_name_scan_bytes = 61;
    assert_eq!(
        evidence(&run(synthetic::identity_image(), l, 6))
            .descriptors()
            .len(),
        3
    );
}
#[test]
fn malformed_metadata_offset_is_structured_and_m23_stays_independent() {
    let mut b = synthetic::identity_image();
    put64(&mut b, 0x6e8, u32::MAX as u64);
    let input = Arc::new(workload::observe(
        SourceArtifact::new(b, None).unwrap(),
        limits(),
    ));
    assert!(matches!(input.outcome(), LinkageOutcome::Complete(_)));
    assert!(matches!(
        identity::observe(
            input,
            IdentityLimits {
                max_identity_records: 6
            }
        )
        .outcome(),
        IdentityOutcome::Failed(IdentityFailure::String {
            dynamic_index: 14,
            ..
        })
    ));
}
#[test]
fn duplicate_and_conflicting_ids_remain_explicit() {
    for conflict in [false, true] {
        let mut b = synthetic::identity_image();
        put64(&mut b, 0x700, 0x61000049);
        put64(
            &mut b,
            0x708,
            (1 << 48) | (1 << 32) | if conflict { 33 } else { 40 },
        );
        let r = report(b, 6);
        let NameEvidence::Encoded { library, .. } = evidence(&r).symbols()[1].evidence else {
            panic!()
        };
        assert_eq!(
            library,
            if conflict {
                ContextEvidence::Conflict {
                    id: 1,
                    declarations: 2,
                }
            } else {
                ContextEvidence::Hypothesis {
                    id: 1,
                    dynamic_index: 15,
                    declarations: 2,
                }
            }
        );
    }
}
#[test]
fn missing_context_does_not_resolve_or_discard_numeric_nid() {
    let mut b = synthetic::identity_image();
    b[0x90d] = b'C';
    let r = report(b, 6);
    assert!(matches!(
        evidence(&r).symbols()[1].evidence,
        NameEvidence::Encoded {
            library: ContextEvidence::Missing { id: 2 },
            ..
        }
    ));
}
#[test]
fn plain_bare_encoded_empty_and_raw_names_are_not_function_identities() {
    for (name, expected) in [
        (b"plain\0".as_slice(), NameEvidence::PlainOrRaw),
        (
            b"H2e8t5ScQGc\0",
            NameEvidence::Candidate(EncodingError::Context),
        ),
        (b"\xff\0", NameEvidence::PlainOrRaw),
        (b"\0", NameEvidence::PlainOrRaw),
    ] {
        let mut b = synthetic::identity_image();
        b[0x901..0x901 + name.len()].copy_from_slice(name);
        let r = report(b, 6);
        let s = &evidence(&r).symbols()[1];
        assert_eq!(s.evidence, expected);
        assert_eq!(
            r.linkage().source().read(&s.name.unwrap()).unwrap(),
            &name[..name.len() - 1]
        );
    }
}
#[test]
fn malformed_suffix_and_noncanonical_nid_remain_candidates() {
    for (at, value) in [(0x90f, b'#'), (0x90b, b'B')] {
        let mut b = synthetic::identity_image();
        b[at] = value;
        let r = report(b, 6);
        assert!(matches!(
            evidence(&r).symbols()[1].evidence,
            NameEvidence::Candidate(_)
        ));
    }
}
#[test]
fn reordered_library_ids_follow_artifact_local_suffixes() {
    let a = report(synthetic::identity_image(), 6);
    let mut b = synthetic::identity_image();
    put64(&mut b, 0x6f8, (2 << 48) | (1 << 32) | 40);
    b[0x90d] = b'C';
    b[0x91d] = b'C';
    let b = report(b, 6);
    let (
        NameEvidence::Encoded { nid: a, .. },
        NameEvidence::Encoded {
            nid: b,
            library: ContextEvidence::Hypothesis { id: 2, .. },
            ..
        },
    ) = (
        evidence(&a).symbols()[1].evidence,
        evidence(&b).symbols()[1].evidence,
    )
    else {
        panic!()
    };
    assert_eq!(a, b);
}
#[test]
fn shared_input_is_immutable_and_repeated_reports_keep_order() {
    let bytes = synthetic::identity_image();
    let source = SourceArtifact::new(bytes.clone(), Some("synthetic immutable".into())).unwrap();
    let input = Arc::new(workload::observe(source.clone(), limits()));
    let a = identity::observe(
        input.clone(),
        IdentityLimits {
            max_identity_records: 6,
        },
    );
    let b = identity::observe(input.clone(), a.limits());
    assert!(Arc::ptr_eq(a.linkage(), &input));
    assert_eq!(evidence(&a).symbols(), evidence(&b).symbols());
    assert_eq!(evidence(&a).descriptors(), evidence(&b).descriptors());
    assert_eq!(
        source
            .read(&source.checked_range(0, bytes.len() as u64).unwrap())
            .unwrap(),
        bytes
    );
    let LinkageOutcome::Complete(e) = input.outcome() else {
        panic!()
    };
    assert_eq!(e.counts().relocations, 4);
    assert_eq!(e.relocations()[2].record.addend, i64::MIN);
}
#[test]
fn duplicate_symbol_names_keep_separate_indices() {
    let mut b = synthetic::identity_image();
    put32(&mut b, 0x430, 1);
    let r = report(b, 6);
    let s = evidence(&r).symbols();
    assert_eq!(s[1].evidence, s[2].evidence);
    assert_ne!(s[1].symbol_index, s[2].symbol_index);
}
#[test]
fn prerequisite_failure_is_not_successful_identity() {
    let r = report(vec![0; 100], 6);
    assert!(matches!(
        r.outcome(),
        IdentityOutcome::Failed(IdentityFailure::LinkagePrerequisite)
    ));
}
