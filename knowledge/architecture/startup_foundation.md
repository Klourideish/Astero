# M32 libkernel/libc startup foundation migration

PS5Rust is the migration baseline after M31. Established behavior is adapted into Astero,
then validated against the workload. Fresh proof is for changed architectural boundaries,
contradictory evidence and known shortcuts, not a mandatory new observation phase per NID.

## Evidence and adaptation

Catalogue-relative routes: 00_START_HERE/README.md and catalogue_map.md;
03_LOADER_LINKER/provider_authority.md; 04_MEMORY/guest_heap.md and checked_hle_access.md;
05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__nids/017.md;
17_TESTS/by_source/ps5-libs__src__kernel__libc/002.md;
02_CORE_EXECUTION/native_boundary.md; 04_MEMORY/tls_dynamic_blocks.md;
15_FIRMWARE_KNOWLEDGE/contracts/sceKernelGetProcParam.md.
Decrypted catalogue routes: 00_START_HERE/key_findings.md and 03_BINDINGS/INDEX.md.
These were read in the approved migration-gap analysis, followed by focused current prototype
source: kernel/libc/exports.rs, libc/nids.rs, kernel/system.rs and cpu/native_exec.rs.

The prototype explicitly registers __stack_chk_guard as a process-owned read-only data object,
NID 0x7fbb8ec58f663355. Its deterministic 0x9e3779b97f4a7c15 value is emulator policy,
not firmware security truth. Its tests check identity, initialization and write refusal.
Prototype malloc/free NIDs are the corrected real bridge identities. strcmp takes two
arguments; the old strncmp substitution caused incorrect equality in IL2CPP lookup.
__error returns the initial thread's stable guest int slot at TCB+8, not the errno value.
sceKernelGetProcParam returns the relocated main process-parameter address.

Rejected shortcuts: host pointers for getenv storage, process-global environment state,
scratch objects, success for unknown providers, NID-only matching and unrestricted callback
addresses. No external emulator or raw firmware was newly consulted.

## Ownership and boundaries

Libs owns guest contracts/NIDs and startup object declarations. HLE owns separate DataExport
and callable Registration types plus a borrowed GuestMemory copy interface. Core composes
live owners and supplies scoped access only at synchronous native HLE boundaries. Memory
owns address allocation, native residency, range/permission checks and copy operations.
Kernel retains process environment identities and typed exit callbacks. No new runtime crate,
global registry or scheduler is introduced. Loader/offline operations remain independent.

NativeImage checked writes now take a shared owner borrow: the owner remains !Send/!Sync,
copies expose no Rust guest references, and release/protection still require exclusive access.
Guest instructions are stopped at the HLE landing while the copy runs. This is not permission
for concurrent guest-thread memory access; full pthread migration must revisit that boundary.

close_startup is the migrated preparation capability. Historical close_entry remains available
for its explicitly narrow preparation/tests. first-entry composes close_startup and still
requires EntryReadyGuest, the M31 supervisor and outer child containment.

The guard is initialized in temporary host bytes, natively mapped read-only and retained by
the closure owner. Exact libkernel/libkernel object relocations receive its guest address;
unknown objects retain NOACCESS traps. No function stub is used as object storage. Addends
must remain inside the declared object. Relocation writes precede RELRO finalization.

## Resource policy and behavior

Named initial policies: 4 MiB reusable RW/NX guest heap, 4096 live allocations, 16-byte alignment,
4096 retained provider calls, 1 MiB per primitive copy/scan, 128 environment entries with
8192-byte name/value bounds, and 256 callbacks. Native startup storage counts against the
caller's existing max-runtime-bytes. Insufficient capacity refuses explicitly; no successful
prefix is presented as complete. Zero-size malloc owns a minimal aligned slot. free(NULL)
succeeds; invalid/interior/double free refuses. Realloc failure retains the old allocation;
realloc(p,0) frees and returns NULL. calloc checks multiplication and clears reused bytes.

Environment starts empty, matching the prepared empty env vector. Values are guest heap
allocations; replacement invalidates the old returned value. No host environment is imported.
The initial errno slot starts zero in owned TCB storage. This adopts prototype initial-thread
policy, not a complete multi-thread TLS ABI. _init_env remains an instrumented no-op.

The controlled bridge return address is stored as RuntimeReturn, distinct from Guest callbacks.
Only the exact live enclosing bridge landing is admitted; adjacent/arbitrary host addresses
are rejected. Callbacks are retained in reverse registration order and not invoked here.
Supported exit/abort/stack-check termination stops through the bridge; no full libc shutdown
or C++ unwind semantics are claimed.

The migrated call policy accepts exact registered keys and stops on missing providers,
operation refusal, termination, guest faults or the existing deadline. Scalar arguments,
return values, refusal, heap state and object identity are retained. Data providers are not
callable. Libc/libc and libkernel/libkernel are explicit context registrations; no arbitrary
module fallback is allowed. No full pthread, filesystem, module loading, graphics or audio.

## Validation and workload

Three explicitly bounded primary_real_elf experiments used the USAGE first-entry command
with wall-ms 250 and containment-ms 15000. All exited 0, preserved host FS/GS, joined the
execution thread, released all native/runtime reservations and retained the original SHA256:
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.

First: 306 us, advanced past the guard to __cxa_atexit, numeric 0xb6cbc49a77a7cf8f.
The captured argument lane contained the guard value. Both original atexit calls succeeded.
The catalogue libSceLibcInternal.sprx/068.md and prototype libc/exports.rs 3206-3222
establish argument/DSO retention; migrated within scope, replacing global registry with bounded
process ownership. Null registration retains the prototype no-op policy; no callbacks execute.

Second: 241 us, two cxa registrations then operator new 0x7c99e9b955416ca9.
Prototype exports.rs 1610 onward establishes the shared 16-byte heap allocation family.
Migrated scalar/array new/delete and sized delete. Rejected prototype null-success on throwing
new exhaustion: Astero reports AccessFailure instead of pretending to throw or succeeding.

Third: 947 us controller interval, 793 us supervisor interval; thread 1716, no preemption,
zero suspend/resume operations. UnresolvedFunction at ordinal 467, NID 0xe942c06b47eae230,
libkernel/libkernel. Catalogue libkernel.sprx/062.md and threading/nids.rs identify
scePthreadRwlockInit, unimplemented here. Captured PC 0x7ff7e90e57c6 is the host import landing;
guest continuation 0x1001c25b6 is explicitly a return address, RSP 0x200800e58.
Arguments: [0x210041678, 0, 0x200800e98, 0xa52c5f6180, 0x78, 0x80]. No fault occurred.
Calls: _init_env 1, atexit 2, __cxa_atexit 14, operator new 15, memcpy 17, memcmp 3: 52 total.
Heap before teardown: 15 live allocations, 1664 aligned bytes, no frees. Guard resides at
0x210040000, heap begins 0x210041000. Sixteen callbacks retained, none invoked.
The final boundary establishes the next coherent threading/synchronization migration pressure;
M33 is not implemented. No more M32 execution experiments are needed.

Supported: owned guard replacement, startup callback identity, real heap allocation/copy calls
and deterministic teardown. Refined: add cxa retention and new/delete within the startup wave.
Unproven: full environment/TLS ABI, callback execution and C++ unwinding. Two explicit libc and
libkernel context registrations are adaptation policy, not a claim of arbitrary namespace equivalence.
Later ENOMEM propagation refinements are synthetic-tested, not an additional real run.
M31's comparison point is the guarded symbol-7 read at RIP 0x100332abf. M32 must preserve
source integrity, supervisor/recovery behavior and zero reservations after owner teardown.


## NID inventory and validation

31 distinct registered identities: 30 callable functions plus the owned data guard. The
runtime retains 53 exact callable context registrations; duplicated libc/libkernel keys are
not additional numeric NIDs. Three unregistered observations remain (two M24 identities and
M32's encountered rwlock initialization). ABI inventory remains two; no full C++ or TLS ABI added.

| Name / scope | Numeric NID | M32 status |
|---|---|---|
| _init_env | `0x6f3404c72d7cf592` | Registered; runtime exercised |
| atexit | `0xf06d8b07e037af38` | Registered; runtime exercised |
| __stack_chk_guard (data) | `0x7fbb8ec58f663355` | Registered; runtime exercised |
| operator new | `0x7c99e9b955416ca9` | Registered; runtime exercised |
| operator new[] | `0x85d9b461f31aed34` | Registered; not runtime exercised |
| operator delete | `0xcfe3fec429d62c19` | Registered; not runtime exercised |
| operator delete[] | `0x30b5a5f7448558d1` | Registered; not runtime exercised |
| operator delete sized | `0x9580f3055139999b` | Registered; not runtime exercised |
| malloc | `0x8105fee060d08e93` | Registered; not runtime exercised |
| free | `0xb4886caa3d2ab051` | Registered; not runtime exercised |
| calloc | `0xd97e5a8058cac4c7` | Registered; not runtime exercised |
| realloc | `0x63b689d6ec9d3cca` | Registered; not runtime exercised |
| memcpy | `0x437541c425e1507b` | Registered; runtime exercised |
| memmove | `0xf8fe854461f82df0` | Registered; not runtime exercised |
| memset | `0xf334c5bc120020df` | Registered; not runtime exercised |
| memcmp | `0x0df8af3c0ae1b9c8` | Registered; runtime exercised |
| strlen | `0x8f856258d1c4830c` | Registered; not runtime exercised |
| strnlen | `0xe6336e6f0e2f9400` | Registered; not runtime exercised |
| strcmp | `0x3af6f675224e02e1` | Registered; not runtime exercised |
| strncmp | `0x69eb328eb1d55b2e` | Registered; not runtime exercised |
| strchr | `0xa1be71016e259ffd` | Registered; not runtime exercised |
| strrchr | `0xf720d63311057495` | Registered; not runtime exercised |
| __error | `0xf41703ca43e6a352` | Registered; not runtime exercised |
| sceKernelGetProcParam | `0xf79f6aadaccf22b8` | Registered; not runtime exercised |
| getenv | `0xb266d0ba47f16093` | Registered; not runtime exercised |
| setenv | `0x3386186d215f27c8` | Registered; not runtime exercised |
| exit | `0xb8c7a2d56f6ec8da` | Registered; not runtime exercised |
| _exit | `0xe99f37b18585940f` | Registered; not runtime exercised |
| abort | `0x2f54814e40be0afc` | Registered; not runtime exercised |
| __stack_chk_fail | `0x3aede22f569bbe78` | Registered; not runtime exercised |
| __cxa_atexit | `0xb6cbc49a77a7cf8f` | Registered; runtime exercised |
| scePthreadRwlockInit | `0xe942c06b47eae230` | Runtime observed; unregistered |

Full final validation: 402 executable Rust tests, 23 doctests, full workspace check/build,
fmt check and warnings-denied Clippy passed. 14 tests added: libs 11, memory 2, core 1.
Existing HLE tests adapted to the scoped memory interface. Core closure tests serialize their
process-exclusive bridge owner; the normal workspace test command passes. No test suppression.
53 Python tests passed (21 policy, 9 structure, 2 native policy, 21 index tests). Policy reports
16 crates, 21 internal edges and 250 homes. Only added dependencies are libs test-only ABI/memory;
no runtime or third-party dependencies added. Supplemental whitespace: 626 files; Git diff check passed.

The generic scalar/memory routines have direct tests; complete malloc/libc conformance is not
claimed. Copy/scan limits may stop valid but larger workloads explicitly. Environment and errno
are initial-thread contracts. Host-side HLE stalls remain subject to outer process containment,
not arbitrary Rust-frame preemption. The existing no-security-sandbox limitation remains.

Registered provider refusal/registry failure stays distinct from an unresolved provider; post-run diagnostic refinement, no additional real execution.
