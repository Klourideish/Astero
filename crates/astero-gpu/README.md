# astero-gpu

Commands, registers, resources, submission and GPU backend.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

[agc/](src/agc/mod.rs), [backend/](src/backend/mod.rs), [command_buffers/](src/command_buffers/mod.rs), [pm4/](src/pm4/mod.rs), [registers/](src/registers/mod.rs), [resources/](src/resources/mod.rs), [submission/](src/submission/mod.rs), [synchronization/](src/synchronization/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Display timing or application lifecycle.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
