//! Bounded dynamic metadata with narrow string/symbol observation owners; no linking.
pub mod entries;
pub mod error;
pub mod observation;
pub mod tags;
pub use observation::{
    DynamicDescriptors, DynamicObservation, DynamicTable, ObservationLimits, observe,
};
pub mod dependencies;
pub mod hash;
pub mod string_table;
pub mod symbol_table;
