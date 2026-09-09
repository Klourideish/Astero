//! Symbol-only structural classification, distinct from M10 linkage eligibility.
mod classify;
mod model;
pub use classify::classify;
pub use model::{
    CandidateRole, ClassificationFailure, ClassificationLimits, ClassificationOutcome,
    ClassificationRecord, SymbolClassificationReport,
};
