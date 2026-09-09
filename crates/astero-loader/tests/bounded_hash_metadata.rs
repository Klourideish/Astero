use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::DynamicLimits,
        hash::{
            HashLimits,
            bounded::{self, HashMetadataFailure, HashMetadataLimits, HashMetadataOutcome},
            error::{HashError, HashFailure},
            extent::CountClaim,
            synthetic::{Variant, image_with_hashes},
        },
        synthetic::*,
    },
};
fn limits(words: u64) -> HashMetadataLimits {
    HashMetadataLimits {
        dynamic: DynamicLimits {
            max_program_headers: 2,
            max_dynamic_entries: 16,
        },
        hash: HashLimits { max_words: words },
    }
}
fn source(b: Vec<u8>) -> SourceArtifact {
    SourceArtifact::new(b, Some("M20 synthetic".into())).unwrap()
}
fn observe(b: Vec<u8>, words: u64) -> bounded::HashMetadataReport {
    bounded::observe(source(b), limits(words))
}
fn evidence(
    r: &bounded::HashMetadataReport,
) -> &astero_loader::elf::dynamic::hash::HashObservation {
    let HashMetadataOutcome::Complete(e) = r.outcome() else {
        panic!("{:?}", r.outcome())
    };
    e
}
#[test]
fn exact_shared_budgets_and_source_proof_without_reading_symbols() {
    for (variant, required, proofs) in [
        (Variant::SysV, 9, 1),
        (Variant::Gnu, 9, 1),
        (Variant::Both, 18, 2),
    ] {
        let bytes = image_with_hashes(variant);
        let s = source(bytes.clone());
        let token = s.checked_range(0, s.len()).unwrap();
        let r = bounded::observe(s.clone(), limits(required));
        let e = evidence(&r);
        let extent = e.extent().unwrap();
        assert_eq!(extent.symbol_count(), 3);
        assert_eq!(extent.evidence().len(), proofs);
        assert_eq!(extent.source_range(), s.checked_range(0x400, 72).unwrap());
        assert_eq!(r.source().identity(), s.identity());
        assert_eq!(r.source().provenance(), s.provenance());
        assert_eq!(e, evidence(&bounded::observe(s.clone(), limits(required))));
        assert_eq!(s.read(&token).unwrap(), bytes);
        assert_eq!(r.limits(), limits(required));
        // 0xff null/name fields and unrelated malformed Needed/Rela do not affect the count.
        for budget in 0..required {
            assert!(matches!(
                bounded::observe(s.clone(), limits(budget)).outcome(),
                HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::At {
                    failure: HashFailure::WorkLimit,
                    ..
                }))
            ));
        }
    }
}
#[test]
fn absence_lower_bound_and_corroboration_remain_distinct() {
    assert!(matches!(
        observe(image_with_hashes(Variant::None), 0).outcome(),
        HashMetadataOutcome::Unavailable
    ));
    let mut none = image(&[(0, 0)]);
    put16(&mut none, 56, 1);
    assert!(matches!(
        observe(none, 0).outcome(),
        HashMetadataOutcome::Unavailable
    ));
    let r = observe(image_with_hashes(Variant::LowerBound), 7);
    let e = evidence(&r);
    assert!(e.extent().is_none());
    assert_eq!(e.gnu().unwrap().count_claim(), CountClaim::LowerBound(1));
    let mut b = image_with_hashes(Variant::Both);
    put32(&mut b, 0x358, 0);
    let r = observe(b, 16);
    let e = evidence(&r);
    assert_eq!(e.extent().unwrap().symbol_count(), 3);
    assert_eq!(
        e.extent().unwrap().evidence()[1].claim,
        CountClaim::LowerBound(1)
    );
    assert!(matches!(
        observe(image_with_hashes(Variant::Conflict), 100).outcome(),
        HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::ConflictingEvidence {
            sysv: 3,
            gnu: 2,
            gnu_exact: true,
            ..
        }))
    ));
}
#[test]
fn descriptors_and_symbol_backing_are_not_overridden_by_hash_counts() {
    for duplicate in [0x1300, 0x1304] {
        let b = image(&[(6, 0x1400), (11, 24), (4, 0x1300), (4, duplicate), (0, 0)]);
        let r = observe(b, 100);
        assert_eq!(r.descriptor_fields().count(), 4);
        assert!(matches!(
            r.outcome(),
            HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::Dynamic(_)))
        ));
    }
    let mut b = image_with_hashes(Variant::SysV);
    put64(&mut b, 0x208, 0x15d0); // only two symbol entries fit
    assert!(matches!(
        observe(b, 100).outcome(),
        HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::SymbolExtent {
            count: 3,
            ..
        }))
    ));
    for value in [0, 23, u64::MAX] {
        let mut b = image_with_hashes(Variant::SysV);
        put64(&mut b, 0x218, value);
        assert!(matches!(
            observe(b, 100).outcome(),
            HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::Dynamic(_)))
        ));
    }
    let mut b = image_with_hashes(Variant::SysV);
    put64(&mut b, 0x200, 99);
    put64(&mut b, 0x210, 99);
    assert!(matches!(
        observe(b, 100).outcome(),
        HashMetadataOutcome::Failed(HashMetadataFailure::Hash(
            HashError::MissingSymbolDescriptor
        ))
    ));
}
#[test]
fn sysv_field_and_source_boundary_sweep_is_structured() {
    for buckets in [0, 1, 2, u32::MAX] {
        for chains in [0, 1, 3, u32::MAX] {
            let mut b = image_with_hashes(Variant::SysV);
            put32(&mut b, 0x300, buckets);
            put32(&mut b, 0x304, chains);
            let r = observe(b, 32);
            if (buckets == 1 || buckets == 2) && chains == 3 {
                assert!(matches!(r.outcome(), HashMetadataOutcome::Complete(_)));
            } else {
                assert!(matches!(r.outcome(), HashMetadataOutcome::Failed(_)));
            }
        }
    }
    // Header/arrays at exact source end and every short boundary. No truncation is count evidence.
    for remaining in 0..=24 {
        let mut b = image_with_hashes(Variant::SysV);
        let at = 0x600 - remaining;
        let words = [1u32, 3, 1, 0, 2, 0];
        let raw: Vec<_> = words.into_iter().flat_map(u32::to_le_bytes).collect();
        b[at..].copy_from_slice(&raw[..remaining]);
        put64(&mut b, 0x228, (0x1000 + at) as u64);
        let r = observe(b, 9);
        assert_eq!(
            matches!(r.outcome(), HashMetadataOutcome::Complete(_)),
            remaining == 24
        );
    }
    for address in [0, 0x1601, u64::MAX] {
        let mut b = image_with_hashes(Variant::SysV);
        put64(&mut b, 0x228, address);
        assert!(matches!(
            observe(b, 100).outcome(),
            HashMetadataOutcome::Failed(_)
        ));
    }
}
#[test]
fn gnu_termination_layout_and_work_boundaries_do_not_guess_counts() {
    for terminal in 0..8 {
        let mut b = image_with_hashes(Variant::Gnu);
        for i in 0..=terminal {
            put32(&mut b, 0x35c + i * 4, if i == terminal { 1 } else { 2 });
        }
        let required = 8 + terminal as u64;
        let r = observe(b.clone(), required);
        assert_eq!(
            evidence(&r).extent().unwrap().symbol_count(),
            terminal as u64 + 2
        );
        assert!(matches!(
            observe(b, required - 1).outcome(),
            HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::At {
                failure: HashFailure::WorkLimit,
                ..
            }))
        ));
    }
    for (at, value) in [
        (0x348, 0),
        (0x348, 3),
        (0x348, u32::MAX),
        (0x344, 0),
        (0x358, 2),
        (0x358, u32::MAX),
    ] {
        let mut b = image_with_hashes(Variant::Gnu);
        put32(&mut b, at, value);
        assert!(matches!(
            observe(b, 100).outcome(),
            HashMetadataOutcome::Failed(_)
        ));
    }
    let mut b = image_with_hashes(Variant::Gnu);
    b[0x35c..].fill(0); // no low-bit terminator, valid source ends
    assert!(matches!(
        observe(b, 1000).outcome(),
        HashMetadataOutcome::Failed(HashMetadataFailure::Hash(HashError::At {
            failure: HashFailure::MissingChainTerminator { .. },
            ..
        }))
    ));
}

#[test]
fn trusted_report_does_not_mutate_candidate_only_symbol_table() {
    use astero_loader::{
        elf::{
            dynamic::{
                ObservationLimits,
                symbol_table::{SymbolError, SymbolTable},
            },
            inspect,
        },
        modules::{ModuleId, ModuleMetadata},
    };
    let mut bytes = image(&[(6, 0x1400), (11, 24), (4, 0x1300), (0, 0)]);
    let full = image_with_hashes(Variant::SysV);
    bytes[0x300..].copy_from_slice(&full[0x300..]);
    let s = source(bytes);
    let elf = inspect(
        s.clone(),
        Some(ModuleMetadata {
            id: ModuleId(20),
            name: "synthetic".into(),
        }),
    )
    .unwrap();
    let table = SymbolTable::new(&elf, ObservationLimits { max_entries: 4 }).unwrap();
    assert!(matches!(
        table.symbol_count(),
        Err(SymbolError::CountUnavailable)
    ));
    let report = bounded::observe(s, limits(9));
    assert_eq!(evidence(&report).extent().unwrap().symbol_count(), 3);
    assert!(matches!(
        table.symbol_count(),
        Err(SymbolError::CountUnavailable)
    ));
}
