use super::super::{
    descriptor::{RelocationFormat, TrustedRelocationExtent},
    error::RelocationError,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TailAlias {
    pub first_dynamic_index: u64,
    pub count: u64,
}
pub(in crate::elf::dynamic::relocations) fn classify(
    tables: &[TrustedRelocationExtent],
) -> Result<Option<TailAlias>, RelocationError> {
    if tables.len() != 2 {
        return Ok(None);
    }
    let a = &tables[0];
    let b = &tables[1];
    let ar = a.source_range().extent();
    let br = b.source_range().extent();
    if ar.size == 0 || br.size == 0 {
        return Ok(None);
    }
    let source_overlap = ar.offset.0 < br.offset.0 + br.size && br.offset.0 < ar.offset.0 + ar.size;
    let address_overlap =
        a.address().0 < b.address().0 + br.size && b.address().0 < a.address().0 + ar.size;
    if !source_overlap && !address_overlap {
        return Ok(None);
    }
    // These sums were proved by M5 translation. Both coordinates must describe the same tail.
    if b.format() == RelocationFormat::Rela
        && br.offset.0 >= ar.offset.0
        && b.address().0 >= a.address().0
        && br.offset.0 + br.size == ar.offset.0 + ar.size
        && b.address().0 + br.size == a.address().0 + ar.size
        && br.offset.0 - ar.offset.0 == b.address().0 - a.address().0
        && (br.offset.0 - ar.offset.0).is_multiple_of(24)
    {
        return Ok(Some(TailAlias {
            first_dynamic_index: (br.offset.0 - ar.offset.0) / 24,
            count: b.entry_count(),
        }));
    }
    Err(RelocationError::DescriptorConflict {
        ordinary: a.source_range(),
        plt: b.source_range(),
    })
}
