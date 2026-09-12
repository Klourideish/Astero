# astero-kernel

Processes, threads, synchronization, clocks and execution.

## Interfaces and status

Native execution/preparation, timing-backed synchronization and process state are implemented; other roots retain explicit scaffold status.

## Module ownership

[filesystem/](src/filesystem/mod.rs).

[objects/](src/objects/mod.rs).

[errno/](src/errno/mod.rs), [execution/](src/execution/mod.rs), [process/](src/process/mod.rs), [signals/](src/signals/mod.rs), [synchronization/](src/synchronization/mod.rs), [threading/](src/threading/mod.rs), [timing/](src/timing/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Guest library export contracts or application composition.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M25 separates the generic asynchronous mechanism into astero-timing. M40 now owns guest clock conversion and cancellable sleep tickets; guest HLE adapters remain in libs.

M29 execution/preparation implements guarded native stack/TLS storage, layout, explicit context and recovery/import-encoding contracts. No thread, FS switch or native transfer occurs. Dependencies on memory/ABI are now active.

M30 execution/host owns the private Windows x64 synthetic-tested bridge, FS restoration and process-exclusive VEH/TLS lifecycle. process/exit_callbacks owns bounded callback storage; it does not invoke callbacks. The only new unsafe exception is execution/host/platform.rs. See [native closure](../../knowledge/architecture/native_entry_closure.md).

M31 uses astero-timing deadlines for exact-thread native supervision in the existing private host leaf. See [first entry](../../knowledge/architecture/first_native_entry.md).

M32 process/environment retains bounded guest-owned value identities; exit callbacks distinguish guest and runtime-return targets. See [startup foundation](../../knowledge/architecture/startup_foundation.md).

M33 [pthread foundation](../../knowledge/architecture/pthread_foundation.md) implements owned synchronization and scoped guest adapters.

M34 [pthread lifecycle](../../knowledge/architecture/pthread_lifecycle.md) adapts owned worker lifetimes, shared process storage and exact lifecycle exports. Offline analysis remains independent.

M36 process/users owns bounded single-user session/event state; no host-account backend.

M37 native call capture retains a checked stack argument address for extended varargs;
process output supports bounded raw-byte append shared by printf-family and puts.

M40 [guest timing](../../knowledge/architecture/kernel_timing.md) shares M25 scheduling across clocks, sleeps and pthread absolute deadlines.

M42 process/guards owns bounded static-initializer exclusion using existing synchronization and timing.

M44 execution/host/sampling aggregates optional actual-PC captures in the existing owned watchdog. Synchronization exposes waiter identities and shutdown cancellations; thread storage exposes committed bytes and the table reports peak live/starting records. No new guest behavior. See [runtime observability](../../knowledge/architecture/runtime_observability.md).

M45 objects/memory and synchronization/semaphore own bounded direct-offset resources and M25-backed counting waits; both real workload frontiers advanced with clean teardown. See [kernel resources](../../knowledge/architecture/kernel_resources.md) for limits and evidence.

M47 filesystem owns bounded descriptors, explicit app0/data mounts and host I/O; see [filesystem startup](../../knowledge/architecture/filesystem_startup.md).
