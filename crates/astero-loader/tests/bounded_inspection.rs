#[allow(dead_code)]
mod elf_fixtures;
use astero_loader::{
    artifact::SourceArtifact,
    elf::{
        self,
        inspect::bounded::{InspectionFailure, InspectionLimits, InspectionOutcome, inspect},
    },
};
use elf_fixtures::*;

#[test]
fn count_budget_sweep_refuses_before_observing_any_prefix() {
    for count in 0..7 {
        for maximum in 0..8 {
            let source = source(header(count));
            let identity = source.identity();
            let report = inspect(
                source,
                InspectionLimits {
                    max_program_headers: maximum,
                },
            );
            assert_eq!(report.source().identity(), identity);
            assert_eq!(report.limits().max_program_headers, maximum);
            if u64::from(count) > maximum {
                assert!(
                    matches!(report.outcome(),InspectionOutcome::Failed(InspectionFailure::HeaderBudget{declared,maximum:m}) if *declared==count && *m==maximum)
                );
            } else {
                let InspectionOutcome::Complete(e) = report.outcome() else {
                    panic!("expected complete")
                };
                assert_eq!(e.program_headers().len(), usize::from(count));
            }
        }
    }
}
#[test]
fn existing_decoders_supply_identical_evidence_and_source_stays_immutable() {
    let source = source(multiple());
    let limits = InspectionLimits {
        max_program_headers: 3,
    };
    let token = source.checked_range(0, source.len()).unwrap();
    let before = source.read(&token).unwrap().to_vec();
    let a = inspect(source.clone(), limits);
    let b = inspect(source.clone(), limits);
    let old = elf::inspect(source.clone(), context()).unwrap();
    let (InspectionOutcome::Complete(ae), InspectionOutcome::Complete(be)) =
        (a.outcome(), b.outcome())
    else {
        panic!("expected complete")
    };
    assert_eq!(ae, be);
    assert_eq!(ae.header(), old.header());
    assert_eq!(
        ae.program_headers(),
        old.program_headers()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    );
    assert_eq!(a.source().identity(), source.identity());
    assert_eq!(a.source().provenance(), source.provenance());
    assert_eq!(source.read(&token).unwrap(), before);
    assert_eq!(
        a.source().read(&token).unwrap().as_ptr(),
        source.read(&token).unwrap().as_ptr()
    );
}
#[test]
fn malformed_and_unsupported_input_preserves_existing_structured_errors() {
    let mut invalid = header(1);
    invalid.truncate(65);
    let mut unsupported = header(0);
    unsupported[4] = 1;
    for bytes in [vec![1, 2, 3], vec![0; 64], invalid, unsupported] {
        let source = source(bytes);
        let expected = elf::inspect(source.clone(), None).unwrap_err();
        let report = inspect(
            source.clone(),
            InspectionLimits {
                max_program_headers: 0,
            },
        );
        assert!(
            matches!(report.outcome(),InspectionOutcome::Failed(InspectionFailure::Header(e)) if *e==expected)
        );
        assert_eq!(report.source().identity(), source.identity());
    }
}
#[test]
fn dynamic_and_section_pointers_remain_uninterpreted_and_no_admission_occurs() {
    let mut bytes = header(1);
    put32(&mut bytes, 64, 2); // PT_DYNAMIC, but no dynamic payload
    put64(&mut bytes, 72, u64::MAX);
    put64(&mut bytes, 96, u64::MAX);
    put64(&mut bytes, 40, u64::MAX); // sections are not followed either
    let report = inspect(
        SourceArtifact::new(bytes, None).unwrap(),
        InspectionLimits {
            max_program_headers: 1,
        },
    );
    let InspectionOutcome::Complete(e) = report.outcome() else {
        panic!("only headers requested")
    };
    assert_eq!(e.program_headers()[0].kind, 2);
    assert_eq!(e.program_headers()[0].file_offset, u64::MAX);
    assert_eq!(e.header().section_offset, u64::MAX);
}
