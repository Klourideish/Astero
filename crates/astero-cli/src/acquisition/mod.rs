//! Explicit acquisition-only CLI selection; no parsing, linkage or session state.
mod arguments;
mod presentation;
mod selection;
pub use arguments::{ArgumentError, parse};
pub use presentation::render;
pub use selection::{Request, Selection, State};

pub const USAGE: &str =
    "Usage: astero-cli acquire --path <native-path> --max-bytes <u64> --max-read-calls <u64>";
