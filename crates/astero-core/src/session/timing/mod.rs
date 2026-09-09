//! Explicit host timing ownership for opted-in sessions. No worker is created by default.
pub use astero_timing::{
    clock::ManualClock,
    scheduler::{Config, Error, Scheduler, TimingEngine},
    time::{Deadline, Span, Tick},
};
