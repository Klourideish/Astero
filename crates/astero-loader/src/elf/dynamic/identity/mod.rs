//! PS5 identity evidence over complete linkage reports; no provider resolution.
pub mod codec;
mod collect;
mod metadata;
mod model;
pub use collect::observe;
pub use model::*;

pub mod synthetic;
