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

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn native_realization_preserves_staged_bytes_plan_and_pending_writes() {
    use astero_core::input::native;
    let mut b = image();
    put32(&mut b, 68, 6);
    let bias = 0x900000000 + (std::process::id() as u64) * 0x100000;
    let p = plan(
        report(b),
        vec![],
        VirtualAddress(bias),
        PlanningLimits {
            max_providers: 1,
            max_plan_records: 256,
        },
    )
    .unwrap();
    let (mut staged, bo) = staged(p);
    let before = staged.snapshot();
    let base = staged.plan().segments()[0].mapping.range.start;
    let (r, o) = native::realize(
        &staged,
        native::NativeLimits {
            max_reserved_bytes: 65536,
            max_committed_bytes: 65536,
        },
    );
    let mut image = r.unwrap();
    assert!(Arc::ptr_eq(image.plan(), staged.plan()));
    assert_eq!(image.staging_snapshot(), before);
    assert_eq!(
        image.read(base.0, 0xa00).unwrap(),
        staged.read(base, 0xa00).unwrap()
    );
    assert_eq!(
        image.state(),
        native::NativeState::NativeBackedWithPendingWork
    );
    image.release().unwrap();
    assert_eq!(image.state(), native::NativeState::Released);
    assert_eq!(o.active_reservations(), 0);
    staged.release();
    assert_eq!(bo.active_mappings(), 0);
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn released_staging_cannot_be_realized() {
    use astero_core::input::native;
    let (mut s, _) = staged(run(image(), vec![]));
    s.release();
    let (r, o) = native::realize(
        &s,
        native::NativeLimits {
            max_reserved_bytes: 65536,
            max_committed_bytes: 65536,
        },
    );
    assert!(matches!(r, Err(native::NativeError::Released)));
    assert_eq!(o.active_reservations(), 0);
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn entry_fixture(
    offset: u64,
) -> (
    astero_core::input::native::NativeBackedGuestImage,
    astero_core::input::native::NativeObserver,
) {
    use astero_core::input::native;
    let mut b = image();
    put32(&mut b, 68, 6);
    let bias = 0x1100000000 + (std::process::id() as u64) * 0x100000 + offset;
    let p = plan(
        report(b),
        vec![],
        VirtualAddress(bias),
        PlanningLimits {
            max_providers: 1,
            max_plan_records: 256,
        },
    )
    .unwrap();
    let (s, _) = staged(p);
    let (r, o) = native::realize(
        &s,
        native::NativeLimits {
            max_reserved_bytes: 65536,
            max_committed_bytes: 65536,
        },
    );
    (r.unwrap(), o)
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn entry_owner_preserves_plan_and_releases_every_reservation() {
    use astero_core::input::entry::*;
    let (image, o) = entry_fixture(0);
    let plan = image.plan().clone();
    let bias = (std::process::id() as u64) * 0x100000;
    let (mut g, obs) = prepare(
        image,
        RuntimeLimits {
            stack_base: 0x1300000000 + bias,
            stack_bytes: 8192,
            tls_base: 0x1400000000 + bias,
            max_runtime_bytes: 16384,
        },
        PreparedRegistry::new(vec![], 0).unwrap(),
        None,
    )
    .unwrap();
    assert!(Arc::ptr_eq(g.image().plan(), &plan));
    assert!(!g.entry_ready());
    assert!(!g.has_timing());
    assert!(g.blockers().contains(&EntryBlocker::RecoveryAdapterMissing));
    assert!(
        g.blockers()
            .iter()
            .any(|b| matches!(b, EntryBlocker::PendingRelocations { .. }))
    );
    assert_eq!(g.context().rsp % 16, 8);
    g.release().unwrap();
    assert_eq!(g.state(), EntryState::Released);
    assert_eq!(o.active_reservations(), 0);
    assert!(obs.iter().all(|o| o.active_reservations() == 0));
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn preparation_refusal_drops_consumed_image() {
    use astero_core::input::entry::*;
    let (image, o) = entry_fixture(0x10000);
    assert!(
        prepare(
            image,
            RuntimeLimits {
                stack_base: 0x1500000000,
                stack_bytes: 8192,
                tls_base: 0x1600000000,
                max_runtime_bytes: 1
            },
            PreparedRegistry::new(vec![], 0).unwrap(),
            None
        )
        .is_err()
    );
    assert_eq!(o.active_reservations(), 0);
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn released_native_image_cannot_prepare() {
    use astero_core::input::entry::*;
    let (mut image, _) = entry_fixture(0x20000);
    image.release().unwrap();
    assert!(matches!(
        prepare(
            image,
            RuntimeLimits {
                stack_base: 0,
                stack_bytes: 0,
                tls_base: 0,
                max_runtime_bytes: 0
            },
            PreparedRegistry::new(vec![], 0).unwrap(),
            None
        ),
        Err(EntryError::Released)
    ));
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn closure_preserves_blocked_authority_and_accepts_only_closed_synthetic_image() {
    use astero_core::input::{entry::*, native};
    for closed in [false, true] {
        let mut b = image();
        put32(&mut b, 68, 5);
        // No write targets in the ready fixture. Original symbols and identity remain.
        if closed {
            put64(&mut b, 0x668, 0);
            put64(&mut b, 0x698, 0);
        } else {
            // Unsupported relocation is retained without attempting a write into RX memory.
            for i in 0..4 {
                put64(&mut b, 0x548 + i * 24, 0xffffeeee);
            }
        }
        let p = plan(
            report(b),
            vec![],
            VirtualAddress(0x2400000000),
            PlanningLimits {
                max_providers: 1,
                max_plan_records: 256,
            },
        )
        .unwrap();
        let (s, _) = staged(p);
        let (n, observer) = native::realize(
            &s,
            native::NativeLimits {
                max_reserved_bytes: 65536,
                max_committed_bytes: 65536,
            },
        );
        let (g, storage) = prepare(
            n.unwrap(),
            RuntimeLimits {
                stack_base: 0x2500000000,
                stack_bytes: 8192,
                tls_base: 0x2600000000,
                max_runtime_bytes: 20480,
            },
            PreparedRegistry::new(vec![], 0).unwrap(),
            None,
        )
        .unwrap();
        let source = g.image().plan().headers().source().identity();
        let result = close_entry(g, 20480, StartupPolicy::ExperimentalEntryOwnedInit).unwrap();
        assert_eq!(
            result
                .prepared()
                .image()
                .plan()
                .headers()
                .source()
                .identity(),
            source
        );
        assert!(result.bridge_validated());
        assert!(result.landing_mapping_active());
        assert_eq!(result.entry_ready(), closed);
        if closed {
            assert!(result.try_ready().is_ok());
        } else {
            assert!(result.try_ready().is_err());
        }
        assert_eq!(observer.active_reservations(), 0);
        assert!(storage.iter().all(|s| s.active_reservations() == 0));
    }
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn signed_object_trap_geometry_refuses_overflow_and_preserves_addends() {
    use astero_core::input::entry::trap_geometry;
    let (s, n) = trap_geometry(0x10000, -16, 4000, 4096).unwrap();
    assert_eq!(n, 4096);
    assert_eq!(s, 0x10010);
    assert_eq!(s - 16, 0x10000);
    assert!(s + 4000 < 0x11000);
    assert!(trap_geometry(u64::MAX - 10, 0, 4096, 4096).is_none());
    assert!(trap_geometry(0, i64::MIN, i64::MAX, 4096).is_none());
    assert!(trap_geometry(0, 1, 2, 4096).is_none());
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn first_entry_requires_factory_to_produce_ready_authority() {
    use astero_core::input::entry::execute_first_entry;
    assert!(
        execute_first_entry(|| Err("not ready".into()), 25)
            .unwrap_err()
            .contains("not ready")
    );
    assert!(execute_first_entry(|| panic!("must not construct"), 0).is_err());
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn worker_containment_distinguishes_completion_crash_and_hang() {
    use astero_core::input::entry::{ContainmentExit, contain_worker};
    use std::process::{Command, Stdio};
    for (code, expected) in [
        (0, ContainmentExit::Clean),
        (7, ContainmentExit::WorkerFailure(Some(7))),
    ] {
        let c = Command::new("cmd.exe")
            .args(["/d", "/c", &format!("exit {code}")])
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        assert_eq!(contain_worker(c, 1000).unwrap(), expected);
    }
    let c = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "Start-Sleep -Seconds 30",
        ])
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    assert_eq!(
        contain_worker(c, 30).unwrap(),
        ContainmentExit::TimeoutKilled
    );
}
