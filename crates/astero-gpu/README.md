# astero-gpu

Commands, registers, resources, submission and GPU backend.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

registers, pm4, resources. Modules belong to this crate's scope.

Forbidden: Display timing or application lifecycle.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
