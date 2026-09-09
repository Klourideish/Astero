use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::DynamicLimits,
        candidates::{
            structural::CandidateRole,
            workload::{
                self, LinkageEvidence, LinkageFailure as F, LinkageLimits, LinkageOutcome as O,
                LinkageReport, ReferenceRole, synthetic::linkage_image,
            },
        },
        hash::{HashLimits, bounded::HashMetadataLimits},
        relocations::error::RelocationError,
        symbol_table::bounded::SymbolObservationLimits,
        synthetic::{put16, put32, put64},
    },
};
fn limits() -> LinkageLimits {
    LinkageLimits {
        hash: HashMetadataLimits {
            dynamic: DynamicLimits {
                max_program_headers: 2,
                max_dynamic_entries: 16,
            },
            hash: HashLimits { max_words: 64 },
        },
        symbols: SymbolObservationLimits {
            max_descriptors: 2,
            max_symbols: 3,
            max_name_lookups: 4,
            max_name_scan_bytes: 6,
            max_total_name_scan_bytes: 24,
        },
        max_relocations: 4,
    }
}
fn observe(b: Vec<u8>, l: LinkageLimits) -> LinkageReport {
    workload::observe(
        SourceArtifact::new(b, Some("M23 synthetic".into())).unwrap(),
        l,
    )
}
fn complete(r: &LinkageReport) -> &LinkageEvidence {
    let O::Complete(e) = r.outcome() else {
        panic!("{:?}", r.outcome());
    };
    e
}
#[test]
fn complete_capability_preserves_raw_relocations_candidates_and_names() {
    let r = observe(linkage_image(), limits());
    let e = complete(&r);
    let c = e.counts();
    assert_eq!(
        (
            c.symbols,
            c.relocations,
            c.symbol_associated,
            c.null_references,
            c.external_candidates,
            c.definitions
        ),
        (3, 4, 3, 1, 1, 1)
    );
    assert_eq!(
        e.relocations()
            .iter()
            .map(|r| r.record.symbol_index())
            .collect::<Vec<_>>(),
        [0, 1, 2, 1]
    );
    assert_eq!(e.relocations()[1].record.relocation_type(), 0xffffeeee);
    assert_eq!(e.relocations()[2].record.addend, i64::MIN);
    assert_eq!(
        e.symbol_uses()[1].role,
        ReferenceRole::ExternalReferenceCandidate
    );
    assert_eq!(
        (e.symbol_uses()[1].ordinary, e.symbol_uses()[1].plt),
        (1, 1)
    );
    assert_eq!(
        r.symbols().entries().nth(2).unwrap().0.role(),
        CandidateRole::DefinitionCandidate
    );
    assert_eq!(e.needed().len(), 2);
    for n in e.needed() {
        assert_eq!(r.source().read(&n.source).unwrap(), b"alpha");
    }
}
#[test]
fn source_and_reports_are_immutable_and_repeated_runs_deterministic() {
    let b = linkage_image();
    let s = SourceArtifact::new(b.clone(), Some("provenance".into())).unwrap();
    let a = workload::observe(s.clone(), limits());
    let other = workload::observe(s.clone(), limits());
    assert_eq!(
        format!("{:?}", a.outcome()),
        format!("{:?}", other.outcome())
    );
    assert_eq!(a.source().identity(), s.identity());
    assert_eq!(a.symbols().input().source().identity(), s.identity());
    assert_eq!(a.source().provenance(), Some("provenance"));
    assert_eq!(s.read(&s.checked_range(0, s.len()).unwrap()).unwrap(), b);
}
#[test]
fn undefined_without_relocations_is_not_promoted() {
    let mut b = linkage_image();
    put64(&mut b, 0x268, 0);
    put64(&mut b, 0x298, 0);
    let r = observe(b, limits());
    let e = complete(&r);
    assert_eq!(e.counts().relocations, 0);
    assert_eq!(e.counts().external_candidates, 0);
    assert_eq!(e.symbol_uses()[1].role, ReferenceRole::StructuralOnly);
}
#[test]
fn ambiguous_attributes_and_names_keep_references_without_resolution() {
    for (info, other, name) in [
        (0x02, 0, 1),
        (0x12, 2, 1),
        (0xef, 0xfd, 1),
        (0x12, 0, 0),
        (0x12, 0, 7),
    ] {
        let mut b = linkage_image();
        b[0x41c] = info;
        b[0x41d] = other;
        put32(&mut b, 0x418, name);
        let r = observe(b, limits());
        assert_eq!(
            complete(&r).symbol_uses()[1].role,
            ReferenceRole::AmbiguousReference
        );
    }
    let mut b = linkage_image();
    put32(&mut b, 0x418, 8);
    let r = observe(b, limits());
    assert_eq!(
        complete(&r).symbol_uses()[1].role,
        ReferenceRole::ExternalReferenceCandidate
    );
}
#[test]
fn invalid_symbol_reference_preserves_raw_failure_without_prefix() {
    let mut b = linkage_image();
    put64(&mut b, 0x560, (3u64 << 32) | 7);
    let r = observe(b, limits());
    assert!(
        matches!(r.outcome(),O::Failed(F::SymbolIndex{raw,count:3}) if raw.record.symbol_index()==3)
    );
}
#[test]
fn exact_relocation_budget_and_refusals() {
    for max in [0, 3, 4] {
        let mut l = limits();
        l.max_relocations = max;
        let r = observe(linkage_image(), l);
        if max < 4 {
            assert!(matches!(
                r.outcome(),
                O::Failed(F::Relocation(RelocationError::EntryBudget { count: 4, .. }))
            ));
        } else {
            assert_eq!(complete(&r).counts().relocations, 4);
        }
    }
}
#[test]
fn symbol_and_needed_names_share_lookup_and_scan_limits() {
    for (lookups, bytes, ok) in [(4, 24, true), (3, 24, false), (4, 23, false)] {
        let mut l = limits();
        l.symbols.max_name_lookups = lookups;
        l.symbols.max_total_name_scan_bytes = bytes;
        assert_eq!(
            matches!(observe(linkage_image(), l).outcome(), O::Complete(_)),
            ok
        );
    }
}
#[test]
fn aliases_are_canonical_and_conflicts_are_structured() {
    let mut b = linkage_image();
    put64(&mut b, 0x288, 0x1570);
    let r = observe(b, limits());
    let e = complete(&r);
    assert_eq!(e.counts().relocations, 3);
    assert_eq!(e.relocations()[2].dynamic_index, Some(2));
    assert_eq!(e.relocations()[2].plt_index, Some(0));
    let mut b = linkage_image();
    put64(&mut b, 0x288, 0x1548);
    assert!(matches!(
        observe(b, limits()).outcome(),
        O::Failed(F::Relocation(RelocationError::DescriptorConflict { .. }))
    ));
}
#[test]
fn malformed_descriptors_and_deferred_formats_do_not_become_complete() {
    for (at, value) in [(0x268, 71), (0x278, 16), (0x258, u64::MAX), (0x2a8, 17)] {
        let mut b = linkage_image();
        put64(&mut b, at, value);
        assert!(matches!(observe(b, limits()).outcome(), O::Failed(_)));
    }
    let mut b = linkage_image();
    put64(&mut b, 0x2d0, 17);
    assert!(matches!(
        observe(b, limits()).outcome(),
        O::Failed(F::UnsupportedRelocationDescriptor { tag: 17, .. })
    ));
}
#[test]
fn missing_extent_failed_symbols_and_special_definitions_remain_explicit() {
    let mut b = linkage_image();
    put64(&mut b, 0x240, 0x123456);
    assert!(matches!(observe(b, limits()).outcome(), O::Unavailable));
    let mut b = linkage_image();
    b[0x400] = 1;
    assert!(matches!(
        observe(b, limits()).outcome(),
        O::Failed(F::Symbols)
    ));
    let mut b = linkage_image();
    put16(&mut b, 0x436, 0xfff1);
    let r = observe(b, limits());
    assert_eq!(complete(&r).counts().definitions, 0);
    assert_eq!(
        r.symbols().entries().nth(2).unwrap().0.role(),
        CandidateRole::SpecialCandidate
    );
}
