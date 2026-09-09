//! Evidence classification only: candidates are not resolved or registered symbols.
pub mod classification;
mod enumerate;
pub mod error;
pub mod evidence;
pub mod exports;
pub mod imports;
pub use enumerate::{CandidateEnumeration, CandidateLimits, enumerate};
pub mod report;

pub mod structural;

pub mod workload;
