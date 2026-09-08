# astero-hle

Provider interfaces, registration, resolution and dispatch.

## Interfaces and status

M0 scaffold only. `providers::Provider` supplies diagnostic identity only.

## Module ownership

[calls/](src/calls/mod.rs), [dispatch/](src/dispatch/mod.rs), [nids/](src/nids/mod.rs), [providers/](src/providers/mod.rs), [registration/](src/registration/mod.rs), [resolution/](src/resolution/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Depending on libs or implementing guest library families.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
