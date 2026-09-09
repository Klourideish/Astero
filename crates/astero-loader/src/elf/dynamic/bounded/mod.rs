//! Independently requested raw dynamic evidence; descriptor semantics remain separate.
mod collect;
mod model;
pub use super::observation::RawTable;
pub use collect::observe;
pub use model::{DynamicFailure, DynamicLimits, DynamicObservationReport, DynamicOutcome};
