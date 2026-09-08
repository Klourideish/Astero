use super::super::error::{HashError, HashFailure, HashKind};
use crate::{
    artifact::{BoundSourceRange, SourceArtifact},
    elf::address_translation::AddressTranslator,
    metadata::VirtualAddress,
};
/// Hash-local prefix reader. Every growing prefix remains in one unambiguous source-backed PT_LOAD.
pub(in crate::elf::dynamic::hash) struct HashReader<'a, 'b> {
    pub source: &'a SourceArtifact,
    pub translator: &'b AddressTranslator<'a>,
    pub base: u64,
    pub kind: HashKind,
    pub remaining: &'b mut u64,
}
impl HashReader<'_, '_> {
    pub fn error(&self, failure: HashFailure) -> HashError {
        HashError::At {
            source: self.source.identity(),
            kind: self.kind,
            address: self.base,
            failure,
        }
    }
    pub fn prefix(&self, size: u64) -> Result<BoundSourceRange, HashError> {
        self.translator
            .translate(VirtualAddress(self.base), size)
            .map_err(|e| self.error(HashFailure::Translation(e)))
    }
    pub fn charge(&mut self, words: u64) -> Result<(), HashError> {
        *self.remaining = self
            .remaining
            .checked_sub(words)
            .ok_or_else(|| self.error(HashFailure::WorkLimit))?;
        Ok(())
    }
    pub fn word(&mut self, offset: u64) -> Result<u32, HashError> {
        let end = offset
            .checked_add(4)
            .ok_or_else(|| self.error(HashFailure::Overflow))?;
        let token = self.prefix(end)?;
        self.charge(1)?;
        let bytes = self
            .source
            .read(&token)
            .map_err(|e| self.error(HashFailure::Source(e)))?;
        let at = offset as usize; // bounded by the translated actual byte slice
        Ok(u32::from_le_bytes([
            bytes[at],
            bytes[at + 1],
            bytes[at + 2],
            bytes[at + 3],
        ]))
    }
}
