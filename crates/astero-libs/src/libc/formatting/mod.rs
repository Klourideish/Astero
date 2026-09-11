//! Bounded libc formatting. Guest ABI and byte buffers are explicit; no host variadic/FILE calls.
pub mod arguments;
pub mod engine;
pub mod exports;
mod render;
use astero_hle::calls::memory::AccessError;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Access(AccessError),
    Format { offset: usize, reason: &'static str },
}
impl Error {
    pub fn contract(reason: &'static str) -> Self {
        Self::Format { offset: 0, reason }
    }
}
impl From<AccessError> for Error {
    fn from(e: AccessError) -> Self {
        Self::Access(e)
    }
}
pub type Result<T> = std::result::Result<T, Error>;
pub const MAX_OUTPUT: usize = 64 * 1024 * 1024;
pub const MAX_FIELD_WIDTH: usize = 1024 * 1024;
pub const MAX_CONVERSIONS: usize = 16384;
