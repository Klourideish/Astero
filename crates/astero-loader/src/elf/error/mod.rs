//! Structured byte-inspection failures, separate from target admission.
mod failure;
pub use failure::{ElfError, Structure};
