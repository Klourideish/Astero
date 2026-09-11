//! Runtime-owned, bounded null-sink ports and AudioOut2 queues. No guest addresses.
use astero_timing::{
    scheduler::{Completion, Label, Scheduler, Ticket},
    time::{Deadline, Span, Tick},
};
use std::sync::{Mutex, MutexGuard};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Handle,
    Capacity,
    Busy,
    Interrupted,
    NotInitialized,
}
pub type Result<T = ()> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Legacy,
    Context,
    Port,
    User,
}
/// Logical output policy, independent of input port channel counts and host hardware.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeakerTopology {
    Stereo,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub frames: u32,
    pub rate: u32,
    pub format: u32,
    pub channels: u8,
    pub sample_bytes: u8,
    pub port_type: u32,
    pub depth: usize,
}
impl Config {
    pub fn pcm(frames: u32, rate: u32, format: u32, port_type: u32) -> Result<Self> {
        let (channels, sample_bytes) = match format {
            0 => (1, 2),
            1 => (2, 2),
            2 | 6 => (8, 2),
            3 => (1, 4),
            4 => (2, 4),
            5 | 7 => (8, 4),
            _ => return Err(Error::Invalid),
        };
        if frames == 0
            || frames > 65536
            || rate != 48000
            || !matches!(port_type, 0..=4 | 10 | 126 | 127)
        {
            return Err(Error::Invalid);
        }
        Ok(Self {
            frames,
            rate,
            format,
            channels,
            sample_bytes,
            port_type,
            depth: 1,
        })
    }
    pub fn context(frames: u32, depth: usize) -> Result<Self> {
        let mut c = Self::pcm(frames, 48000, 4, 0)?;
        if !(1..=32).contains(&depth) {
            return Err(Error::Invalid);
        }
        c.depth = depth;
        Ok(c)
    }
    pub fn bytes(&self) -> usize {
        self.frames as usize * self.channels as usize * self.sample_bytes as usize
    }
    fn period(&self) -> Span {
        Span::from_nanos((self.frames as u64 * 1_000_000_000).div_ceil(self.rate as u64))
    }
}
#[derive(Clone, Debug)]
pub struct Record {
    pub id: u64,
    pub kind: Kind,
    pub parent: Option<u64>,
    pub config: Config,
    pub volume: [i32; 8],
    pub submitted: u64,
    pub completed: u64,
    pub bytes: u64,
    pub pending: usize,
    pub next_deadline: Option<u64>,
}
struct Pending {
    ticket: Ticket,
    deadline: Tick,
    _pcm: Vec<u8>,
}
struct Object {
    record: Record,
    queue: Vec<Pending>,
}
struct State {
    retired: Vec<Record>,
    objects: Vec<Object>,
    next: u64,
    initialized: bool,
    mastering: bool,
    speaker_queries: u64,
    topology: SpeakerTopology,
    stopped: bool,
    limit: Option<Deadline>,
    submitted: u64,
    completed: u64,
    cancelled: u64,
    bytes: u64,
    waits: u64,
    opened: u64,
    closed: u64,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub retired: Vec<Record>,
    pub backend: &'static str,
    pub initialized: bool,
    pub mastering: bool,
    pub speaker_queries: u64,
    pub topology: SpeakerTopology,
    pub stopped: bool,
    pub objects: Vec<Record>,
    pub submitted: u64,
    pub completed: u64,
    pub cancelled: u64,
    pub bytes: u64,
    pub waits: u64,
    pub opened: u64,
    pub closed: u64,
}
pub struct AudioService {
    state: Mutex<State>,
    scheduler: Scheduler,
}
impl AudioService {
    pub fn new(scheduler: Scheduler) -> Self {
        Self {
            scheduler,
            state: Mutex::new(State {
                retired: vec![],
                objects: vec![],
                next: 1,
                initialized: false,
                mastering: false,
                speaker_queries: 0,
                topology: SpeakerTopology::Stereo,
                stopped: false,
                limit: None,
                submitted: 0,
                completed: 0,
                cancelled: 0,
                bytes: 0,
                waits: 0,
                opened: 0,
                closed: 0,
            }),
        }
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    fn now(&self) -> Result<Tick> {
        self.scheduler.now().map_err(|_| Error::Interrupted)
    }
    fn live(&self, s: &State) -> Result {
        if s.stopped
            || s.limit
                .is_some_and(|d| self.now().is_ok_and(|n| d.is_due(n)))
        {
            Err(Error::Interrupted)
        } else {
            Ok(())
        }
    }
    fn index(s: &State, id: u64, kind: Kind) -> Result<usize> {
        s.objects
            .iter()
            .position(|o| o.record.id == id && o.record.kind == kind)
            .ok_or(Error::Handle)
    }
    fn reap(&self, s: &mut State) {
        for o in &mut s.objects {
            while o.queue.first().is_some_and(|p| {
                matches!(p.ticket.poll(), Some(Completion::Fired(_)))
                    && self.now().is_ok_and(|n| n >= p.deadline)
            }) {
                o.queue.remove(0);
                o.record.completed += 1;
                s.completed += 1;
            }
        }
    }
    pub fn arm(&self, d: Deadline) {
        self.lock().limit = Some(d)
    }
    pub fn initialize(&self) -> Result {
        let mut s = self.lock();
        self.live(&s)?;
        s.initialized = true;
        Ok(())
    }
    /// Global query: firmware takes output + selector, never a port handle.
    pub fn speaker_ready(&self) -> Result<SpeakerTopology> {
        let s = self.lock();
        self.live(&s)?;
        if !s.initialized {
            return Err(Error::NotInitialized);
        }
        Ok(s.topology)
    }
    pub fn speaker_queried(&self) -> Result {
        let mut s = self.lock();
        self.live(&s)?;
        s.speaker_queries = s.speaker_queries.saturating_add(1);
        Ok(())
    }
    pub fn mastering(&self) -> Result {
        let mut s = self.lock();
        self.live(&s)?;
        s.mastering = true;
        Ok(())
    }
    pub fn create(&self, kind: Kind, parent: Option<u64>, config: Config) -> Result<u64> {
        let mut s = self.lock();
        self.live(&s)?;
        if let Some(p) = parent {
            Self::index(&s, p, Kind::Context)?;
        }
        let expected = Config::pcm(config.frames, config.rate, config.format, config.port_type)?;
        if expected.channels != config.channels
            || expected.sample_bytes != config.sample_bytes
            || !(1..=32).contains(&config.depth)
            || (kind == Kind::Port && parent.is_none())
        {
            return Err(Error::Invalid);
        }
        let maximum = if kind == Kind::Legacy { 16 } else { 32 };
        if s.objects.iter().filter(|o| o.record.kind == kind).count() >= maximum {
            return Err(Error::Capacity);
        }
        // Bound cumulative identities and diagnostics without handle reuse.
        if s.opened >= 4096 {
            return Err(Error::Capacity);
        }
        s.objects.try_reserve(1).map_err(|_| Error::Capacity)?;
        let mut queue = Vec::new();
        queue
            .try_reserve_exact(config.depth)
            .map_err(|_| Error::Capacity)?;
        let id = s.next;
        s.next = id.checked_add(1).ok_or(Error::Capacity)?;
        s.opened += 1;
        s.objects.push(Object {
            record: Record {
                id,
                kind,
                parent,
                config,
                volume: [127; 8],
                submitted: 0,
                completed: 0,
                bytes: 0,
                pending: 0,
                next_deadline: None,
            },
            queue,
        });
        Ok(id)
    }
    pub fn record(&self, id: u64, kind: Kind) -> Result<Record> {
        let mut s = self.lock();
        self.live(&s)?;
        self.reap(&mut s);
        let i = Self::index(&s, id, kind)?;
        let o = &s.objects[i];
        let mut r = o.record.clone();
        r.pending = o.queue.len();
        r.next_deadline = o.queue.first().map(|p| p.deadline.as_nanos());
        Ok(r)
    }
    pub fn close(&self, id: u64, kind: Kind) -> Result {
        let mut s = self.lock();
        self.live(&s)?;
        let i = Self::index(&s, id, kind)?;
        if s.objects.iter().any(|o| o.record.parent == Some(id)) {
            return Err(Error::Busy);
        }
        let o = s.objects.remove(i);
        s.retired.push(o.record.clone());
        for p in o.queue {
            let _ = self.scheduler.cancel(&p.ticket);
            s.cancelled += 1;
        }
        s.closed += 1;
        Ok(())
    }
    pub fn volume(&self, id: u64, mask: u32, values: [i32; 8]) -> Result {
        if mask & !255 != 0 {
            return Err(Error::Invalid);
        }
        let mut s = self.lock();
        self.live(&s)?;
        let i = Self::index(&s, id, Kind::Legacy)?;
        for (c, v) in values.iter().enumerate() {
            if mask & (1 << c) != 0 {
                s.objects[i].record.volume[c] = *v
            }
        }
        Ok(())
    }
    /// Atomically admits a pre-copied batch. No guest reads or central lock during waiting.
    pub fn submit(&self, items: Vec<(u64, Vec<u8>)>, kind: Kind, blocking: bool) -> Result {
        if items.is_empty()
            || items.len() > 25
            || !matches!(kind, Kind::Legacy | Kind::Context)
            || items.iter().map(|(_, p)| p.len()).sum::<usize>() > 32 * 1024 * 1024
        {
            return Err(Error::Invalid);
        }
        loop {
            let mut s = self.lock();
            self.live(&s)?;
            self.reap(&mut s);
            let mut indices = Vec::new();
            let mut wait = None;
            for (id, pcm) in &items {
                let i = Self::index(&s, *id, kind)?;
                if indices.contains(&i) {
                    return Err(Error::Invalid);
                }
                let o = &s.objects[i];
                if kind == Kind::Legacy && pcm.len() != o.record.config.bytes() {
                    return Err(Error::Invalid);
                }
                if o.queue.len() >= o.record.config.depth {
                    wait = Some(o.queue[0].ticket.clone());
                }
                indices.push(i);
            }
            if let Some(ticket) = wait {
                if !blocking {
                    return Err(Error::Busy);
                }
                s.waits += 1;
                drop(s);
                if !matches!(ticket.wait(), Completion::Fired(_)) {
                    return Err(Error::Interrupted);
                }
                continue;
            }
            let now = self.now()?;
            let mut prepared: Vec<(usize, Pending)> = Vec::new();
            // Compute every deadline before installing any ticket.
            let ends: Vec<_> = indices
                .iter()
                .map(|i| {
                    let o = &s.objects[*i];
                    o.queue
                        .last()
                        .map_or(now, |p| p.deadline.max(now))
                        .checked_add(o.record.config.period())
                        .map_err(|_| Error::Invalid)
                })
                .collect::<Result<_>>()?;
            prepared
                .try_reserve_exact(indices.len())
                .map_err(|_| Error::Capacity)?;
            for ((i, (_, pcm)), end) in indices.into_iter().zip(items).zip(ends) {
                let deadline = s
                    .limit
                    .map_or(Deadline::at(end), |d| d.min(Deadline::at(end)));
                let ticket = match self.scheduler.schedule(
                    deadline,
                    Label::new("audio null-sink period").expect("constant"),
                ) {
                    Ok(t) => t,
                    Err(_) => {
                        for (_, p) in prepared {
                            let _ = self.scheduler.cancel(&p.ticket);
                        }
                        return Err(Error::Capacity);
                    }
                };
                prepared.push((
                    i,
                    Pending {
                        ticket,
                        deadline: end,
                        _pcm: pcm,
                    },
                ));
            }
            for (i, p) in prepared {
                s.submitted += 1;
                s.bytes += p._pcm.len() as u64;
                let o = &mut s.objects[i];
                o.record.submitted += 1;
                o.record.bytes += p._pcm.len() as u64;
                o.queue.push(p);
            }
            return Ok(());
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let mut s = self.lock();
        self.reap(&mut s);
        Snapshot {
            retired: s.retired.clone(),
            backend: "TimedNullSink (no host playback)",
            initialized: s.initialized,
            mastering: s.mastering,
            speaker_queries: s.speaker_queries,
            topology: s.topology,
            stopped: s.stopped,
            objects: s
                .objects
                .iter()
                .map(|o| {
                    let mut r = o.record.clone();
                    r.pending = o.queue.len();
                    r.next_deadline = o.queue.first().map(|p| p.deadline.as_nanos());
                    r
                })
                .collect(),
            submitted: s.submitted,
            completed: s.completed,
            cancelled: s.cancelled,
            bytes: s.bytes,
            waits: s.waits,
            opened: s.opened,
            closed: s.closed,
        }
    }
    pub fn shutdown(&self) {
        let mut s = self.lock();
        if s.stopped {
            return;
        }
        s.stopped = true;
        self.reap(&mut s);
        let objects = std::mem::take(&mut s.objects);
        for o in objects {
            s.retired.push(o.record.clone());
            for p in o.queue {
                let _ = self.scheduler.cancel(&p.ticket);
                s.cancelled += 1;
            }
            s.closed += 1;
        }
    }
}
impl Drop for AudioService {
    fn drop(&mut self) {
        self.shutdown()
    }
}
