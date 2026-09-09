use crate::{artifact::BoundSourceRange, elf::dynamic::tags::DynamicTag, metadata::VirtualAddress};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableKind {
    Dynamic,
    Plt,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationFormat {
    Rela,
    DeferredRel,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DescriptorOrigin {
    pub tag: DynamicTag,
    pub dynamic_entry_index: u64,
}
/// A source-bound table, not an application plan. REL extents do not authorize RELA decoding.
/// ```compile_fail
/// use astero_loader::elf::dynamic::relocations::descriptor::TrustedRelocationExtent;
/// fn forge(t: &mut TrustedRelocationExtent) { t.count = 999; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedRelocationExtent {
    pub(super) kind: TableKind,
    pub(super) format: RelocationFormat,
    pub(super) address: VirtualAddress,
    pub(super) source: BoundSourceRange,
    pub(super) count: u64,
    pub(super) width: u64,
    pub(super) origins: Vec<DescriptorOrigin>,
}
impl TrustedRelocationExtent {
    pub fn kind(&self) -> TableKind {
        self.kind
    }
    pub fn format(&self) -> RelocationFormat {
        self.format
    }
    pub fn address(&self) -> VirtualAddress {
        self.address
    }
    pub fn source_range(&self) -> BoundSourceRange {
        self.source
    }
    pub fn entry_count(&self) -> u64 {
        self.count
    }
    pub fn entry_size(&self) -> u64 {
        self.width
    }
    pub fn origins(&self) -> &[DescriptorOrigin] {
        &self.origins
    }
}
