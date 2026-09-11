//! Owns synchronization within astero-kernel; runtime implementation is planned.
pub mod condvar;
pub mod event_flag;
pub mod mutex;
pub mod rwlock;
pub mod semaphore;

pub mod owned;
