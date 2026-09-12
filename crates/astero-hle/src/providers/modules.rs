//! Owned module request state. Provider identity is separate from a sysmodule request.
use crate::dispatch::prepared::ProviderKey;
use std::sync::{Arc, Mutex};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Backing {
    Hle,
    Artifact { source: String },
    Partial,
}
#[derive(Clone, Debug)]
pub struct Declaration {
    pub id: u16,
    pub name: String,
    pub backing: Backing,
    pub providers: Vec<ProviderKey>,
    pub dependencies: Vec<u16>,
}
#[derive(Clone, Debug)]
pub struct Record {
    pub declaration: Declaration,
    pub references: u32,
    pub leases: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Unknown,
    Unsupported,
    Conflict,
    MissingProvider,
    Dependency,
    Busy,
    Capacity,
    Stopped,
}
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub records: Vec<Record>,
    pub loads: u64,
    pub failed: u64,
    pub unloads: u64,
    pub last: Option<u16>,
    pub stopped: bool,
}
pub struct Modules {
    state: Mutex<Snapshot>,
    maximum: usize,
}
impl Modules {
    pub fn new(maximum: usize) -> Self {
        Self {
            state: Mutex::new(Snapshot::default()),
            maximum,
        }
    }
    pub fn declare(&self, d: Declaration, available: &[ProviderKey]) -> Result<(), Error> {
        if d.name.len() > 128
            || d.dependencies.len() > 16
            || d.providers.len() > 512
            || d.providers.is_empty()
        {
            return Err(Error::Capacity);
        }
        if d.providers
            .iter()
            .enumerate()
            .any(|(i, k)| d.providers[..i].contains(k))
        {
            return Err(Error::Conflict);
        }
        if d.backing == Backing::Hle && d.providers.iter().any(|k| !available.contains(k)) {
            return Err(Error::MissingProvider);
        }
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.stopped {
            return Err(Error::Stopped);
        }
        if s.records.iter().any(|r| r.declaration.id == d.id) {
            return Err(Error::Conflict);
        }
        if s.records.len() >= self.maximum {
            return Err(Error::Capacity);
        }
        s.records.push(Record {
            declaration: d,
            references: 0,
            leases: 0,
        });
        Ok(())
    }
    fn closure(
        records: &[Record],
        id: u16,
        path: &mut Vec<u16>,
        out: &mut Vec<usize>,
    ) -> Result<(), Error> {
        if path.contains(&id) {
            return Err(Error::Dependency);
        }
        let i = records
            .iter()
            .position(|r| r.declaration.id == id)
            .ok_or(Error::Unknown)?;
        if records[i].declaration.backing != Backing::Hle {
            return Err(Error::Unsupported);
        }
        if out.contains(&i) {
            return Ok(());
        }
        path.push(id);
        for d in &records[i].declaration.dependencies {
            Self::closure(records, *d, path, out)?;
        }
        path.pop();
        out.push(i);
        Ok(())
    }
    pub fn load(&self, id: u16) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        s.last = Some(id);
        s.loads = s.loads.saturating_add(1);
        let result = (|| {
            if s.stopped {
                return Err(Error::Stopped);
            }
            let mut indexes = vec![];
            Self::closure(&s.records, id, &mut vec![], &mut indexes)?;
            if indexes.iter().any(|i| s.records[*i].references == u32::MAX) {
                return Err(Error::Capacity);
            }
            for i in indexes {
                s.records[i].references += 1;
            }
            Ok(())
        })();
        if result.is_err() {
            s.failed = s.failed.saturating_add(1);
        }
        result
    }
    pub fn is_loaded(&self, id: u16) -> bool {
        let s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        !s.stopped
            && s.records
                .iter()
                .any(|r| r.declaration.id == id && r.references > 0)
    }
    pub fn unload(&self, id: u16) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.stopped {
            return Err(Error::Stopped);
        }
        let mut indexes = vec![];
        Self::closure(&s.records, id, &mut vec![], &mut indexes)?;
        if s.records
            .iter()
            .any(|r| r.references > 0 && r.declaration.dependencies.contains(&id))
        {
            return Err(Error::Busy);
        }
        if indexes.iter().any(|i| s.records[*i].references == 0) {
            return Err(Error::Unknown);
        }
        if indexes.iter().any(|i| s.records[*i].leases > 0) {
            return Err(Error::Busy);
        }
        for i in indexes {
            s.records[i].references -= 1;
        }
        s.unloads = s.unloads.saturating_add(1);
        Ok(())
    }
    pub fn lease(self: &Arc<Self>, id: u16) -> Result<Lease, Error> {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.stopped {
            return Err(Error::Stopped);
        }
        let r = s
            .records
            .iter_mut()
            .find(|r| r.declaration.id == id && r.references > 0)
            .ok_or(Error::Unknown)?;
        r.leases = r.leases.checked_add(1).ok_or(Error::Capacity)?;
        Ok(Lease {
            owner: self.clone(),
            id,
        })
    }
    /// Gate a real HLE handler behind this module's load state and execution lease.
    /// Artifact declarations cannot be turned into host-callable implementations.
    pub fn publish(
        self: &Arc<Self>,
        id: u16,
        mut entry: crate::dispatch::prepared::Registration,
    ) -> Result<crate::dispatch::prepared::Registration, Error> {
        use crate::dispatch::prepared::{CallResult, ProviderKind};
        {
            let s = self.state.lock().unwrap_or_else(|p| p.into_inner());
            let r = s
                .records
                .iter()
                .find(|r| r.declaration.id == id)
                .ok_or(Error::Unknown)?;
            if r.declaration.backing != Backing::Hle
                || entry.kind != ProviderKind::HleImplementation
                || !r.declaration.providers.contains(&entry.key)
            {
                return Err(Error::Conflict);
            }
        }
        let handler = entry.handler.take().ok_or(Error::MissingProvider)?;
        let owner = self.clone();
        entry.handler = Some(Box::new(move |f, m| {
            let Ok(_lease) = owner.lease(id) else {
                return CallResult::Unsupported;
            };
            handler(f, m)
        }));
        Ok(entry)
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }
    pub fn shutdown(&self) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap_or_else(|p| p.into_inner());
        if s.records.iter().any(|r| r.leases > 0) {
            return Err(Error::Busy);
        }
        s.stopped = true;
        s.records.clear();
        Ok(())
    }
}
pub struct Lease {
    owner: Arc<Modules>,
    id: u16,
}
impl Drop for Lease {
    fn drop(&mut self) {
        let mut s = self.owner.state.lock().unwrap_or_else(|p| p.into_inner());
        if let Some(r) = s.records.iter_mut().find(|r| r.declaration.id == self.id) {
            r.leases -= 1;
        }
    }
}
