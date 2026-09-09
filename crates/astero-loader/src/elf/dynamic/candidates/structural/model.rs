use crate::elf::dynamic::symbol_table::bounded::{
    SymbolObservationReport, SymbolOutcome, SymbolRecord,
};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateRole {
    NullSymbol,
    UndefinedCandidate,
    DefinitionCandidate,
    SpecialCandidate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassificationLimits {
    pub max_classifications: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassificationRecord {
    pub(super) symbol_index: u64,
    pub(super) role: CandidateRole,
}
impl ClassificationRecord {
    pub fn symbol_index(&self) -> u64 {
        self.symbol_index
    }
    pub fn role(&self) -> CandidateRole {
        self.role
    }
}
#[derive(Debug)]
pub enum ClassificationFailure {
    PrerequisiteFailed,
    Budget { count: u64, maximum: u64 },
    Allocation { count: u64 },
}
impl std::fmt::Display for ClassificationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "structural classification: {self:?}")
    }
}
impl std::error::Error for ClassificationFailure {}
#[derive(Debug)]
pub enum ClassificationOutcome {
    Complete(Vec<ClassificationRecord>),
    Unavailable,
    Failed(ClassificationFailure),
}
/// Immutable roles plus the original complete observation; no copied names, fields or proof.
/// ```compile_fail
/// use astero_loader::elf::dynamic::candidates::structural::{SymbolClassificationReport,ClassificationOutcome};
/// fn replace(r:&mut SymbolClassificationReport){r.outcome=ClassificationOutcome::Unavailable;}
/// ```
#[derive(Debug)]
pub struct SymbolClassificationReport {
    pub(super) input: Arc<SymbolObservationReport>,
    pub(super) limits: ClassificationLimits,
    pub(super) outcome: ClassificationOutcome,
}
impl SymbolClassificationReport {
    pub fn input(&self) -> &Arc<SymbolObservationReport> {
        &self.input
    }
    pub fn limits(&self) -> ClassificationLimits {
        self.limits
    }
    pub fn outcome(&self) -> &ClassificationOutcome {
        &self.outcome
    }
    /// Complete-only ordered views, paired with the original immutable M21 record.
    pub fn entries(&self) -> impl Iterator<Item = (&ClassificationRecord, &SymbolRecord)> {
        let (roles, symbols): (&[ClassificationRecord], &[SymbolRecord]) =
            match (&self.outcome, self.input.outcome()) {
                (ClassificationOutcome::Complete(r), SymbolOutcome::Complete(s)) => (r, s),
                _ => (&[], &[]),
            };
        roles.iter().zip(symbols)
    }
}
