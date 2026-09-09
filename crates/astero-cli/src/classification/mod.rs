//! Separate classify-symbols request, no linkage or ELF semantics in the frontend.
mod arguments;
mod presentation;
mod request;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use request::classify_acquired;
pub const USAGE: &str = "Usage: astero-cli classify-symbols --path <native-path> --max-bytes <u64> --max-read-calls <u64> --max-program-headers <u64> --max-dynamic-entries <u64> --max-hash-words <u64> --max-descriptors <u64> --max-symbols <u64> --max-name-lookups <u64> --max-name-scan-bytes <u64> --max-total-name-scan-bytes <u64> --max-classifications <u64>";
