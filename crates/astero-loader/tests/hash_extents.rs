mod dynamic_fixtures;
use astero_loader::elf::{
    address_translation::TranslationError,
    dynamic::{
        self, ObservationLimits,
        hash::{
            self, HashLimits,
            error::{HashError, HashFailure, HashKind},
            extent::CountClaim,
        },
        symbol_table::{EnumerationLimits, SymbolError, SymbolTable},
    },
};
use dynamic_fixtures::{image, parsed, program, put32, put64};
fn dl() -> ObservationLimits {
    ObservationLimits { max_entries: 32 }
}
fn hl() -> HashLimits {
    HashLimits { max_words: 4096 }
}
fn el() -> EnumerationLimits {
    EnumerationLimits {
        max_symbols: 64,
        max_name_scan_bytes: 32,
        max_total_name_scan_bytes: 128,
    }
}
fn fixture(sysv: bool, gnu: bool) -> Vec<u8> {
    let mut tags = vec![(6, 0x2000), (11, 24), (5, 0x2800), (10, 8)];
    if sysv {
        tags.push((4, 0x1600));
    }
    if gnu {
        tags.push((0x6ffffef5, 0x1800));
    }
    tags.push((0, 0));
    let mut b = image(&tags);
    b.resize(0x2000, 0);
    program(&mut b, 0, 1, 0x100, 0x1100, 0x1f00, 0x2000);
    b[0x1800..0x1808].copy_from_slice(b"\0alpha\0\0");
    put32(&mut b, 0x1018, 1);
    b[0x101c] = 0x12;
    put32(&mut b, 0x1030, 1);
    b[0x1034] = 0x21;
    b
}
fn sysv(b: &mut [u8], buckets: &[u32], chains: &[u32]) {
    put32(b, 0x600, buckets.len() as u32);
    put32(b, 0x604, chains.len() as u32);
    for (i, v) in buckets.iter().chain(chains).enumerate() {
        put32(b, 0x608 + i * 4, *v);
    }
}
fn gnu(b: &mut [u8], offset: u32, buckets: &[u32], chains: &[u32]) {
    put32(b, 0x800, buckets.len() as u32);
    put32(b, 0x804, offset);
    put32(b, 0x808, 1);
    put32(b, 0x80c, 5);
    put64(b, 0x810, 0xabcdef01);
    for (i, v) in buckets.iter().chain(chains).enumerate() {
        put32(b, 0x818 + i * 4, *v);
    }
}
fn report(b: Vec<u8>) -> Result<hash::HashObservation, HashError> {
    hash::observe(&parsed(b), dl(), hl())
}
#[test]
fn sysv_count_provenance_and_lazy_enumeration_preserve_names() {
    let mut b = fixture(true, false);
    sysv(&mut b, &[1, 2], &[0, 0, 0]);
    let elf = parsed(b);
    let r = hash::observe(&elf, dl(), hl()).unwrap();
    assert_eq!(r, hash::observe(&elf, dl(), hl()).unwrap());
    let extent = r.extent().unwrap();
    assert_eq!(extent.symbol_count(), 3);
    assert_eq!(
        extent.source_range(),
        elf.artifact().source().checked_range(0x1000, 72).unwrap()
    );
    let h = r.sysv().unwrap();
    assert_eq!(h.bucket_count(), 2);
    assert_eq!(h.chain_count(), 3);
    assert_eq!(
        h.source_range(),
        elf.artifact().source().checked_range(0x600, 28).unwrap()
    );
    assert_eq!(extent.evidence()[0].kind, HashKind::SysV);
    let table = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let symbols = table
        .enumerate(el())
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(symbols.len(), 3);
    assert_eq!(symbols[1].name.unwrap().as_bytes(), b"alpha");
    assert_eq!(symbols[2].name.unwrap().as_bytes(), b"alpha");
    assert_eq!(
        symbols,
        table
            .enumerate(el())
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    );
    assert!(elf.artifact().description().imports.is_empty());
    assert!(elf.artifact().description().exports.is_empty());
    assert_eq!(
        SymbolTable::new(&elf, dl()).unwrap().symbol_count(),
        Err(SymbolError::CountUnavailable)
    );
}
#[test]
fn minimal_sysv_and_zero_counts_have_deliberate_policy() {
    let mut b = fixture(true, false);
    sysv(&mut b, &[0], &[0]);
    assert_eq!(report(b).unwrap().extent().unwrap().symbol_count(), 1);
    for (buckets, chains, expected) in [
        (0, 1, HashFailure::ZeroBuckets),
        (1, 0, HashFailure::ZeroSymbols),
    ] {
        let mut b = fixture(true, false);
        put32(&mut b, 0x600, buckets);
        put32(&mut b, 0x604, chains);
        assert!(matches!(report(b),Err(HashError::At{failure,..}) if failure==expected));
    }
}
#[test]
fn sysv_reference_and_cycle_failures_are_structured() {
    let mut b = fixture(true, false);
    sysv(&mut b, &[2], &[0, 0]);
    assert!(matches!(
        report(b),
        Err(HashError::At {
            failure: HashFailure::InvalidReference { .. },
            ..
        })
    ));
    let mut b = fixture(true, false);
    sysv(&mut b, &[1], &[0, 2]);
    assert!(matches!(
        report(b),
        Err(HashError::At {
            failure: HashFailure::InvalidReference { .. },
            ..
        })
    ));
    let mut b = fixture(true, false);
    sysv(&mut b, &[1], &[0, 1]);
    assert!(matches!(
        report(b),
        Err(HashError::At {
            failure: HashFailure::Cycle { .. },
            ..
        })
    ));
}
#[test]
fn widened_sysv_counts_never_wrap_into_short_arrays() {
    for (nb, nc) in [(u32::MAX, 1), (1, u32::MAX), (u32::MAX, u32::MAX)] {
        let mut b = fixture(true, false);
        put32(&mut b, 0x600, nb);
        put32(&mut b, 0x604, nc);
        assert!(matches!(
            report(b),
            Err(HashError::At {
                failure: HashFailure::Translation(_),
                ..
            })
        ));
    }
}
#[test]
fn hash_header_boundaries_bss_and_address_overflow_are_checked() {
    for (tag, width) in [(4, 8), (0x6ffffef5, 16)] {
        for address in [0x3000 - (width - 1), 0x3000, 0x4000, u64::MAX - 3] {
            let mut b = fixture(false, false); // replace DT_NULL with hash, append terminator
            put64(&mut b, 0x240, tag);
            put64(&mut b, 0x248, address);
            program(&mut b, 1, 2, 0x200, 0x1200, 96, 96);
            assert!(matches!(
                report(b),
                Err(HashError::At {
                    failure: HashFailure::Translation(_),
                    ..
                })
            ));
        }
    }
}
#[test]
fn gnu_terminated_suffix_and_bloom_are_source_bound() {
    let mut b = fixture(false, true);
    gnu(&mut b, 1, &[1, 2], &[0x101, 0x202, 0x303]);
    let elf = parsed(b);
    let r = hash::observe(&elf, dl(), hl()).unwrap();
    let h = r.gnu().unwrap();
    assert_eq!(h.first_symbol(), 1);
    assert_eq!(h.bloom_size(), 1);
    assert_eq!(h.bloom_shift(), 5);
    assert_eq!(h.bucket_count(), 2);
    assert_eq!(h.count_claim(), CountClaim::Exact(4));
    assert_eq!(
        h.source_range(),
        elf.artifact().source().checked_range(0x800, 44).unwrap()
    );
    assert_eq!(
        SymbolTable::with_hash(&elf, dl(), hl())
            .unwrap()
            .symbol_count(),
        Ok(4)
    );
}
#[test]
fn absent_and_empty_gnu_evidence_never_guesses_count() {
    let elf = parsed(fixture(false, false));
    let table = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(matches!(
        table.enumerate(el()),
        Err(SymbolError::CountUnavailable)
    ));
    for buckets in [vec![], vec![0], vec![0, 0]] {
        let mut b = fixture(false, true);
        gnu(&mut b, 3, &buckets, &[]);
        let elf = parsed(b);
        let r = hash::observe(&elf, dl(), hl()).unwrap();
        assert_eq!(r.gnu().unwrap().count_claim(), CountClaim::LowerBound(3));
        assert!(r.extent().is_none());
        assert!(matches!(
            SymbolTable::with_hash(&elf, dl(), hl())
                .unwrap()
                .enumerate(el()),
            Err(SymbolError::CountUnavailable)
        ));
    }
}
#[test]
fn gnu_invalid_layouts_and_huge_bloom_never_authorize_enumeration() {
    for size in [0, 3, u32::MAX] {
        let mut b = fixture(false, true);
        gnu(&mut b, 1, &[1], &[1]);
        put32(&mut b, 0x808, size);
        assert!(matches!(
            report(b),
            Err(HashError::At {
                failure: HashFailure::InvalidBloomSize(_),
                ..
            })
        ));
    }
    let mut b = fixture(false, true);
    gnu(&mut b, 1, &[1], &[1]);
    put32(&mut b, 0x808, 1 << 31);
    assert!(matches!(
        report(b),
        Err(HashError::At {
            failure: HashFailure::Translation(_),
            ..
        })
    ));
    for (offset, buckets) in [
        (0, vec![1]),
        (2, vec![1]),
        (1, vec![1, 1]),
        (1, vec![2]),
        (1, vec![u32::MAX]),
    ] {
        let mut b = fixture(false, true);
        gnu(&mut b, offset, &buckets, &[1]);
        assert!(matches!(
            report(b),
            Err(HashError::At {
                failure: HashFailure::InvalidSymbolOffset(_)
                    | HashFailure::InvalidBucketLayout { .. },
                ..
            })
        ));
    }
}
#[test]
fn gnu_missing_termination_and_work_limit_are_separate() {
    let mut b = fixture(false, true);
    gnu(&mut b, 1, &[1], &[2]);
    b[0x81c..].fill(0); // no terminator anywhere in source prefix
    assert!(matches!(
        report(b.clone()),
        Err(HashError::At {
            failure: HashFailure::MissingChainTerminator { .. },
            ..
        })
    ));
    let elf = parsed(b);
    assert!(matches!(
        hash::observe(&elf, dl(), HashLimits { max_words: 8 }),
        Err(HashError::At {
            failure: HashFailure::WorkLimit,
            ..
        })
    ));
}
#[test]
fn two_evidence_sources_must_agree_including_partial_lower_bound() {
    for (count, gnu_count, ok) in [(3, 3, true), (2, 3, false), (4, 3, false)] {
        let mut b = fixture(true, true);
        sysv(&mut b, &[0], &vec![0; count]);
        gnu(&mut b, 1, &[1], &vec![1; gnu_count - 1]);
        // one bucket is a single contiguous chain, only final word terminates
        for i in 0..gnu_count - 2 {
            put32(&mut b, 0x81c + i * 4, 2);
        }
        let r = report(b);
        assert_eq!(r.is_ok(), ok);
        if ok {
            assert_eq!(r.unwrap().extent().unwrap().evidence().len(), 2);
        } else {
            assert!(matches!(
                r,
                Err(HashError::ConflictingEvidence {
                    gnu_exact: true,
                    ..
                })
            ));
        }
    }
    for count in [2, 3, 4] {
        let mut b = fixture(true, true);
        sysv(&mut b, &[0], &vec![0; count]);
        gnu(&mut b, 3, &[0], &[]);
        assert_eq!(report(b).is_ok(), count >= 3);
    }
}
#[test]
fn trusted_count_still_requires_full_symbol_source_extent() {
    for remaining in [71, 72, 73] {
        let mut b = fixture(true, false);
        sysv(&mut b, &[0], &[0, 0, 0]);
        put64(&mut b, 0x208, 0x3000 - remaining);
        let elf = parsed(b);
        let r = SymbolTable::with_hash(&elf, dl(), hl());
        assert_eq!(r.is_ok(), remaining >= 72);
        if remaining < 72 {
            assert!(matches!(
                r,
                Err(SymbolError::Hash(HashError::SymbolExtent { count: 3, .. }))
            ));
        }
    }
}
#[test]
fn enumeration_budgets_and_name_errors_stop_without_fabricated_success() {
    let mut b = fixture(true, false);
    sysv(&mut b, &[0], &[0, 0, 0]);
    let elf = parsed(b);
    let table = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    assert!(matches!(
        table.enumerate(EnumerationLimits {
            max_symbols: 2,
            ..el()
        }),
        Err(SymbolError::EnumerationLimit { .. })
    ));
    let mut iter = table
        .enumerate(EnumerationLimits {
            max_total_name_scan_bytes: 6,
            ..el()
        })
        .unwrap();
    assert!(iter.next().unwrap().is_ok());
    assert!(iter.next().unwrap().is_ok());
    assert!(matches!(
        iter.next(),
        Some(Err(SymbolError::Name { index: 2, .. }))
    ));
    assert!(iter.next().is_none());
    assert!(iter.next().is_none());
    let mut b = fixture(true, false);
    sysv(&mut b, &[0], &[0, 0]);
    put32(&mut b, 0x1018, 100);
    let elf = parsed(b);
    let table = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let mut iter = table.enumerate(el()).unwrap();
    assert!(iter.next().unwrap().is_ok());
    assert!(matches!(
        iter.next(),
        Some(Err(SymbolError::Name { index: 1, .. }))
    ));
    assert!(iter.next().is_none());
}
#[test]
fn sysv_count_source_and_symbol_extent_sweep() {
    for buckets in 1..=4 {
        for count in 1..=16 {
            let mut b = fixture(true, false);
            sysv(&mut b, &vec![0; buckets], &vec![0; count]);
            let r = report(b).unwrap();
            assert_eq!(r.extent().unwrap().symbol_count(), count as u64);
            assert_eq!(
                r.sysv().unwrap().source_range().extent().size,
                (8 + 4 * (buckets + count)) as u64
            );
        }
    }
}
#[test]
fn gnu_termination_and_prefix_boundary_sweep() {
    for chain_len in 1..=16usize {
        for missing in [false, true] {
            let mut b = fixture(false, true);
            let mut words = vec![2; chain_len];
            if !missing {
                words[chain_len - 1] = 3;
            };
            gnu(&mut b, 1, &[1], &words);
            // End the PT_LOAD source exactly after these chain words, keeping memory tail.
            program(
                &mut b,
                0,
                1,
                0x100,
                0x1100,
                (0x81c + 4 * chain_len - 0x100) as u64,
                0x2000,
            );
            // Keep the required first symbol and string table within the smaller file prefix.
            put64(&mut b, 0x208, 0x1400);
            put64(&mut b, 0x228, 0x1500);
            let r = report(b);
            if missing {
                assert!(matches!(
                    r,
                    Err(HashError::At {
                        failure: HashFailure::MissingChainTerminator {
                            error: TranslationError::CrossesSourceBoundary { .. }
                                | TranslationError::ZeroFill { .. },
                            ..
                        },
                        ..
                    })
                ));
            } else {
                assert_eq!(
                    r.unwrap().extent().unwrap().symbol_count(),
                    chain_len as u64 + 1
                );
            }
        }
    }
}
#[test]
fn duplicate_hash_tags_and_missing_symbol_descriptor_fail_explicitly() {
    let mut b = fixture(true, false);
    sysv(&mut b, &[0], &[0]);
    put64(&mut b, 0x250, 4);
    put64(&mut b, 0x258, 0x1600);
    program(&mut b, 1, 2, 0x200, 0x1200, 112, 112);
    assert!(matches!(
        report(b),
        Err(HashError::Dynamic(
            dynamic::error::DynamicError::DuplicateTag { .. }
        ))
    ));
    let mut b = fixture(true, false);
    sysv(&mut b, &[0], &[0]);
    put64(&mut b, 0x200, 0x1234);
    put64(&mut b, 0x210, 0x1235);
    assert!(matches!(report(b), Err(HashError::MissingSymbolDescriptor)));
}

#[test]
fn exact_hash_array_end_and_one_byte_short_are_distinct() {
    for short in [false, true] {
        let mut b = fixture(true, false);
        put64(&mut b, 0x248, 0x2ff0);
        put32(&mut b, 0x1ff0, 1);
        put32(&mut b, 0x1ff4, 1);
        if short {
            program(&mut b, 0, 1, 0x100, 0x1100, 0x1eff, 0x2000);
        }
        let result = report(b);
        if short {
            assert!(matches!(
                result,
                Err(HashError::At {
                    failure: HashFailure::Translation(
                        TranslationError::CrossesSourceBoundary { .. }
                    ),
                    ..
                })
            ));
        } else {
            assert_eq!(result.unwrap().extent().unwrap().symbol_count(), 1);
        }
    }
    let mut b = fixture(false, true);
    gnu(&mut b, 1, &[1], &[1]);
    put32(&mut b, 0x800, u32::MAX);
    assert!(matches!(
        report(b),
        Err(HashError::At {
            failure: HashFailure::Translation(_),
            ..
        })
    ));
}
