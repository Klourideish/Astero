use super::{Binding, DynamicSymbolObservation, Section, SymbolError, SymbolType, Visibility};
use crate::{
    elf::{
        ElfInspection,
        address_translation::AddressTranslator,
        dynamic::{self, DynamicObservation, ObservationLimits, string_table::DynamicStringTable},
    },
    metadata::VirtualAddress,
};
/// Tied to the original immutable inspection. Enumeration requires internally derived hash evidence.
pub struct SymbolTable<'a> {
    elf: &'a ElfInspection,
    translator: AddressTranslator<'a>,
    address: u64,
    strings: Option<DynamicStringTable>,
    pub(super) extent: Option<crate::elf::dynamic::hash::extent::TrustedSymbolExtent>,
}
impl<'a> SymbolTable<'a> {
    pub fn new(elf: &'a ElfInspection, limits: ObservationLimits) -> Result<Self, SymbolError> {
        let dynamic = dynamic::observe(elf, limits).map_err(SymbolError::Dynamic)?;
        let DynamicObservation::Present(table) = dynamic else {
            return Err(SymbolError::TableUnavailable);
        };
        let descriptor = table
            .descriptors()
            .symbols
            .as_ref()
            .ok_or(SymbolError::TableUnavailable)?;
        let strings = DynamicStringTable::from_dynamic(&table)
            .map_err(|error| SymbolError::Name { index: 0, error })?;
        let translator = AddressTranslator::new(elf)
            .map_err(|error| SymbolError::Translation { index: 0, error })?;
        Ok(Self {
            elf,
            translator,
            address: descriptor.address.0,
            strings,
            extent: None,
        })
    }
    /// Exact count is available only after with_hash validates evidence and the entire source extent.
    pub fn symbol_count(&self) -> Result<u64, SymbolError> {
        self.extent
            .as_ref()
            .map(|e| e.symbol_count())
            .ok_or(SymbolError::CountUnavailable)
    }
    /// Reads one caller-requested candidate. Success proves byte backing, not table membership.
    /// st_name == 0 denotes no name without dereferencing string offset zero.
    pub fn read_candidate(
        &self,
        index: u64,
        max_name_scan_bytes: u64,
    ) -> Result<DynamicSymbolObservation<'_>, SymbolError> {
        let address = index
            .checked_mul(24)
            .and_then(|offset| self.address.checked_add(offset))
            .ok_or(SymbolError::IndexOverflow { index })?;
        let source = self
            .translator
            .translate(VirtualAddress(address), 24)
            .map_err(|error| SymbolError::Translation { index, error })?;
        let bytes = self
            .elf
            .artifact()
            .source()
            .read(&source)
            .map_err(SymbolError::Source)?;
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
        let name = if name_offset == 0 {
            None
        } else {
            Some(
                self.strings
                    .as_ref()
                    .ok_or(SymbolError::StringsUnavailable {
                        index,
                        offset: name_offset,
                    })?
                    .lookup(u64::from(name_offset), max_name_scan_bytes)
                    .map_err(|error| SymbolError::Name { index, error })?,
            )
        };
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
        Ok(DynamicSymbolObservation {
            index,
            source,
            name_offset,
            name,
            info,
            other,
            binding,
            symbol_type,
            visibility,
            section,
            value,
            size,
        })
    }
}
