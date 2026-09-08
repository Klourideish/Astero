# astero-abi

Guest ABI layouts, identifiers and constants.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

layouts, identifiers. Modules belong to this crate's scope.

Forbidden: Host-side application types or service mechanisms.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
