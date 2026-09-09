//! Explicit budgeted header evidence only. No admission, dynamic inspection or linkage.
mod collect;
mod model;
pub use collect::inspect;
pub use model::{
    HeaderEvidence, InspectionFailure, InspectionLimits, InspectionOutcome, InspectionReport,
};
