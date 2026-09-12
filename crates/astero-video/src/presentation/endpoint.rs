use super::*;
use std::sync::{Arc, Mutex, Weak};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Ready,
    Active,
    Failed,
    Closed,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub sink: &'static str,
    pub state: State,
    pub received: u64,
    pub presented: u64,
    pub dropped: u64,
    pub refused: u64,
    pub last_id: Option<u64>,
    pub width: u32,
    pub height: u32,
    pub format: Option<Format>,
    pub checksum: Option<u64>,
    pub bytes: u64,
    pub backing_id: Option<u64>,
    pub last_presented_id: Option<u64>,
}
struct Inner {
    snapshot: Snapshot,
    pending: Option<Frame>,
    in_flight: Option<u64>,
}
/// Thread-safe latest-frame mailbox; backend objects never cross the producer boundary.
#[derive(Clone)]
pub struct Endpoint {
    inner: Arc<Mutex<Inner>>,
}
pub trait PresentationSink {
    fn present(&self, frame: Frame) -> Result<(), Error>;
    fn snapshot(&self) -> Snapshot;
    fn close(&self);
}
impl Endpoint {
    pub fn new(sink: &'static str) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                snapshot: Snapshot {
                    sink,
                    state: State::Ready,
                    received: 0,
                    presented: 0,
                    dropped: 0,
                    refused: 0,
                    last_id: None,
                    width: 0,
                    height: 0,
                    format: None,
                    checksum: None,
                    bytes: 0,
                    backing_id: None,
                    last_presented_id: None,
                },
                pending: None,
                in_flight: None,
            })),
        }
    }
    /// Weak read-only observation never retains frame backing or a backend.
    pub fn observer(&self) -> Observation {
        Observation {
            inner: Arc::downgrade(&self.inner),
        }
    }
    /// One consumer frame can be outstanding. Pending newer frames still replace each other.
    pub fn take(&self) -> Option<Frame> {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if s.in_flight.is_some() {
            return None;
        }
        let f = s.pending.take()?;
        s.in_flight = Some(f.id);
        Some(f)
    }
    pub fn completed(&self, frame: &Frame) {
        // Hash outside the mailbox lock; producer publication remains a short operation.
        let checksum = match &frame.backing {
            Backing::Cpu(b) => Some(b.iter().fold(0xcbf29ce484222325, |h, b| {
                (h ^ *b as u64).wrapping_mul(0x100000001b3)
            })),
            Backing::Host(_) => None,
        };
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(s.snapshot.state, State::Closed | State::Failed)
            || s.in_flight != Some(frame.id)
        {
            return;
        }
        s.in_flight = None;
        s.snapshot.presented += 1;
        s.snapshot.last_presented_id = Some(frame.id);
        s.snapshot.bytes = u64::from(frame.stride) * u64::from(frame.height);
        s.snapshot.backing_id = match &frame.backing {
            Backing::Host(r) => Some(r.identity()),
            _ => None,
        };
        s.snapshot.state = State::Active;
        s.snapshot.width = frame.width;
        s.snapshot.height = frame.height;
        s.snapshot.format = Some(frame.format);
        s.snapshot.checksum = checksum;
    }

    /// Backend abandons a receipt on suspension; a newer pending frame remains available.
    pub fn discard(&self, frame: &Frame) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if s.in_flight == Some(frame.id) {
            s.in_flight = None;
            s.snapshot.dropped += 1;
        }
    }
    pub fn failed(&self) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        s.snapshot.state = State::Failed;
        if s.pending.take().is_some() {
            s.snapshot.dropped += 1;
        }
        if s.in_flight.take().is_some() {
            s.snapshot.refused += 1;
        }
    }
}
impl PresentationSink for Endpoint {
    fn present(&self, frame: Frame) -> Result<(), Error> {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let error = if matches!(s.snapshot.state, State::Closed | State::Failed) {
            Some(Error::Closed)
        } else if let Err(e) = frame.validate() {
            Some(e)
        } else if s.snapshot.last_id.is_some_and(|id| frame.id <= id) {
            Some(Error::Order)
        } else {
            None
        };
        if let Some(e) = error {
            s.snapshot.refused += 1;
            return Err(e);
        }
        s.snapshot.received += 1;
        s.snapshot.last_id = Some(frame.id);
        if s.pending.replace(frame).is_some() {
            s.snapshot.dropped += 1;
        }
        Ok(())
    }
    fn snapshot(&self) -> Snapshot {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .snapshot
            .clone()
    }
    fn close(&self) {
        let mut s = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if s.pending.take().is_some() {
            s.snapshot.dropped += 1;
        }
        if s.in_flight.take().is_some() {
            s.snapshot.dropped += 1;
        }
        s.snapshot.state = State::Closed;
    }
}
pub struct Headless {
    endpoint: Endpoint,
}
impl Default for Headless {
    fn default() -> Self {
        Self {
            endpoint: Endpoint::new("headless"),
        }
    }
}
impl Headless {
    pub fn endpoint(&self) -> Endpoint {
        self.endpoint.clone()
    }
    pub fn drain(&self) -> Result<bool, Error> {
        let Some(f) = self.endpoint.take() else {
            return Ok(false);
        };
        if !matches!(f.backing, Backing::Cpu(_)) {
            self.endpoint.failed();
            return Err(Error::Unsupported);
        }
        self.endpoint.completed(&f);
        Ok(true)
    }
}
impl PresentationSink for Headless {
    fn present(&self, f: Frame) -> Result<(), Error> {
        self.endpoint.present(f)
    }
    fn snapshot(&self) -> Snapshot {
        self.endpoint.snapshot()
    }
    fn close(&self) {
        self.endpoint.close();
    }
}
impl Drop for Headless {
    fn drop(&mut self) {
        self.close();
    }
}

/// Does not keep the endpoint, pending frames, or presentation resources alive.
#[derive(Clone)]
pub struct Observation {
    inner: Weak<Mutex<Inner>>,
}
impl Observation {
    pub fn snapshot(&self) -> Option<Snapshot> {
        self.inner
            .upgrade()
            .map(|s| s.lock().unwrap_or_else(|e| e.into_inner()).snapshot.clone())
    }
}
