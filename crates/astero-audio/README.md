# astero-audio

Audio processing, mixing and host output.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

[buffers/](src/buffers/mod.rs), [devices/](src/devices/mod.rs), [mixing/](src/mixing/mod.rs), [output/](src/output/mod.rs), [timing/](src/timing/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Guest library contracts or session composition.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
