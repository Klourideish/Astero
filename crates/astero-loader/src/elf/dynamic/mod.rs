//! Bounded dynamic metadata observations; no strings, symbols, relocations or linking are interpreted.
pub mod entries;
pub mod error;
pub mod observation;
pub mod tags;
pub use observation::{
    DynamicDescriptors, DynamicObservation, DynamicTable, ObservationLimits, observe,
};
pub mod dependencies;
pub mod string_table;
