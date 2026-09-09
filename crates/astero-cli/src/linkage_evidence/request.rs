use crate::{
    acquisition::{Selection, State},
    inspection::RequestError,
};
use astero_core::input::linkage_evidence::{self, LinkageLimits, LinkageReport};
pub fn observe_acquired(
    selection: &Selection,
    limits: LinkageLimits,
) -> Result<LinkageReport, RequestError> {
    match selection.state() {
        State::Acquired(source) => Ok(linkage_evidence::observe(source.clone(), limits)),
        _ => Err(RequestError::NotAcquired),
    }
}
