mod dynamic_fixtures;
use astero_loader::elf::{
    ElfInspection,
    dynamic::{
        ObservationLimits,
        candidates::{
            classification::Classification,
            error::CandidateError,
            report::{
                self, BudgetReason, Completeness, LinkageEvidenceReport, ReportBudget, ReportError,
                UnavailableReason,
            },
        },
        hash::HashLimits,
        relocations::{RelocationLimits, RelocationTables},
        symbol_table::{SymbolError, SymbolTable},
    },
};
use dynamic_fixtures::{image, parsed, program, put16, put32, put64};
fn dl() -> ObservationLimits {
    ObservationLimits { max_entries: 64 }
}
fn hl() -> HashLimits {
    HashLimits { max_words: 256 }
}
fn budget() -> ReportBudget {
    ReportBudget {
        max_symbols: 64,
        max_details: 64,
        max_retained_name_bytes: 1024,
        max_name_scan_bytes: 32,
        max_total_name_scan_bytes: 1024,
        relocations: RelocationLimits {
            max_entries: 64,
            max_name_scan_bytes: 32,
            max_total_name_scan_bytes: 1024,
        },
    }
}
fn evidence() -> ElfInspection {
    let mut b = fixture(8, &[0, 1, 1], &[1]);
    for (i, name, info, visibility, section) in [
        (1, 1, 0x12, 0, 0),
        (2, 1, 0x12, 0, 1),
        (3, 0, 2, 0, 1),
        (4, 6, 0xa2, 7, 0),
        (5, 8, 0x12, 0, 0),
        (6, 1, 0x1f, 0, 0),
        (7, 1, 0x12, 0, 0),
    ] {
        symbol(&mut b, i, name, info, visibility, section);
    }
    parsed(b)
}
fn collect(elf: &ElfInspection, b: ReportBudget) -> LinkageEvidenceReport {
    let s = SymbolTable::with_hash(elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(elf, dl()).unwrap();
    report::collect(elf, &s, &r, b)
}
fn fixture(count: u32, ordinary: &[u32], plt: &[u32]) -> Vec<u8> {
    let mut tags = vec![(6, 0x2000), (11, 24), (5, 0x2800), (10, 9), (4, 0x1600)];
    if !ordinary.is_empty() {
        tags.extend([(7, 0x1800), (8, ordinary.len() as u64 * 24), (9, 24)]);
    }
    if !plt.is_empty() {
        tags.extend([(23, 0x1c00), (2, plt.len() as u64 * 24), (20, 7)]);
    }
    tags.push((0, 0));
    let mut b = image(&tags);
    b.resize(0x2000, 0);
    program(&mut b, 0, 1, 0x100, 0x1100, 0x1f00, 0x2000);
    put32(&mut b, 0x600, 1);
    put32(&mut b, 0x604, count);
    b[0x1800..0x1809].copy_from_slice(b"\0name\0\xff\0\0");
    for (at, entries) in [(0x800, ordinary), (0xc00, plt)] {
        for (i, &symbol) in entries.iter().enumerate() {
            put64(&mut b, at + 24 * i, 0x1234 + i as u64);
            put64(
                &mut b,
                at + 24 * i + 8,
                (u64::from(symbol) << 32) | 0xffffeeee,
            );
            b[at + 24 * i + 16..at + 24 * i + 24].copy_from_slice(&(-17i64).to_le_bytes());
        }
    }
    b
}
fn symbol(b: &mut [u8], i: usize, name: u32, info: u8, other: u8, section: u16) {
    let at = 0x1000 + 24 * i;
    put32(b, at, name);
    b[at + 4] = info;
    b[at + 5] = other;
    put16(b, at + 6, section);
    put64(b, at + 8, 0x123456);
    put64(b, at + 16, 37);
}

#[test]
fn complete_owned_report_preserves_counts_details_provenance_and_duplicates() {
    let elf = evidence();
    let report = collect(&elf, budget());
    assert_eq!(report.completeness(), &Completeness::Complete);
    let c = report.counts();
    assert_eq!(
        (
            c.observed,
            c.imports,
            c.exports,
            c.internal,
            c.unclassified,
            c.null
        ),
        (8, 3, 1, 1, 2, 1)
    );
    assert_eq!((c.unnamed, c.empty_names, c.non_utf8_names), (2, 1, 1));
    assert_eq!(
        (c.unknown_binding, c.unknown_type, c.unknown_visibility),
        (1, 1, 1)
    );
    assert_eq!(
        (
            c.relocations.unique,
            c.relocations.ordinary,
            c.relocations.plt
        ),
        (4, 3, 1)
    );
    assert_eq!(report.source(), elf.artifact().identity());
    assert_eq!(report.module(), Some(astero_loader::modules::ModuleId(5)));
    assert_eq!(report.extent().unwrap().symbol_count(), 8);
    assert!(!report.extent().unwrap().evidence().is_empty());
    assert_eq!(report.details()[1].name, report.details()[7].name);
    assert_ne!(report.details()[1].index, report.details()[7].index);
    assert_eq!(report.details()[4].name, Some(vec![255]));
    assert_eq!(report.details()[0].classification, Classification::Null);
    assert_eq!(report, collect(&elf, budget()));
    drop(elf);
    assert_eq!(
        report.details()[1].name.as_deref(),
        Some(b"name".as_slice())
    );
}
#[test]
fn symbol_detail_and_name_limits_preserve_coherent_prefixes() {
    let elf = evidence();
    for maximum in 0..=9 {
        let mut b = budget();
        b.max_symbols = maximum;
        let r = collect(&elf, b);
        assert_eq!(r.counts().observed, maximum.min(8));
        assert_eq!(r.details().len() as u64, r.counts().observed);
        assert_eq!(
            matches!(r.completeness(), Completeness::Complete),
            maximum >= 8
        );
    }
    let mut b = budget();
    b.max_details = 2;
    let r = collect(&elf, b);
    assert!(matches!(
        r.completeness(),
        Completeness::Partial {
            observed: 2,
            remaining: Some(6),
            reason: BudgetReason::DetailedRecords
        }
    ));
    assert_eq!(r.counts().imports, 1);
    b = budget();
    b.max_retained_name_bytes = 3;
    let r = collect(&elf, b);
    assert!(matches!(
        r.completeness(),
        Completeness::Partial {
            observed: 1,
            reason: BudgetReason::RetainedNameBytes,
            ..
        }
    ));
    assert_eq!(r.details().len(), 1);
}
#[test]
fn relocation_budget_is_partial_without_unvalidated_associations() {
    let elf = evidence();
    let mut b = budget();
    b.relocations.max_entries = 3;
    let r = collect(&elf, b);
    assert!(matches!(
        r.completeness(),
        Completeness::Partial {
            observed: 0,
            reason: BudgetReason::Evidence(CandidateError::Relocation(_)),
            ..
        }
    ));
    assert_eq!(r.counts().relocations.unique, 0);
    assert!(r.details().is_empty());
    b = budget();
    b.relocations.max_total_name_scan_bytes = 4;
    assert!(matches!(
        collect(&elf, b).completeness(),
        Completeness::Partial {
            reason: BudgetReason::Evidence(_),
            ..
        }
    ));
}
#[test]
fn missing_extent_and_mismatched_evidence_remain_distinct() {
    let elf = evidence();
    let s = SymbolTable::new(&elf, dl()).unwrap();
    let r = RelocationTables::new(&elf, dl()).unwrap();
    let report = report::collect(&elf, &s, &r, budget());
    assert_eq!(
        report.completeness(),
        &Completeness::Unavailable(UnavailableReason::TrustedExtentMissing)
    );
    assert!(report.extent().is_none());
    let other = evidence();
    let s = SymbolTable::with_hash(&other, dl(), hl()).unwrap();
    assert!(matches!(
        report::collect(&elf, &s, &r, budget()).completeness(),
        Completeness::Failed(ReportError::SourceMismatch { .. })
    ));
    let s = SymbolTable::with_hash(&elf, dl(), hl()).unwrap();
    let r = RelocationTables::new(&other, dl()).unwrap();
    assert!(matches!(
        report::collect(&elf, &s, &r, budget()).completeness(),
        Completeness::Failed(ReportError::Candidate(
            CandidateError::SourceMismatch { .. }
        ))
    ));
}
#[test]
fn name_failure_keeps_structured_error_and_only_successful_prefix() {
    let mut b = fixture(3, &[], &[]);
    symbol(&mut b, 1, 1, 0x12, 0, 0);
    symbol(&mut b, 2, 9, 0x12, 0, 0);
    let elf = parsed(b);
    let r = collect(&elf, budget());
    assert_eq!(r.counts().observed, 2);
    assert_eq!(r.details().len(), 2);
    assert!(matches!(
        r.completeness(),
        Completeness::Failed(ReportError::Candidate(CandidateError::Symbol(
            SymbolError::Name { index: 2, .. }
        )))
    ));
    let mut b = budget();
    b.max_total_name_scan_bytes = 4;
    assert!(matches!(
        collect(&elf, b).completeness(),
        Completeness::Partial {
            observed: 1,
            reason: BudgetReason::Evidence(_),
            ..
        }
    ));
}
