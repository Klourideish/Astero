//! Explicit DT_NEEDED byte evidence only; no whole-table enumeration or dependency declarations.
mod collect;
mod error;
mod model;
pub use collect::observe;
pub use error::StringReferenceFailure;
pub use model::{
    StringEncoding, StringReferenceLimits, StringReferenceObservationReport,
    StringReferenceOutcome, StringReferenceRecord,
};
// Reuse M6's established budget value type, not its dependency-observation operation.
pub use super::dependencies::StringLimits;
