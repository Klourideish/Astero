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

M28 mapping/windows_native owns exact identity reservation, commitment, final protections/query/readback and release. Only its private platform.rs may use unsafe FFI; crate-wide deny remains. No dependency added. See [native VM](../../knowledge/architecture/windows_native_vm.md).

M29 adds exact-page write removal/query for RELRO closure in the existing private Windows leaf. Guarded stack/TLS owners reuse native realization; memory does not decide guest entry readiness.
