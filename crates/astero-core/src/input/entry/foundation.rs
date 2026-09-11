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
pub(crate) fn access_error(
    error: astero_memory::mapping::windows_native::NativeError,
) -> AccessError {
    use astero_memory::mapping::windows_native::NativeError;
    match error {
        NativeError::Os {
            operation,
            address,
            size,
            code,
        } => AccessError::HostCopy {
            operation,
            address,
            size,
            code,
        },
        NativeError::Allocation => AccessError::Allocation,
        _ => AccessError::Range,
    }
}
pub struct Foundation {
    pub formatting: std::sync::Arc<astero_libs::libc::formatting::exports::Formatting>,
    pub users: std::sync::Arc<std::sync::Mutex<astero_kernel::process::users::Users>>,
    pub image: NativeImage,
    pub heap: std::sync::Mutex<GuestHeap>,
    pub guard: DataExport,
    pub output: std::sync::Arc<std::sync::Mutex<astero_kernel::process::output::Output>>,
    pub access_budget: astero_hle::calls::budget::AccessBudget,
}
impl Foundation {
    pub fn build(base: u64, page: u64, available: u64) -> Result<Self, ClosureError> {
        let bytes = page.checked_add(HEAP_BYTES).ok_or(ClosureError::Budget)?;
        if bytes > available {
            return Err(ClosureError::Budget);
        }
        let mut guard = vec![0; page as usize];
        guard[..8].copy_from_slice(&astero_libs::libc::startup::STACK_GUARD_VALUE.to_le_bytes());
        guard[16..24].copy_from_slice(b"STDOUT37");
        guard[32..40].copy_from_slice(b"STDERR37");
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
        let output = std::sync::Arc::new(std::sync::Mutex::new(
            astero_kernel::process::output::Output::new(65536),
        ));
        Ok(Self {
            formatting: std::sync::Arc::new(
                astero_libs::libc::formatting::exports::Formatting::new(
                    output.clone(),
                    base + 16,
                    base + 32,
                ),
            ),
            users: std::sync::Arc::new(std::sync::Mutex::new(
                astero_kernel::process::users::Users::new(
                    astero_libs::user_service::exports::USER_ID,
                ),
            )),
            output,
            access_budget: astero_hle::calls::budget::AccessBudget::new(
                astero_hle::calls::budget::AccessLimits {
                    max_operation_bytes: astero_hle::calls::memory::MAX_OPERATION_BYTES,
                    max_total_bytes: 512 * 1024 * 1024,
                },
            ),
            image: image.map_err(|error| ClosureError::Native {
                operation: "startup residency",
                error,
            })?,
            heap: std::sync::Mutex::new(
                GuestHeap::new(heap_base, HEAP_BYTES, MAX_ALLOCATIONS)
                    .map_err(|_| ClosureError::Allocation)?,
            ),
            guard: astero_libs::libc::startup::stack_guard(base),
        })
    }
}
pub struct Access<'a> {
    pub guest: &'a PreparedGuest,
    pub foundation: &'a Foundation,
}
impl GuestMemory for Access<'_> {
    fn charge(&self, n: u64) -> Result<(), AccessError> {
        self.foundation.access_budget.charge(n)
    }
    fn validate(&self, a: u64, n: u64, write: bool) -> Result<(), AccessError> {
        let [stack, tls] = self.guest.thread.native_regions();
        astero_memory::access::native::validate(
            &[
                self.guest.image.native_owner(),
                stack,
                tls,
                &self.foundation.image,
            ],
            a,
            n,
            write,
        )
        .map_err(access_error)
    }
    fn read_window(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        let [stack, tls] = self.guest.thread.native_regions();
        astero_memory::access::native::read_window(
            &[
                self.guest.image.native_owner(),
                stack,
                tls,
                &self.foundation.image,
            ],
            a,
            n,
        )
        .map_err(access_error)
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        if n > astero_hle::calls::memory::COPY_CHUNK_BYTES {
            return Err(AccessError::Limit);
        }
        let [stack, tls] = self.guest.thread.native_regions();
        astero_memory::access::native::read(
            &[
                self.guest.image.native_owner(),
                stack,
                tls,
                &self.foundation.image,
            ],
            a,
            n,
        )
        .map_err(access_error)
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        if b.len() as u64 > astero_hle::calls::memory::COPY_CHUNK_BYTES {
            return Err(AccessError::Limit);
        }
        let [stack, tls] = self.guest.thread.native_regions();
        astero_memory::access::native::write(
            &[
                self.guest.image.native_owner(),
                stack,
                tls,
                &self.foundation.image,
            ],
            a,
            b,
        )
        .map_err(access_error)
    }
    fn allocate_aligned(
        &mut self,
        size: u64,
        alignment: u64,
    ) -> std::result::Result<u64, AccessError> {
        self.foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .allocate_aligned(size, alignment)
            .map_err(|_| AccessError::Allocation)
    }
    fn usable_size(&self, a: u64) -> std::result::Result<u64, AccessError> {
        self.foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .usable_size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
    fn allocate(&mut self, n: u64) -> Result<u64, AccessError> {
        self.foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .allocate(n)
            .map_err(|_| AccessError::Allocation)
    }
    fn free(&mut self, a: u64) -> Result<(), AccessError> {
        self.foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .free(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, a: u64) -> Result<u64, AccessError> {
        self.foundation
            .heap
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .size(a)
            .map_err(|_| AccessError::InvalidAllocation)
    }
}
