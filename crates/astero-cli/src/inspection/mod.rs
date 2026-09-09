//! Explicit opt-in header inspection; separate from acquisition and synthetic linkage.
mod arguments;
mod presentation;
mod request;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use request::{RequestError, inspect_acquired};
pub const USAGE: &str = "Usage: astero-cli inspect --path <native-path> --max-bytes <u64> --max-read-calls <u64> --max-program-headers <u64>";
