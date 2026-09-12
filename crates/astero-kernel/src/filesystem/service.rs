//! Bounded descriptor owner. Host I/O is serialized per descriptor; no host pointer is a guest ID.
use super::mounts::Mounts;
use crate::process::output::Output;
use std::{
    collections::BTreeMap,
    fs::{File, Metadata, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotFound,
    Permission,
    Invalid,
    BadDescriptor,
    Capacity,
    Io,
    Stopped,
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        use std::io::ErrorKind::*;
        match e.kind() {
            NotFound => Self::NotFound,
            PermissionDenied => Self::Permission,
            InvalidInput => Self::Invalid,
            _ => Self::Io,
        }
    }
}
impl Error {
    pub fn errno(self) -> u32 {
        match self {
            Self::NotFound => 2,
            Self::Permission => 13,
            Self::Invalid => 22,
            Self::BadDescriptor => 9,
            Self::Capacity => 24,
            Self::Io => 5,
            Self::Stopped => 4,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mode {
    pub read: bool,
    pub write: bool,
    pub create: bool,
    pub truncate: bool,
    pub append: bool,
}
impl Mode {
    pub fn flags(f: u64) -> Result<Self, Error> {
        if f & !0x60b != 0 || f & 3 == 3 {
            return Err(Error::Invalid);
        }
        Ok(Self {
            read: f & 3 != 1,
            write: f & 3 != 0,
            create: f & 0x200 != 0,
            truncate: f & 0x400 != 0,
            append: f & 8 != 0,
        })
    }
    pub fn parse(s: &str) -> Result<Self, Error> {
        let mut c = s.chars();
        let first = c.next().ok_or(Error::Invalid)?;
        let mut plus = false;
        let mut binary = false;
        for x in c {
            match x {
                '+' if !plus => plus = true,
                'b' if !binary => binary = true,
                _ => return Err(Error::Invalid),
            }
        }
        match first {
            'r' => Self::flags(if plus { 2 } else { 0 }),
            'w' => Self::flags(0x600 | if plus { 2 } else { 1 }),
            'a' => Self::flags(0x208 | if plus { 2 } else { 1 }),
            _ => Err(Error::Invalid),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub open: usize,
    pub streams: usize,
    pub peak: usize,
    pub opens: u64,
    pub closes: u64,
    pub read: u64,
    pub written: u64,
    pub seeks: u64,
    pub stats: u64,
    pub failures: u64,
    pub last_path: String,
}
struct Entry {
    _guest_path: String,
    _host_path: Option<std::path::PathBuf>,
    file: Option<File>,
    mode: Mode,
    eof: bool,
    error: bool,
    closed: bool,
}
struct State {
    entries: BTreeMap<u32, Arc<Mutex<Entry>>>,
    streams: BTreeMap<u64, u32>,
    next: u32,
    metrics: Snapshot,
    stopped: bool,
}
pub struct Filesystem {
    mounts: Mutex<Option<Mounts>>,
    state: Mutex<State>,
    output: Arc<Mutex<Output>>,
    maximum: usize,
}
impl Filesystem {
    pub fn new(maximum: usize, output: Arc<Mutex<Output>>) -> Self {
        let mut entries = BTreeMap::new();
        for fd in 0..3 {
            entries.insert(
                fd,
                Arc::new(Mutex::new(Entry {
                    _guest_path: format!("<stdio:{fd}>"),
                    _host_path: None,
                    file: None,
                    mode: Mode::flags(if fd == 0 { 0 } else { 1 }).unwrap(),
                    eof: false,
                    error: false,
                    closed: false,
                })),
            );
        }
        Self {
            mounts: Mutex::new(None),
            state: Mutex::new(State {
                entries,
                streams: BTreeMap::new(),
                next: 3,
                metrics: Snapshot::default(),
                stopped: false,
            }),
            output,
            maximum,
        }
    }
    pub fn configure(&self, mounts: Mounts) -> Result<(), Error> {
        let s = self.state.lock().unwrap();
        if s.metrics.opens != 0 || s.stopped {
            return Err(Error::Invalid);
        }
        *self.mounts.lock().unwrap() = Some(mounts);
        Ok(())
    }
    fn path(&self, p: &str, w: bool) -> Result<std::path::PathBuf, Error> {
        {
            let mut s = self.state.lock().unwrap();
            if s.stopped {
                return Err(Error::Stopped);
            }
            s.metrics.last_path = p.chars().take(256).collect();
        }
        let r = self
            .mounts
            .lock()
            .unwrap()
            .as_ref()
            .ok_or(Error::NotFound)
            .and_then(|m| m.resolve(p, w));
        if r.is_err() {
            self.state.lock().unwrap().metrics.failures += 1;
        }
        r
    }
    pub fn open(&self, p: &str, mode: Mode) -> Result<u32, Error> {
        let path = self.path(p, mode.write)?;
        if path.exists() && !path.is_file() {
            return Err(Error::Invalid);
        }
        let file = OpenOptions::new()
            .read(mode.read)
            .write(mode.write)
            .create(mode.create)
            .truncate(mode.truncate)
            .append(mode.append)
            .open(&path)
            .map_err(|e| {
                self.state.lock().unwrap().metrics.failures += 1;
                Error::from(e)
            })?;
        let mut s = self.state.lock().unwrap();
        if s.stopped {
            return Err(Error::Stopped);
        }
        if s.entries.len() >= self.maximum {
            return Err(Error::Capacity);
        }
        let fd = s.next;
        s.next = s.next.checked_add(1).ok_or(Error::Capacity)?;
        s.entries.insert(
            fd,
            Arc::new(Mutex::new(Entry {
                _guest_path: p.into(),
                _host_path: Some(path),
                file: Some(file),
                mode,
                eof: false,
                error: false,
                closed: false,
            })),
        );
        s.metrics.opens += 1;
        s.metrics.peak = s.metrics.peak.max(s.entries.len());
        Ok(fd)
    }
    fn entry(&self, fd: u32) -> Result<Arc<Mutex<Entry>>, Error> {
        let s = self.state.lock().unwrap();
        if s.stopped {
            return Err(Error::Stopped);
        }
        s.entries.get(&fd).cloned().ok_or(Error::BadDescriptor)
    }
    pub fn close(&self, fd: u32) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap();
        let e = s.entries.remove(&fd).ok_or(Error::BadDescriptor)?;
        let mut e = e.lock().unwrap();
        e.closed = true;
        e.file.take();
        s.streams.retain(|_, v| *v != fd);
        s.metrics.closes += 1;
        Ok(())
    }
    pub fn bind_stream(&self, token: u64, fd: u32) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap();
        if token == 0 || s.streams.contains_key(&token) || !s.entries.contains_key(&fd) {
            return Err(Error::Invalid);
        }
        s.streams.insert(token, fd);
        Ok(())
    }
    pub fn stream(&self, t: u64) -> Result<u32, Error> {
        self.state
            .lock()
            .unwrap()
            .streams
            .get(&t)
            .copied()
            .ok_or(Error::BadDescriptor)
    }
    pub fn read(&self, fd: u32, b: &mut [u8], at: Option<u64>) -> Result<usize, Error> {
        let e = self.entry(fd)?;
        let mut e = e.lock().unwrap();
        if e.closed || !e.mode.read {
            e.error = true;
            return Err(Error::BadDescriptor);
        }
        let result = if let Some(f) = e.file.as_mut() {
            let old = if at.is_some() {
                Some(f.stream_position()?)
            } else {
                None
            };
            if let Some(p) = at {
                f.seek(SeekFrom::Start(p))?;
            }
            let r = f.read(b);
            if let Some(p) = old {
                f.seek(SeekFrom::Start(p))?;
            }
            r.map_err(Error::from)
        } else {
            Ok(0)
        };
        match result {
            Ok(n) => {
                e.eof = n < b.len();
                drop(e);
                self.state.lock().unwrap().metrics.read += n as u64;
                Ok(n)
            }
            Err(x) => {
                e.error = true;
                Err(x)
            }
        }
    }
    pub fn write(&self, fd: u32, b: &[u8], at: Option<u64>) -> Result<usize, Error> {
        let e = self.entry(fd)?;
        let mut e = e.lock().unwrap();
        if at.is_some() && e.mode.append {
            return Err(Error::Invalid);
        }
        if e.closed || !e.mode.write {
            e.error = true;
            return Err(Error::BadDescriptor);
        }
        let result = if let Some(f) = e.file.as_mut() {
            let old = if at.is_some() {
                Some(f.stream_position()?)
            } else {
                None
            };
            if let Some(p) = at {
                f.seek(SeekFrom::Start(p))?;
            }
            let r = f.write(b);
            if let Some(p) = old {
                f.seek(SeekFrom::Start(p))?;
            }
            r.map_err(Error::from)
        } else {
            self.output
                .lock()
                .unwrap()
                .append(b)
                .map(|_| b.len())
                .map_err(|_| Error::Capacity)
        };
        match result {
            Ok(n) => {
                drop(e);
                self.state.lock().unwrap().metrics.written += n as u64;
                Ok(n)
            }
            Err(x) => {
                e.error = true;
                Err(x)
            }
        }
    }
    pub fn seek(&self, fd: u32, offset: i64, whence: u64) -> Result<u64, Error> {
        let e = self.entry(fd)?;
        let mut e = e.lock().unwrap();
        if e.closed {
            return Err(Error::BadDescriptor);
        }
        let from = match whence {
            0 => SeekFrom::Start(u64::try_from(offset).map_err(|_| Error::Invalid)?),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return Err(Error::Invalid),
        };
        let p = e.file.as_mut().ok_or(Error::BadDescriptor)?.seek(from)?;
        e.eof = false;
        drop(e);
        self.state.lock().unwrap().metrics.seeks += 1;
        Ok(p)
    }
    pub fn indicators(&self, fd: u32) -> Result<(bool, bool), Error> {
        let e = self.entry(fd)?;
        let e = e.lock().unwrap();
        Ok((e.eof, e.error))
    }
    pub fn clear(&self, fd: u32) -> Result<(), Error> {
        let e = self.entry(fd)?;
        let mut e = e.lock().unwrap();
        e.eof = false;
        e.error = false;
        Ok(())
    }
    pub fn flush(&self, fd: u32) -> Result<(), Error> {
        let e = self.entry(fd)?;
        let mut e = e.lock().unwrap();
        if let Some(f) = e.file.as_mut() {
            f.flush()?;
        }
        Ok(())
    }
    pub fn metadata(&self, p: &str) -> Result<Metadata, Error> {
        let path = self.path(p, false)?;
        let m = std::fs::metadata(path)?;
        self.state.lock().unwrap().metrics.stats += 1;
        Ok(m)
    }
    pub fn fmetadata(&self, fd: u32) -> Result<Metadata, Error> {
        let e = self.entry(fd)?;
        let e = e.lock().unwrap();
        let m = e.file.as_ref().ok_or(Error::BadDescriptor)?.metadata()?;
        drop(e);
        self.state.lock().unwrap().metrics.stats += 1;
        Ok(m)
    }
    pub fn truncate(&self, fd: u32, size: u64) -> Result<(), Error> {
        let e = self.entry(fd)?;
        let e = e.lock().unwrap();
        if !e.mode.write {
            return Err(Error::Permission);
        }
        e.file.as_ref().ok_or(Error::BadDescriptor)?.set_len(size)?;
        Ok(())
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.state.lock().unwrap();
        let mut m = s.metrics.clone();
        m.open = s.entries.keys().filter(|k| **k >= 3).count();
        m.streams = s.streams.len();
        m
    }
    pub fn shutdown(&self) {
        let mut s = self.state.lock().unwrap();
        s.stopped = true;
        for e in s.entries.values() {
            let mut e = e.lock().unwrap();
            e.closed = true;
            e.file.take();
        }
        s.entries.clear();
        s.streams.clear();
    }
}
