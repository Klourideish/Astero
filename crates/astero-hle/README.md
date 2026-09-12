# astero-hle

Provider interfaces, registration, resolution and dispatch.

## Interfaces and status

Exact callable/data provider contracts, prepared dispatch and scoped checked-memory access are implemented. Shared byte-work limits remain separate from mapping validity.

## Module ownership

[calls/](src/calls/mod.rs), [dispatch/](src/dispatch/mod.rs), [nids/](src/nids/mod.rs), [providers/](src/providers/mod.rs), [registration/](src/registration/mod.rs), [resolution/](src/resolution/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Depending on libs or implementing guest library families.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M29 dispatch/prepared implements a bounded immutable exact-key host-model registry consuming ABI call frames. Artifact declarations are not callable; native guest dispatch and registrations remain absent.

M32 calls/memory provides scoped checked copies to handlers; providers/data is distinct from callable registrations. See [startup foundation](../../knowledge/architecture/startup_foundation.md).

M35: [large libc memory/runtime continuation](../../knowledge/architecture/libc_large_memory.md) documents checked chunks, budgets, concurrency and real results.

M44 calls/metrics owns bounded atomic per-import counters with exact raw provider keys and coherent last-ordinal/thread pairs. Registration identity counting distinguishes runtime keys from grouped index records. See [runtime observability](../../knowledge/architecture/runtime_observability.md).

M46 providers/modules owns bounded declared-provider request/refcount/dependency state and lease-gated HLE publication. Artifact declarations fail explicitly until a real loader authority is supplied.
