//! Explicit acquisition/proof selection and structural symbol presentation, not linkage.
mod arguments;
mod presentation;
mod request;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use request::observe_acquired;
pub const USAGE: &str = "Usage: astero-cli symbols --path <native-path> --max-bytes <u64> --max-read-calls <u64> --max-program-headers <u64> --max-dynamic-entries <u64> --max-hash-words <u64> --max-descriptors <u64> --max-symbols <u64> --max-name-lookups <u64> --max-name-scan-bytes <u64> --max-total-name-scan-bytes <u64>";
