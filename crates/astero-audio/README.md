# astero-audio

Audio processing, mixing and host output.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

mixing, output. Modules belong to this crate's scope.

Forbidden: Guest library contracts or session composition.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
