# astero-timing

Owns consumer-independent host/manual monotonic clocks and a real bounded asynchronous deadline worker.
No dependencies, unsafe code, guest semantics, callback execution, wall clock or hidden singleton.

Narrow homes: [time](src/time/mod.rs), [clock](src/clock/mod.rs),
[scheduler](src/scheduler/mod.rs), [diagnostics](src/diagnostics/mod.rs).
TimingEngine owns/join-stops the worker; Scheduler and ManualClock are weak handles. Tickets retain
one immutable terminal outcome. Config requires pending capacity and snapshot bound; labels are
bounded to 48 UTF-8 bytes. No default worker exists. Consumers may block on a ticket or poll it.

Use `cargo test -p astero-timing` or `cargo run -p astero-timing --example deadlines`.
Read [architecture](../../knowledge/architecture/asynchronous_timing.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[boundaries](../../knowledge/architecture/crate_boundaries.md) and
[validation](../../knowledge/architecture/validation.md). No PS5 timer/HLE APIs are implemented.
