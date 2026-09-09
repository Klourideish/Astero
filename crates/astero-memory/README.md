# astero-memory

Guest mappings, protections, allocation and checked access.

## Interfaces and status

M27 implements isolated guest byte regions; other module roots remain scaffolded.

## Module ownership

[access/](src/access/mod.rs), [address/](src/address/mod.rs), [allocation/](src/allocation/mod.rs), [mapping/](src/mapping/mod.rs), [protection/](src/protection/mod.rs), [regions/](src/regions/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Process scheduling or library exports.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M27 mapping owns bounded, non-executable byte regions and checked staging writes. Finalization disables writes; per-image observers verify teardown. Native executable backing is deliberately separate. See [staging](../../knowledge/architecture/guest_image_staging.md).
