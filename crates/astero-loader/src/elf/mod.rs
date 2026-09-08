//! Bounded ELF64 observations and adaptation; no dynamic linking or memory application.
mod decoding;
pub mod error;
pub mod header;
pub mod identification;
pub mod inspect;
pub mod program_headers;
pub use error::ElfError;
pub use inspect::{ElfInspection, inspect};
