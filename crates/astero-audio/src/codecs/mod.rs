//! Audio decoding/encoding mechanisms, with future codec-specific children.
//! Guest codec exports belong in astero-libs; mixing and host output have separate owners.
//! AJM lifecycle is owned here; no compressed audio decoding is claimed.
pub mod ajm;
pub mod atrac9;
