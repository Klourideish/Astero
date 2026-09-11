# M33 pthread synchronization foundation

PS5Rust is the migration baseline. Kernel owns synchronization state; libs owns guest exports and
error/attribute conversion; core owns the opted-in native runtime's timing engine and service.
No runtime dependency is added. Core tests add an allowed ABI dependency for native call frames.

## Focused evidence

Catalogue-relative routes: `00_START_HERE/README.md`, `catalogue_map.md`, `05_KERNEL_HLE/INDEX.md`,
`05_KERNEL_HLE/pthread_attributes.md`; source indexes for
`ps5-libs__src__kernel__threading__rwlock`, `__mutex`, `__condvar`, then rwlock `001.md`, mutex
`002.md` and condvar `002.md`. Read original source narrowly: `kernel/threading/rwlock.rs`,
`mutex.rs`, `condvar.rs`, `state.rs`, `exports.rs`, `nids.rs`, and
`kernel/threading.rs::reset_process_state`.
Firmware documentary corroboration only: `15_FIRMWARE_KNOWLEDGE/INDEX.md`,
`document_guides/403_SYNC.md.md`, `contracts/umtx_op.md`. These do not prove private pthread layout;
no raw firmware, external emulator or decrypted corpus re-analysis was needed for these contracts.
Current Astero M25/M29-M32 architecture and source remain ownership authority.

Preserved: opaque u64 handles, owner-aware recursive/errorcheck mutexes, reader/writer exclusion,
atomic cond registration/release, reacquisition on ordinary timeout, explicit attribute identity,
SCE versus POSIX errors and corrected destroy NIDs. Rejected: global registries/reset handle reuse,
forgotten host guards/unsafe force-unlock, saturating recursive depth, 1 ms timeout polling,
clearing mutex registries while waiters can still exist, leaving stale destroyed guest slots,
permissive normalization of invalid attributes and conflating encoded/raw errno values.

## Objects, identity and lifetime

Guest slots contain one little-endian u64 token, not a host pointer or primitive. Identity checks
require token + original guest slot + kind; copies/stale tokens cannot alias a live object.
Tokens increase without reuse within a runtime. Explicit init requires a readable/writable slot;
failed publication rolls back the object. Double init refuses. Destroy rejects holders and waiters,
including condition waiters reacquiring their mutex, then clears the slot. Runtime teardown differs
from successful guest destroy. No static zero/lazy object behavior was found in reviewed prototype
lookups; zero handles refuse rather than creating guessed objects. Zero-filled slots before explicit
init are supported. Arbitrary inline libc static representations remain unsupported.
Guest memory uses scoped checked copies; M33 does not establish concurrent native guest memory access.
Initial caller is explicit Thread(1), not host ThreadId. Kernel tests use multiple logical callers;
real pthread creation remains absent.

## Semantics

Mutex type 0 normalizes to prototype default errorcheck=1; recursive=2, normal=3, adaptive=4.
Adaptive has normal blocking semantics, no spin optimization. Recursive depths are checked.
Trylock of a held nonrecursive mutex returns Busy; errorcheck self-lock returns Deadlock.
Wrong-owner unlock returns Permission. Normal self-lock can wait until execution cancellation.
RW locks track per-caller read depths and an exclusive writer. Upgrade/self-write misuse returns
Deadlock. Unlock and destroy preserve exclusion and owner checks. Fairness is not claimed;
rwattr type 1 supported, type 2 refuses instead of falsely promising writer preference.
Cond signal marks one queued waiter; broadcast marks all existing queued waiters. No future credit.
Registration and associated mutex release share the state lock, preventing lost wakes.
Wait reacquires the mutex before normal success/timeout. Recursive depth >1 refuses cond wait.
At runtime cancellation reacquisition may be impossible: Interrupted stops native HLE execution,
never returns guest success with an unowned mutex. Guest destroy cannot invalidate active wait state.

## Timing and bounds

All waits use astero-timing tickets. No host sleeps, polling timers, callbacks or new scheduler.
Unlock/signal cancels tickets as control wakes; service predicates remain authoritative.
Runtime stop cancels tickets. Each real run arms a service deadline using its execution wall limit.
Expiry at this limit returns Interrupted to the bridge, avoiding indefinite Rust HLE blocking.
Native supervision and outer process containment remain intact; timer wake lateness is not a precise
CPU stop guarantee. Rust host frames are not asynchronously redirected.

Absolute timespec is two signed 64-bit seconds/nanoseconds, with checked range/arithmetic.
Explicit compatibility conversion: clock 0 uses host Unix realtime sampled at call to compute
remaining time, then schedules on M25 monotonic time; clock 4 uses this engine's origin.
This is not a complete guest clock/pause API. No mid-wait calendar adjustment. Invalid/overflowing
values refuse. SCE timed calls use the same absolute-timespec baseline as PS5Rust.

Limits: 1024 objects including attrs, 128 waiters/tickets and distinct readers per lock;
existing 4096 HLE call and memory-copy limits retained. Metadata vectors are bounded/fallibly reserved.
Snapshots obey these capacities. Counts/generations expose create/destroy/wait/wake/timeout.
Runtime owns service/engine; timing shutdown joins. Poisoning does not become guest errno.
SCE failures encode 0x80020000 | FreeBSD errno; POSIX returns errno directly, no TLS errno writes.
Permission=1, Deadlock=11, Busy=16, Invalid=22, capacity=35, timeout=60. Interrupted stops execution.
Attrs support init/destroy/get/set type or clock and process-private/no-priority-protocol queries.
Unsupported pshared/protocol/priority behavior refuses; no scheduler enforcement is fabricated.

## Validation and runtime experiment

One controlled real run, primary_real_elf, unchanged USAGE command with wall-ms 250 and
containment-ms 15000. CLI exit 0 / Clean. Total 74 completed provider calls. Three pthread calls:
scePthreadRwlockInit at 0x210041678 (id 1), scePthreadCondInit at 0x2100417a8 (id 2),
scePthreadMutexInit at 0x2100417b0 (id 3), each returned 0 once. All unowned at stop, no guest destroy.
No actual lock wait/wake/timeout occurred; M25 wait participation is synthetically proven, not inferred
from this run. Objects are dropped with runtime, separate from guest-visible destroy.

New stop: UnresolvedFunction scePthreadAttrInit, NID 0x9ec628351cb0c0d8, libkernel/libkernel,
ordinal 119. Initial RIP 0x100000070, RSP 0x200800fb8, FS 0x210000000. At stop RSP 0x200800e68;
boundary RIP 0x7ff72df56d06 is host import landing, not actual guest PC. Guest continuation address
0x1001c2814; args [0x210041818,0x1005fdb1e,0,0xa5bf9f54b0,0x78,0x80]. No exception/object fault.
Elapsed 1406 us, supervisor interval 1251 us, thread 8720, no redirection, zero suspends/resumes.
FS restored, GS preserved, joined thread, native/runtime reservations zero, no release errors.
Heap at stop: 23 live /2128 bytes, 25 allocations and 2 frees. 16 callbacks retained, none executed.
Source SHA256 before/after A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Raw local evidence: target/m33-real-first.log and target/m33-real-first-hash.log (ignored).

Other provider calls: _init_env 1, atexit 2, __cxa_atexit 14, operator new 25, memcpy 22, memcmp 3,
strlen 2, operator delete 2. No further real execution in M33. scePthreadAttrInit is general thread
lifecycle, not mutex/rwlock/cond attributes. Recommended M34 is the thread-lifecycle migration wave
starting with this observed demand; no M34 implementation has begun.

Supported: explicit opaque object creation and successful startup advancement. Refined: runtime
cancellation returns a bridge stop instead of resuming a cond waiter without its mutex. Unproven:
full guest realtime epoch/pause behavior, arbitrary inline static layouts and waiter fairness.
These are deliberate adaptation boundaries, not rediscovery of prototype fundamentals.
ABI index remains at 2: no full firmware pthread layout claim. 82 synchronization NIDs implemented
and registered, exactly three runtime-exercised here. Other migrated NIDs are test/source-backed,
not claimed runtime-confirmed. New scePthreadAttrInit record is observed/unregistered.

Preflight passed complete check/build/Clippy/workspace Rust tests and synthetic native supervision.
50 ms synthetic loop stopped after 55,525 us, balanced one suspend/resume, FS/GS restored, joined.
Focused manual-clock tests cover signal/reacquire, timeout, shutdown, ownership and limits.
A post-run synthetic native HLE test also blocks on a held mutex, expires through M25, returns to
bridge and joins with no tickets; it executes only fixed Astero assembly, no second real run.


## Migrated identities

All below register exactly libkernel/libkernel. SCE and POSIX identities remain distinct.

| Source identity | Numeric NID | Real run |
|---|---|---|
| SCE_PTHREAD_MUTEX_INIT | `0x726a3544862f6bda` | once, success |
| SCE_PTHREAD_MUTEX_DESTROY | `0xd8e7f47fede68611` | not exercised |
| SCE_PTHREAD_MUTEX_LOCK | `0xf542b5bcb6507ede` | not exercised |
| SCE_PTHREAD_MUTEX_TRYLOCK | `0xba9a15af330715e1` | not exercised |
| SCE_PTHREAD_MUTEX_UNLOCK | `0xb67dd5943d211bad` | not exercised |
| SCE_PTHREAD_MUTEX_TIMEDLOCK | `0x21a7c8d8fc5c3e74` | not exercised |
| SCE_PTHREAD_RWLOCK_INIT | `0xe942c06b47eae230` | once, success |
| SCE_PTHREAD_RWLOCK_DESTROY | `0x041fa46f4f1397d0` | not exercised |
| SCE_PTHREAD_RWLOCK_RDLOCK | `0x3b1f62d1cecbe70d` | not exercised |
| SCE_PTHREAD_RWLOCK_TRYRDLOCK | `0x5c3de60dec9b0a79` | not exercised |
| SCE_PTHREAD_RWLOCK_WRLOCK | `0x9aa74da2bac1fa02` | not exercised |
| SCE_PTHREAD_RWLOCK_TRYWRLOCK | `0x6c81e86424e89ac2` | not exercised |
| SCE_PTHREAD_RWLOCK_UNLOCK | `0xf8bf7c3c86c6b6d9` | not exercised |
| SCE_PTHREAD_RWLOCK_TIMEDRDLOCK | `0x88fb594562028eb3` | not exercised |
| SCE_PTHREAD_RWLOCK_TIMEDWRLOCK | `0x69d87fffa9c8a939` | not exercised |
| SCE_PTHREAD_COND_INIT | `0xd936fddaaba9ae5d` | once, success |
| SCE_PTHREAD_COND_DESTROY | `0x83e3d977686269c8` | not exercised |
| SCE_PTHREAD_COND_WAIT | `0x58a0172785c13d0e` | not exercised |
| SCE_PTHREAD_COND_TIMEDWAIT | `0x06632363199ec35c` | not exercised |
| SCE_PTHREAD_COND_SIGNAL | `0x90387f35fc6032d1` | not exercised |
| SCE_PTHREAD_COND_BROADCAST | `0x246823ed4beb97e0` | not exercised |
| SCE_PTHREAD_MUTEXATTR_INIT | `0x17c6d41f0006dbce` | not exercised |
| SCE_PTHREAD_MUTEXATTR_DESTROY | `0xb2658492d8b2c86d` | not exercised |
| SCE_PTHREAD_MUTEXATTR_SETTYPE | `0x88ca7c42913e5cee` | not exercised |
| SCE_PTHREAD_MUTEXATTR_GETTYPE | `0x82ab84841ad2da2c` | not exercised |
| SCE_PTHREAD_MUTEXATTR_SETPROTOCOL | `0xd451af5348bdb1a4` | not exercised |
| SCE_PTHREAD_MUTEXATTR_GETPROTOCOL | `0x1a84e615eba2fa14` | not exercised |
| SCE_PTHREAD_MUTEXATTR_SETPSHARED | `0x9b12b1f5bc571762` | not exercised |
| SCE_PTHREAD_MUTEXATTR_GETPSHARED | `0x968b04b9b1dceb87` | not exercised |
| SCE_PTHREAD_RWLOCKATTR_INIT | `0xc8e7c683f2356482` | not exercised |
| SCE_PTHREAD_RWLOCKATTR_DESTROY | `0x8b689f6777d2d9fa` | not exercised |
| SCE_PTHREAD_RWLOCKATTR_SETTYPE | `0x87f3a27e2a2e05df` | not exercised |
| SCE_PTHREAD_RWLOCKATTR_GETTYPE | `0x2b296cd42845cab7` | not exercised |
| SCE_PTHREAD_RWLOCKATTR_SETPSHARED | `0xfd9bd01f5f23d747` | not exercised |
| SCE_PTHREAD_RWLOCKATTR_GETPSHARED | `0x2dc3990471aa6c59` | not exercised |
| SCE_PTHREAD_CONDATTR_INIT | `0x9b9ff66ec35fbfbb` | not exercised |
| SCE_PTHREAD_CONDATTR_DESTROY | `0xc1a3dcc58891dd60` | not exercised |
| SCE_PTHREAD_CONDATTR_SETCLOCK | `0x73f6f18f4dbb733b` | not exercised |
| SCE_PTHREAD_CONDATTR_GETCLOCK | `0xeaa33790ee52dcea` | not exercised |
| SCE_PTHREAD_CONDATTR_SETPSHARED | `0xeb131ec3dfab6702` | not exercised |
| SCE_PTHREAD_CONDATTR_GETPSHARED | `0x0e7fc34568bdb79e` | not exercised |
| PTHREAD_MUTEX_INIT | `0xb6d1cd7d4faa0c15` | not exercised |
| PTHREAD_MUTEX_DESTROY | `0x96d09f686af62461` | not exercised |
| PTHREAD_MUTEX_LOCK | `0xec7d224ce7224cba` | not exercised |
| PTHREAD_MUTEX_TRYLOCK | `0x2bf8d785bb76827e` | not exercised |
| PTHREAD_MUTEX_UNLOCK | `0xd99f8fa58e826898` | not exercised |
| PTHREAD_MUTEX_TIMEDLOCK | `0x228f7e9d329766d0` | not exercised |
| PTHREAD_RWLOCK_INIT | `0xcad4142cdfe784be` | not exercised |
| PTHREAD_RWLOCK_DESTROY | `0xd78ef56a33f3c61d` | not exercised |
| PTHREAD_RWLOCK_RDLOCK | `0x8868ecaf5580b48d` | not exercised |
| PTHREAD_RWLOCK_TRYRDLOCK | `0x485c5330e7ee0a41` | not exercised |
| PTHREAD_RWLOCK_WRLOCK | `0xb08951bd0aac3766` | not exercised |
| PTHREAD_RWLOCK_TRYWRLOCK | `0x5e15879fa3f947b5` | not exercised |
| PTHREAD_RWLOCK_UNLOCK | `0x12098ba3a11682ca` | not exercised |
| PTHREAD_RWLOCK_TIMEDRDLOCK | `0x95bf259d8a3fa3b9` | not exercised |
| PTHREAD_RWLOCK_TIMEDWRLOCK | `0xf73925cc097d0863` | not exercised |
| PTHREAD_COND_INIT | `0xd13c959383122edd` | not exercised |
| PTHREAD_COND_DESTROY | `0x4575ea8b80ad17cc` | not exercised |
| PTHREAD_COND_WAIT | `0x3a9f130466392878` | not exercised |
| PTHREAD_COND_TIMEDWAIT | `0xdbb6c08222663a1d` | not exercised |
| PTHREAD_COND_SIGNAL | `0xd8c3b2fab51fba14` | not exercised |
| PTHREAD_COND_BROADCAST | `0x9a4c767d584d32c8` | not exercised |
| PTHREAD_MUTEXATTR_INIT | `0x7501d612c26da04e` | not exercised |
| PTHREAD_MUTEXATTR_DESTROY | `0x1c5ee52b8eb1ce36` | not exercised |
| PTHREAD_MUTEXATTR_SETTYPE | `0x9839a030e19552a8` | not exercised |
| PTHREAD_MUTEXATTR_GETTYPE | `0x19916523b461b90a` | not exercised |
| PTHREAD_MUTEXATTR_SETPROTOCOL | `0xe6dc4a7dc3140289` | not exercised |
| PTHREAD_MUTEXATTR_GETPROTOCOL | `0xc83696c54139d2cd` | not exercised |
| PTHREAD_MUTEXATTR_SETPSHARED | `0x117bf7ced1aab433` | not exercised |
| PTHREAD_MUTEXATTR_GETPSHARED | `0x3e62ff4f0294cd72` | not exercised |
| PTHREAD_RWLOCKATTR_INIT | `0xc4579bb00e18b052` | not exercised |
| PTHREAD_RWLOCKATTR_DESTROY | `0xaac7668178ea4a09` | not exercised |
| PTHREAD_RWLOCKATTR_SETTYPE_NP | `0xf0db8e1e24ebd55c` | not exercised |
| PTHREAD_RWLOCKATTR_GETTYPE_NP | `0x97e6c6e5fb189218` | not exercised |
| PTHREAD_RWLOCKATTR_SETPSHARED | `0x3ae2a0fa44430fb5` | not exercised |
| PTHREAD_RWLOCKATTR_GETPSHARED | `0x56a10cb82bffa876` | not exercised |
| PTHREAD_CONDATTR_INIT | `0x98aa13c74dc74560` | not exercised |
| PTHREAD_CONDATTR_DESTROY | `0x74972e4159fafc8c` | not exercised |
| PTHREAD_CONDATTR_SETCLOCK | `0x123965680a803d9a` | not exercised |
| PTHREAD_CONDATTR_GETCLOCK | `0x7130d8c5350d3e13` | not exercised |
| PTHREAD_CONDATTR_SETPSHARED | `0xdc1a4ff39d21053e` | not exercised |
| PTHREAD_CONDATTR_GETPSHARED | `0x874a94a92b8e982f` | not exercised |


## Final validation

426 executable Rust tests and 23 doctests pass; 24 tests added (19 kernel synchronization,
4 core guest-provider tests, 1 native bridge blocked-HLE test). All workspace targets pass
formatting, check, build and warnings-denied Clippy. Existing CLI integration and supervision tests
remain included. All 53 Python tests pass: policy/state/structure/native/index suites.
Policy reports 16 crates, 22 internal edges, 250 declared homes. No runtime dependency change;
core's ABI edge is test-only. Supplemental whitespace scans 634 files; Git diff check passes.
Indexes: implementation 1193, subsystems 137, modules 543, sources 529, tests 504, diagnostics 37,
NIDs 116 (113 registered, 3 unregistered), ABI 2; total 3061. Index freshness passes.

The shared `pthread/exports.rs` is deliberately the single typed guest-slot/error/NID adapter:
family tables share identical admission and publication rules, and contain no host synchronization.
The narrower kernel mutex/rwlock/condvar operation homes own their mechanisms. The synchronization
root owns the shared object/wait registry because cond release and mutex handoff must linearize
under the same lock. This avoids parallel ownership registries or duplicate slot decoders.

A post-run regression reproduced timing-engine shutdown falsely returning cond success. The shared
wait paths now treat Stopped tickets as runtime interruption; the new regression passes. No real
workload replay was needed for this shutdown-only correction.

Final M33 index regeneration was byte-identical across all 17 index-directory files; freshness,
policy/state/structure, all 53 Python tests and whitespace checks passed after the shutdown fix.

M38 supersedes only the mutex publication-address identity restriction: valid copied opaque mutex
tokens remain usable and helper output slots may be reused. Kind/stale/ownership checks remain.
See [correction and real evidence](audio_startup.md). Other object kinds retain their M33 contract.
