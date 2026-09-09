use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::DynamicLimits,
        hash::{
            HashLimits,
            bounded::{self as hash, HashMetadataLimits},
        },
        symbol_table::{
            Binding, Section, SymbolError, SymbolType, Visibility,
            bounded::{
                self, SymbolObservationFailure as Failure, SymbolObservationLimits,
                SymbolObservationReport, SymbolOutcome,
            },
            synthetic::{Variant, image_with_symbols},
        },
        synthetic::*,
    },
};
use std::sync::Arc;
fn limits() -> SymbolObservationLimits {
    SymbolObservationLimits {
        max_descriptors: 2,
        max_symbols: 3,
        max_name_lookups: 2,
        max_name_scan_bytes: 6,
        max_total_name_scan_bytes: 12,
    }
}
fn source(b: Vec<u8>) -> SourceArtifact {
    SourceArtifact::new(b, Some("M21 synthetic".into())).unwrap()
}
fn proof(s: &SourceArtifact) -> Arc<hash::HashMetadataReport> {
    Arc::new(hash::observe(
        s.clone(),
        HashMetadataLimits {
            dynamic: DynamicLimits {
                max_program_headers: 2,
                max_dynamic_entries: 16,
            },
            hash: HashLimits { max_words: 64 },
        },
    ))
}
fn observe(b: Vec<u8>, l: SymbolObservationLimits) -> SymbolObservationReport {
    let s = source(b);
    bounded::observe(s.clone(), proof(&s), l)
}
fn records(r: &SymbolObservationReport) -> &[bounded::SymbolRecord] {
    let SymbolOutcome::Complete(v) = r.outcome() else {
        panic!("{:?}", r.outcome())
    };
    v
}
#[test]
fn exact_membership_order_raw_fields_names_and_shared_proof_are_preserved() {
    for variant in [Variant::SysV, Variant::Gnu, Variant::Both, Variant::Unknown] {
        let b = image_with_symbols(variant);
        let s = source(b.clone());
        let p = proof(&s);
        let r = bounded::observe(s.clone(), p.clone(), limits());
        let v = records(&r);
        assert_eq!(
            v.iter().map(|v| v.fields().index).collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!(v[0].name_bytes(), None);
        assert_eq!(v[1].name_bytes(), Some(b"alpha".as_slice()));
        assert_eq!(v[2].name_bytes(), v[1].name_bytes());
        assert_eq!(v[2].fields().source, s.checked_range(0x430, 24).unwrap());
        assert_eq!(v[1].name_range(), Some(s.checked_range(0x501, 5).unwrap()));
        assert!(Arc::ptr_eq(r.proof(), &p));
        assert_eq!(r.source().identity(), s.identity());
        assert_eq!(r.source().provenance(), s.provenance());
        assert_eq!(s.read(&s.checked_range(0, s.len()).unwrap()).unwrap(), b);
        assert_eq!(v, records(&bounded::observe(s, p, limits())));
        if matches!(variant, Variant::Unknown) {
            let f = v[2].fields();
            assert_eq!(
                (f.info, f.other, f.shndx, f.value, f.size),
                (0xef, 0xfd, 0xff20, u64::MAX, 8)
            );
            assert_eq!(
                (f.binding, f.symbol_type, f.visibility, f.section),
                (
                    Binding::Unknown(14),
                    SymbolType::Unknown(15),
                    Visibility::Unknown(5),
                    Section::Reserved(0xff20)
                )
            );
        }
    }
}
#[test]
fn unavailable_and_mismatched_proofs_never_authorize_observation() {
    for variant in [Variant::None, Variant::LowerBound] {
        assert!(matches!(
            observe(image_with_symbols(variant), limits()).outcome(),
            SymbolOutcome::Unavailable
        ));
    }
    let s = source(image_with_symbols(Variant::SysV));
    let p = proof(&s);
    let other = source(image_with_symbols(Variant::SysV));
    assert!(matches!(
        bounded::observe(other, p, limits()).outcome(),
        SymbolOutcome::Failed(Failure::SourceMismatch { .. })
    ));
    let mut b = image_with_symbols(Variant::Both);
    put32(&mut b, 0x35c, 1);
    assert!(matches!(
        observe(b, limits()).outcome(),
        SymbolOutcome::Failed(Failure::EvidenceFailed)
    ));
    let mut b = image_with_symbols(Variant::SysV);
    put32(&mut b, 0x304, 0);
    assert!(matches!(
        observe(b, limits()).outcome(),
        SymbolOutcome::Failed(Failure::EvidenceFailed)
    ));
}
#[test]
fn complete_extent_and_entry_budget_precede_decoding() {
    for maximum in 0..3 {
        let mut b = image_with_symbols(Variant::SysV);
        b[0x400] = 1;
        let mut l = limits();
        l.max_symbols = maximum;
        assert!(matches!(
            observe(b, l).outcome(),
            SymbolOutcome::Failed(Failure::EntryBudget { count: 3, .. })
        ));
    }
    let mut b = image_with_symbols(Variant::SysV);
    b[0x400] = 1;
    assert!(matches!(
        observe(b, limits()).outcome(),
        SymbolOutcome::Failed(Failure::Symbol {
            error: SymbolError::InvalidNullSymbol,
            ..
        })
    ));
    // Source-backed symbol extent ends exactly at EOF, versus one byte short.
    for at in [0x5b8, 0x5b9] {
        let mut b = image_with_symbols(Variant::SysV);
        let sym = b[0x400..0x448].to_vec();
        let n = (0x600 - at).min(72);
        b[at..at + n].copy_from_slice(&sym[..n]);
        put64(&mut b, 0x208, (0x1000 + at) as u64);
        let r = observe(b, limits());
        assert_eq!(
            matches!(r.outcome(), SymbolOutcome::Complete(_)),
            at == 0x5b8
        );
    }
    let mut b = image_with_symbols(Variant::SysV);
    put32(&mut b, 0x304, 1);
    put32(&mut b, 0x308, 0);
    let mut l = limits();
    l.max_symbols = 1;
    l.max_name_lookups = 0;
    l.max_name_scan_bytes = 0;
    l.max_total_name_scan_bytes = 0;
    assert_eq!(records(&observe(b, l)).len(), 1);
}
#[test]
fn lookup_budget_sweep_charges_nonzero_offsets_and_nul_without_partial_success() {
    for lookups in 0..=2 {
        for per in 0..=6 {
            for total in 0..=12 {
                let mut l = limits();
                l.max_name_lookups = lookups;
                l.max_name_scan_bytes = per;
                l.max_total_name_scan_bytes = total;
                let r = observe(image_with_symbols(Variant::SysV), l);
                assert_eq!(
                    matches!(r.outcome(), SymbolOutcome::Complete(_)),
                    lookups == 2 && per == 6 && total == 12
                );
            }
        }
    }
    for (variant, name, bytes, attempts) in [
        (Variant::Raw, Some(vec![255]), 8, 2),
        (Variant::Empty, Some(vec![]), 7, 2),
        (Variant::Unnamed, None, 6, 1),
    ] {
        let mut l = limits();
        l.max_total_name_scan_bytes = bytes;
        l.max_name_lookups = attempts;
        let r = observe(image_with_symbols(variant), l);
        assert_eq!(records(&r)[2].name_bytes(), name.as_deref());
    }
}
#[test]
fn malformed_names_fail_with_index_and_prior_work_context() {
    let r = observe(image_with_symbols(Variant::BadName), limits());
    assert!(matches!(
        r.outcome(),
        SymbolOutcome::Failed(Failure::Symbol {
            index: 2,
            completed: 2,
            lookups: 2,
            error: SymbolError::Name { .. },
            ..
        })
    ));
    let mut b = image_with_symbols(Variant::SysV);
    b[0x501..0x50a].fill(b'x');
    let mut l = limits();
    l.max_name_scan_bytes = 10;
    l.max_total_name_scan_bytes = 20;
    assert!(matches!(
        observe(b, l).outcome(),
        SymbolOutcome::Failed(Failure::Symbol {
            index: 1,
            error: SymbolError::Name { .. },
            ..
        })
    ));
    let mut b = image_with_symbols(Variant::SysV);
    put64(&mut b, 0x220, 99);
    put64(&mut b, 0x230, 99);
    assert!(matches!(
        observe(b, limits()).outcome(),
        SymbolOutcome::Failed(Failure::Symbol {
            index: 1,
            error: SymbolError::StringsUnavailable { .. },
            ..
        })
    ));
}
