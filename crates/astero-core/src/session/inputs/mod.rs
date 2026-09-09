//! Typed immutable evidence input. Construction does not load a guest.
mod composition;
pub use astero_loader::elf::dynamic::candidates::report::{
    CandidateDetail, Completeness, LinkageEvidenceReport, ReportCounts,
};
pub use composition::{EvidenceOrigin, EvidenceTarget, InputError, SessionInputs};
pub mod synthetic;
pub use astero_loader::elf::dynamic::candidates::evidence::NameState;
