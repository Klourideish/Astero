//! CLI selection/presentation only; hash semantics remain in loader.
mod arguments;
mod presentation;
mod request;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use request::observe_acquired;
pub const USAGE: &str = "Usage: astero-cli hash-metadata --path <native-path> --max-bytes <u64> --max-read-calls <u64> --max-program-headers <u64> --max-dynamic-entries <u64> --max-hash-words <u64>";
