//! Initial-thread-owned storage. No FS/GS change or thread is created.
use super::{
    layout::{PreparationError, ThreadLayout},
    process_arguments,
};
use astero_memory::mapping::{
    GuestAddress,
    windows_native::{
        self, NativeError, NativeImage, NativeLimits, NativeObserver, NativeRegion, Protection,
    },
};
#[derive(Debug)]
pub enum StorageError {
    Layout(PreparationError),
    Native(NativeError),
}
impl From<NativeError> for StorageError {
    fn from(e: NativeError) -> Self {
        Self::Native(e)
    }
}
pub struct ThreadStorage {
    stack: NativeImage,
    tls: NativeImage,
    layout: ThreadLayout,
}
impl ThreadStorage {
    pub fn build(
        layout: ThreadLayout,
        template: &[u8],
    ) -> Result<(Self, [NativeObserver; 2]), StorageError> {
        let g = windows_native::host_geometry()?;
        let checked = super::layout::plan(
            super::layout::RuntimeLimits {
                stack_base: layout.guard.start.0,
                stack_bytes: layout.stack.size,
                tls_base: layout.tls.start.0,
                max_runtime_bytes: layout.reserved_bytes,
            },
            g,
            layout.tls_template_size,
            layout.tls_memory_size,
            layout.tls_alignment,
        )
        .map_err(StorageError::Layout)?;
        // Caller layouts are value evidence, not permission to index arbitrary memory.
        if checked != layout {
            return Err(StorageError::Layout(PreparationError::TlsShape));
        }
        if template.len() as u64 != layout.tls_template_size {
            return Err(StorageError::Layout(PreparationError::TlsShape));
        }
        fn zeros(n: u64) -> Result<Vec<u8>, StorageError> {
            let n = usize::try_from(n)
                .map_err(|_| StorageError::Layout(PreparationError::Allocation))?;
            let mut b = Vec::new();
            b.try_reserve_exact(n)
                .map_err(|_| StorageError::Layout(PreparationError::Allocation))?;
            b.resize(n, 0);
            Ok(b)
        }
        let mut stack = zeros(layout.stack.size)?;
        let guard = zeros(layout.guard.size)?;
        let mut tls = zeros(layout.tls.size)?;
        let offset = (layout.params - layout.stack.start.0) as usize;
        stack[offset..offset + 32].copy_from_slice(&process_arguments(layout.argv0));
        stack[offset + 32..offset + 38].copy_from_slice(b"guest\0");
        tls[..template.len()].copy_from_slice(template);
        let tp = (layout.thread_pointer - layout.tls.start.0) as usize;
        tls[tp..tp + 8].copy_from_slice(&layout.thread_pointer.to_le_bytes());
        let rw = Protection {
            read: true,
            write: true,
            execute: false,
        };
        let limits = NativeLimits {
            max_reserved_bytes: layout.reserved_bytes,
            max_committed_bytes: layout.reserved_bytes,
        };
        let regions = [
            NativeRegion {
                range: layout.guard,
                bytes: &guard,
                protection: Protection::default(),
            },
            NativeRegion {
                range: layout.stack,
                bytes: &stack,
                protection: rw,
            },
        ];
        let (s, so) = windows_native::realize(&regions, limits);
        let stack = s?;
        let (t, to) = windows_native::realize(
            &[NativeRegion {
                range: layout.tls,
                bytes: &tls,
                protection: rw,
            }],
            limits,
        );
        let tls = t?;
        Ok((Self { stack, tls, layout }, [so, to]))
    }
    pub fn native_regions(&self) -> [&NativeImage; 2] {
        [&self.stack, &self.tls]
    }
    pub fn observers(&self) -> [NativeObserver; 2] {
        [self.stack.observer(), self.tls.observer()]
    }
    pub fn layout(&self) -> &ThreadLayout {
        &self.layout
    }
    pub fn read_stack(&self, address: u64, size: u64) -> Result<Vec<u8>, NativeError> {
        self.stack.read(GuestAddress(address), size)
    }
    pub fn read_tls(&self, address: u64, size: u64) -> Result<Vec<u8>, NativeError> {
        self.tls.read(GuestAddress(address), size)
    }
    pub fn install_return(&self, landing: u64) -> Result<(), NativeError> {
        if landing == 0 {
            return Err(NativeError::Unreadable);
        }
        self.stack
            .write(GuestAddress(self.layout.rsp), &landing.to_le_bytes())
    }
    /// Checked copies only; no reference to guest bytes escapes.
    pub fn write(&self, address: u64, bytes: &[u8]) -> Result<(), NativeError> {
        if address >= self.layout.stack.start.0
            && address - self.layout.stack.start.0 < self.layout.stack.size
        {
            self.stack
                .write(astero_memory::mapping::GuestAddress(address), bytes)
        } else {
            self.tls
                .write(astero_memory::mapping::GuestAddress(address), bytes)
        }
    }
    pub fn release(&mut self) -> Result<(), NativeError> {
        let a = self.stack.release();
        let b = self.tls.release();
        a.and(b)
    }
}
