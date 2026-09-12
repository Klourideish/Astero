//! High-level host lifecycle only. There is no guest loader or executor.
use super::lifecycle::Lifecycle;
use super::observation::{
    Availability, Diagnostic, ObservationError, ObserveSession, SessionSnapshot, Statistics,
    SubsystemStatus,
};
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Unique within this host process. Not a persistent or cross-process identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "session-{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    IdentityExhausted,
    InvalidTransition,
    NoGuestLoaded,
    ExecutionUnsupported,
    EmptyFault,
    StateUnavailable,
    TimingShutdownFailed,
}

/// Sole strong owner of session state. Frontends retain only a weak observer.
pub struct Session {
    state: Arc<Mutex<SessionSnapshot>>,
    timing: Option<super::timing::TimingEngine>,
    presentation: Option<astero_video::presentation::Endpoint>,
}

#[derive(Clone)]
pub struct SessionObserver {
    state: Weak<Mutex<SessionSnapshot>>,
}

impl Session {
    pub fn attach_presentation(&mut self, endpoint: astero_video::presentation::Endpoint) {
        self.detach_presentation();
        self.presentation = Some(endpoint);
    }
    pub fn presentation(&self) -> Option<astero_video::presentation::Endpoint> {
        self.presentation.clone()
    }
    pub fn detach_presentation(&mut self) {
        use astero_video::presentation::PresentationSink;
        if let Some(p) = self.presentation.take() {
            p.close();
        }
    }

    pub fn new() -> Result<Self, SessionError> {
        Self::with_inputs(super::inputs::SessionInputs::default())
    }

    /// Accept already checked immutable observational inputs; loaded_target remains None.
    pub fn with_inputs(inputs: super::inputs::SessionInputs) -> Result<Self, SessionError> {
        let id = NEXT_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| SessionError::IdentityExhausted)?;
        Ok(Self {
            timing: None,
            presentation: None,
            state: Arc::new(Mutex::new(SessionSnapshot {
                id: SessionId(id),
                lifecycle: Lifecycle::Created,
                loaded_target: None,
                inputs,
                statistics: Statistics::default(),
                subsystems: [
                    "memory",
                    "loader",
                    "kernel/execution",
                    "HLE/libraries",
                    "shader",
                    "GPU",
                    "video",
                    "audio",
                ]
                .into_iter()
                .map(|name| SubsystemStatus {
                    name,
                    availability: Availability::NotImplemented,
                })
                .collect(),
                diagnostics: vec![Diagnostic::NoGuestLoaded],
            })),
        })
    }

    /// Explicit ownership transfer; all existing constructors remain worker-free.
    pub fn with_timing(
        inputs: super::inputs::SessionInputs,
        timing: super::timing::TimingEngine,
    ) -> Result<Self, SessionError> {
        let mut session = Self::with_inputs(inputs)?;
        session.timing = Some(timing);
        Ok(session)
    }

    pub fn timing(&self) -> Option<super::timing::Scheduler> {
        self.timing.as_ref().map(|engine| engine.scheduler())
    }

    pub fn observer(&self) -> SessionObserver {
        SessionObserver {
            state: Arc::downgrade(&self.state),
        }
    }

    /// Ready means host session initialization, not guest readiness.
    pub fn initialize(&mut self) -> Result<(), SessionError> {
        self.change(Lifecycle::Ready, None)
    }

    pub fn stop(&mut self) -> Result<(), SessionError> {
        self.change(Lifecycle::Stopped, None)?;
        self.detach_presentation();
        if let Some(timing) = &self.timing {
            timing
                .shutdown()
                .map_err(|_| SessionError::TimingShutdownFailed)?;
        }
        Ok(())
    }

    /// Record a real host orchestration failure; this is not guest fault capture.
    pub fn report_host_fault(&mut self, reason: &str) -> Result<(), SessionError> {
        if reason.trim().is_empty() {
            return Err(SessionError::EmptyFault);
        }
        self.change(Lifecycle::Faulted, Some(reason))
    }

    pub fn run(&mut self) -> Result<(), SessionError> {
        let state = self
            .state
            .lock()
            .map_err(|_| SessionError::StateUnavailable)?;
        if state.lifecycle != Lifecycle::Ready {
            return Err(SessionError::InvalidTransition);
        }
        if state.loaded_target.is_none() {
            return Err(SessionError::NoGuestLoaded);
        }
        Err(SessionError::ExecutionUnsupported)
    }

    fn change(&mut self, to: Lifecycle, fault: Option<&str>) -> Result<(), SessionError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| SessionError::StateUnavailable)?;
        let valid = matches!(
            (state.lifecycle, to),
            (
                Lifecycle::Created,
                Lifecycle::Ready | Lifecycle::Stopped | Lifecycle::Faulted
            ) | (Lifecycle::Ready, Lifecycle::Stopped | Lifecycle::Faulted)
                | (Lifecycle::Faulted, Lifecycle::Stopped)
        );
        if !valid {
            return Err(SessionError::InvalidTransition);
        }
        state.lifecycle = to;
        state.statistics.lifecycle_changes += 1;
        if let Some(reason) = fault {
            state
                .diagnostics
                .push(Diagnostic::HostFault(reason.to_owned()));
        }
        Ok(())
    }
}

impl ObserveSession for SessionObserver {
    fn snapshot(&self) -> Result<SessionSnapshot, ObservationError> {
        let state = self
            .state
            .upgrade()
            .ok_or(ObservationError::SessionClosed)?;
        state
            .lock()
            .map(|s| s.clone())
            .map_err(|_| ObservationError::StateUnavailable)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.detach_presentation();
    }
}
