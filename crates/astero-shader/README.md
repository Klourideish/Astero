# astero-shader

Shader decoding, IR, transformations and host generation.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

[analysis/](src/analysis/mod.rs), [ir/](src/ir/mod.rs), [lowering/](src/lowering/mod.rs), [rdna2/](src/rdna2/mod.rs), [spirv/](src/spirv/mod.rs), [ssa/](src/ssa/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: GPU submission or presentation.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
