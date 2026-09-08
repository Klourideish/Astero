use crate::{
    artifact::SourceArtifact,
    elf::dynamic::hash::{extent::TrustedSymbolExtent, gnu::GnuHash, sysv::SysVHash},
};
/// Caller-selected total decoded/traversed 32-bit words across both tables; no hidden PS5 ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HashLimits {
    pub max_words: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HashObservation {
    pub(super) source: SourceArtifact,
    pub(super) sysv: Option<SysVHash>,
    pub(super) gnu: Option<GnuHash>,
    pub(super) extent: Option<TrustedSymbolExtent>,
}
impl HashObservation {
    pub fn source(&self) -> &SourceArtifact {
        &self.source
    }
    pub fn sysv(&self) -> Option<&SysVHash> {
        self.sysv.as_ref()
    }
    pub fn gnu(&self) -> Option<&GnuHash> {
        self.gnu.as_ref()
    }
    pub fn extent(&self) -> Option<&TrustedSymbolExtent> {
        self.extent.as_ref()
    }
}
