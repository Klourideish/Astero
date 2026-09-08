//! Shader decoding, IR, transformations and host generation.
//! M0 foundation; consult the crate README before extending ownership.
pub mod analysis;
pub mod ir;
pub mod lowering;
pub mod rdna2;
pub mod spirv;
pub mod ssa;

// Compatibility path retained from M0/M1; implementation has one physical owner.
pub use rdna2::decode as decoding;
