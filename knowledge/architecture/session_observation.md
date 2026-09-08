# Session ownership and observation — M1

Core owns `Session` and its private state. The executable composition root creates and initializes
that owner, keeps it alive during the frontend, then drops it. Frontend presentation receives
`SessionObserver`, a weak, read-only handle; it cannot extend session lifetime or mutate state.
Core has no debugger/toolkit dependency. Debug consumes the deliberate `ObserveSession` trait.
CLI and GUI both use debug's `Inspection`, containing the unmodified core `SessionSnapshot`
and capability inventory. The GUI adapter does not create a parallel model.

## Lifecycle contract

| Operation | Accepted source | Result |
|---|---|---|
| create | no session | Created, no target, zero host lifecycle changes |
| initialize | Created | Ready: host initialized; no guest readiness implied |
| stop | Created, Ready, Faulted | Stopped; terminal |
| report_host_fault(nonempty reason) | Created, Ready | Faulted with host diagnostic |
| run | Ready with no target | NoGuestLoaded error; no mutation |
| run | other reachable states | InvalidTransition error; no mutation |

Running and Paused are reserved values with no transition into them in M1. No load API exists.
LoadedTarget is a display-name placeholder; real session snapshots always contain None.
Invalid transitions leave observed values unchanged. Host faults are orchestration reports, not guest
fault capture. Successful transitions increment the host counter; no guest statistics are fabricated.
Session IDs use a checked atomic sequence unique within one process, not persistent IDs.

## Snapshot semantics

`snapshot()` clones one owned value under a mutex, releasing the lock before formatting/rendering.
Lifecycle, diagnostics and counters belong to one coherent observation. Observation is side-effect free.
Editing a detached snapshot cannot affect the owner. Dropped owners return SessionClosed; poison
returns StateUnavailable. A concurrent read which already upgraded its weak handle may finish as
the owner drops; later reads fail.

Snapshot fields are detached public values, never mutable references into core. This is not a
persisted schema yet. No service lock or guest operation may be performed while holding the mutex.

## Dependency correction and pressure points

M1 replaces M0's core → debug permission with debug → core: the application observation contract
lives in core. Runtime services depend on neither. CLI/GUI depend on core and debug.
Do not bridge the direction with a cycle or put host observation types in ABI.

A dedicated observation/interface crate may be considered if independent consumers or service-level
contracts outgrow core. None is created now. Multi-service consistency, bounded observation cost,
versioning, target metadata provenance and execution-control authorization need future design.
Toolkit types may not cross this boundary.

Tests live under core/tests, debug/tests, cli/tests and gui/tests. See [validation](validation.md).

## Physical homes after structural scaffolding

The existing owner implementation lives at core/src/session/owner.rs; lifecycle vocabulary is in
session/lifecycle/state.rs, observations in session/observation/snapshot.rs, and counters in
session/statistics/counters.rs. Directory roots only declare modules/re-exports. The public
astero_core::observation path is a compatibility re-export of session::observation.
Debugger inspection/capabilities/control now live under debug/src/session/, with their original
public module paths re-exported. No lifecycle transition, statistic or inspection operation was added.
