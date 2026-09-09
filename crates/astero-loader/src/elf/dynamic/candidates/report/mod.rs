//! Owned, bounded linkage observations; completeness never implies resolution.
mod collect;
mod model;
mod status;
pub use collect::collect;
pub use model::{
    CandidateDetail, LinkageEvidenceReport, RelocationCounts, ReportBudget, ReportCounts,
};
pub use status::{BudgetReason, Completeness, ReportError, UnavailableReason};
pub mod synthetic;
