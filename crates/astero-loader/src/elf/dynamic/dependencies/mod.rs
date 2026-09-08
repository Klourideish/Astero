//! DT_NEEDED string interpretation into generic declarations; no name resolution or loading.
mod error;
mod inspect;
mod model;
pub use error::DependencyObservationError;
pub use inspect::observe;
pub use model::{DependencyObservation, NeededReference, StringLimits};
