# astero-libs

Guest library exports and delegation to owning services.

## Interfaces and status

M0 scaffold only. Re-exports the HLE `Provider` contract; exports and dispatch are not implemented.

## Module ownership

families, nids. Modules belong to this crate's scope.

Forbidden: Owning mutex machinery, scheduling or memory mapping mechanisms.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
