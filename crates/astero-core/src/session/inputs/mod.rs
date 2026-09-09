//! Typed immutable evidence input. Construction does not load a guest.
mod composition;
pub use astero_loader::elf::dynamic::candidates::report::{Completeness, LinkageEvidenceReport};
pub use composition::{EvidenceTarget, InputError, SessionInputs};
