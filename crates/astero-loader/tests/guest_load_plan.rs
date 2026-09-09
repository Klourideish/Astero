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
            SourceArtifact::new(b, Some("M26 synthetic".into())).unwrap(),
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
#[test]
fn mapping_copy_zero_fill_permissions_and_entry_preserve_proof() {
    let p = run(image(), vec![]);
    let s = &p.segments()[0];
    assert_eq!(s.mapping.range.start.0, 0x11100);
    assert_eq!(s.mapping.zero_fill.unwrap().size, 0x100);
    assert!(s.mapping.permissions.execute && s.mapping.permissions.write);
    assert_eq!(p.entry(), Some(VirtualAddress(0x11100)));
    assert_eq!(
        p.input()
            .linkage()
            .source()
            .read(&s.mapping.copy.as_ref().unwrap().source)
            .unwrap()
            .len(),
        0x900
    );
}
#[test]
fn exact_context_and_nid_select_provider_without_runtime_binding() {
    let p = run(image(), vec![provider(image())]);
    assert!(
        p.references()
            .iter()
            .any(|r| matches!(r.resolution, PlannedResolution::Selected(_)))
    );
    assert_eq!(p.readiness(), Readiness::Blocked);
    assert!(
        p.blockers()
            .iter()
            .any(|b| matches!(b, Blocker::ProviderRequiresOwnLoadPlan { .. }))
    );
    assert_eq!(p.relocations()[3].value, Some(0x21120));
}
#[test]
fn same_nid_wrong_library_never_matches() {
    let mut b = image();
    b[0x928] = b'X';
    let p = run(image(), vec![provider(b)]);
    assert!(
        p.references()
            .iter()
            .all(|r| !matches!(r.resolution, PlannedResolution::Selected(_)))
    );
}
#[test]
fn two_compatible_providers_are_ambiguous_not_first_wins() {
    let p = run(image(), vec![provider(image()), provider(image())]);
    assert!(
        p.references()
            .iter()
            .any(|r| matches!(&r.resolution,PlannedResolution::Ambiguous(v) if v.len()==2))
    );
    assert!(p.dependencies().iter().all(|d| d.supplied.len() == 2));
}
#[test]
fn missing_alias_does_not_guess_filename_or_strip_prx() {
    let mut pr = provider(image());
    pr.dependency_alias = None;
    let p = run(image(), vec![pr]);
    assert!(p.dependencies().iter().all(|d| d.supplied.is_empty()));
}
#[test]
fn known_catalogue_nid_stays_unimplemented_without_provider() {
    let p = run(image(), vec![]);
    let r = p
        .references()
        .iter()
        .find(|r| r.nid == Some(0x1f67bcb7949c4067))
        .unwrap();
    assert_eq!(r.resolution, PlannedResolution::Unresolved);
    let known = catalogue_correlation(r.nid.unwrap()).unwrap();
    assert_eq!(known.name, "__cxa_finalize");
    assert!(!known.astero_registered);
}
#[test]
fn unknown_relocation_and_raw_addend_are_retained() {
    let p = run(image(), vec![]);
    assert_eq!(p.relocations()[1].action, Action::Unsupported(0xffffeeee));
    assert_eq!(p.relocations()[1].raw.record.addend, 19);
    assert_eq!(p.relocations()[1].value, None);
}
#[test]
fn relative_absolute_pc32_and_symbol_writes_have_checked_formulas() {
    for (kind, expected) in [(1, 0x11238), (2, 0xfffffa28), (6, 0x11234), (7, 0x11234)] {
        let mut b = image();
        put64(&mut b, 0x578, (2 << 32) | kind);
        let p = run(b, vec![]);
        assert_eq!(p.relocations()[2].value, Some(expected), "kind {kind}");
        assert_eq!(p.relocations()[0].value, Some(0xfff0));
    }
}
#[test]
fn relative_nonzero_symbol_and_overflow_are_blocked() {
    let mut b = image();
    put64(&mut b, 0x548, (1 << 32) | 8);
    let p = run(b, vec![]);
    assert!(p.blockers().iter().any(|b| matches!(
        b,
        Blocker::Relocation {
            reason: RelocationProblem::NonzeroRelativeSymbol,
            ..
        }
    )));
    let mut b = image();
    put64(&mut b, 0x580, i64::MAX as u64);
    put64(&mut b, 0x578, (2 << 32) | 2);
    let p = run(b, vec![]);
    assert!(p.blockers().iter().any(|b| matches!(
        b,
        Blocker::Relocation {
            reason: RelocationProblem::Arithmetic,
            ..
        }
    )));
}
#[test]
fn patch_outside_image_and_overlapping_patch_are_blocked() {
    let mut b = image();
    put64(&mut b, 0x540, u64::MAX);
    let p = run(b, vec![]);
    assert!(p.blockers().iter().any(|b| matches!(
        b,
        Blocker::Relocation {
            reason: RelocationProblem::TargetOutsideImage,
            ..
        }
    )));
    let mut b = image();
    put64(&mut b, 0x570, 0x1804);
    let p = run(b, vec![]);
    assert!(p.blockers().iter().any(|b| matches!(
        b,
        Blocker::Relocation {
            reason: RelocationProblem::Overlap,
            ..
        }
    )));
}
#[test]
fn malformed_segment_and_entry_block_readiness() {
    let mut b = image();
    put64(&mut b, 24, 0x5000);
    put64(&mut b, 112, 3);
    let p = run(b, vec![]);
    assert_eq!(p.readiness(), Readiness::Blocked);
    assert!(p.blockers().iter().any(|b| matches!(
        b,
        Blocker::Segment {
            reason: SegmentProblem::Alignment,
            ..
        }
    )));
}
#[test]
fn budgets_refuse_whole_plan_at_zero_and_exact_boundary_succeeds() {
    let input = report(image());
    let mut exact = None;
    for n in 0..128 {
        match plan(
            input.clone(),
            vec![],
            VirtualAddress(0x10000),
            PlanningLimits {
                max_providers: 0,
                max_plan_records: n,
            },
        ) {
            Ok(_) => {
                exact = Some(n);
                break;
            }
            Err(PlanningError::Budget { .. }) => (),
            e => panic!("{e:?}"),
        }
    }
    let n = exact.unwrap();
    assert!(n > 0);
    assert!(matches!(
        plan(
            input,
            vec![provider(image())],
            VirtualAddress(0),
            PlanningLimits {
                max_providers: 0,
                max_plan_records: 256
            }
        ),
        Err(PlanningError::ProviderBudget { .. })
    ));
}
#[test]
fn repeated_planning_retains_input_and_deterministic_order() {
    let r = report(image());
    let bytes = r
        .linkage()
        .source()
        .read(
            &r.linkage()
                .source()
                .checked_range(0, r.linkage().source().len())
                .unwrap(),
        )
        .unwrap()
        .to_vec();
    let build = || {
        plan(
            r.clone(),
            vec![],
            VirtualAddress(0x10000),
            PlanningLimits {
                max_providers: 0,
                max_plan_records: 256,
            },
        )
        .unwrap()
    };
    let a = build();
    let b = build();
    assert!(Arc::ptr_eq(a.input(), &r));
    assert_eq!(a.relocations(), b.relocations());
    assert_eq!(a.segments(), b.segments());
    assert_eq!(
        r.linkage()
            .source()
            .read(
                &r.linkage()
                    .source()
                    .checked_range(0, r.linkage().source().len())
                    .unwrap()
            )
            .unwrap(),
        bytes
    );
}
#[test]
fn failed_linkage_still_preserves_segment_and_entry_evidence() {
    let mut b = image();
    put64(&mut b, 0x608, u64::MAX);
    let p = run(b, vec![]);
    assert_eq!(p.segments().len(), 1);
    assert!(p.blockers().contains(&Blocker::EvidenceUnavailable));
    assert!(p.references().is_empty());
}
#[test]
fn local_hidden_wrong_type_and_version_providers_are_not_selected() {
    for variant in 0..4 {
        let mut pr = provider(image());
        let mut b = image();
        put64(&mut b, 0x6f0, 0x61000047);
        put16(&mut b, 0x41e, 1);
        put64(&mut b, 0x420, 0x1120);
        match variant {
            0 => b[0x41c] = 2,
            1 => b[0x41d] = 2,
            2 => b[0x41c] = 0x11,
            _ => put64(&mut b, 0x6f8, (1 << 48) | (2 << 32) | 40),
        }
        pr.evidence = report(b);
        let p = run(image(), vec![pr]);
        assert!(
            p.references()
                .iter()
                .all(|r| !matches!(r.resolution, PlannedResolution::Selected(_))),
            "variant {variant}"
        );
    }
}

#[test]
fn ready_is_only_for_closed_supported_image_and_unknown_identity_is_experimental() {
    let mut b = image();
    for i in 0..4 {
        put64(&mut b, 0x548 + i * 24, 0);
    }
    for at in [0x6b0, 0x6c0] {
        put64(&mut b, at, (-42i64) as u64);
    }
    let p = run(b.clone(), vec![]);
    assert_eq!(p.readiness(), Readiness::Experimental);
    for at in [0x6e0, 0x6f0, 0x700] {
        put64(&mut b, at, (-42i64) as u64);
    }
    let p = run(b, vec![]);
    assert_eq!(p.readiness(), Readiness::ReadyForLoad);
    assert!(p.blockers().is_empty());
}

#[test]
fn empty_tls_is_observable_without_an_unnecessary_load_blocker() {
    let mut b = image();
    put16(&mut b, 56, 3);
    astero_loader::elf::dynamic::synthetic::program(&mut b, 2, 7, 0, 0, 0, 0);
    let mut l = limits();
    l.hash.dynamic.max_program_headers = 3;
    let r = Arc::new(identity::observe(
        Arc::new(workload::observe(SourceArtifact::new(b, None).unwrap(), l)),
        IdentityLimits {
            max_identity_records: 16,
        },
    ));
    let p = plan(
        r,
        vec![],
        VirtualAddress(0x10000),
        PlanningLimits {
            max_providers: 0,
            max_plan_records: 256,
        },
    )
    .unwrap();
    assert!(
        !p.blockers()
            .iter()
            .any(|b| matches!(b, Blocker::ProgramSemantics { kind: 7, .. }))
    );
    assert_eq!(p.segments().len(), 1);
}

#[test]
fn local_tls_symbol_value_is_not_an_image_address() {
    let mut b = image();
    b[0x434] = 0x26;
    let p = run(b, vec![]);
    assert_eq!(p.relocations()[2].value, None);
    assert!(p.blockers().iter().any(|b| matches!(
        b,
        Blocker::Relocation {
            index: 2,
            reason: RelocationProblem::SymbolUnavailable
        }
    )));
}
