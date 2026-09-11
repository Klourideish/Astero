# M43 AJM and C11 startup migration

## Scope and ownership

M43 adapts established PS5Rust startup mechanisms into the existing audio and libc homes.
Audio `codecs/ajm` owns bounded context, codec declarations, instances and registered ranges.
Libs `media/ajm` owns exact libSceAjm identities and checked guest serialization.
Libs `libc/c11` delegates to the same runtime-owned kernel Synchronization and explicit Thread
identity already used by pthread. Core owns both service lifetimes. No new crates or unsafe code.
Libs has a new **dev-only** timing dependency to test shared-service adapters. Production adapters
never construct timing engines. Existing source roots are the narrow declared owners; there is
no loose top-level subsystem file or parallel synchronization engine.

## Evidence consulted

PS5Rust Knowledge Catalogue relative routes/records:

- `05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__c11_sync/INDEX.md`, `001.md`, `002.md`:
  pointer-sized opaque slots, ownership, waiter registration, wrapper return mapping.
- `13_NETWORK_PLATFORM/INDEX.md`, `source/ps5-libs__src__platform/INDEX.md`, `006.md`:
  AJM registration routing and surrounding platform implementation.
- `11_AUDIO_MEDIA/INDEX.md`, `atrac9_decode.md`: actual decode is a separate codec mechanism;
  lifecycle registration cannot substantiate decoded PCM.
- `15_FIRMWARE_KNOWLEDGE/INDEX.md`; `modules/libSceAjm.sprx/INDEX.md`, `001.md`, `002.md`:
  exact export identities, invalid-context/parameter constants and configuration veneer.
- `15_FIRMWARE_KNOWLEDGE/modules/libSceAjm.native.sprx/INDEX.md`, `001.md`:
  batch cancellation evidence, not a complete job-stream contract.
- `15_FIRMWARE_KNOWLEDGE/modules/libSceLibcInternal.sprx/043.md`:
  `_Mtx_init` delegates to `_Mtx_init_with_name`; bounded constants do not prove complete type semantics.

Read-only current prototype source, relative to PS5Rust:
`crates/ps5-libs/src/platform.rs` (AJM registrations and batch handlers),
`kernel/libc/c11_sync.rs`, `kernel/libc/exports.rs` and `kernel/libc/nids.rs` under the same src root;
`atrac9/config.rs` and `atrac9/tables.rs` for configuration bitfields/tables.
`kernel/memory.rs` and `kernel/sync.rs` identify the final unresolved imports only.
Approved `docs/firmware/403/generated/veneer_records.jsonl` records for `_Mtx_init` and `_Thrd_create`
were inspected as bounded documentary evidence. No raw firmware payload or external emulator
implementation was opened. Prototype comments referencing SharpEmu/Kyty are secondary attribution,
not an independent comparative validation in M43.

Current artifact evidence is M42's durable observations plus the contained M43 executions below.
M25/M33/M34/M38/M39/M40/M42 architecture and existing code remain the ownership baseline.
No claim that the two decrypted executables have equivalent ABI/behavior follows from their shared imports.

## AJM mechanism and limits

One runtime-owned mutex protects AJM bookkeeping. IDs are monotonic u32 values shared across
context/instance allocation, never reused; context ownership is checked for instance operations.
Default core capacity is 4096 aggregate context/module/instance/memory records. Exact boundary
refuses structurally rather than inventing an AJM allocation error. Failed guest handle output
rolls back creation; IDs may be consumed but cannot alias a later object.

Codec registration retains declarations, including unknown IDs, and is **not decoder availability**.
Instance flags are retained opaque; no fabricated channel/format interpretation. Instances require
registration in their own context. Unregistering a codec with live instances refuses; this is an
Astero lifetime adaptation of the prototype's unconditional teardown. Range registration retains
base and length after complete checked readable-range validation; it is not pinning or ownership of
those guest bytes. No asynchronous consumer retains raw guest memory. Any future job must revalidate
or copy its inputs and bind their lifetime explicitly.

Initialize/finalize, module register/unregister, memory register/unregister, instance create/destroy
and ATRAC9 configuration are implemented. ATRAC9 configuration consumes four big-endian bytes and
writes exactly five little-endian u32 fields (20 bytes): channels, rate, frame samples, superframe
samples and superframe bytes. Reserved channel indices 6/7 refuse instead of indexing past the
prototype table. The reference vector FE740BF0 yields 2,48000,256,1024,384. No decoder is migrated.

Batch/job execution, wait/poll/result publication remain **unsupported and unregistered**. This is
an explicit limitation of the migration, not successful null decoding. The inspected prototype
BatchInitialize selects the first context, Start selects the latest batch and Wait can label it
completed without associated work. That is not a reusable association contract. No batch can be
submitted in Astero, so there are no job buffers, outstanding completions or audio timing tickets.
An attempted job import takes the existing structured unresolved-provider stop. A future batch
implementation must associate an explicit context, serialized job buffer and per-job result before
submission, and must not publish completion without codec work. The real workload did not reach
that architecture-changing path; M43 does not implement the full requested future batch lifecycle.

Shutdown clears all owned AJM records and permanently prevents creation. It runs on both controller
and worker stop paths; no audio device/backend or independent timer is involved. Snapshot counts
are observed after shutdown; zero counts do not mean no context was created during the run.

## C11 adapters

Nine exact libc/libc Dinkumware wrappers reuse M33 synchronization. An eight-byte guest slot stores
an internal token, never a host pointer. Kind and address validation remains in the owning service;
M38's copied-mutex-token policy is preserved. Condition slots retain their original-address identity.
Successful destroy clears the slot; busy destroy retains both slot and object.

Mutex lock uses the prototype non-recursive/errorcheck behavior: own-lock deadlock maps to 3,
invalid to 4, allocation/init failure to 2, success to 0. `_Mtx_unlock` and void destroy/signal/wait
wrappers discard underlying pthread errors as the reviewed prototype does; they still enforce
ownership internally. A discarded error does not unlock another thread's object. Guest range
errors are structural HLE failures. Runtime interruption is always StopRequested, never a fabricated
successful wake or resume with an unreacquired mutex.

Mode 0 and mode 2 are admitted with the prototype errorcheck policy. The first second-title run
refused mode 2; the subsequent run validated successful initialization. Exact timed/recursive flag
semantics remain an explicit limited-policy hypothesis; arbitrary other values refuse. This is not
full standard-C11 conformance. No timed or try NIDs were invented where the reviewed prototype did
not provide them.

Condition waits register their M25 ticket and release the mutex under the kernel's shared lock,
then wait without that lock and reacquire before normal return. Signal/broadcast and shutdown use
existing mechanisms. No polling sleep, host-thread identity registry or unsafe forced host unlock.
Synthetic adapter tests prove wait/reacquisition and shutdown interruption; the real second run
only exercised initialization, not these waits.

`_Execute_once` is deliberately unregistered: its known callback signature requires nested native
execution from HLE, which the current single bridge authority does not support. Reusing M42 guards
alone cannot truthfully execute the callback. No callback is silently skipped or reported complete.
Thread creation/self helpers have no new C11 registration here; M34 remains the mechanism owner.

## NIDs and ABI

18 registrations (9 AJM, 9 C11); only four identities returned through real guest calls.
Two prior observations are promoted; two new kernel boundaries remain observations. Totals:
340 registered, 4 observation-only/unregistered; 344 NID records. ABI remains at six records;
configuration serialization and opaque slot adaptation are documented here without claiming a
new firmware-complete ABI contract.

| Export | Numeric NID | M43 status |
|---|---|---|
| sceAjmInitialize | `0x765FB87874B352EE` | Runtime returned |
| sceAjmFinalize | `0x307BABEAA0AC52EB` | Registered; not runtime-confirmed |
| sceAjmModuleRegister | `0x43777216EC069FAE` | Runtime returned |
| sceAjmModuleUnregister | `0x5A2EC3B652D5F8A2` | Registered; not runtime-confirmed |
| sceAjmMemoryRegister | `0x6E44471181BA9443` | Registered; not runtime-confirmed |
| sceAjmMemoryUnregister | `0xA48A4689A6241E43` | Registered; not runtime-confirmed |
| sceAjmInstanceCreate | `0x031A03AC8369E09F` | Registered; not runtime-confirmed |
| sceAjmInstanceDestroy | `0x45B2DBB8ABFCCE1A` | Registered; not runtime-confirmed |
| sceAjmDecAt9ParseConfigData | `0xD6DDE2C58357CAE7` | Registered; not runtime-confirmed |
| _Mtx_init | `0x61A1DCDC64BBCBB8` | Runtime returned |
| _Mtx_lock | `0x892E1A59B5289E5D` | Registered; not runtime-confirmed |
| _Mtx_unlock | `0x813B974303FDAEBB` | Registered; not runtime-confirmed |
| _Mtx_destroy | `0xE4B7F9D63BE88534` | Registered; not runtime-confirmed |
| _Cnd_init | `0x4AB799C9B4915A95` | Runtime returned |
| _Cnd_signal | `0xD2EBAA811CFDA9FA` | Registered; not runtime-confirmed |
| _Cnd_broadcast | `0x56C3F775A2609950` | Registered; not runtime-confirmed |
| _Cnd_wait | `0xBC46AA13FEC86587` | Registered; not runtime-confirmed |
| _Cnd_destroy | `0xEF230581C4BC10F0` | Registered; not runtime-confirmed |

## Real workload observations

Both commands use LOCAL_TEST_CORPUS roles and the unchanged first-entry controller: 250 ms wall,
15000 ms containment, 65536 HLE calls. Exact reusable commands are in USAGE. No extra title ran.

Primary (`primary_real_elf`, `target/m43-primary.ps1`): Initialize(0,0x21005ee38) returned 0,
context ID1 was consumed by four successful ModuleRegister calls for IDs1,0,2,0x18.
No instance, registered memory, batch, decode, completion or wait call occurred.
New stop: UnresolvedFunction, `sceKernelGetDirectMemorySize`, NID0xA4EF7A4F0CCE9B91,
libkernel/libkernel, ordinal502. Arguments [1,0x210042be0,0,0x600000,9,0x2008007e0].
Boundary RIP0x7ff649f177b6, RSP0x200800778; **separate guest import return address**0x1003c8006.
169.286 ms overall,168.166 ms supervised native interval. Eight workers joined; no active waiters.

Second (`named_title_elf`, `target/m43-second.ps1`): first run passed CndInit and refused
MtxInit(mode2). Final run passed both: condition slot0x101b13498 and mutex slot0x101b134a0.
One call each; no C11 waits/locks/signals observed. New stop: UnresolvedFunction,
`sceKernelCreateSema`, NID0xD7CF31E7B258A748, libkernel/libkernel, ordinal1209.
Arguments [0x200800f40,0x10174b2ac,1,0,0x7fffffff,0]. Boundary RIP0x7ff7515477d6,
RSP0x200800f38; separate guest import return address0x1014e5936. Overall1.171 ms,
0.902 ms supervised. No guest worker created. Semaphores are outside this C11 cluster.

Both boundary RIPs are Astero import landings, not sampled guest instruction PCs. No guest fault
or guarded object caused these stops. Supervisor remained armed; no suspension/redirection needed.
FS restored, GS preserved, thread joined, zero native/runtime reservations, no release errors,
Clean containment. AJM counts zero after shutdown; synchronization waiters zero. Source hashes:

- primary: A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A
- second: 3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327

Before/after equality was verified for every M43 run. Historical Windows error487 remains
unresolved/unreproduced. M43 did not change placement or infer that failure's cause.

## Findings and next pressure

Supported: owned AJM initialization/codec declarations advance primary; shared-service C11 slots
advance second; checked publication/teardown work. Refined: rejecting all nonzero mutex modes was
too restrictive; mode2 uses the explicitly limited prototype policy. Rejected: global registries,
unsafe cross-call host mutex guards, latest/first batch association and fabricated decode completion.
Unresolved: full C11 mode semantics/copyable condition tokens, nested once callbacks, exact AJM
job-stream/result association and compressed decoding. None gained unobserved runtime confirmation.

Recommend M44 as a kernel resource foundation wave driven by the two observed boundaries:
direct-memory query/allocation policy and semaphore lifecycle/waits, retaining existing memory and
timing owners. This is a recommendation only; M44 has not started.

Validation commands/results are recorded in validation.md. M43 remains active pending cleanup
approval. No commit, push, history rewrite or source-artifact mutation.

## Changed-file manifest

- `Cargo.lock`
- `USAGE.md`
- `crates/astero-audio/README.md`
- `crates/astero-audio/src/codecs/mod.rs`
- `crates/astero-cli/src/entry/first.rs`
- `crates/astero-core/src/input/entry/closure.rs`
- `crates/astero-core/src/input/entry/workers.rs`
- `crates/astero-libs/Cargo.toml`
- `crates/astero-libs/README.md`
- `crates/astero-libs/src/libc/mod.rs`
- `crates/astero-libs/src/media/mod.rs`
- `knowledge/architecture/README.md`
- `knowledge/architecture/crate_boundaries.md`
- `knowledge/architecture/dependency_policy.json`
- `knowledge/architecture/validation.md`
- `knowledge/indexes/DIAGNOSTIC_INDEX.md`
- `knowledge/indexes/IMPLEMENTATION_INDEX.md`
- `knowledge/indexes/MODULE_INDEX.md`
- `knowledge/indexes/NID_INDEX.md`
- `knowledge/indexes/SUBSYSTEM_INDEX.md`
- `knowledge/indexes/TEST_INDEX.md`
- `tools/index_links.json`
- `crates/astero-audio/src/codecs/ajm.rs`
- `crates/astero-audio/src/codecs/atrac9.rs`
- `crates/astero-libs/src/libc/c11.rs`
- `crates/astero-libs/src/media/ajm.rs`
- `crates/astero-libs/tests/media_c11.rs`
- `knowledge/architecture/ajm_c11_startup.md`
