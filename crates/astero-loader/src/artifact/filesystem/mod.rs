//! Bounded host file acquisition only; not guest VFS, parsing, admission or loading.
mod acquire;
mod error;
mod read;
pub use acquire::{AcquisitionLimits, acquire};
pub use error::{AcquisitionError, Failure, Operation};
