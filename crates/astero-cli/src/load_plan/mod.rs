//! Explicit acquisition and offline plan presentation. Loader owns every semantic rule.
mod arguments;
mod presentation;
mod run;
pub use arguments::{Request, parse};
pub use presentation::render;
pub use run::{Failure, execute};
pub const USAGE: &str = "Usage: astero-cli load-plan <all ps5-identity arguments> --image-bias <u64> --max-providers <u64> --max-plan-records <u64> [--provider <native-path> [--provider-alias <exact-name>]]";
