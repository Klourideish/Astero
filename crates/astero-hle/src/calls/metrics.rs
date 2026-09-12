//! Bounded per-import counters; never invokes providers or retains guest resources.
use crate::dispatch::prepared::{CallResult, ProviderKey, RegistryError};
use std::sync::atomic::{
    AtomicU64,
    Ordering::{Acquire, Relaxed, Release},
};
#[derive(Debug, Clone)]
pub struct Count {
    pub key: Option<ProviderKey>,
    pub ordinal: usize,
    pub calls: u64,
    pub returned: u64,
    pub unknown: u64,
    pub refused: u64,
}
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub entries: Vec<Count>,
    pub last_ordinal: Option<usize>,
    pub last_thread: Option<u64>,
    pub invalid_ordinals: u64,
}
struct Counters {
    calls: AtomicU64,
    returned: AtomicU64,
    unknown: AtomicU64,
    refused: AtomicU64,
}
pub struct Metrics {
    keys: Vec<Option<ProviderKey>>,
    counts: Vec<Counters>,
    last: AtomicU64,
    invalid: AtomicU64,
}
impl Metrics {
    pub fn new(keys: Vec<Option<ProviderKey>>) -> Result<Self, &'static str> {
        if keys.len() > 65536
            || keys
                .iter()
                .flatten()
                .any(|k| k.library.len() > 256 || k.module.len() > 256)
        {
            return Err("metrics capacity");
        }
        let counts = keys
            .iter()
            .map(|_| Counters {
                calls: 0.into(),
                returned: 0.into(),
                unknown: 0.into(),
                refused: 0.into(),
            })
            .collect();
        Ok(Self {
            keys,
            counts,
            last: 0.into(),
            invalid: 0.into(),
        })
    }
    pub fn begin(&self, ordinal: u32, thread: u64) {
        // One atomic pair prevents attributing another producer's ordinal to this thread.
        let pair = if ordinal < 65536 {
            u32::try_from(thread).map_or(0, |t| (u64::from(t) << 32) | (u64::from(ordinal) + 1))
        } else {
            0
        };
        self.last.store(pair, Relaxed);
        if let Some(c) = self.counts.get(ordinal as usize) {
            c.calls.fetch_add(1, Relaxed);
        } else {
            self.invalid.fetch_add(1, Relaxed);
        }
    }
    pub fn complete(&self, ordinal: u32, result: Result<CallResult, RegistryError>) {
        if let Some(c) = self.counts.get(ordinal as usize) {
            let counter = match result {
                Ok(CallResult::Returned) => &c.returned,
                Err(RegistryError::Missing) => &c.unknown,
                Ok(CallResult::StopRequested) => return,
                _ => &c.refused,
            };
            counter.fetch_add(1, Release);
        }
    }
    /// Completion publication precedes the calls read; still an observation window, not a process checkpoint.
    pub fn snapshot(&self) -> Snapshot {
        let last = self.last.load(Relaxed);
        Snapshot {
            entries: self
                .counts
                .iter()
                .enumerate()
                .filter_map(|(i, c)| {
                    let returned = c.returned.load(Acquire);
                    let unknown = c.unknown.load(Acquire);
                    let refused = c.refused.load(Acquire);
                    let calls = c.calls.load(Acquire);
                    (calls > 0).then(|| Count {
                        key: self.keys[i].clone(),
                        ordinal: i,
                        calls,
                        returned,
                        unknown,
                        refused,
                    })
                })
                .collect(),
            last_ordinal: (last & 0xffff_ffff).checked_sub(1).map(|n| n as usize),
            last_thread: (last != 0).then_some(last >> 32),
            invalid_ordinals: self.invalid.load(Relaxed),
        }
    }
}
