# astero-hle

Provider interfaces, registration, resolution and dispatch.

## Interfaces and status

M0 scaffold only. `providers::Provider` supplies diagnostic identity only.

## Module ownership

providers, resolution. Modules belong to this crate's scope.

Forbidden: Depending on libs or implementing guest library families.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
