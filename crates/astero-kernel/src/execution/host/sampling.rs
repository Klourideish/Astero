//! Bounded actual-PC observations. Capture is private to the native supervisor; no execution authority.
use std::sync::Mutex;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Domain {
    GuestImage,
    ImportLanding,
    RuntimeBridge,
    HostOrHle,
}
#[derive(Clone, Debug)]
pub struct Range {
    pub start: u64,
    pub size: u64,
    pub domain: Domain,
    pub source: u32,
}
#[derive(Clone, Debug)]
pub struct Sample {
    pub thread: u32,
    pub rip: u64,
    pub rsp: u64,
    pub domain: Domain,
    pub source: Option<u32>,
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub samples: Vec<Sample>,
    pub total: u64,
    pub dropped: u64,
    pub suspends: u64,
    pub resumes: u64,
}
pub struct Sampler {
    interval_ms: u64,
    capacity: usize,
    ranges: Vec<Range>,
    state: Mutex<Snapshot>,
}
impl Sampler {
    pub fn new(
        interval_ms: u64,
        capacity: usize,
        ranges: Vec<Range>,
    ) -> Result<Self, &'static str> {
        if !(5..=1000).contains(&interval_ms)
            || !(1..=4096).contains(&capacity)
            || ranges.len() > 65536
            || ranges
                .iter()
                .any(|r| r.size == 0 || r.start.checked_add(r.size).is_none())
        {
            return Err("invalid sampling limits");
        }
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(capacity)
            .map_err(|_| "sample allocation")?;
        Ok(Self {
            interval_ms,
            capacity,
            ranges,
            state: Mutex::new(Snapshot {
                samples,
                ..Default::default()
            }),
        })
    }
    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }
    pub fn classify(&self, rip: u64) -> (Domain, Option<u32>) {
        self.ranges
            .iter()
            .find(|r| rip >= r.start && rip - r.start < r.size)
            .map_or((Domain::HostOrHle, None), |r| (r.domain, Some(r.source)))
    }
    /// Called only after the suspension is paired with a successful resume.
    pub(super) fn record(&self, thread: u32, rip: u64, rsp: u64, bridge: bool) {
        let (mut domain, source) = self.classify(rip);
        if domain == Domain::HostOrHle && bridge {
            domain = Domain::RuntimeBridge;
        }
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.total += 1;
        s.suspends += 1;
        s.resumes += 1;
        if s.samples.len() < self.capacity {
            s.samples.push(Sample {
                thread,
                rip,
                rsp,
                domain,
                source,
            });
        } else {
            s.dropped += 1;
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_classification_preserves_unknown() {
        let p = Sampler::new(
            5,
            2,
            vec![
                Range {
                    start: 4096,
                    size: 4096,
                    domain: Domain::GuestImage,
                    source: 3,
                },
                Range {
                    start: 8192,
                    size: 4096,
                    domain: Domain::ImportLanding,
                    source: 3,
                },
            ],
        )
        .unwrap();
        assert_eq!(p.classify(8191), (Domain::GuestImage, Some(3)));
        assert_eq!(p.classify(8192), (Domain::ImportLanding, Some(3)));
        assert_eq!(p.classify(12288), (Domain::HostOrHle, None));
    }
    #[test]
    fn overflow_and_capacity_refuse() {
        assert!(Sampler::new(4, 1, vec![]).is_err());
        assert!(Sampler::new(5, 0, vec![]).is_err());
        assert!(
            Sampler::new(
                5,
                1,
                vec![Range {
                    start: u64::MAX,
                    size: 1,
                    domain: Domain::GuestImage,
                    source: 0
                }]
            )
            .is_err()
        );
    }
    #[test]
    fn aggregate_after_capacity_is_explicit() {
        let p = Sampler::new(5, 1, vec![]).unwrap();
        p.record(1, 8, 16, true);
        p.record(1, 9, 16, false);
        let s = p.snapshot();
        assert_eq!((s.total, s.dropped, s.samples.len()), (2, 1, 1));
        assert_eq!(s.samples[0].domain, Domain::RuntimeBridge);
        assert_eq!(s.suspends, s.resumes);
    }
}
