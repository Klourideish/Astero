//! Composition of owned startup objects/heap and checked provider memory access.
use super::{ClosureError, PreparedGuest};
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    providers::data::DataExport,
};
use astero_memory::{
    allocation::heap::GuestHeap,
    mapping::{
        GuestAddress,
        windows_native::{self, GuestRange, NativeImage, NativeLimits, NativeRegion, Protection},
    },
};
pub const HEAP_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_ALLOCATIONS: usize = 4096;
pub const MAX_PROVIDER_CALLS: usize = 4096;
pub struct Foundation {
    pub image: NativeImage,
    pub heap: GuestHeap,
    pub guard: DataExport,
}
impl Foundation {
    pub fn build(base: u64, page: u64, available: u64) -> Result<Self, ClosureError> {
        let bytes = page.checked_add(HEAP_BYTES).ok_or(ClosureError::Budget)?;
        if bytes > available {
            return Err(ClosureError::Budget);
        }
        let mut guard = vec![0; page as usize];
        guard[..8].copy_from_slice(&astero_libs::libc::startup::STACK_GUARD_VALUE.to_le_bytes());
        let heap_base = base.checked_add(page).ok_or(ClosureError::Budget)?;
        let heap_bytes = vec![0; HEAP_BYTES as usize];
        let (image, _) = windows_native::realize(
            &[
                NativeRegion {
                    range: GuestRange {
                        start: GuestAddress(base),
                        size: page,
                    },
                    bytes: &guard,
                    protection: Protection {
                        read: true,
                        write: false,
                        execute: false,
                    },
                },
                NativeRegion {
                    range: GuestRange {
                        start: GuestAddress(heap_base),
                        size: HEAP_BYTES,
                    },
                    bytes: &heap_bytes,
                    protection: Protection {
                        read: true,
                        write: true,
                        execute: false,
                    },
                },
            ],
            NativeLimits {
                max_reserved_bytes: bytes,
                max_committed_bytes: bytes,
            },
        );
        Ok(Self {
            image: image.map_err(|error| ClosureError::Native {
                operation: "startup residency",
                error,
            })?,
            heap: GuestHeap::new(heap_base, HEAP_BYTES, MAX_ALLOCATIONS)
                .map_err(|_| ClosureError::Allocation)?,
            guard: astero_libs::libc::startup::stack_guard(base),
        })
    }
}
pub struct Access<'a> {
    pub guest: &'a PreparedGuest,
    pub foundation: &'a mut Foundation,
}
impl GuestMemory for Access<'_> {
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        if n > astero_libs::libc::primitives::MAX_OPERATION_BYTES {
            return Err(AccessError::Limit);
        }
        if n == 0 {
            return Ok(Vec::new());
        }
        let g = &self.guest;
        g.image
            .read(a, n)
            .or_else(|_| g.thread.read_stack(a, n))
            .or_else(|_| g.thread.read_tls(a, n))
            .or_else(|_| self.foundation.image.read(GuestAddress(a), n))
            .map_err(|_| AccessError::Range)
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        if b.len() as u64 > astero_libs::libc::primitives::MAX_OPERATION_BYTES {
            return Err(AccessError::Limit);
        }
        if b.is_empty() {
            return Ok(());
        }
        self.guest
            .image
            .native_owner()
            .write(GuestAddress(a), b)
            .or_else(|_| self.guest.thread.write(a, b))
            .or_else(|_| self.foundation.image.write(GuestAddress(a), b))
            .map_err(|_| AccessError::Range)
    }
    fn allocate(&mut self, n: u64) -> Result<u64, AccessError> {
        self.foundation
            .heap
            .allocate(n)
            .map_err(|_| AccessError::Allocation)
    }
    fn free(&mut self, a: u64) -> Result<(), AccessError> {
        self.foundation
            .heap
            .free(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, a: u64) -> Result<u64, AccessError> {
        self.foundation
            .heap
            .size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
}
