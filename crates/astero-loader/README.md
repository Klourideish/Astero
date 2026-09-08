# astero-loader

Binary parsing, load plans, relocations and import metadata.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

loading, relocations. Modules belong to this crate's scope.

Forbidden: Executing guest code or owning session lifecycle.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
