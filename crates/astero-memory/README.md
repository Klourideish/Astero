# astero-memory

Guest mappings, protections, allocation and checked access.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

mappings, access. Modules belong to this crate's scope.

Forbidden: Process scheduling or library exports.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
