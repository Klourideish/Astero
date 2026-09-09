//! Hard evidence failures remain distinct from conservative classification outcomes.
use crate::{
    artifact::SourceId,
    elf::dynamic::{relocations::error::RelocationError, symbol_table::SymbolError},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateError {
    Symbol(SymbolError),
    Relocation(RelocationError),
    SourceMismatch {
        symbols: SourceId,
        relocations: SourceId,
    },
}

impl std::fmt::Display for CandidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CandidateError {}
