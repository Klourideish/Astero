//! Lazy program-header decoding over a previously bounded immutable source table.
mod decode;
pub use decode::{ProgramHeader, ProgramHeaders};
