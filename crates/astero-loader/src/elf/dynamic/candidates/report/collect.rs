use super::{status::evidence_status, *};
use crate::elf::{
    ElfInspection,
    dynamic::{
        candidates::{
            self, CandidateLimits, classification::Classification as C, evidence::NameState,
        },
        relocations::RelocationTables,
        symbol_table::{Binding, EnumerationLimits, SymbolTable, SymbolType, Visibility},
    },
};
/// No classification is duplicated here: all rows originate in M10's trusted enumeration.
pub fn collect(
    elf: &ElfInspection,
    symbols: &SymbolTable<'_>,
    relocations: &RelocationTables,
    budget: ReportBudget,
) -> LinkageEvidenceReport {
    let mut report = LinkageEvidenceReport {
        source: elf.artifact().identity(),
        module: elf.artifact().description().module.as_ref().map(|m| m.id),
        role: elf.artifact().description().role,
        extent: None,
        completeness: Completeness::Unavailable(UnavailableReason::TrustedExtentMissing),
        counts: ReportCounts::default(),
        details: Vec::new(),
    };
    let Some(extent) = symbols.extent() else {
        return report;
    };
    if extent.source_range().source_id() != report.source {
        report.completeness = Completeness::Failed(ReportError::SourceMismatch {
            report: report.source,
            evidence: extent.source_range().source_id(),
        });
        return report;
    }
    report.extent = Some(extent.clone());
    let count = extent.symbol_count();
    // M10 preflights the entire relocation inventory. A budget failure grants no associations.
    // Symbol work is bounded by the loop below, not by untrusted allocation or count truncation.
    let limits = CandidateLimits {
        symbols: EnumerationLimits {
            max_symbols: count,
            max_name_scan_bytes: budget.max_name_scan_bytes,
            max_total_name_scan_bytes: budget.max_total_name_scan_bytes,
        },
        relocations: budget.relocations,
    };
    let mut rows = match candidates::enumerate(symbols, relocations, limits) {
        Ok(rows) => rows,
        Err(e) => {
            report.completeness = evidence_status(e, count, 0);
            return report;
        }
    };
    let mut remaining_names = budget.max_retained_name_bytes;
    while report.counts.observed < count {
        let observed = report.counts.observed;
        let reason = if observed >= budget.max_symbols {
            Some(BudgetReason::Symbols)
        } else if observed >= budget.max_details {
            Some(BudgetReason::DetailedRecords)
        } else {
            None
        };
        if let Some(reason) = reason {
            report.completeness = Completeness::Partial {
                observed,
                remaining: Some(count - observed),
                reason,
            };
            return report;
        }
        let row = match rows.next() {
            Some(Ok(row)) => row,
            Some(Err(e)) => {
                report.completeness = evidence_status(e, count, observed);
                return report;
            }
            None => {
                report.completeness = Completeness::Failed(ReportError::UnexpectedEnd {
                    expected: count,
                    observed,
                });
                return report;
            }
        };
        let e = row.evidence();
        let s = e.symbol();
        let length = s.name.map_or(0, |n| n.as_bytes().len() as u64);
        if length > remaining_names {
            report.completeness = Completeness::Partial {
                observed,
                remaining: Some(count - observed),
                reason: BudgetReason::RetainedNameBytes,
            };
            return report;
        }
        remaining_names -= length;
        let refs = RelocationCounts {
            unique: e.relocations().len() as u64,
            ordinary: e
                .relocations()
                .iter()
                .filter(|r| r.dynamic_index.is_some())
                .count() as u64,
            plt: e
                .relocations()
                .iter()
                .filter(|r| r.plt_index.is_some())
                .count() as u64,
        };
        let c = &mut report.counts;
        c.observed += 1;
        match row.classification() {
            C::ImportCandidate => c.imports += 1,
            C::ExportCandidate => c.exports += 1,
            C::InternalDefined(_) => c.internal += 1,
            C::UndefinedUnclassified(_) => {
                c.unclassified += 1;
                c.undefined_unclassified += 1;
            }
            C::Unclassified(_) => c.unclassified += 1,
            C::Null => c.null += 1,
            C::Absolute => c.absolute += 1,
            C::Common => c.common += 1,
        }
        match e.name_state() {
            NameState::Absent => c.unnamed += 1,
            NameState::Empty => c.empty_names += 1,
            NameState::Bytes => c.non_utf8_names += 1,
            NameState::Utf8 => {}
        }
        c.unknown_binding += u64::from(matches!(s.binding, Binding::Unknown(_)));
        c.unknown_type += u64::from(matches!(s.symbol_type, SymbolType::Unknown(_)));
        c.unknown_visibility += u64::from(matches!(s.visibility, Visibility::Unknown(_)));
        c.relocations.unique += refs.unique;
        c.relocations.ordinary += refs.ordinary;
        c.relocations.plt += refs.plt;
        report.details.push(CandidateDetail {
            index: s.index,
            source: s.source,
            classification: row.classification(),
            name_state: e.name_state(),
            name: s.name.map(|n| n.as_bytes().to_vec()),
            name_source: s.name.map(|n| n.source_range()),
            name_offset: s.name_offset,
            info: s.info,
            other: s.other,
            binding: s.binding,
            symbol_type: s.symbol_type,
            visibility: s.visibility,
            section: s.section,
            value: s.value,
            size: s.size,
            relocations: refs,
        });
    }
    report.completeness = Completeness::Complete;
    report
}
