use crate::artifact::BoundSourceRange;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SysVHash {
    pub(in crate::elf::dynamic::hash) source: BoundSourceRange,
    pub(in crate::elf::dynamic::hash) buckets: u32,
    pub(in crate::elf::dynamic::hash) chains: u32,
}
impl SysVHash {
    pub fn source_range(&self) -> BoundSourceRange {
        self.source
    }
    pub fn bucket_count(&self) -> u32 {
        self.buckets
    }
    pub fn chain_count(&self) -> u32 {
        self.chains
    }
}
