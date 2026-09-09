//! Format adaptation into source-bound observations; admission remains a separate call.
mod adapter;
pub use adapter::{ElfInspection, inspect};

pub mod bounded;
pub mod synthetic;
