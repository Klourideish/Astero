use super::{model::State, *};
use crate::{
    clock::{ManualClock, Source},
    diagnostics::Counters,
    time::Span,
};
use std::sync::{Arc, Condvar, Mutex, atomic::AtomicU64};
use std::thread::JoinHandle;
/// Sole worker owner. Producer/manual/ticket handles do not prevent Drop from joining it.
pub struct TimingEngine {
    pub(super) shared: Arc<Shared>,
    worker: Mutex<Option<JoinHandle<()>>>,
}
impl TimingEngine {
    pub fn real(config: Config) -> Result<Self, Error> {
        Self::start(config, Source::Real(std::time::Instant::now()))
    }
    pub fn manual(config: Config) -> Result<(Self, ManualClock), Error> {
        let engine = Self::start(config, Source::Manual(AtomicU64::new(0)))?;
        let clock = ManualClock {
            shared: Arc::downgrade(&engine.shared),
        };
        Ok((engine, clock))
    }
    fn start(config: Config, clock: Source) -> Result<Self, Error> {
        let mut events = Vec::new();
        events
            .try_reserve_exact(config.max_pending)
            .map_err(|_| Error::Allocation {
                entries: config.max_pending,
            })?;
        let shared = Arc::new(Shared {
            clock,
            config,
            wake: Condvar::new(),
            state: Mutex::new(State {
                events,
                next_sequence: 1,
                counters: Counters::default(),
                stopped: None,
                worker_waiting: false,
                peak_pending: 0,
                max_lateness: Span::ZERO,
                last_terminal: None,
            }),
        });
        let work = shared.clone();
        let worker = std::thread::Builder::new()
            .name("astero-timing".into())
            .spawn(move || {
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker::run(&work)))
                    .is_err()
                {
                    worker::stop(&work, StopReason::WorkerPanicked);
                }
            })
            .map_err(Error::WorkerStart)?;
        Ok(Self {
            shared,
            worker: Mutex::new(Some(worker)),
        })
    }
    pub fn scheduler(&self) -> Scheduler {
        Scheduler {
            shared: Arc::downgrade(&self.shared),
        }
    }
    /// Serializes joins, so every successful concurrent return means the worker has exited.
    pub fn shutdown(&self) -> Result<(), Error> {
        worker::stop(&self.shared, StopReason::Shutdown);
        let mut worker = self.worker.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(h) = worker.take() {
            h.join().map_err(|_| Error::WorkerPanicked)?;
        }
        if self.shared.lock().stopped == Some(StopReason::WorkerPanicked) {
            Err(Error::WorkerPanicked)
        } else {
            Ok(())
        }
    }
}
impl Drop for TimingEngine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
