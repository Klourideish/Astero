use super::model::{
    DynamicDescriptors, PltDescriptor, PltRelocationKind, SymbolTableDescriptor, TableDescriptor,
};
use crate::{
    elf::{
        address_translation::AddressTranslator,
        dynamic::{entries::DynamicEntry, error::DynamicError, tags::DynamicTag},
    },
    metadata::VirtualAddress,
};
use DynamicTag::*;
use std::collections::BTreeMap;
type Fields = BTreeMap<DynamicTag, (u64, u64)>;
pub(in crate::elf::dynamic) fn collect<'a>(
    entries: impl IntoIterator<Item = &'a DynamicEntry>,
    translator: &AddressTranslator<'_>,
) -> Result<DynamicDescriptors, DynamicError> {
    let mut fields = Fields::new();
    let mut result = DynamicDescriptors::default();
    for e in entries {
        match e.tag {
            Needed => result.needed_offsets.push(e.value),
            Unknown(_) | Null => {}
            tag => {
                if let Some((first, _)) = fields.insert(tag, (e.index, e.value)) {
                    return Err(DynamicError::DuplicateTag {
                        tag,
                        first,
                        second: e.index,
                    });
                }
            }
        }
    }
    if let Some(v) = group(&fields, &[StrTab, StrSz])? {
        result.strings = Some(table(translator, StrTab, v[0], v[1], None)?);
    }
    if let Some(v) = group(&fields, &[SymTab, SymEnt])? {
        entry_size(SymEnt, v[1], 24)?;
        let first = table(translator, SymTab, v[0], v[1], Some(v[1]))?;
        result.symbols = Some(SymbolTableDescriptor {
            address: first.address,
            entry_size: v[1],
            first_entry: first.source,
        });
    }
    if let Some(v) = group(&fields, &[Rela, RelaSz, RelaEnt])? {
        entry_size(RelaEnt, v[2], 24)?;
        result.rela = Some(table(translator, Rela, v[0], v[1], Some(v[2]))?);
    }
    if let Some(v) = group(&fields, &[JmpRel, PltRelSz, PltRel])? {
        let (kind, width) = match v[2] {
            7 => (PltRelocationKind::Rela, 24),
            17 => (PltRelocationKind::Rel, 16),
            other => return Err(DynamicError::UnsupportedPltRelocationKind(other)),
        };
        result.plt = Some(PltDescriptor {
            kind,
            table: table(translator, JmpRel, v[0], v[1], Some(width))?,
        });
    }
    if let Some(v) = group(&fields, &[InitArray, InitArraySz])? {
        result.init_array = Some(table(translator, InitArray, v[0], v[1], Some(8))?);
    }
    if let Some(v) = group(&fields, &[FiniArray, FiniArraySz])? {
        result.fini_array = Some(table(translator, FiniArray, v[0], v[1], Some(8))?);
    }
    result.init = fields.get(&Init).map(|(_, v)| VirtualAddress(*v));
    result.fini = fields.get(&Fini).map(|(_, v)| VirtualAddress(*v));
    for (index, &offset) in result.needed_offsets.iter().enumerate() {
        let strings = result
            .strings
            .as_ref()
            .ok_or(DynamicError::IncompleteDescriptor {
                present: Needed,
                missing: StrTab,
            })?;
        if offset >= strings.size {
            return Err(DynamicError::NeededOffsetOutOfBounds {
                index,
                offset,
                string_size: strings.size,
            });
        }
    }
    Ok(result)
}
fn group(fields: &Fields, tags: &[DynamicTag]) -> Result<Option<Vec<u64>>, DynamicError> {
    let Some(&present) = tags.iter().find(|tag| fields.contains_key(tag)) else {
        return Ok(None);
    };
    let mut values = Vec::new();
    for &tag in tags {
        values.push(
            fields
                .get(&tag)
                .ok_or(DynamicError::IncompleteDescriptor {
                    present,
                    missing: tag,
                })?
                .1,
        );
    }
    Ok(Some(values))
}
fn entry_size(tag: DynamicTag, observed: u64, expected: u64) -> Result<(), DynamicError> {
    if observed != expected {
        return Err(DynamicError::UnsupportedEntrySize {
            tag,
            observed,
            expected,
        });
    }
    Ok(())
}
fn table(
    translator: &AddressTranslator<'_>,
    tag: DynamicTag,
    address: u64,
    size: u64,
    entry_size: Option<u64>,
) -> Result<TableDescriptor, DynamicError> {
    if let Some(width) = entry_size
        && !size.is_multiple_of(width)
    {
        return Err(DynamicError::InvalidDescriptorSize {
            tag,
            size,
            entry_size: width,
        });
    }
    let address = VirtualAddress(address);
    let source =
        translator
            .translate(address, size)
            .map_err(|error| DynamicError::Translation {
                tag: Some(tag),
                error,
            })?;
    Ok(TableDescriptor {
        address,
        size,
        entry_size,
        source,
    })
}
