//! ELF64 header observations; section fields are retained without following them.
mod decode;
pub use decode::Elf64Header;
pub(super) use decode::decode;
