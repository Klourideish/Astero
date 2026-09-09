use astero_core::input::acquisition::{self, AcquisitionError, AcquisitionLimits, SourceArtifact};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub path: PathBuf,
    pub limits: AcquisitionLimits,
}
#[derive(Debug)]
pub enum State {
    Ready,
    Acquired(SourceArtifact),
    Failed(AcquisitionError),
}
/// One immutable selection and its latest synchronous acquisition result, not a guest session.
#[derive(Debug)]
pub struct Selection {
    request: Request,
    state: State,
}
impl Selection {
    pub fn new(request: Request) -> Self {
        Self {
            request,
            state: State::Ready,
        }
    }
    pub fn request(&self) -> &Request {
        &self.request
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn acquire(&mut self) {
        self.state = match acquisition::acquire(&self.request.path, self.request.limits) {
            Ok(source) => State::Acquired(source),
            Err(error) => State::Failed(error),
        };
    }
}
