use super::{
    descriptors,
    model::{DynamicObservation, DynamicTable, ObservationLimits},
    raw::observe_raw,
};
use crate::elf::{
    ElfInspection, address_translation::AddressTranslator, dynamic::error::DynamicError,
};
/// Existing M5 descriptor observation; raw traversal is shared with M17.
pub fn observe(
    elf: &ElfInspection,
    limits: ObservationLimits,
) -> Result<DynamicObservation, DynamicError> {
    let programs = elf
        .program_headers()
        .collect::<Result<Vec<_>, _>>()
        .map_err(DynamicError::Header)?;
    let source = elf.artifact().source();
    let Some(raw) = observe_raw(source, &programs, limits)? else {
        return Ok(DynamicObservation::Absent);
    };
    let translator = AddressTranslator::new(elf)
        .map_err(|error| DynamicError::Translation { tag: None, error })?;
    let descriptors = descriptors::collect(&raw.entries, &translator)?;
    Ok(DynamicObservation::Present(Box::new(DynamicTable {
        source: source.clone(),
        source_range: raw.source_range,
        program_index: raw.program_index,
        entries: raw.entries,
        descriptors,
    })))
}
