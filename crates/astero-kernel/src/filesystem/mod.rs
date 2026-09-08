//! Guest filesystem mechanisms: VFS paths/mounts, descriptor I/O and host I/O adapters.
//! Future children should separate vfs, descriptors, asynchronous I/O and host adapters.
//! Guest exports belong in astero-libs::filesystem; target parsing belongs in loader.
//! Planned home only; no filesystem implementation exists.
