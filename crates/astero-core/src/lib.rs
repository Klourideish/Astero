//! Session composition and application observation contracts.
pub mod session;
// Preserve the M1 observation path without a second implementation.
pub use session::observation;
