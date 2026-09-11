# M34 pthread lifecycle migration

This capability adapts the PS5Rust pthread lifecycle into Astero's native Windows runtime.
It retains the M33 `Thread` identity and synchronization service, the M30 bridge and M31 supervisor.
It does not introduce another CPU execution model, guest scheduler or host timer implementation.

## Evidence and adaptations

Catalogue-relative routing: `00_START_HERE/README.md`, `catalogue_map.md`,
`05_KERNEL_HLE/source/ps5-libs__src__kernel__threading__pthread_attr/INDEX.md` and `001.md`, `002.md`;
`02_CORE_EXECUTION/native_boundary.md`,
`02_CORE_EXECUTION/source/ps5-core__src__cpu__native_exec/INDEX.md`, and
`02_CORE_EXECUTION/source/ps5-core__src__cpu__target_thread/INDEX.md`, `003.md`.
Focused original prototype source inspected (paths relative to PS5Rust):

- `crates/ps5-libs/src/kernel/threading/{exports,pthread_attr,pthread_misc,state,nids}.rs`:
  exact registrations, opaque attribute slots, two-argument 32-byte getname, SCE error forwarding,
  2 MiB default stack, all-ones guard sentinel, create hooks and join/detach behavior.
- `crates/ps5-core/src/cpu/dispatcher.rs`: `spawn_guest_thread_with_hooks_and_cleanup`,
  guarded worker stack, target identity, post-TLS cleanup, join/detach and shutdown paths.
- `crates/ps5-core/src/cpu/native_exec.rs`: per-thread TLS template/TCB, scratch cleanup,
  documented fixes for shared TLS and leaking host argument registers into guest entry.

These source observations are the migration baseline, not freshly run PS5Rust tests. No external
emulator was needed. Existing M25/M29/M30/M31/M33 architecture records remain authoritative.
Prototype shortcuts rejected: dropping detached host handles, reporting a faulted join as success,
failing to publish join's result, launching before output publication, permissive non-executable
start pointers, loose global registries and shared TCB storage. No prototype source was modified.

## Ownership and lifecycle

Kernel `threading/thread` owns bounded attributes and the worker table. Libs `pthread/thread` and `pthread/attributes`
own exact guest calls and error adaptation. Core `input/entry/workers` composes memory, provider
views and native execution. There are no new crate dependencies. Frontends only present results.

Attribute slots contain an opaque 64-bit token; validation requires token and original slot address.
Thread handles are the existing `Thread`: main=1, monotonically allocated workers from 2, no reuse
within this runtime. Main identity is not the Windows thread ID. Names are bounded raw bytes,
never ownership keys. Guest destroy invalidates the slot; it does not accept copied token aliases.

Worker transitions: Created -> Running -> Exited -> Joined; detached Exited -> Reclaimed at host
reaping. Failure before publication becomes Failed. Create validates executable image coverage and
the output slot, prepares storage, then spawns behind a publication gate. Failed publication joins
the unstarted host. Join rejects self/detached/invalid/already joined/concurrent join; it waits on an
M25 ticket and retains the native return value. Faulted worker completion is explicit failure.
Guest detach never drops an active host JoinHandle. Storage is released at worker completion;
bounded diagnostic tombstones/host handles remain owned until reaping. No guest callbacks or TLS
destructors execute on completion in this wave.

Runtime shutdown prevents creation, interrupts synchronization/join tickets and joins every worker
before image/landings/traps are released. A native loop retains the existing asynchronous supervisor;
shutdown may wait until the already armed runtime deadline. Host/HLE PCs are never redirected as
guest PCs. Per-worker native bounds use remaining runtime monotonic time, capped at 500 ms, with
millisecond rounding and host scheduling latency; there is no instruction-count guarantee.

## Storage, bridge and process sharing

Default workers receive 2 MiB usable stacks, minimum 16 KiB and maximum 8 MiB, page multiples.
The memory backend owns exact VM placement, guard protection and release. The initial placement
policy uses a separate worker address band with 16 MiB slots; address conflicts refuse, never relocate
silently. Each worker copies the same PT_TLS template into independent storage, zero fills tbss/TCB
slack and gets its own FS/errno. M29's 0x200-byte variant-II TCB remains experimental beyond tested
template/self-pointer/errno behavior. Dynamic TLS modules and key destructor ordering are deferred.

Worker entry uses RDI=start argument, cleared other guest argument registers, owned stack RET
landing and the existing bridge's full host preservation. A transferable adapter lease retains the
single process VEH/TLS slot across thread-affine attachments; each host thread owns its own frame.
Nested active frames on one host thread refuse. Last adapter release removes VEH then frees TLS;
another runtime remains excluded until all leases end. Worker return and pthread_exit converge into
completion, with FS restored and GS preserved. Unresolved providers and faults stop the runtime.

Published native memory is shared through Arc owners; release/protection need exclusive ownership.
The private memory backend uses OS checked read/write copies, never Rust references into guest
bytes. Concurrent guest writes are not a readback-equality guarantee. Immutable image/RELRO ownership
cannot disappear during a worker lease. Process heap/environment/callback storage are shared under
owned locks; per-thread registries carry the correct errno and M33 caller identity. No lock spans a
blocking HLE call merely to protect all guest memory.

## Supported boundary and limits

The exact migration table is `astero-libs/src/pthread/thread/exports.rs`. It includes SCE/POSIX
attribute lifecycle/getters/setters, create, join, detach, exit, self/equal/thread ID and name helpers.
Unsupported custom stacks and nondefault host scheduling refuse with ENOTSUP; metadata-only success
is not a claim of host affinity/priority enforcement. SCE wrappers retain 0x80020000 error namespace;
POSIX returns raw errno. No new errno mutation is invented for pthread return-code APIs.

Policy bounds: 32 worker creation records per run (including failed/retired records, no reuse),
256 live attribute objects, maximum 16 MiB worker storage each, 31 name bytes, 256 provider entries,
4096 process-wide provider attempts, and existing M33 object/waiter/timing limits. Refusal is explicit.
There is no full pthread scheduler, recursive loader, filesystem, GPU, audio or C++ unwind support.
The trusted-input native runtime is not a security sandbox.

## Real workload outcome (one run, 2026-09-11)

`primary_real_elf` was selected through LOCAL_TEST_CORPUS.json. The exact validated command is in
USAGE.md. Required bounds remained 250 ms native execution and 15000 ms parent containment.
Preflight fmt/check/build/warnings-denied Clippy, complete workspace Rust tests, synthetic native
workers/supervision and policy checks passed before this single real execution. Exit code was 0,
containment Clean. The previous M33 AttrInit stop was passed.

Eight instances each of AttrInit, AttrSetstacksize(0x4000), AttrSetschedpolicy(1),
AttrSetinheritsched(0), Create and AttrDestroy succeeded. Eight AttrSetschedparam calls explicitly
returned SCE ENOTSUP (0x8002002d); nondefault priority was not installed. The workload continued
with the retained default 700. This is runtime-confirmed refusal, not implemented host scheduling.
All eight workers entered start RIP 0x1001c2a20, with raw name `SampleUtilJobQueue`.

| Guest thread | Guarded stack usable start | Initial RSP | FS/TCB | Argument |
|---|---|---|---|---|
| 2 | 0x240001000 | 0x240004fb8 | 0x240900000 | 0x2100417e0 |
| 3 | 0x241001000 | 0x241004fb8 | 0x241900000 | 0x210041890 |
| 4 | 0x242001000 | 0x242004fb8 | 0x242900000 | 0x210041940 |
| 5 | 0x243001000 | 0x243004fb8 | 0x243900000 | 0x2100419f0 |
| 6 | 0x244001000 | 0x244004fb8 | 0x244900000 | 0x210041aa0 |
| 7 | 0x245001000 | 0x245004fb8 | 0x245900000 | 0x210041b50 |
| 8 | 0x246001000 | 0x246004fb8 | 0x246900000 | 0x210041c00 |
| 9 | 0x247001000 | 0x247004fb8 | 0x247900000 | 0x210041cb0 |

Each had 16 KiB usable stack, 4 KiB guard and independent 4 KiB TLS mapping; PT_TLS template and
memory extent were zero. Successful workers corroborate the tested initial TCB subset, not full
libc/dynamic TLS semantics. Host Windows IDs were 0x3158, 0x1100, 0x7e0, 0x60c, 0x57a0, 0x2868,
0x52d8 and 0x5eb8 respectively. Guest IDs remain independent of these ephemeral host identifiers.

Seven workers completed MutexLock and entered CondWait. Objects: rwlock 0x210041678,
condition 0x2100417a8, mutex 0x2100417b0. Seven waits, zero signalled wakes, zero timeouts;
all waiters were interrupted by runtime shutdown. The eighth worker reached the mutex landing
while shutdown was already requested and did not dispatch. M25 tickets participated in blocked
waits; the native supervisor remained armed. No worker required suspend/context redirection in
this real run. Every worker restored host FS and preserved GS, then joined during owner teardown.
No real guest join, detach, normal worker return or pthread_exit was observed; those have synthetic
coverage only. Final waiters=0, stopped=true.

The main thread stopped at **ProviderRefused / AccessError::Limit**, libc/libc `memset`, NID
**0xf334c5bc120020df**, ordinal 40. Arguments were
`[0x100906e68, 0, 0x14a140, 0x210041c70, 0x78, 0x80]`.
The 1,352,000-byte request exceeded the existing M32 1,048,576-byte operation limit; the checked
count refused before any bytes of that operation were written. This is outside lifecycle scope.
No limit was raised and no second real run was made.

Initial main RIP=0x100000070, RSP=0x200800fb8, FS=0x210000000. Captured boundary
RIP=0x7ff7803a1436 is a **host import landing**, not an executing guest PC;
RSP=0x200800f08. Separate guest return/continuation address=0x1001bf259. No exception/fault.
Elapsed report time=10,121 us; armed main native interval=8,975 us; main Windows thread=6012.
198 main provider records (197 returned, final memset refused), 14 dispatched worker records
(7 locks returned, 7 condition waits interrupted). Heap: 62 allocations/10 frees, 52 live allocations,
3392 bytes before teardown. Sixteen callbacks retained, none invoked.

All execution threads joined; image/main/worker mappings and reservations=0, release errors=[];
source SHA256 before and after was
`A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A`.
The ignored raw audit log is target/m34-real-1.log. It originally rendered all nonnormal worker
stops as a fault boolean and printed the RAX lane without completion disposition. Post-run diagnostics
were refined to typed ThreadEnd (Interrupted vs ProviderStop vs NativeFault) and CallResult;
synthetic blocked-worker tests verify Interrupted/StopRequested. Zero in an interrupted wait's
return lane is **not** a successful guest wait return. The original run was not repeated or relabelled
as a successful wake.

## Limits, hypotheses and next pressure

Guard attr 0 is retained as a request, but this experimental native backend always enforces a minimum
4 KiB safety guard; MAX means default. It does not claim arbitrary guard/custom-stack support.
The observed 16 KiB stacks, independent TCBs and shared synchronization support the adapted worker
bootstrap. Publication rollback, join/detach/exit and errno isolation are synthetically validated.
Unresolved: full TCB/DTV and TLS destructors; scheduling beyond default; detach reclamation latency
(host handles retained until reaping); comprehensive guest attr layout compatibility. No callbacks,
constructors or TLS destructors are executed during worker teardown.

The workload-driven next recommendation is bounded larger libc memory operations/startup continuation,
starting with the observed memset refusal. M35 has not started. ABI remains two records: process
arguments and synthetic native boundary. Worker start RDI/return preservation is covered by the latter;
M34 does not claim a new complete firmware pthread-layout ABI.

## Migrated identities

61 lifecycle identities are registered with exact libkernel/libkernel context: 60 new records plus
promotion of the existing AttrInit observation. Total: 176 NIDs, 174 registered, 2 observation-only.
Unexercised registrations remain migrated, not runtime-confirmed. Unsupported attr setters are
registered explicit refusals, not fabricated success. Full final validation is in validation.md.

| Export | NID | M34 real observation |
|---|---|---|
| SCE_PTHREAD_CREATE | 0xE9482DC15FB4CDBE | 8 successful calls |
| SCE_PTHREAD_JOIN | 0xA27358F41CA7FD6F | Not exercised |
| SCE_PTHREAD_DETACH | 0xE2A1AB47A7A83FD6 | Not exercised |
| PTHREAD_CREATE | 0x3B184807C2C1FCF4 | Not exercised |
| PTHREAD_CREATE_NAME_NP | 0x2668BEF70F6ED04E | Not exercised |
| PTHREAD_JOIN | 0x87D09C3F7274A153 | Not exercised |
| PTHREAD_DETACH | 0xF94D51E16B57BE87 | Not exercised |
| PTHREAD_EXIT | 0x149AD3E4BB940405 | Not exercised |
| SCE_PTHREAD_EXIT | 0xDE483BAD3D0D408B | Not exercised |
| SCE_PTHREAD_GETTHREADID | 0x108FF9FE396AD9D1 | Not exercised |
| PTHREAD_GETTHREADID_NP | 0xDDEAACDFB1BBE3FB | Not exercised |
| SCE_PTHREAD_ATTR_INIT | 0x9EC628351CB0C0D8 | 8 successful calls |
| PTHREAD_ATTR_INIT | 0xC2D92DFED791D6CA | Not exercised |
| SCE_PTHREAD_ATTR_SETSTACKSIZE | 0x5135F325B5A18531 | 8 successful calls |
| PTHREAD_ATTR_SETSTACKSIZE | 0xD90D33EAB9C1AD31 | Not exercised |
| SCE_PTHREAD_ATTR_SETSCHEDPARAM | 0x0F3112F61405E1FE | 8 calls; ENOTSUP |
| PTHREAD_ATTR_SETSCHEDPARAM | 0x7AE291826D159F63 | Not exercised |
| SCE_PTHREAD_ATTR_SETSCHEDPOLICY | 0xE3E87D133C0A1782 | 8 successful calls |
| PTHREAD_ATTR_SETSCHEDPOLICY | 0x25AACC232F242846 | Not exercised |
| SCE_PTHREAD_ATTR_GETSCHEDPARAM | 0x1573D61CD93C39FD | Not exercised |
| PTHREAD_ATTR_GETSCHEDPARAM | 0xAA593DA522EC5263 | Not exercised |
| SCE_PTHREAD_ATTR_SETINHERITSCHED | 0x7976D44A911A4EC0 | 8 successful calls |
| PTHREAD_ATTR_SETINHERITSCHED | 0xED99406A411FD108 | Not exercised |
| SCE_PTHREAD_ATTR_SETDETACHSTATE | 0xFD6ADEA6BB6ED10B | Not exercised |
| PTHREAD_ATTR_SETDETACHSTATE | 0x13EB72A37969E4BC | Not exercised |
| SCE_PTHREAD_ATTR_GETSTACKSIZE | 0xFDF03EED99460D0B | Not exercised |
| PTHREAD_ATTR_GETSTACKSIZE | 0xD2A3AD091FD91DC9 | Not exercised |
| SCE_PTHREAD_ATTR_SETSTACK | 0x06F9FBE2F8FAA0BA | Not exercised |
| PTHREAD_ATTR_SETSTACK | 0xFD2ADB5E9191D5FD | Not exercised |
| SCE_PTHREAD_ATTR_GETSTACK | 0xFEAB8F6B8484254C | Not exercised |
| PTHREAD_ATTR_GETSTACK | 0xBD09B87C312C5A2F | Not exercised |
| SCE_PTHREAD_ATTR_SETSCOPE | 0x61D65F1197D19CF9 | Not exercised |
| PTHREAD_ATTR_SETSCOPE | 0xC5EB2695223F2822 | Not exercised |
| SCE_PTHREAD_ATTR_GETSCOPE | 0xFBB07600428A9ECF | Not exercised |
| PTHREAD_ATTR_GETSCOPE | 0x7B61BE71D1243A65 | Not exercised |
| PTHREAD_ATTR_GETDETACHSTATE | 0x5544F5652AC74F42 | Not exercised |
| PTHREAD_ATTR_GETGUARDSIZE | 0x24D91556C54398E9 | Not exercised |
| PTHREAD_ATTR_GETINHERITSCHED | 0xA0B8CFA942A1CDEB | Not exercised |
| PTHREAD_ATTR_GETSCHEDPOLICY | 0x46D2D157FA414D36 | Not exercised |
| PTHREAD_ATTR_GETSTACKADDR | 0x0F198831443FC176 | Not exercised |
| PTHREAD_ATTR_SETGUARDSIZE | 0x24AC86DD25B2035D | Not exercised |
| PTHREAD_ATTR_SETSTACKADDR | 0xB2E0AB11BAF4C484 | Not exercised |
| SCE_PTHREAD_ATTR_GETDETACHSTATE | 0x25A44CCBE41CA5E5 | Not exercised |
| SCE_PTHREAD_ATTR_GETGUARDSIZE | 0xB711ED9E027E7B27 | Not exercised |
| SCE_PTHREAD_ATTR_GETINHERITSCHED | 0x96930FF0786405B8 | Not exercised |
| SCE_PTHREAD_ATTR_GETSCHEDPOLICY | 0x34CC8843D5A059B5 | Not exercised |
| SCE_PTHREAD_ATTR_GETSTACKADDR | 0x46EDFA7E24ED2730 | Not exercised |
| SCE_PTHREAD_ATTR_SETGUARDSIZE | 0x125F9C436D03CA75 | Not exercised |
| SCE_PTHREAD_ATTR_SETSTACKADDR | 0x17EC9F99DB88041F | Not exercised |
| SCE_PTHREAD_ATTR_DESTROY | 0xEB6282C04326CDC3 | 8 successful calls |
| PTHREAD_ATTR_DESTROY | 0xCC772163C7EDE699 | Not exercised |
| SCE_PTHREAD_SELF | 0x688F8E782CFCC6B4 | Not exercised |
| PTHREAD_SELF | 0x128B51F1ADC049FE | Not exercised |
| SCE_PTHREAD_EQUAL | 0xDCFB55EA9DD0357E | Not exercised |
| PTHREAD_EQUAL | 0xED7976E7B33854D2 | Not exercised |
| SCE_PTHREAD_RENAME | 0x181518EF2C1D50B1 | Not exercised |
| SCE_PTHREAD_GETNAME | 0x1E8C3B07C39EB7A9 | Not exercised |
| PTHREAD_GETNAME_NP | 0xF47CDF85DB444A2A | Not exercised |
| SCE_PTHREAD_SET_NAME | 0x5DE4EAC3ED19975D | Not exercised |
| PTHREAD_RENAME_NP | 0xF6FC8FE99EDBAB37 | Not exercised |
| PTHREAD_SET_NAME_NP | 0xA31329F2E3EA6BE5 | Not exercised |
