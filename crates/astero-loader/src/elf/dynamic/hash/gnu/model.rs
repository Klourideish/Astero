use crate::{artifact::BoundSourceRange, elf::dynamic::hash::extent::CountClaim};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GnuHash {
    pub(in crate::elf::dynamic::hash) source: BoundSourceRange,
    pub(in crate::elf::dynamic::hash) buckets: u32,
    pub(in crate::elf::dynamic::hash) symoffset: u32,
    pub(in crate::elf::dynamic::hash) bloom_size: u32,
    pub(in crate::elf::dynamic::hash) bloom_shift: u32,
    pub(in crate::elf::dynamic::hash) claim: CountClaim,
}
impl GnuHash {
    pub fn source_range(&self) -> BoundSourceRange {
        self.source
    }
    pub fn bucket_count(&self) -> u32 {
        self.buckets
    }
    pub fn first_symbol(&self) -> u32 {
        self.symoffset
    }
    pub fn bloom_size(&self) -> u32 {
        self.bloom_size
    }
    pub fn bloom_shift(&self) -> u32 {
        self.bloom_shift
    }
    pub fn count_claim(&self) -> CountClaim {
        self.claim
    }
}
