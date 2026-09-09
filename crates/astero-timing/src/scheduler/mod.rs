//! Owned worker, weak producer handles and one-shot completion tickets.
mod client;
mod model;
mod owner;
mod ticket;
mod worker;
pub use client::Scheduler;
pub(crate) use model::Shared;
pub use model::{Config, Error, Label, StopReason};
pub use owner::TimingEngine;
pub use ticket::{CancelOutcome, Completion, Expiry, Ticket};
