//! Direct-offset resources and guest VA views. Windows handles remain private to memory.
use astero_memory::mapping::{
    GuestAddress,
    windows_native::{
        GuestRange, NativeBacking, NativeError, NativeImage, NativeObserver, Protection,
    },
};
use std::sync::{Arc, Mutex, MutexGuard};
pub const DIRECT_SIZE: u64 = 16 * 1024 * 1024 * 1024;
pub const COMMIT_LIMIT: u64 = 512 * 1024 * 1024;
pub const VM_BASE: u64 = 0x400000000;
pub const VM_END: u64 = 0x500000000;
const PAGE: u64 = 16384;
#[derive(Debug)]
pub enum Error {
    Invalid,
    Capacity,
    NotFound,
    Busy,
    Unsupported,
    Stopped,
    Native(NativeError),
}
impl From<NativeError> for Error {
    fn from(e: NativeError) -> Self {
        Self::Native(e)
    }
}
pub type Result<T = ()> = std::result::Result<T, Error>;
#[derive(Clone, Debug)]
pub struct Allocation {
    pub offset: u64,
    pub size: u64,
    pub memory_type: i32,
    pub released: bool,
}
struct Resource {
    info: Allocation,
    backing: Arc<NativeBacking>,
}
pub struct View {
    pub direct: bool,
    pub address: u64,
    pub size: u64,
    pub offset: u64,
    pub protection: u32,
    pub name: Vec<u8>,
    image: NativeImage,
}
struct State {
    resources: Vec<Resource>,
    views: Vec<View>,
    observers: Vec<NativeObserver>,
    stopped: bool,
    allocated: u64,
    mapped: u64,
}
#[derive(Clone, Debug)]
pub struct Mapping {
    pub direct: bool,
    pub address: u64,
    pub size: u64,
    pub offset: u64,
    pub protection: u32,
    pub memory_type: i32,
    pub name: Vec<u8>,
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub allocations: Vec<Allocation>,
    pub mappings: Vec<Mapping>,
    pub allocated_total: u64,
    pub mapped_total: u64,
    pub active_native_resources: u64,
    pub release_errors: Vec<u32>,
}
pub struct MemoryResources {
    state: Mutex<State>,
}
fn aligned(v: u64, a: u64) -> Result<u64> {
    v.checked_add(a - 1)
        .map(|x| x & !(a - 1))
        .ok_or(Error::Invalid)
}
fn alignment(a: u64) -> Result<u64> {
    let a = if a == 0 { PAGE } else { a };
    if a < PAGE || !a.is_power_of_two() {
        Err(Error::Invalid)
    } else {
        Ok(a)
    }
}
fn protection(p: u32) -> Result<Protection> {
    if p & 6 == 6 {
        return Err(Error::Unsupported);
    }
    Ok(Protection {
        read: p & 3 != 0,
        write: p & 2 != 0,
        execute: p & 4 != 0,
    })
}
impl Default for MemoryResources {
    fn default() -> Self {
        Self::new()
    }
}
impl MemoryResources {
    pub fn reserve(&self, hint: u64, size: u64, flags: u32, a: u64) -> Result<u64> {
        let a = alignment(a)?.max(65536);
        if size == 0 || !size.is_multiple_of(PAGE) || flags & !16 != 0 {
            return Err(Error::Invalid);
        }
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Stopped);
        }
        if s.views.len() >= 256 || s.observers.len() >= 4096 {
            return Err(Error::Capacity);
        }
        let fixed = flags & 16 != 0;
        let mut address = if fixed {
            hint
        } else {
            aligned(hint.max(VM_BASE), a)?
        };
        if !fixed {
            for v in &s.views {
                if address.checked_add(size).is_some_and(|e| e <= v.address) {
                    break;
                }
                if address < v.address + v.size {
                    address = aligned(v.address + v.size, a)?
                }
            }
        }
        if address < VM_BASE
            || address.checked_add(size).is_none_or(|e| e > VM_END)
            || !address.is_multiple_of(a)
        {
            return Err(Error::Unsupported);
        }
        if s.views
            .iter()
            .any(|v| address < v.address + v.size && v.address < address + size)
        {
            return Err(Error::Busy);
        }
        let image = NativeImage::reserve(
            GuestRange {
                start: GuestAddress(address),
                size,
            },
            VM_END - VM_BASE,
        )?;
        s.observers.push(image.observer());
        s.views.push(View {
            direct: false,
            address,
            size,
            offset: 0,
            protection: 0,
            name: vec![],
            image,
        });
        s.views.sort_by_key(|v| v.address);
        s.mapped = s.mapped.saturating_add(1);
        Ok(address)
    }
    pub fn new() -> Self {
        Self {
            state: Mutex::new(State {
                resources: vec![],
                views: vec![],
                observers: vec![],
                stopped: false,
                allocated: 0,
                mapped: 0,
            }),
        }
    }
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }
    fn available_locked(s: &State, start: u64, end: u64, a: u64) -> Result<(u64, u64)> {
        if start >= end || end > DIRECT_SIZE {
            return Err(Error::Invalid);
        }
        let mut at = aligned(start, a)?;
        let mut best = (0, 0);
        for r in &s.resources {
            let re = r.info.offset + r.info.size;
            if re <= at {
                continue;
            }
            if r.info.offset >= end {
                break;
            }
            if r.info.offset > at && r.info.offset - at > best.1 {
                best = (at, r.info.offset - at)
            }
            at = aligned(re, a)?;
        }
        if end > at && end - at > best.1 {
            best = (at, end - at)
        }
        if best.1 == 0 {
            Err(Error::Capacity)
        } else {
            Ok(best)
        }
    }
    pub fn available(&self, start: u64, end: u64, a: u64) -> Result<(u64, u64)> {
        Self::available_locked(&self.lock(), start, end, alignment(a)?)
    }
    pub fn allocate(
        &self,
        start: u64,
        end: u64,
        size: u64,
        a: u64,
        memory_type: i32,
    ) -> Result<u64> {
        let a = alignment(a)?;
        if size == 0
            || !size.is_multiple_of(PAGE)
            || start >= end
            || end > DIRECT_SIZE
            || memory_type < 0
        {
            return Err(Error::Invalid);
        }
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Stopped);
        }
        if s.resources.len() >= 256
            || s.observers.len() >= 4096
            || s.resources
                .iter()
                .map(|r| r.info.size)
                .sum::<u64>()
                .checked_add(size)
                .is_none_or(|n| n > COMMIT_LIMIT)
        {
            return Err(Error::Capacity);
        }
        let mut at = aligned(start, a)?;
        for r in &s.resources {
            if at.checked_add(size).is_some_and(|e| e <= r.info.offset) {
                break;
            }
            if at < r.info.offset + r.info.size {
                at = aligned(r.info.offset + r.info.size, a)?
            }
        }
        if at.checked_add(size).is_none_or(|e| e > end) {
            return Err(Error::Capacity);
        }
        let backing = NativeBacking::new(size, COMMIT_LIMIT)?;
        s.observers.push(backing.observer());
        s.resources.push(Resource {
            info: Allocation {
                offset: at,
                size,
                memory_type,
                released: false,
            },
            backing,
        });
        s.resources.sort_by_key(|r| r.info.offset);
        s.allocated = s.allocated.saturating_add(1);
        Ok(at)
    }
    pub fn query_direct(&self, offset: u64, next: bool) -> Result<Allocation> {
        let s = self.lock();
        s.resources
            .iter()
            .find(|r| {
                !r.info.released
                    && (offset >= r.info.offset && offset < r.info.offset + r.info.size
                        || next && r.info.offset > offset)
            })
            .map(|r| r.info.clone())
            .ok_or(Error::NotFound)
    }
    pub fn map(
        &self,
        hint: u64,
        size: u64,
        p: u32,
        flags: u32,
        offset: u64,
        a: u64,
    ) -> Result<u64> {
        let prot = protection(p)?;
        let a = alignment(a)?.max(65536);
        if size == 0 || !size.is_multiple_of(PAGE) || flags & !0x10 != 0 {
            return Err(Error::Unsupported);
        }
        let mut s = self.lock();
        if s.stopped {
            return Err(Error::Stopped);
        }
        if s.views.len() >= 256 || s.observers.len() >= 4096 {
            return Err(Error::Capacity);
        }
        let r = s
            .resources
            .iter()
            .find(|r| {
                !r.info.released
                    && offset >= r.info.offset
                    && offset
                        .checked_add(size)
                        .is_some_and(|e| e <= r.info.offset + r.info.size)
            })
            .ok_or(Error::NotFound)?;
        let relative = offset - r.info.offset;
        let backing = r.backing.clone();
        let fixed = flags & 0x10 != 0;
        let mut address = if fixed {
            hint
        } else {
            aligned(hint.max(VM_BASE), a)?
        };
        if !fixed {
            for v in &s.views {
                if address.checked_add(size).is_some_and(|e| e <= v.address) {
                    break;
                }
                if address < v.address + v.size {
                    address = aligned(v.address + v.size, a)?
                }
            }
        }
        if address == 0
            || !address.is_multiple_of(a)
            || address.checked_add(size).is_none_or(|e| e > VM_END)
            || address < VM_BASE
        {
            return Err(Error::Unsupported);
        }
        let replacement = s
            .views
            .iter()
            .position(|v| !v.direct && v.address == address && v.size == size);
        if s.views.iter().enumerate().any(|(i, v)| {
            Some(i) != replacement && address < v.address + v.size && v.address < address + size
        }) {
            return Err(Error::Busy);
        }
        let range = GuestRange {
            start: GuestAddress(address),
            size,
        };
        if let Some(i) = replacement {
            s.views[i].image.release()?;
        }
        let image = match backing.map(range, relative, prot) {
            Ok(image) => image,
            Err(e) => {
                if let Some(i) = replacement {
                    match NativeImage::reserve(range, VM_END - VM_BASE) {
                        Ok(image) => {
                            s.observers.push(image.observer());
                            s.views[i].image = image;
                        }
                        Err(restore) => {
                            s.views.remove(i);
                            return Err(Error::Native(restore));
                        }
                    }
                }
                return Err(Error::Native(e));
            }
        };
        if let Some(i) = replacement {
            s.views.remove(i);
        }
        s.observers.push(image.observer());
        s.views.push(View {
            direct: true,
            address,
            size,
            offset,
            protection: p,
            name: vec![],
            image,
        });
        s.views.sort_by_key(|v| v.address);
        s.mapped = s.mapped.saturating_add(1);
        Ok(address)
    }
    pub fn unmap(&self, address: u64, size: u64) -> Result {
        let mut s = self.lock();
        let i = s
            .views
            .iter()
            .position(|v| v.address == address && v.size == size)
            .ok_or(Error::NotFound)?;
        s.views[i].image.release()?;
        s.views.remove(i);
        Self::reap(&mut s);
        Ok(())
    }
    fn reap(s: &mut State) {
        s.resources.retain(|r| {
            !r.info.released
                || s.views.iter().any(|v| {
                    v.direct && v.offset >= r.info.offset && v.offset < r.info.offset + r.info.size
                })
        });
    }
    pub fn release(&self, offset: u64, size: u64) -> Result {
        let mut s = self.lock();
        let r = s
            .resources
            .iter_mut()
            .find(|r| r.info.offset == offset && r.info.size == size && !r.info.released)
            .ok_or(Error::NotFound)?;
        r.info.released = true;
        Self::reap(&mut s);
        Ok(())
    }
    pub fn protect(&self, address: u64, size: u64, p: u32) -> Result {
        let prot = protection(p)?;
        let mut s = self.lock();
        let v = s
            .views
            .iter_mut()
            .find(|v| v.address == address && v.size == size)
            .ok_or(Error::Unsupported)?;
        v.image.protect_all(prot)?;
        v.protection = p;
        Ok(())
    }
    fn mapping(s: &State, v: &View) -> Mapping {
        Mapping {
            direct: v.direct,
            address: v.address,
            size: v.size,
            offset: v.offset,
            protection: v.protection,
            memory_type: s
                .resources
                .iter()
                .find(|r| {
                    v.direct && v.offset >= r.info.offset && v.offset < r.info.offset + r.info.size
                })
                .map_or(0, |r| r.info.memory_type),
            name: v.name.clone(),
        }
    }
    pub fn query(&self, address: u64, next: bool) -> Result<Mapping> {
        let s = self.lock();
        s.views
            .iter()
            .find(|v| {
                address >= v.address && address < v.address + v.size || next && v.address > address
            })
            .map(|v| Self::mapping(&s, v))
            .ok_or(Error::NotFound)
    }
    pub fn name(&self, address: u64, size: u64, name: &[u8]) -> Result {
        if name.len() > 32 {
            return Err(Error::Invalid);
        }
        let mut s = self.lock();
        let v = s
            .views
            .iter_mut()
            .find(|v| v.address == address && v.size == size)
            .ok_or(Error::NotFound)?;
        v.name = name.to_vec();
        Ok(())
    }
    pub fn with_regions<T>(&self, f: impl FnOnce(&[&NativeImage]) -> T) -> T {
        let s = self.lock();
        let regions: Vec<_> = s.views.iter().map(|v| &v.image).collect();
        f(&regions)
    }
    pub fn shutdown(&self) {
        let mut s = self.lock();
        s.stopped = true;
        s.views.clear();
        s.resources.clear();
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.lock();
        Snapshot {
            allocations: s.resources.iter().map(|r| r.info.clone()).collect(),
            mappings: s.views.iter().map(|v| Self::mapping(&s, v)).collect(),
            allocated_total: s.allocated,
            mapped_total: s.mapped,
            active_native_resources: s.observers.iter().map(|o| o.active_reservations()).sum(),
            release_errors: s
                .observers
                .iter()
                .map(|o| o.release_error())
                .filter(|c| *c != 0)
                .collect(),
        }
    }
}
impl Drop for MemoryResources {
    fn drop(&mut self) {
        self.shutdown()
    }
}
