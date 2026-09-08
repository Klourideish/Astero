# astero-kernel

Processes, threads, synchronization, clocks and execution.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

execution, synchronization. Modules belong to this crate's scope. Native execution starts in `src/execution/`; host-specific backends must be nested below it when implemented.

Forbidden: Guest library export contracts or application composition.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
