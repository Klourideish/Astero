use crate::{artifact::BoundSourceRange, elf::dynamic::hash::error::HashKind};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountClaim {
    Exact(u64),
    LowerBound(u64),
}
/// Read-only diagnostic value. Constructing a detached value cannot grant trusted extent access.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtentEvidence {
    pub kind: HashKind,
    pub source: BoundSourceRange,
    pub claim: CountClaim,
}
/// Only hash validation can construct this token; source range covers every 24-byte symbol.
/// ```compile_fail
/// use astero_loader::elf::dynamic::hash::extent::TrustedSymbolExtent;
/// fn forge(e: &mut TrustedSymbolExtent) { e.count = 100; }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrustedSymbolExtent {
    pub(super) count: u64,
    pub(super) symbols: BoundSourceRange,
    pub(super) evidence: Vec<ExtentEvidence>,
}
impl TrustedSymbolExtent {
    pub fn symbol_count(&self) -> u64 {
        self.count
    }
    pub fn source_range(&self) -> BoundSourceRange {
        self.symbols
    }
    pub fn evidence(&self) -> &[ExtentEvidence] {
        &self.evidence
    }
}
