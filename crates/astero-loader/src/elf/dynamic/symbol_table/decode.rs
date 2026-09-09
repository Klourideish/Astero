//! Sole fixed-field ELF64 symbol decoder, shared by indexed and proven-membership observations.
use super::{Binding, Section, SymbolError, SymbolFields, SymbolType, Visibility};
use crate::artifact::{BoundSourceRange, SourceArtifact};
pub(super) fn decode(
    source_artifact: &SourceArtifact,
    source: BoundSourceRange,
    index: u64,
) -> Result<SymbolFields, SymbolError> {
    let bytes = source_artifact.read(&source).map_err(SymbolError::Source)?;
    // Internal callers must supply one complete entry; malformed tokens never reach indexing.
    if bytes.len() != 24 {
        return Err(SymbolError::InvalidEntryRange {
            index,
            size: bytes.len() as u64,
        });
    }
    // Full 24-byte translation/read proof precedes every fixed field access.
    let name_offset = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let info = bytes[4];
    let other = bytes[5];
    let shndx = u16::from_le_bytes([bytes[6], bytes[7]]);
    let value = u64::from_le_bytes([
        bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    ]);
    let size = u64::from_le_bytes([
        bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
    ]);
    if index == 0 && bytes.iter().any(|&b| b != 0) {
        return Err(SymbolError::InvalidNullSymbol);
    }
    let binding = match info >> 4 {
        0 => Binding::Local,
        1 => Binding::Global,
        2 => Binding::Weak,
        n => Binding::Unknown(n),
    };
    let symbol_type = match info & 15 {
        0 => SymbolType::NoType,
        1 => SymbolType::Object,
        2 => SymbolType::Function,
        3 => SymbolType::Section,
        4 => SymbolType::File,
        5 => SymbolType::Common,
        6 => SymbolType::Tls,
        n => SymbolType::Unknown(n),
    };
    // Generic ABI visibility uses the low three bits; extensions remain numerically observable.
    let visibility = match other & 7 {
        0 => Visibility::Default,
        1 => Visibility::Internal,
        2 => Visibility::Hidden,
        3 => Visibility::Protected,
        n => Visibility::Unknown(n),
    };
    let section = match shndx {
        0 => Section::Undefined,
        0xfff1 => Section::Absolute,
        0xfff2 => Section::Common,
        0xffff => Section::Extended,
        0xff00..=0xfffe => Section::Reserved(shndx),
        n => Section::Index(n),
    };
    Ok(SymbolFields {
        index,
        source,
        name_offset,
        info,
        other,
        shndx,
        binding,
        symbol_type,
        visibility,
        section,
        value,
        size,
    })
}
