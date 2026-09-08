use crate::metadata::VirtualAddress;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationKind {
    /// Synthetic eight-byte patch description only; not an ELF or PS5 relocation number.
    SyntheticAbsolute64,
    Unsupported,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelocationTarget {
    ImportIndex(usize),
    LocalAddress(VirtualAddress),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relocation {
    pub site: VirtualAddress,
    pub kind: RelocationKind,
    pub target: RelocationTarget,
    /// Preserved for a future evaluator; never applied in M2.
    pub addend: i64,
}
