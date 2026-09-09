use crate::{
    artifact::SourceId,
    elf::dynamic::{
        candidates::error::CandidateError, relocations::error::RelocationError,
        string_table::StringTableError, symbol_table::SymbolError,
    },
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportError {
    UnexpectedEnd {
        expected: u64,
        observed: u64,
    },
    SourceMismatch {
        report: SourceId,
        evidence: SourceId,
    },
    Candidate(CandidateError),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BudgetReason {
    Symbols,
    DetailedRecords,
    RetainedNameBytes,
    Evidence(CandidateError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnavailableReason {
    TrustedExtentMissing,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Completeness {
    Complete,
    Partial {
        observed: u64,
        remaining: Option<u64>,
        reason: BudgetReason,
    },
    Unavailable(UnavailableReason),
    Failed(ReportError),
}
pub(super) fn evidence_status(error: CandidateError, count: u64, observed: u64) -> Completeness {
    let limited = matches!(
        &error,
        CandidateError::Symbol(
            SymbolError::EnumerationLimit { .. }
                | SymbolError::Name {
                    error: StringTableError::ScanLimit { .. },
                    ..
                }
        ) | CandidateError::Relocation(
            RelocationError::EntryBudget { .. }
                | RelocationError::SymbolObservation {
                    error: SymbolError::Name {
                        error: StringTableError::ScanLimit { .. },
                        ..
                    },
                    ..
                }
        )
    );
    if limited {
        Completeness::Partial {
            observed,
            remaining: Some(count - observed),
            reason: BudgetReason::Evidence(error),
        }
    } else {
        Completeness::Failed(ReportError::Candidate(error))
    }
}
