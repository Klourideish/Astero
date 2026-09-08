use super::super::{
    decoding::read,
    error::{ElfError, Structure},
};
use crate::artifact::SourceArtifact;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identification {
    pub class: u8,
    pub byte_order: u8,
    pub version: u8,
    pub os_abi: u8,
    pub abi_version: u8,
}
pub(in crate::elf) fn decode(source: &SourceArtifact) -> Result<Identification, ElfError> {
    let b = read(source, 0, 16, Structure::Identification)?;
    // Fixed indices are bounded by the exact 16-byte read above.
    let observed = [b[0], b[1], b[2], b[3]];
    if observed != *b"\x7fELF" {
        return Err(ElfError::InvalidMagic { observed });
    }
    if b[4] != 2 {
        return Err(ElfError::UnsupportedClass(b[4]));
    }
    if b[5] != 1 {
        return Err(ElfError::UnsupportedByteOrder(b[5]));
    }
    if b[6] != 1 {
        return Err(ElfError::UnsupportedVersion {
            structure: Structure::Identification,
            version: u32::from(b[6]),
        });
    }
    Ok(Identification {
        class: b[4],
        byte_order: b[5],
        version: b[6],
        os_abi: b[7],
        abi_version: b[8],
    })
}
