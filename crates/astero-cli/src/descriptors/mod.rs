//! Explicit descriptor selection/presentation; no ELF decoding or payload semantics.
mod arguments;
mod presentation;
mod request;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use request::observe_acquired;
pub const USAGE: &str = "Usage: astero-cli descriptors --path <native-path> --max-bytes <u64> --max-read-calls <u64> --max-program-headers <u64> --max-dynamic-entries <u64> --max-descriptors <u64>";
