//! Explicit PS5 identity presentation; no parsing or provider lookup here.
mod arguments;
mod presentation;
pub use arguments::{Request, parse};
pub use presentation::render;
pub const USAGE: &str =
    "Usage: astero-cli ps5-identity <all linkage-evidence arguments> --max-identity-records <u64>";
