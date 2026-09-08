/// Loader-local virtual-address intent, not a host pointer or allocated address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtualAddress(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileOffset(pub u64);
/// Half-open source extent; observation can contain invalid extents until admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceRange {
    pub offset: FileOffset,
    pub size: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddressRange {
    pub start: VirtualAddress,
    pub size: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegionObservation {
    pub address: VirtualAddress,
    pub source: SourceRange,
    pub memory_size: u64,
    pub alignment: u64,
    pub permissions: Permissions,
}
