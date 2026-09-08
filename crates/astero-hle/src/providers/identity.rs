//! Owns providers within astero-hle; runtime implementation is planned.

/// Provider identity only. Registration and dispatch are planned.
pub trait Provider {
    /// Stable diagnostic name; this does not imply export support.
    fn name(&self) -> &str;
}
