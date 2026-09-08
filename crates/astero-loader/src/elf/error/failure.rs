use crate::artifact::SourceError;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Structure {
    Identification,
    Header,
    ProgramHeaderTable,
    ProgramHeader { index: u16 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElfError {
    Source {
        structure: Structure,
        error: SourceError,
    },
    InvalidMagic {
        observed: [u8; 4],
    },
    UnsupportedClass(u8),
    UnsupportedByteOrder(u8),
    UnsupportedVersion {
        structure: Structure,
        version: u32,
    },
    UnsupportedHeaderSize(u16),
    UnsupportedProgramHeaderSize(u16),
    UnsupportedExtendedProgramCount,
    InvalidTableOffset {
        offset: u64,
        count: u16,
    },
    TableOverflow {
        offset: u64,
        count: u16,
        entry_size: u16,
    },
    MalformedField {
        offset: usize,
        width: usize,
    },
}
impl std::fmt::Display for ElfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ELF inspection: {self:?}")
    }
}
impl std::error::Error for ElfError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source { error, .. } => Some(error),
            _ => None,
        }
    }
}
