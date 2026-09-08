//! Dependencies contracts; see the loader architecture record.
mod requirement;
pub use requirement::Dependency;
mod name;
pub use name::{DependencyName, DependencyNameError};
