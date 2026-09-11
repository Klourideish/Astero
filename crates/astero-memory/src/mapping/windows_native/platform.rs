//! Sole M28 unsafe leaf. Every address passed to copy/protect/query is inside this owner's
//! successfully reserved envelope; runtime copies touch only committed owned RW/NX pages.
//! No callable pointer escapes. x86-64 Windows ABI declarations are pinned below.
use super::*;
use std::{
    ffi::c_void,
    mem::{MaybeUninit, size_of},
    ptr,
    sync::atomic::Ordering,
};
#[repr(C)]
struct SystemInfo {
    architecture: u16,
    reserved: u16,
    page: u32,
    min: *mut c_void,
    max: *mut c_void,
    mask: usize,
    processors: u32,
    kind: u32,
    granularity: u32,
    level: u16,
    revision: u16,
}
#[repr(C)]
struct MemoryInfo {
    base: *mut c_void,
    allocation: *mut c_void,
    allocation_protect: u32,
    partition: u16,
    region_size: usize,
    state: u32,
    protect: u32,
    kind: u32,
}
const _: () = assert!(size_of::<SystemInfo>() == 48 && size_of::<MemoryInfo>() == 48);
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetNativeSystemInfo(info: *mut SystemInfo);
    fn GetLastError() -> u32;
    fn GetCurrentProcess() -> *mut c_void;
    fn VirtualAlloc(address: *mut c_void, size: usize, kind: u32, protection: u32) -> *mut c_void;
    fn VirtualFree(address: *mut c_void, size: usize, kind: u32) -> i32;
    fn VirtualProtect(address: *mut c_void, size: usize, protection: u32, old: *mut u32) -> i32;
    fn VirtualQuery(address: *const c_void, info: *mut MemoryInfo, size: usize) -> usize;
    fn FlushInstructionCache(process: *mut c_void, address: *const c_void, size: usize) -> i32;
    fn ReadProcessMemory(
        process: *mut c_void,
        address: *const c_void,
        buffer: *mut c_void,
        size: usize,
        read: *mut usize,
    ) -> i32;
    fn WriteProcessMemory(
        process: *mut c_void,
        address: *mut c_void,
        buffer: *const c_void,
        size: usize,
        written: *mut usize,
    ) -> i32;
}
fn error(operation: &'static str, address: u64, size: u64) -> NativeError {
    // SAFETY: GetLastError has no pointer arguments and runs before cleanup alters the error.
    NativeError::Os {
        operation,
        address,
        size,
        code: unsafe { GetLastError() },
    }
}
pub fn host_geometry() -> Result<Geometry, NativeError> {
    let mut s = MaybeUninit::<SystemInfo>::zeroed();
    // SAFETY: API writes exactly the verified x64 SYSTEM_INFO layout to valid storage.
    let s = unsafe {
        GetNativeSystemInfo(s.as_mut_ptr());
        s.assume_init()
    };
    if s.architecture != 9 {
        return Err(NativeError::UnsupportedHost);
    }
    Ok(Geometry {
        page_size: s.page as u64,
        allocation_granularity: s.granularity as u64,
    })
}
fn flags(p: Protection) -> u32 {
    match (p.read, p.write, p.execute) {
        (_, true, true) => 0x40,
        (true, false, true) => 0x20,
        (false, false, true) => 0x10,
        (_, true, false) => 4,
        (true, false, false) => 2,
        _ => 1,
    }
}
struct Reservation {
    base: *mut c_void,
    size: u64,
    observer: NativeObserver,
}
impl Reservation {
    fn release(&mut self) -> Result<(), NativeError> {
        if self.base.is_null() {
            return Ok(());
        }
        // SAFETY: base is exactly this owner's successful reservation, not a committed subrange.
        if unsafe { VirtualFree(self.base, 0, 0x8000) } == 0 {
            let e = error("release", self.base as u64, self.size);
            if let NativeError::Os { code, .. } = e {
                self.observer.failure.store(code, Ordering::SeqCst);
            }
            return Err(e);
        }
        self.base = ptr::null_mut();
        self.observer.active.store(0, Ordering::SeqCst);
        self.observer.failure.store(0, Ordering::SeqCst);
        Ok(())
    }
}
impl Drop for Reservation {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
/// Owned VM may be shared after protection finalization. Release/protection require exclusive
/// ownership; checked copies never borrow guest bytes as Rust references.
pub struct NativeImage {
    reservation: Reservation,
    layout: NativeLayout,
    snapshot: NativeSnapshot,
}
// SAFETY: the reservation is process VM, not thread-local storage. Metadata is immutable through
// shared access. Only OS checked copies touch published bytes (which native guest threads may
// mutate). Exclusive release/protection cannot race an Arc-held execution/storage lease. No raw
// pointer or Rust reference to guest memory escapes; final drop releases on any owning thread.
unsafe impl Send for NativeImage {}
unsafe impl Sync for NativeImage {}
impl NativeImage {
    pub fn observer(&self) -> NativeObserver {
        self.reservation.observer.clone()
    }
    pub fn snapshot(&self) -> NativeSnapshot {
        let mut s = self.snapshot;
        s.active = !self.reservation.base.is_null();
        if !s.active {
            s.committed_bytes = 0;
        }
        s
    }
    pub fn pages(&self) -> &[NativePage] {
        self.layout.pages()
    }
    pub fn release(&mut self) -> Result<(), NativeError> {
        self.reservation.release()
    }
    /// Remove write permission from complete owned pages only. Failure quarantines/releases
    /// the whole image rather than exposing partially finalized protection as success.
    pub fn seal_read_only(&mut self, range: GuestRange) -> Result<(), NativeError> {
        if self.reservation.base.is_null() {
            return Err(NativeError::Released);
        }
        let page = self.layout.geometry.page_size;
        if range.size == 0
            || !range.start.0.is_multiple_of(page)
            || !range.size.is_multiple_of(page)
        {
            return Err(NativeError::Geometry);
        }
        let end = range
            .start
            .0
            .checked_add(range.size)
            .ok_or(NativeError::Overflow)?;
        let first = self
            .layout
            .pages
            .iter()
            .position(|p| p.range.start == range.start)
            .ok_or(NativeError::Unreadable)?;
        let count = usize::try_from(range.size / page).map_err(|_| NativeError::Overflow)?;
        let stop = first.checked_add(count).ok_or(NativeError::Overflow)?;
        if stop > self.layout.pages.len() {
            return Err(NativeError::Unreadable);
        }
        for (i, p) in self.layout.pages[first..stop].iter().enumerate() {
            if p.range.start.0 != range.start.0 + i as u64 * page
                || p.range.start.0 >= end
                || !p.protection.read
                || p.protection.execute
            {
                return Err(NativeError::Unreadable);
            }
        }
        for index in first..stop {
            let address = self.layout.pages[index].range.start.0;
            let mut old = 0;
            // SAFETY: prevalidated committed, readable, non-executable owned page; removing write only.
            let ok = unsafe { VirtualProtect(address as *mut c_void, page as usize, 2, &mut old) };
            if ok == 0 {
                let e = error("seal", address, page);
                let _ = self.release();
                return Err(e);
            }
            let mut info = MaybeUninit::<MemoryInfo>::zeroed();
            // SAFETY: valid initialized storage of pinned x64 layout; VirtualQuery never dereferences guest data.
            let n = unsafe {
                VirtualQuery(
                    address as *const c_void,
                    info.as_mut_ptr(),
                    size_of::<MemoryInfo>(),
                )
            };
            if n != size_of::<MemoryInfo>() {
                let e = error("seal-query", address, page);
                let _ = self.release();
                return Err(e);
            }
            // SAFETY: full output layout returned successfully.
            let info = unsafe { info.assume_init() };
            if info.protect != 2 || info.allocation != self.reservation.base || info.state != 0x1000
            {
                let _ = self.release();
                return Err(NativeError::ProtectionMismatch { address });
            }
            self.layout.pages[index].protection.write = false;
        }
        Ok(())
    }
    /// Writes only existing owned RW/NX pages, with complete preflight and OS checked copy.
    /// No permission elevation, executable patching, or arbitrary pointer is exposed.
    pub fn write(
        &self,
        address: crate::mapping::GuestAddress,
        bytes: &[u8],
    ) -> Result<(), NativeError> {
        if self.reservation.base.is_null() {
            return Err(NativeError::Released);
        }
        let end = address
            .0
            .checked_add(bytes.len() as u64)
            .ok_or(NativeError::Overflow)?;
        let page = self.layout.geometry.page_size;
        let mut at = address.0 & !(page - 1);
        while at < end {
            let p = self
                .layout
                .pages
                .iter()
                .find(|p| p.range.start.0 == at)
                .ok_or(NativeError::Unreadable)?;
            if !p.protection.write || p.protection.execute {
                return Err(NativeError::Unreadable);
            }
            at = at.checked_add(page).ok_or(NativeError::Overflow)?;
        }
        if bytes.is_empty() {
            return Ok(());
        }
        let mut written = 0;
        // SAFETY: complete owned RW/NX coverage, independent source slice, live shared owner.
        // Readback is deliberately not a concurrency guarantee: another guest may write after us.
        if unsafe {
            WriteProcessMemory(
                GetCurrentProcess(),
                address.0 as *mut c_void,
                bytes.as_ptr().cast(),
                bytes.len(),
                &mut written,
            )
        } == 0
            || written != bytes.len()
        {
            return Err(error("write", address.0, bytes.len() as u64));
        }
        Ok(())
    }
    pub fn read(
        &self,
        address: crate::mapping::GuestAddress,
        size: u64,
    ) -> Result<Vec<u8>, NativeError> {
        if self.reservation.base.is_null() {
            return Err(NativeError::Released);
        }
        let end = address.0.checked_add(size).ok_or(NativeError::Overflow)?;
        if size == 0 {
            return Ok(Vec::new());
        }
        let page = self.layout.geometry.page_size;
        let mut at = address.0 & !(page - 1);
        while at < end {
            let p = self
                .layout
                .pages
                .iter()
                .find(|p| p.range.start.0 == at)
                .ok_or(NativeError::Unreadable)?;
            if !p.protection.read && !p.protection.write {
                return Err(NativeError::Unreadable);
            }
            at = at.checked_add(page).ok_or(NativeError::Overflow)?;
        }
        let len = usize::try_from(size).map_err(|_| NativeError::Allocation)?;
        let mut out = Vec::new();
        out.try_reserve_exact(len)
            .map_err(|_| NativeError::Allocation)?;
        out.resize(len, 0);
        let mut read = 0;
        // SAFETY: validated owned readable page coverage; OS checked copy writes only to sized Vec.
        if unsafe {
            ReadProcessMemory(
                GetCurrentProcess(),
                address.0 as *const c_void,
                out.as_mut_ptr().cast(),
                len,
                &mut read,
            )
        } == 0
            || read != len
        {
            return Err(error("read", address.0, size));
        }
        Ok(out)
    }
}
pub fn realize(
    regions: &[NativeRegion<'_>],
    limits: NativeLimits,
) -> (Result<NativeImage, NativeError>, NativeObserver) {
    let observer = NativeObserver::default();
    let result = construct(regions, limits, observer.clone());
    (result, observer)
}
fn construct(
    regions: &[NativeRegion<'_>],
    limits: NativeLimits,
    observer: NativeObserver,
) -> Result<NativeImage, NativeError> {
    construct_checked(regions, limits, observer, |_| Ok(()))
}
// Private checkpoint enables deterministic rollback tests without a public fault API.
fn construct_checked(
    regions: &[NativeRegion<'_>],
    limits: NativeLimits,
    observer: NativeObserver,
    checkpoint: impl Fn(&str) -> Result<(), NativeError>,
) -> Result<NativeImage, NativeError> {
    let layout = plan_layout(regions, host_geometry()?, limits)?;
    let range = layout.envelope;
    let size = usize::try_from(range.size).map_err(|_| NativeError::Overflow)?;
    // SAFETY: aligned checked nonzero address/size; reservation never replaces existing mappings.
    let base = unsafe { VirtualAlloc(range.start.0 as *mut c_void, size, 0x2000, 1) };
    if base.is_null() {
        return Err(error(
            "reserve_exact_or_collision",
            range.start.0,
            range.size,
        ));
    }
    observer.active.store(1, Ordering::SeqCst);
    let reservation = Reservation {
        base,
        size: range.size,
        observer,
    };
    if base as u64 != range.start.0 {
        return Err(NativeError::PlacementMismatch);
    }
    // RAII reservation exists before any fallible commit/copy/protection operation.
    for p in &layout.pages {
        // SAFETY: each full aligned page is inside our reservation; initially RW, never executable.
        if unsafe {
            VirtualAlloc(
                p.range.start.0 as *mut c_void,
                p.range.size as usize,
                0x1000,
                4,
            )
        }
        .is_null()
        {
            return Err(error("commit", p.range.start.0, p.range.size));
        }
    }
    checkpoint("after_commit")?;
    for r in regions {
        // SAFETY: layout proves non-overlap and full committed RW coverage; input is a valid slice
        // disjoint from fresh reservation (VirtualAlloc could not overlap any existing source allocation).
        unsafe {
            ptr::copy_nonoverlapping(
                r.bytes.as_ptr(),
                base.cast::<u8>()
                    .add((r.range.start.0 - range.start.0) as usize),
                r.bytes.len(),
            );
        }
        // SAFETY: same owned initialized RW extent; borrowed only here, before protection/release.
        let copied = unsafe {
            std::slice::from_raw_parts(
                base.cast::<u8>()
                    .add((r.range.start.0 - range.start.0) as usize),
                r.bytes.len(),
            )
        };
        if copied != r.bytes {
            return Err(NativeError::Readback {
                address: r.range.start.0,
            });
        }
    }
    checkpoint("before_protect")?;
    for p in &layout.pages {
        let mut old = 0;
        // SAFETY: committed aligned page within our reservation; old is writable DWORD storage.
        if unsafe {
            VirtualProtect(
                p.range.start.0 as *mut c_void,
                p.range.size as usize,
                flags(p.protection),
                &mut old,
            )
        } == 0
        {
            return Err(error("protect", p.range.start.0, p.range.size));
        }
        checkpoint("after_protect")?;
        if p.protection.execute {
            // SAFETY: valid process pseudo-handle, owned committed code range; never calls that code.
            if unsafe {
                FlushInstructionCache(
                    GetCurrentProcess(),
                    p.range.start.0 as *const c_void,
                    p.range.size as usize,
                )
            } == 0
            {
                return Err(error("flush", p.range.start.0, p.range.size));
            }
        }
        let mut info = MaybeUninit::<MemoryInfo>::zeroed();
        // SAFETY: VirtualQuery writes checked x64 MEMORY_BASIC_INFORMATION layout; failure not read.
        let n = unsafe {
            VirtualQuery(
                p.range.start.0 as *const c_void,
                info.as_mut_ptr(),
                size_of::<MemoryInfo>(),
            )
        };
        if n != size_of::<MemoryInfo>() {
            return Err(error("query", p.range.start.0, p.range.size));
        }
        // SAFETY: full structure initialized by successful VirtualQuery.
        let info = unsafe { info.assume_init() };
        if info.allocation != base || info.state != 0x1000 || info.protect != flags(p.protection) {
            return Err(NativeError::ProtectionMismatch {
                address: p.range.start.0,
            });
        }
    }
    let snapshot = NativeSnapshot {
        guest_envelope: range,
        host_envelope: range,
        geometry: layout.geometry,
        committed_bytes: layout.pages.len() as u64 * layout.geometry.page_size,
        copied_bytes: regions.iter().map(|r| r.range.size).sum(),
        widened_pages: layout.pages.iter().filter(|p| p.widened).count(),
        active: true,
        readback_before_protection: true,
        protections_verified: true,
        instruction_cache_flushed: layout.pages.iter().any(|p| p.protection.execute),
    };
    let image = NativeImage {
        reservation,
        layout,
        snapshot,
    };
    // Verify readable portions again after protection transitions. Execute-only/no-access
    // portions were checked while RW; never dereference them after finalization.
    for r in regions {
        let mut at = r.range.start.0;
        let end = at + r.range.size;
        while at < end {
            let page = image.layout.geometry.page_size;
            let base = at & !(page - 1);
            let stop = end.min(base + page);
            let p = image
                .layout
                .pages
                .iter()
                .find(|p| p.range.start.0 == base)
                .expect("validated page");
            if p.protection.read || p.protection.write {
                let bytes = image.read(crate::mapping::GuestAddress(at), stop - at)?;
                if bytes
                    != r.bytes[(at - r.range.start.0) as usize..(stop - r.range.start.0) as usize]
                {
                    return Err(NativeError::Readback { address: at });
                }
            }
            at = stop;
        }
    }
    Ok(image)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failures_after_commit_and_before_protection_release_real_reservation() {
        let bytes = [0; 4096];
        let limits = NativeLimits {
            max_reserved_bytes: 65536,
            max_committed_bytes: 65536,
        };
        for (i, point) in ["after_commit", "before_protect", "after_protect"]
            .into_iter()
            .enumerate()
        {
            let address = 0x680000000 + (std::process::id() as u64) * 65536 + i as u64 * 65536;
            let regions = [NativeRegion {
                range: GuestRange {
                    start: crate::mapping::GuestAddress(address),
                    size: 4096,
                },
                bytes: &bytes,
                protection: Protection {
                    read: true,
                    write: false,
                    execute: true,
                },
            }];
            let observer = NativeObserver::default();
            let r = construct_checked(&regions, limits, observer.clone(), |p| {
                if p == point {
                    Err(NativeError::Os {
                        operation: "injected checkpoint",
                        address,
                        size: 4096,
                        code: 5,
                    })
                } else {
                    Ok(())
                }
            });
            assert!(r.is_err());
            assert_eq!(observer.active_reservations(), 0);
            assert_eq!(observer.release_error(), 0);
            let (retry, o) = realize(&regions, limits);
            drop(retry.unwrap());
            assert_eq!(o.active_reservations(), 0);
        }
    }
}
