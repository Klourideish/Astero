//! Explicit referenced-byte presentation only; no dependency or ELF parsing semantics.
mod arguments;
mod presentation;
mod request;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use request::observe_acquired;
pub const USAGE: &str = "Usage: astero-cli string-references --path <native-path> --max-bytes <u64> --max-read-calls <u64> --max-program-headers <u64> --max-dynamic-entries <u64> --max-descriptors <u64> --max-string-references <u64> --max-scan-bytes-per-reference <u64> --max-total-scan-bytes <u64>";
