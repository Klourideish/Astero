use super::{DescriptorOrigin, RelocationFormat, TableKind, TrustedRelocationExtent};
use crate::elf::dynamic::{
    DynamicTable,
    observation::{PltRelocationKind, TableDescriptor},
    tags::DynamicTag,
};
pub(in crate::elf::dynamic::relocations) fn discover(
    table: &DynamicTable,
) -> Vec<TrustedRelocationExtent> {
    let mut result = Vec::new();
    if let Some(d) = &table.descriptors().rela {
        result.push(adapt(
            table,
            d,
            TableKind::Dynamic,
            RelocationFormat::Rela,
            &[DynamicTag::Rela, DynamicTag::RelaSz, DynamicTag::RelaEnt],
            24,
        ));
    }
    if let Some(p) = &table.descriptors().plt {
        let (format, width) = match p.kind {
            PltRelocationKind::Rela => (RelocationFormat::Rela, 24),
            PltRelocationKind::Rel => (RelocationFormat::DeferredRel, 16),
        };
        result.push(adapt(
            table,
            &p.table,
            TableKind::Plt,
            format,
            &[DynamicTag::JmpRel, DynamicTag::PltRelSz, DynamicTag::PltRel],
            width,
        ));
    }
    result
}
fn adapt(
    table: &DynamicTable,
    d: &TableDescriptor,
    kind: TableKind,
    format: RelocationFormat,
    tags: &[DynamicTag],
    width: u64,
) -> TrustedRelocationExtent {
    // Only private M5 storage reaches here; M5 already proved size divisibility, width and translation.
    let origins = table
        .entries()
        .iter()
        .filter(|e| tags.contains(&e.tag))
        .map(|e| DescriptorOrigin {
            tag: e.tag,
            dynamic_entry_index: e.index,
        })
        .collect();
    TrustedRelocationExtent {
        kind,
        format,
        address: d.address,
        source: d.source,
        count: d.size / width,
        width,
        origins,
    }
}
