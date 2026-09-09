//! Source-bound table discovery, raw reads and lazy reference-validated enumeration.
mod enumerate;
mod model;
mod read;
pub use enumerate::{RelocationIterator, RelocationLimits};
pub use model::{RawRelocation, RelocationObservation, RelocationTables};

mod raw;
pub use raw::RawRelocationIterator;
