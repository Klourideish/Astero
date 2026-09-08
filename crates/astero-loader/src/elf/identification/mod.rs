//! ELF identification and encoding support; OS ABI is observed, not inferred.
mod decode;
pub use decode::Identification;
pub(super) use decode::decode;
