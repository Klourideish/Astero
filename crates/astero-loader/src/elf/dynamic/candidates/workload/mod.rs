//! Complete bounded linkage evidence from one immutable artifact; no provider resolution.
mod collect;
mod model;
mod names;
pub use collect::observe;
pub use model::*;

pub mod synthetic;
