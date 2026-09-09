# astero-kernel

Processes, threads, synchronization, clocks and execution.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

[filesystem/](src/filesystem/mod.rs).

[objects/](src/objects/mod.rs).

[errno/](src/errno/mod.rs), [execution/](src/execution/mod.rs), [process/](src/process/mod.rs), [signals/](src/signals/mod.rs), [synchronization/](src/synchronization/mod.rs), [threading/](src/threading/mod.rs), [timing/](src/timing/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Guest library export contracts or application composition.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M25 separates the generic asynchronous mechanism into astero-timing. This crate retains future guest clocks, waits and timer policy adapters; no timing HLE is implemented here.
