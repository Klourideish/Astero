//! Shared development-only composition; fixture generation and evidence semantics stay in loader.
use super::{EvidenceOrigin, EvidenceTarget, SessionInputs};
use crate::session::{Session, SessionError};
pub use astero_loader::elf::dynamic::candidates::report::synthetic::{
    SyntheticCase, SyntheticError,
};
use std::sync::Arc;
#[derive(Debug)]
pub enum SyntheticSessionError {
    Evidence(SyntheticError),
    Input(super::InputError),
    Session(SessionError),
}
pub fn session(max_symbols: u64) -> Result<Session, SyntheticSessionError> {
    session_with_case(SyntheticCase::Complete, max_symbols)
}
pub fn session_with_case(
    case: SyntheticCase,
    max_symbols: u64,
) -> Result<Session, SyntheticSessionError> {
    let report = Arc::new(
        astero_loader::elf::dynamic::candidates::report::synthetic::report(case, max_symbols)
            .map_err(SyntheticSessionError::Evidence)?,
    );
    let target = EvidenceTarget {
        source: report.source(),
        module: report.module(),
    };
    let mut inputs =
        SessionInputs::new(Some(target), Some(report)).map_err(SyntheticSessionError::Input)?;
    inputs.origin = EvidenceOrigin::Synthetic;
    Session::with_inputs(inputs).map_err(SyntheticSessionError::Session)
}
