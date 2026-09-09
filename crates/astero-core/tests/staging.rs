use astero_loader::{
    artifact::SourceArtifact,
    elf::dynamic::{
        bounded::DynamicLimits,
        candidates::workload::{self, LinkageLimits},
        hash::{HashLimits, bounded::HashMetadataLimits},
        identity::{self, *},
        symbol_table::bounded::SymbolObservationLimits,
        synthetic::{put16, put32, put64},
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
use astero_loader::{load_plan::link::*, metadata::VirtualAddress};
fn image() -> Vec<u8> {
    let mut b = synthetic::identity_image();
    put64(&mut b, 24, 0x1100);
    put32(&mut b, 68, 7);
    put64(&mut b, 104, 0xa00);
    for i in 0..4 {
        put64(&mut b, 0x540 + i * 24, 0x1800 + i as u64 * 8);
    }
    put64(&mut b, 0x580, 4);
    b
}
fn report(b: Vec<u8>) -> Arc<Ps5IdentityEvidenceReport> {
    Arc::new(identity::observe(
        Arc::new(workload::observe(
            SourceArtifact::new(b, Some("M27 synthetic".into())).unwrap(),
            limits(),
        )),
        IdentityLimits {
            max_identity_records: 16,
        },
    ))
}
fn run(b: Vec<u8>, providers: Vec<ProviderInput>) -> GuestLoadPlan {
    plan(
        report(b),
        providers,
        VirtualAddress(0x10000),
        PlanningLimits {
            max_providers: 4,
            max_plan_records: 256,
        },
    )
    .unwrap()
}
fn provider(mut b: Vec<u8>) -> ProviderInput {
    put64(&mut b, 0x6f0, 0x61000047);
    put16(&mut b, 0x41e, 1);
    put64(&mut b, 0x420, 0x1120);
    ProviderInput {
        evidence: report(b),
        dependency_alias: Some(b"sample".to_vec()),
        image_bias: Some(VirtualAddress(0x20000)),
    }
}

use astero_core::input::staging::{
    self, ProtectionEnforcement, RelocationState, StageStatus, StagingLimits,
};
fn staged(p: GuestLoadPlan) -> (staging::StagedGuestImage, staging::MappingObserver) {
    let (r, o) = staging::stage(
        Arc::new(p),
        StagingLimits {
            max_mapped_bytes: 0x10000,
        },
    );
    (r.unwrap(), o)
}
#[test]
fn copy_zero_tail_and_provenance_survive_independent_writes() {
    let p = Arc::new(run(image(), vec![]));
    let original = p.headers().source().clone();
    let (r, o) = staging::stage(
        p.clone(),
        StagingLimits {
            max_mapped_bytes: 0xa00,
        },
    );
    let mut s = r.unwrap();
    assert_eq!(
        s.read(VirtualAddress(0x11100), 16).unwrap(),
        original
            .read(&original.checked_range(0x100, 16).unwrap())
            .unwrap()
    );
    assert!(
        s.read(VirtualAddress(0x11a00), 0x100)
            .unwrap()
            .iter()
            .all(|b| *b == 0)
    );
    assert_eq!(s.snapshot().zero_filled_bytes, 0x100);
    assert_eq!(
        s.snapshot().protections,
        ProtectionEnforcement::MetadataOnly
    );
    assert!(Arc::ptr_eq(s.plan(), &p));
    assert!(!s.snapshot().ready_for_execution);
    assert_eq!(o.active_mappings(), 1);
    s.release();
    s.release();
    assert_eq!(o.active_mappings(), 0);
    assert_eq!(s.snapshot().status, StageStatus::Released);
    assert!(s.read(VirtualAddress(0x11100), 1).is_err());
}
#[test]
fn writes_match_plan_pending_bytes_are_never_fabricated() {
    let (s, _) = staged(run(image(), vec![]));
    let p = s.plan();
    for (r, state) in p.relocations().iter().zip(s.relocations()) {
        if *state == RelocationState::Applied {
            assert_eq!(
                s.read(r.place.unwrap(), r.width as u64).unwrap(),
                &r.value.unwrap().to_le_bytes()[..r.width as usize]
            );
        } else if *state == RelocationState::Pending {
            let address = r.place.unwrap().0;
            let m = &p.segments()[0].mapping;
            let off = m.copy.as_ref().unwrap().source.extent().offset.0 + address - m.range.start.0;
            assert_eq!(
                s.read(r.place.unwrap(), r.width as u64).unwrap(),
                p.headers()
                    .source()
                    .read(
                        &p.headers()
                            .source()
                            .checked_range(off, r.width as u64)
                            .unwrap()
                    )
                    .unwrap()
            );
        }
    }
    assert!(s.snapshot().applied_relocations > 0);
    assert!(s.snapshot().pending_relocations > 0);
}
#[test]
fn selected_but_unstaged_provider_value_remains_pending() {
    let (s, _) = staged(run(image(), vec![provider(image())]));
    assert!(s.plan().relocations()[3].value.is_some());
    assert_eq!(s.relocations()[3], RelocationState::Pending);
}
#[test]
fn bounded_refusal_allocates_nothing() {
    for maximum in [0, 0x9ff] {
        let (r, o) = staging::stage(
            Arc::new(run(image(), vec![])),
            StagingLimits {
                max_mapped_bytes: maximum,
            },
        );
        assert!(matches!(r, Err(staging::StagingError::Budget { .. })));
        assert_eq!(o.active_mappings(), 0);
    }
}
#[test]
fn invalid_target_refuses_before_memory_creation() {
    let mut b = image();
    put64(&mut b, 0x540, 0xffff_ffff);
    let (r, o) = staging::stage(
        Arc::new(run(b, vec![])),
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    assert!(r.is_err());
    assert_eq!(o.active_mappings(), 0);
}
#[test]
fn malformed_segments_do_not_stage_a_valid_prefix() {
    let mut b = image();
    put64(&mut b, 104, 1);
    let (r, o) = staging::stage(
        Arc::new(run(b, vec![])),
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    assert!(r.is_err());
    assert_eq!(o.active_mappings(), 0);
}
#[test]
fn drop_releases_mapping_and_repeated_staging_is_deterministic() {
    let p = Arc::new(run(image(), vec![]));
    let (a, oa) = staging::stage(
        p.clone(),
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    let (b, ob) = staging::stage(
        p,
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    let a = a.unwrap();
    let b = b.unwrap();
    assert_eq!(a.snapshot(), b.snapshot());
    assert_eq!(
        a.read(VirtualAddress(0x11100), 0xa00).unwrap(),
        b.read(VirtualAddress(0x11100), 0xa00).unwrap()
    );
    drop(a);
    drop(b);
    assert_eq!(oa.active_mappings() + ob.active_mappings(), 0);
}
#[test]
fn blocker_classes_do_not_promote_execution_readiness() {
    use staging::{BlockerClass::*, classify_blocker as c};
    assert_eq!(c(&Blocker::EntryUnavailable), ExecutionBlocking);
    assert_eq!(c(&Blocker::Reference { symbol: 1 }), ResolutionBlocking);
    assert_eq!(
        c(&Blocker::SegmentOverlap {
            first: 0,
            second: 1
        }),
        StageBlocking
    );
    assert_eq!(
        c(&Blocker::ProgramSemantics {
            index: 0,
            kind: 0xdead
        }),
        StageBlocking
    );
}

#[test]
fn failure_after_copy_discards_all_backend_regions() {
    use astero_loader::load_plan::{
        MappingIntent,
        staging::{ProtectionEnforcement, StagingBackend},
    };
    use astero_memory::mapping::{GuestAddress, MemoryError, OwnedAddressSpace};
    struct Fail {
        m: OwnedAddressSpace,
        writes: usize,
    }
    impl StagingBackend for Fail {
        type Error = MemoryError;
        fn reserve_image(&mut self, _m: &[SegmentPlan]) -> Result<(), MemoryError> {
            Ok(())
        }
        fn map(&mut self, m: &MappingIntent) -> Result<(), MemoryError> {
            self.m
                .map_zeroed(GuestAddress(m.range.start.0), m.range.size)
        }
        fn write(&mut self, a: VirtualAddress, b: &[u8]) -> Result<(), MemoryError> {
            self.writes += 1;
            if self.writes > 1 {
                return Err(MemoryError::Finalized);
            }
            self.m.write(GuestAddress(a.0), b)
        }
        fn read(&self, a: VirtualAddress, n: u64) -> Result<&[u8], MemoryError> {
            self.m.read(GuestAddress(a.0), n)
        }
        fn finalize(&mut self, _m: &[SegmentPlan]) -> Result<ProtectionEnforcement, MemoryError> {
            Ok(ProtectionEnforcement::MetadataOnly)
        }
        fn clear(&mut self) {
            self.m.clear();
        }
    }
    let m = OwnedAddressSpace::new(65536);
    let o = m.observer();
    let r = astero_loader::load_plan::staging::stage(
        Arc::new(run(image(), vec![])),
        Fail { m, writes: 0 },
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    assert!(matches!(
        r,
        Err(staging::StagingError::Backend {
            operation: "relocation",
            ..
        })
    ));
    assert_eq!(o.active_mappings(), 0);
}
#[test]
fn bad_alignment_and_overlapping_relocations_are_stage_blockers() {
    let mut b = image();
    put64(&mut b, 112, 3);
    let (r, o) = staging::stage(
        Arc::new(run(b, vec![])),
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    assert!(r.is_err());
    assert_eq!(o.active_mappings(), 0);
    let mut b = image();
    put64(&mut b, 0x570, 0x1804);
    let (r, o) = staging::stage(
        Arc::new(run(b, vec![])),
        StagingLimits {
            max_mapped_bytes: 65536,
        },
    );
    assert!(r.is_err());
    assert_eq!(o.active_mappings(), 0);
}
