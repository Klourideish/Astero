use crate::{
    admission::{TargetMetadata, ValidatedTarget},
    metadata::{AddressRange, Permissions, SourceRange, VirtualAddress},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyIntent {
    pub source: SourceRange,
    pub destination: VirtualAddress,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappingIntent {
    pub range: AddressRange,
    pub alignment: u64,
    pub permissions: Permissions,
    pub copy: Option<CopyIntent>,
    pub zero_fill: Option<AddressRange>,
}
/// Immutable work description; contains neither handles nor executable callbacks.
/// ```compile_fail
/// use astero_loader::load_plan::LoadPlan;
/// fn alter(plan: &mut LoadPlan) { plan.mappings()[0].alignment = 0; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadPlan {
    metadata: TargetMetadata,
    mappings: Vec<MappingIntent>,
}
impl LoadPlan {
    pub fn metadata(&self) -> &TargetMetadata {
        &self.metadata
    }
    pub fn mappings(&self) -> &[MappingIntent] {
        &self.mappings
    }
}
/// Pure planning: no allocation of guest memory, I/O, resolution or relocation execution.
/// Region order is canonical by requested address; descriptor order preserves import indices.
pub fn plan(target: &ValidatedTarget) -> LoadPlan {
    let mappings = target
        .regions()
        .iter()
        .map(|region| {
            let copied = region.source.size;
            MappingIntent {
                range: region.range,
                alignment: region.alignment,
                permissions: region.permissions,
                copy: (copied != 0).then_some(CopyIntent {
                    source: region.source,
                    destination: region.range.start,
                }),
                zero_fill: (region.range.size > copied).then(|| AddressRange {
                    // Admission proved source.size <= memory size and virtual end cannot overflow.
                    start: VirtualAddress(region.range.start.0 + copied),
                    size: region.range.size - copied,
                }),
            }
        })
        .collect();
    LoadPlan {
        metadata: target.metadata().clone(),
        mappings,
    }
}
