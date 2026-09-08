# astero-loader

Binary parsing, load plans, relocations and import metadata.

## Interfaces and status

M0 scaffold only. Module roots document future ownership; no runtime mechanisms are implemented.

## Module ownership

[admission/](src/admission/mod.rs), [artifact/](src/artifact/mod.rs), [dependencies/](src/dependencies/mod.rs), [elf/](src/elf/mod.rs), [exports/](src/exports/mod.rs), [imports/](src/imports/mod.rs), [load_plan/](src/load_plan/mod.rs), [metadata/](src/metadata/mod.rs), [modules/](src/modules/mod.rs), [relocations/](src/relocations/mod.rs), [self_format/](src/self_format/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Executing guest code or owning session lifecycle.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
