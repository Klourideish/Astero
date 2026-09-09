use super::StringTableError;
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::dynamic::DynamicTable,
};
/// Owns a shared source handle, never a copied string table. Constructed only from validated M5/M18 observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicStringTable {
    source: SourceArtifact,
    range: BoundSourceRange,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StringReference<'a> {
    bytes: &'a [u8],
    source: BoundSourceRange,
}
impl StringReference<'_> {
    /// Excludes the terminator. Empty strings and arbitrary non-UTF-8 bytes are valid views.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes
    }
    pub fn source_range(&self) -> BoundSourceRange {
        self.source
    }
    pub fn as_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.bytes)
    }
    /// Lookup work charged including the terminating NUL.
    pub fn scanned_bytes(&self) -> u64 {
        self.source.extent().size + 1
    }
}
impl DynamicStringTable {
    pub fn from_dynamic(dynamic: &DynamicTable) -> Result<Option<Self>, StringTableError> {
        let Some(descriptor) = dynamic.descriptors().strings.as_ref() else {
            return Ok(None);
        };
        Self::from_range(dynamic.source(), descriptor.source).map(Some)
    }
    /// Accept only a privately constructed M18 record; tokens are checked against the supplied source.
    pub fn from_descriptor(
        source: &SourceArtifact,
        record: &crate::elf::dynamic::descriptors::DescriptorRecord,
    ) -> Result<Option<Self>, StringTableError> {
        let crate::elf::dynamic::descriptors::DescriptorValue::Strings(descriptor) = record.value()
        else {
            return Ok(None);
        };
        Self::from_range(source, descriptor.source).map(Some)
    }
    fn from_range(
        source: &SourceArtifact,
        range: BoundSourceRange,
    ) -> Result<Self, StringTableError> {
        source.read(&range).map_err(StringTableError::Source)?;
        Ok(Self {
            source: source.clone(),
            range,
        })
    }
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn source_range(&self) -> BoundSourceRange {
        self.range
    }
    /// Scan no further than DT_STRSZ or the explicit byte budget, including NUL.
    pub fn lookup(
        &self,
        offset: u64,
        max_scan_bytes: u64,
    ) -> Result<StringReference<'_>, StringTableError> {
        let extent = self.range.extent();
        if offset >= extent.size {
            return Err(StringTableError::OffsetOutOfBounds {
                offset,
                table: self.range,
            });
        }
        let bytes = self
            .source
            .read(&self.range)
            .map_err(StringTableError::Source)?;
        // offset and budget fit usize because both are bounded by this actual source-backed slice.
        let available = extent.size - offset;
        let scan = available.min(max_scan_bytes);
        let suffix = &bytes[offset as usize..(offset + scan) as usize];
        let Some(length) = suffix.iter().position(|byte| *byte == 0) else {
            return Err(if scan < available {
                StringTableError::ScanLimit {
                    offset,
                    table: self.range,
                    limit: max_scan_bytes,
                }
            } else {
                StringTableError::MissingTerminator {
                    offset,
                    table: self.range,
                }
            });
        };
        // The full table source proof bounds this addition and the substring extent.
        let source = self
            .source
            .checked_range(extent.offset.0 + offset, length as u64)
            .map_err(StringTableError::Source)?;
        let bytes = self
            .source
            .read(&source)
            .map_err(StringTableError::Source)?;
        Ok(StringReference { bytes, source })
    }
}
