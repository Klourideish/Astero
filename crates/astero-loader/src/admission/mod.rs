//! Explicit admission, independent of inspection and runtime application.
mod rejection;
mod validated;
mod validation;
pub use rejection::*;
pub use validated::*;
pub use validation::admit;
