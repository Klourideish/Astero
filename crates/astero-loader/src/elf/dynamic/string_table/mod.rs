//! Source-backed bounded byte strings; no dependency identity or text normalization.
mod error;
mod view;
pub use error::StringTableError;
pub use view::{DynamicStringTable, StringReference};
