# M42 native placement diagnostics, second-title execution and static initialization

## Outcome and evidence boundary

The same named_title_elf successfully mapped at the unchanged exact 4 GiB image bias and
executed to libc cnd_init. The primary passed eleven guard acquire/release pairs and reached
sceAjmInitialize. No title-specific placement, relocation, hash or call-site rule was added.

**The historical M41 Windows487 cause is not established.** That process was reaped without
VirtualQuery evidence. M42 cannot retrospectively identify its occupant. A host/process-layout
collision remains a hypothesis, not an observed ASLR diagnosis. M42 improves failure diagnostics,
not a demonstrated placement-policy defect. The requested range is accepted unchanged now.
Do not describe this as a reproduced-and-fixed historical collision or guarantee arbitrary-title placement.

## Focused references

Paths below are relative to the read-only PS5Rust Knowledge Catalogue, routed from
00_START_HERE/README.md and 04_MEMORY/INDEX.md / 05_KERNEL_HLE/INDEX.md:

- 04_MEMORY/address_space_contract.md: exact guest identity and typed failures.
- 04_MEMORY/source/ps5-core__src__memory__address_space/INDEX.md,
  002.md (main/dynamic address constants), 009.md (direct/placeholder allocation and teardown),
  010.md (exact-address regression leads). Prototype main base is 0x800000000; not a mandate
  to change Astero's already-selected image bias.
- 05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__exports/INDEX.md and 001.md,005.md
  route the large registration function; focused current source supplies the actual guard body.
- 05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__nids/INDEX.md,015.md:
  acquire/release/abort constants and finalize identity.
- Current PS5Rust crates/ps5-core/src/memory/address_space.rs:138-170,1910-2165:
  distinct page/allocation granularity, exact allocation, placeholder use for non-granular bases,
  and preserved-placeholder release ownership. No whole-address-space arena migration is justified
  by this already-aligned request. The source's prior unrelated high-address failure is not M41 proof.
- Current PS5Rust crates/ps5-libs/src/kernel/libc/exports.rs:3224-3350:
  finalize calls guest destructors; corrected competing-initializer serialization, guard completion,
  abort and controlled pure-virtual termination. nids.rs:338-357 supplies exact identities.
  libc.rs:875-930 has serialization/retry test source (not claimed executed here).
- Current PS5Rust kernel/libc/nids.rs:494 identifies second cnd_init; platform.rs:76 identifies AJM.
- Existing Astero windows_native_vm.md, runtime_entry_preparation.md, native_entry_closure.md,
  first_native_entry.md, pthread_foundation.md, pthread_lifecycle.md and libc_scalar_math.md.

No new firmware disassembly, external emulator implementation or broad catalogue import was needed.
The M41 real source hashes, load plans and current call records are the decrypted artifact evidence.
Windows API corroboration: [VirtualAlloc](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc)
and [VirtualQuery](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualquery).
Allocation and commitment granularities remain distinct; an existing reservation cannot be replaced
by another reserve. Query reports region metadata without reading its payload.

## Placement and ownership

Memory retains one exact identity reservation per NativeImage, rounded down to host allocation
granularity with page-rounded end. It reserves the complete image before commitment, copying,
protection and publication. Holes remain inaccessible. RAII rollback/release is unchanged; no RWX,
random fallback or host/guest translation was introduced. The second envelope is
0x100000000..0x101bc9000 (29,134,848 bytes). Base is 64 KiB aligned; page is 4 KiB.

NativeError::ReservationRefused now retains original Windows error, requested range, host geometry,
and at most 64 VirtualQuery records covering the failed envelope. Records retain raw state,
protection, type, allocation base and region range; complete/query_error explicitly mark incomplete
snapshots. Query runs only after failure and never reads payloads. There is no silent complete claim
on truncation. This is a racy diagnostic observation, never permission to overwrite another owner.
The original error is captured before any query can replace thread last-error.

Known intentional-collision tests identify their retained owner by allocation base. Production metadata
cannot name an unrelated host allocator from VirtualQuery alone; owner remains unknown. Initial image
realization precedes Astero guest stacks/TLS/landings/heap/audio/worker allocations. Source/staging
host storage and host thread stacks already exist; none is proven the M41 competing owner.
Main stack 0x200000000, TLS 0x210000000, later landing/foundation offsets and worker band
0x240000000 (16 MiB stride) are outside both image envelopes. No guest arena/early reservation change
was made without evidence of necessity. Genuine collisions still refuse safely, now with evidence.

## Static initialization ownership and ABI

Kernel process::guards owns a bounded private guard-to-synchronization-ID table; libs runtime::guards
owns exact libc/libc contracts. Core injects the runtime scheduler and existing guest Thread identity.
There are no new crates/dependencies/global maps/unsafe exceptions. All native unsafe remains in the
existing private memory leaf. Four providers form the migration: acquire,release,abort,pure_virtual.
Existing __cxa_atexit remains owned and records real callbacks; no host-pointer shortcut was added.

An aligned eight-byte readable/writable guest guard uses bit0 of byte0 as the inherited completion
indicator. Acquire takes exclusion before rereading guest bytes: zero returns1 with logical ownership
retained, complete releases exclusion and returns0. Release validates owner before publishing bit0,
preserves all other bytes, then wakes a waiter. Abort releases without completion so initialization
can retry. Guest fast-path reads observe the actual completion byte, not a side-table-only flag.
The x86-64 native memory path uses checked OS copies, not overlapping Rust guest references.
The existing mutex synchronization orders HLE participants; no cross-architecture atomic claim is made.

The state is Uninitialized/Owned-by-thread/Completed-by-guest-byte; no Rust MutexGuard survives an
HLE return. Recursive initialization stops as a diagnosed unsupported/deadlock policy rather than
hanging; wrong owner cannot publish. Null/misaligned/invalid/RO ranges refuse. Failed publication
stops without claiming completion; runtime shutdown cancels remaining exclusion waits.

4096 process guards,64 pending guard waiters, plus existing scheduler/global call/resource limits.
The guard table uses a private Synchronization instance so guest pthread handles cannot name it.
Its M25 tickets share the runtime deadline; shutdown/engine-stop interrupts waits, never pretends
initialization succeeded. Main/worker teardown cancels this service before joining workers.
No second timer, host sleep polling or host ThreadId ownership was introduced.

Prototype shortcuts rejected: null/read failure reported as initialized, ignored write errors,
process-global guard maps, forgotten host lock guards and force_unlock. Pure-virtual is a controlled
StopRequested with exact provider context, never a successful return. Abort and pure-virtual are
synthetic-tested only; no real throw/unwind behavior is claimed.

__cxa_finalize was reviewed but not migrated: it requires safe nested guest destructor invocation,
which the current first-entry bridge does not provide. Silently consuming callbacks or returning
success would be wrong. It remains explicitly unresolved. Full exceptions/destructors are deferred;
the real primary reached AJM without requesting them. One scoped guard ABI record is added (six total).

## Real commands and results

Local scripts target/m42-placement-probe.ps1, target/m42-second.ps1 and target/m42-primary.ps1
use LOCAL_TEST_CORPUS.json only. Exact repeatable commands are in USAGE. Both execution runs use
250 ms wall,15000 ms containment,65536 HLE calls. No extra title was selected and no depth-chasing
rerun followed either new out-of-scope stop. Synthetic and real code remain clearly separated.

First diagnostic native-map of named_title_elf succeeded: 7107 committed pages, exact envelope,
protections/readback/cache finalization verified, zero native/byte mappings after release, no code.
The subsequent contained first-entry run issued EntryReady and executed:

| Evidence | Primary | named_title_elf |
|---|---|---|
| Source bytes | 9,225,340 | 27,908,888 |
| Image envelope | 0x100000000..0x1010a0000 | 0x100000000..0x101bc9000 |
| Segments/dependencies | 5 / 38 | 5 / 36 |
| External references / relocations | 822 / 30,898 | 579 / 29,569 |
| Raw / native entry | 0x70 / 0x100000070 | 0x70 / 0x100000070 |
| Initial RSP | 0x200800fb8 | 0x200800fb8 |
| Initial FS | 0x210000000 | 0x2100000a0 |
| Result | UnresolvedFunction sceAjmInitialize | UnresolvedFunction cnd_init |
| Key | 0x765FB87874B352EE libSceAjm/libSceAjm | 0x4AB799C9B4915A95 libc/libc |
| Ordinal | 570 | 1681 |
| Captured boundary RIP (Astero landing) | 0x7ff64723e816 | 0x7ff78d10bc76 |
| RSP at boundary | 0x2008007d8 | 0x200800f58 |
| Separate guest import return address | 0x1003c6ab3 | 0x101676d50 |
| Overall/native interval | 180.274 / 179.003 ms | 0.578 / 0.311 ms |
| Guest workers | 8 | 0 |
| Retained callbacks (not invoked) | 42 | 7 |
| Supervisor redirect/suspend/resume | 0 / 0 / 0 | 0 / 0 / 0 |
| FS restored / GS preserved | yes / yes | yes / yes |
| Threads joined / reservations / release errors | yes / 0 / [] | yes / 0 / [] |
| Containment | Clean | Clean |

Primary passed11 acquire/release pairs (abort0), including guards0x100fe0d40,0x100fe0d60,
0x100fe0d80. Acquires returned1, releases0. Existing process/libc/sync/audio/timing paths continued;
10514 sincosf calls,heap peak1,055,728 bytes,101 allocations/11 frees. This is real completion evidence,
not real contention/abort proof. Eight existing workers stopped/joined; waiting condition calls interrupted.

Second calls: _init_env1,atexit2,__cxa_atexit5; no worker,heap allocation,audio submission or timing wait.
Arguments at cnd_init: [0x101b13498,0x101a46148,0,0x101ae2c68,0xd0,0xe0].
Primary AJM arguments: [0,0x21005ee38,0x10073ec40,0x66fc8,0xd0,0xe0].
Neither stop is a guest fault or guarded-object fault. Reported RIP is an import landing, not a
falsely inferred executing guest PC. Both exact source SHA256 values remain unchanged:

- primary A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A
- second 3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327

Unchanged loading/native identity, bridge, TLS preparation, exact startup providers, recovery and
teardown generalize to this second source. Worker/audio/math demand differs substantially. No physical
sound/game-boot/security-sandbox claim follows. Second-title behavior was observed only; no C11 provider
was migrated to chase it.

## Findings and handoff

Supported: exact second envelope and second native startup work on this host; guard serialization,
abort and interruption pass synthetic tests; real primary guard publication advances to AJM.
Refined: the M41 failure does not prove deterministic invalid geometry or title-incompatible placement.
Unresolved: historical competing allocation, arbitrary future collisions, native guard fast-path
semantics outside this ABI, initializer exceptions/destructor invocation. No fabricated root cause.
M43 recommendation: AJM/media-startup migration for primary, with a bounded C11 condition/mutex adapter
wave for the second cnd_init boundary; retain placement telemetry for any recurrence. M43 not started.

## Changed files

- `USAGE.md`
- `crates/astero-core/README.md`
- `crates/astero-core/src/input/entry/closure.rs`
- `crates/astero-core/src/input/entry/workers.rs`
- `crates/astero-core/tests/cxx_guards.rs`
- `crates/astero-kernel/README.md`
- `crates/astero-kernel/src/process/guards.rs`
- `crates/astero-kernel/src/process/mod.rs`
- `crates/astero-kernel/src/synchronization/mutex/operations.rs`
- `crates/astero-libs/README.md`
- `crates/astero-libs/src/runtime/guards.rs`
- `crates/astero-libs/src/runtime/mod.rs`
- `crates/astero-memory/README.md`
- `crates/astero-memory/src/mapping/windows_native/model.rs`
- `crates/astero-memory/src/mapping/windows_native/platform.rs`
- `crates/astero-memory/tests/native.rs`
- `knowledge/architecture/README.md`
- `knowledge/architecture/native_placement_static_init.md`
- `knowledge/architecture/validation.md`
- `knowledge/indexes/ABI_INDEX.md`
- `knowledge/indexes/DIAGNOSTIC_INDEX.md`
- `knowledge/indexes/IMPLEMENTATION_INDEX.md`
- `knowledge/indexes/MODULE_INDEX.md`
- `knowledge/indexes/NID_INDEX.md`
- `knowledge/indexes/SUBSYSTEM_INDEX.md`
- `knowledge/indexes/TEST_INDEX.md`
- `tools/index_links.json`
