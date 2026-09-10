# astero-libs

Guest library exports and delegation to owning services.

## Interfaces and status

M0 scaffold only. Re-exports the HLE `Provider` contract; exports and dispatch are not implemented.

## Module ownership

[media/](src/media/mod.rs).

[agc/](src/agc/mod.rs), [audio/](src/audio/mod.rs), [families/](src/families/mod.rs), [filesystem/](src/filesystem/mod.rs), [kernel/](src/kernel/mod.rs), [libc/](src/libc/mod.rs), [network/](src/network/mod.rs), [nids/](src/nids/mod.rs), [pthread/](src/pthread/mod.rs), [runtime/](src/runtime/mod.rs), [sysmodule/](src/sysmodule/mod.rs), [videoout/](src/videoout/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Owning mutex machinery, scheduling or memory mapping mechanisms.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M30 libc/startup implements exact libc/libc startup registrations: instrumented experimental _init_env no-op and bounded atexit retention. Kernel owns callback storage. No callback executes. See [native closure](../../knowledge/architecture/native_entry_closure.md).

M32 libc/primitives and libc/process adapt the PS5Rust startup cluster; startup publishes the owned guard contract. Tests alone add ABI/memory dependencies. See [startup foundation](../../knowledge/architecture/startup_foundation.md).
