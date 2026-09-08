use super::{
    DependencyObservationError,
    model::{DependencyObservation, NeededReference, StringLimits},
};
use crate::{
    artifact::{self, InputArtifact},
    dependencies::{Dependency, DependencyName},
    elf::{
        ElfInspection,
        dynamic::{
            self, DynamicObservation, ObservationLimits, string_table::DynamicStringTable,
            tags::DynamicTag,
        },
    },
};
/// One source-bound pipeline. Named declarations do not replace module identity or clear admission requirements.
pub fn observe(
    elf: &ElfInspection,
    dynamic_limits: ObservationLimits,
    string_limits: StringLimits,
) -> Result<DependencyObservation, DependencyObservationError> {
    let dynamic =
        dynamic::observe(elf, dynamic_limits).map_err(DependencyObservationError::Dynamic)?;
    let mut description = elf.artifact().description().clone();
    let mut references = Vec::new();
    let mut string_table = None;
    if let DynamicObservation::Present(table) = &dynamic {
        string_table = DynamicStringTable::from_dynamic(table)
            .map_err(DependencyObservationError::StringTable)?;
        let mut remaining = string_limits.max_total_scan_bytes;
        for entry in table
            .entries()
            .iter()
            .filter(|entry| entry.tag == DynamicTag::Needed)
        {
            let strings =
                string_table
                    .as_ref()
                    .ok_or(DependencyObservationError::TableUnavailable {
                        entry_index: entry.index,
                        offset: entry.value,
                    })?;
            let view = strings
                .lookup(
                    entry.value,
                    remaining.min(string_limits.max_scan_bytes_per_reference),
                )
                .map_err(|error| DependencyObservationError::Lookup {
                    entry_index: entry.index,
                    error,
                })?;
            remaining -= view.scanned_bytes(); // successful lookup proves its charged work fits the supplied budget
            let name = DependencyName::new(view.as_bytes().to_vec()).map_err(|error| {
                DependencyObservationError::InvalidName {
                    entry_index: entry.index,
                    offset: entry.value,
                    error,
                }
            })?;
            description.dependencies.push(Dependency::Named(name));
            references.push(NeededReference {
                entry_index: entry.index,
                string_offset: entry.value,
                source: view.source_range(),
            });
        }
    }
    Ok(DependencyObservation {
        artifact: artifact::inspect(InputArtifact {
            source: elf.artifact().source().clone(),
            description,
        }),
        dynamic,
        string_table,
        references,
    })
}
