# astero-libs

Guest library exports and delegation to owning services.

## Interfaces and status

Startup and large-memory libc/libkernel, pthread synchronization and pthread lifecycle exports delegate through exact HLE registrations; other families retain explicit scaffold status.

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

M33 [pthread foundation](../../knowledge/architecture/pthread_foundation.md) implements owned synchronization and scoped guest adapters.

M34 [pthread lifecycle](../../knowledge/architecture/pthread_lifecycle.md) adapts owned worker lifetimes, shared process storage and exact lifecycle exports. Offline analysis remains independent.

M35: [large libc memory/runtime continuation](../../knowledge/architecture/libc_large_memory.md) documents checked chunks, budgets, concurrency and real results.

M36 [user_service/](src/user_service/mod.rs) delegates exact UserService contracts to process-owned state.

M37 implements the checked printf family in `libc/formatting`, with explicit guest varargs,
raw bytes, truncation semantics and owned console output. See [contract](../../knowledge/architecture/libc_formatting.md).

M38 adds checked legacy AudioOut and AudioOut2 adapters under audio/exports; mechanisms use astero-audio.

M40 kernel/timing registers the scoped sleep/clock family, with checked outputs and distinct SCE/POSIX errors.

M41 libc/math adds 53 exact scalar math adapters with checked outputs and native XMM validation. See [math/cross-title record](../../knowledge/architecture/libc_scalar_math.md).

M42 runtime/guards provides exact C++ guard and pure-virtual contracts; nested destructor execution remains unsupported.

M43 media/ajm supplies checked lifecycle exports; libc/c11 reuses kernel synchronization. A dev-only timing dependency exercises those adapters. See [M43](../../knowledge/architecture/ajm_c11_startup.md).

M45 kernel/memory and kernel/semaphore adapt exact kernel resource identities; owned mechanisms remain in kernel/memory.
