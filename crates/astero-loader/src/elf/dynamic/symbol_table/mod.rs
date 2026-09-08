//! Explicit indexed symbol candidates; source backing does not prove symbol-count membership.
mod error;
mod model;
mod read;
pub use error::SymbolError;
pub use model::{Binding, DynamicSymbolObservation, Section, SymbolType, Visibility};
pub use read::SymbolTable;
