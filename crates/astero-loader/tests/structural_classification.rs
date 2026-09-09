use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::DynamicLimits,
        hash::{
            HashLimits,
            bounded::{self as hash, HashMetadataLimits},
        },
        symbol_table::{
            bounded::{self, SymbolObservationLimits, SymbolObservationReport, SymbolOutcome},
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

use astero_loader::elf::dynamic::candidates::structural::{
    self, CandidateRole as Role, ClassificationFailure as Failure, ClassificationLimits,
    ClassificationOutcome as Outcome,
};
#[test]
fn roles_keep_shared_original_evidence_names_order_and_identity() {
    for variant in [
        Variant::SysV,
        Variant::Raw,
        Variant::Empty,
        Variant::Unnamed,
        Variant::Unknown,
    ] {
        let input = Arc::new(observe(image_with_symbols(variant), limits()));
        let before = format!("{input:?}");
        let report = structural::classify(
            input.clone(),
            ClassificationLimits {
                max_classifications: 3,
            },
        );
        assert!(Arc::ptr_eq(report.input(), &input));
        let roles: Vec<_> = report
            .entries()
            .map(|(r, s)| {
                assert_eq!(r.symbol_index(), s.fields().index);
                r.role()
            })
            .collect();
        assert_eq!(roles[0], Role::NullSymbol);
        assert_eq!(roles[1], Role::UndefinedCandidate);
        assert_eq!(
            roles[2],
            if matches!(variant, Variant::Unknown) {
                Role::SpecialCandidate
            } else {
                Role::DefinitionCandidate
            }
        );
        for ((_, actual), original) in report.entries().zip(records(&input)) {
            assert!(std::ptr::eq(actual, original));
        }
        let again = structural::classify(
            input.clone(),
            ClassificationLimits {
                max_classifications: 3,
            },
        );
        assert_eq!(
            format!("{:?}", report.outcome()),
            format!("{:?}", again.outcome())
        );
        assert_eq!(before, format!("{input:?}"));
    }
}
#[test]
fn every_record_consumes_budget_before_any_successful_prefix() {
    let input = Arc::new(observe(image_with_symbols(Variant::SysV), limits()));
    for maximum in 0..=3 {
        let r = structural::classify(
            input.clone(),
            ClassificationLimits {
                max_classifications: maximum,
            },
        );
        assert_eq!(r.limits().max_classifications, maximum);
        if maximum < 3 {
            assert!(
                matches!(r.outcome(),Outcome::Failed(Failure::Budget{count:3,maximum:m}) if *m==maximum)
            );
            assert_eq!(r.entries().count(), 0);
        } else {
            assert_eq!(r.entries().count(), 3);
        }
    }
}
#[test]
fn prerequisites_remain_unavailable_or_structured_failure() {
    for variant in [Variant::None, Variant::LowerBound, Variant::BadName] {
        let input = Arc::new(observe(image_with_symbols(variant), limits()));
        let r = structural::classify(
            input.clone(),
            ClassificationLimits {
                max_classifications: 3,
            },
        );
        assert!(Arc::ptr_eq(r.input(), &input));
        assert_eq!(r.entries().count(), 0);
        if matches!(variant, Variant::BadName) {
            assert!(matches!(
                r.outcome(),
                Outcome::Failed(Failure::PrerequisiteFailed)
            ));
        } else {
            assert!(matches!(r.outcome(), Outcome::Unavailable));
        }
    }
}
#[test]
fn bounded_attribute_sweep_uses_only_section_role_and_preserves_unknowns() {
    for section in [0, 1, 0xfff1, 0xfff2, 0xff20, 0xffff] {
        for info in [0x00, 0x11, 0x12, 0x21, 0xef] {
            for other in [0, 1, 2, 3, 0xfd] {
                let mut b = image_with_symbols(Variant::Raw);
                put16(&mut b, 0x436, section);
                b[0x434] = info;
                b[0x435] = other;
                let input = Arc::new(observe(b, limits()));
                let r = structural::classify(
                    input,
                    ClassificationLimits {
                        max_classifications: 3,
                    },
                );
                let (role, s) = r.entries().nth(2).unwrap();
                assert_eq!(
                    (s.fields().info, s.fields().other, s.fields().shndx),
                    (info, other, section)
                );
                assert_eq!(s.name_bytes(), Some([255].as_slice()));
                assert_eq!(
                    role.role(),
                    match section {
                        0 => Role::UndefinedCandidate,
                        1 => Role::DefinitionCandidate,
                        _ => Role::SpecialCandidate,
                    }
                );
            }
        }
    }
}
