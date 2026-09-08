//! Read-only session adapter. No toolkit types or duplicate state model.
use astero_core::{observation::ObservationError, session::SessionObserver};
use astero_debug::inspection::{Inspection, inspect_session};

pub struct SessionView {
    observer: SessionObserver,
}
impl SessionView {
    pub fn new(observer: SessionObserver) -> Self {
        Self { observer }
    }
    pub fn inspect(&self) -> Result<Inspection, ObservationError> {
        inspect_session(&self.observer)
    }
}
