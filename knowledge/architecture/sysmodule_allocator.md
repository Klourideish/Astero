# M46 sysmodule and allocator continuation

Evidence reviewed: PS5Rust catalogue 00_START_HERE/README.md;
03_LOADER_LINKER/INDEX.md, provider_authority.md, module_queries_unload.md,
source/ps5-libs__src__kernel__module_loader/INDEX.md; 06_SYSMODULES/INDEX.md;
05_KERNEL_HLE/source/ps5-libs__src__kernel__system/INDEX.md, 002.md, 006.md;
05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__exports/INDEX.md, 001.md.
Current prototype system.rs SystemModuleProviderState and seven sysmodule adapters;
libc/exports.rs malloc_stats_fast and malloc_usable_size; libc/nids.rs constants;
lib.rs sole declared AppContent provider; docs/memory/NIDS.md abi.malloc_stats_fast.
Firmware catalogue libSceLibcInternal.sprx index/search did not establish stats layout.
Decrypted catalogue 00_START_HERE/README.md and catalogue_map.md routed artifact evidence.

Prototype declares availability separately from request state, rejects unsupported IDs,
and gates unload on native/TLS users in its artifact loader. No first-loaded provider
fallback or fabricated module is migrated. HLE providers/modules owns bounded declarations,
exact provider identity, transactional dependency refcounts, leases and gated publication.
Libs/sysmodule owns seven exact guest adapters. Artifact/Partial declarations refuse load;
no fabricated mapping, guessed path or init call. Runtime currently has no verified provider
declaration for observed ID0x10b. Returning unsupported must not be described as loaded.
The existing native artifact loader remains the future authoritative route, not a parallel parser.

Stats investigation: prototype malloc_stats_fast only returns zero. Real M45 named_title_elf
caller at source VA0xfedc6c passes rbp-0x90, stores header0x10028 at0xfedc8a,
zeros remaining36 bytes, calls at0xfedcb9, tests EAX then reads offsets16 and32.
Its 40-byte version-one output uses a checked explicit ABI encoding. Fields16/current arena
and32/current in-use are inferred from caller subtraction and allocator context. Peak lanes8/24
follow a limited managed-size model; all semantic mapping remains experimental, not firmware proof.
The actual guest heap snapshot supplies values; no host allocator statistics or constant fake use.
Unknown versions refuse. Full output is preflighted, reserved4 bytes zero; usable-size already exists.

Comparative searches found no additional firm PS5 ABI evidence. OpenOrbis Sysmodule.h and
_types/sysmodule.h corroborate public signatures and AppContent ID0xb4, not PS5 ID0x10b:
https://github.com/OpenOrbis/OpenOrbis-PS4-Toolchain/blob/master/include/orbis/Sysmodule.h
https://github.com/OpenOrbis/OpenOrbis-PS4-Toolchain/blob/master/include/orbis/_types/sysmodule.h
shadPS4 libc_internal.h/cpp inspection yielded no stats implementation. Search results for
Orbis user_malloc showed managed-size names only; no implementation copied or layout proof claimed.

## Ownership and supported boundary

No crate/dependency/unsafe additions. HLE owns provider-request lifetime in its existing
providers home because it operates on the existing exact ProviderKey/Registration contracts;
libs owns the guest syscall adapters; core owns the runtime instance. This is not a loader
or a new kernel execution path. The heap mechanism remains memory-owned, accessed under
its existing process heap mutex via a borrowed HLE snapshot interface. ABI owns byte encoding.

128 declarations per runtime,16 dependencies per declaration,512 callable keys per declaration,
128-byte names, checked u32 references/leases and bounded last-request counters. Dependency
closure is validated before any refcount changes; missing dependencies/cycles roll back without
publication. Repeated successful loads increment references, adapting artifact-loader refcount
behavior rather than the prototype HLE set-only behavior. Loaded dependents or execution leases
refuse unload; shutdown refuses live leases. Gated handlers obtain an RAII lease and return
Unsupported before load/after unload. Provider gates retain owners until safe release.

Declaration, loaded request and handler authority are separate. Only explicitly available HLE
callables can be declared as available; missing exact keys refuse. Publish checks declaration,
key and HLE kind. Artifact/Partial declarations cannot manufacture a host handler or loaded
state. Native artifacts, data exports, TLS callbacks and init/fini are not covered by this gate;
they require existing loader/native lifetime authority and are explicitly unsupported here.
No artifact was selected, searched for, loaded or initialized in M46. This is an intentional
limited foundation, not complete dynamic module loading. Runtime has no verified declaration
for0x10b, nor an AppContent implementation with which to declare prototype ID0xb4.

Seven reviewed wrappers are load/unload/query, their internal variants and internal load-with-arg.
Only the prototype's bounded u16 IDs and empty argument tuple are accepted. WithArg output is
preflighted and a failed publication rolls back its load. Unsupported IDs return0x80020002;
invalid shapes return0x80020003; busy unload returns0x80020010. These are limited inherited
policies, not all firmware error precedence. Internal high-bit IDs remain unsupported.
No lookup-by-name, native handle query, arbitrary PRX loading or init/fini success is fabricated.

## Additional identity checks

Read firmware catalogue `15_FIRMWARE_KNOWLEDGE/modules/libSceSysmodule.sprx/001.md`
and `07_NIDS/by_sysmodule/libSceSysmodule.sprx/001.md`, then its exact current record
`docs/firmware/403/generated/veneer_records.jsonl` for LoadModuleInternal. The recorded
source binary path was stale; an Astero ps5-identity acquisition attempt failed NotFound,
so no fresh firmware parse is claimed. Encoded identity is39iV5E1HoCk#C#A.
Comparative [shadPS4 sysmodule registration](https://github.com/shadps4-emu/shadPS4/blob/main/src/core/libraries/sysmodule/sysmodule.cpp)
corroborates the shared libSceSysmodule namespace for public/internal identities. Its
implementation and extra stub exports were not copied. Searches of libc_internal.h/cpp
and missing memory_management.cpp yielded no stats layout proof. OpenOrbis supplied
signature metadata only. Failed source lookups are not positive evidence.

## Allocator behavior and limitations

The versioned record encodes header0x10028,4 reserved zero bytes, then four u64 byte
counts at8/16/24/32: peak arena/current arena/peak use/current use. Current fixed owned
arena makes the first pair equal (4MiB), not a fabricated host capacity. Used/peak values
come from the same real GuestHeap snapshot as the dashboard. The owner lock makes each
snapshot internally coherent; it is released before the checked guest write. Existing
malloc/free/realloc/aligned allocation/usable-size remain the shared allocation family;
no second heap or counters. Other malloc_stats variants lack credible exact PS5 identity/
ABI evidence in the reviewed source and are not guessed into registration.

The complete40-byte output is checked before mutation. Unknown version returns nonzero22,
invalid memory is structured AccessFailure, failed copy is not success. No rollback promise
is made for a host copy failure after mutation. Only header, size, consumed lanes and successful
continuation are runtime-corroborated. Peak-lane names/ordering remain experimental. No independent
post-write byte dump was captured from the real run; encoding tests establish bytes synthetically.

## Two-workload execution results

Exact local commands: `& ./target/m46-primary-1.ps1`, `& ./target/m46-second-1.ps1`.
Both use existing first-entry CLI,250ms wall,15000ms containment,65536 HLE-call limit,
no-dashboard, report-json and log-file. Full reusable command is in USAGE. PC sampling
was disabled. Real guest execution is trusted experimental input, not a security sandbox.

Primary primary_real_elf: one LoadModule(0x10b), return0x80020002 (unsupported).
**Zero modules loaded, zero providers added, zero artifacts mapped by sysmodule service.**
Guest handled that failure and continued; this is progress beyond dispatch, not module coverage.
It subsequently allocated/mapped6MiB through M45 and stopped at **sceKernelMunmap**,
NID0x71091EF54B8140E9, libkernel/libkernel, ordinal582, arguments address0x400000000,
size0x600000. Source identity corroborated in prototype kernel/system.rs; no wrapper migrated.
Overall212.122ms, supervised native209.773ms,11006HLE calls,59 unique providers,
9threads including main. Boundary RIP0x7ff7ce2782e6 (RuntimeBridge), RSP0x2008007d8,
guest continuation0x1003c8156, native thread31284. Heap peak1055728, live513216,
89live allocations/90peak. Eight workers and their waits were cleaned up.

Second named_title_elf: one malloc_stats_fast call at output0x200800ed0 returned0;
guest consumed the record and continued to **fopen**, NID0xC5E60EE2EEEEC89D,
libc/libc, ordinal1147. Source identity corroborated in prototype kernel/file.rs.
Overall27.005ms, native26.384ms,1563HLE calls,34 unique providers, main only.
Boundary RIP0x7ff7ce2782e6 (RuntimeBridge), RSP0x200800b58,
guest continuation0x100be0fc5, native thread17324. Heap live/peak137616bytes,
2live/peak allocations. Real heap arena4MiB; record contents follow that snapshot,
not separately captured output evidence. Nine semaphores and1MiB direct memory remained
at stop; no semaphore wait/signal occurred. No sysmodule requests on this workload.

Both final stops are unresolved functions in different approved-scope-external families;
no AV, guarded object or supervisor intervention. Boundary RIPs above are **host landings**,
not executing guest PCs. No return-address approximation is described as actual sampling.
Teardown: thread joined,8/0workers joined,FS restored,GS preserved,zero reservations,
release errors,waiters and sleep tickets; clean child containment. Source SHA256 unchanged:
- primary A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A
- second 3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327

## Observability, validation and handoff

Additive schema1 modules object reports requests/failed loads/unloads,last ID,loaded IDs,
HLE/artifact counts and contributed provider references. Older JSON defaults the object to zero.
Module health becomes Partial for a failed load, not Active merely because an error returned.
Dashboard adds one concise row. Normal heap counters remain authoritative for both consumers.

23 new tests:12 module owner/dependency/lease/publication/concurrency;10 libs stats/error/
output/adapter tests;1 JSON compatibility. Artifact test proves refusal, **not artifact load**.
Existing allocator/supervisor/worker/resource regressions are included in the full suite.
Validation commands/counts are retained in validation.md. No real reruns chase unrelated services.

Eight registrations (seven sysmodule,one stats), two old observations promoted; two new
unresolved observations. NID total375:371 registered/4 observation-only. ABI total7 includes
one explicitly experimental version-one managed-size representation, not a full allocator ABI.
Only load-error dispatch and stats-success dispatch were observed; other wrappers remain
synthetically tested. No runtime-confirmed successful module load is claimed.

Promoted: checked stats call progresses and zero-return stub is unnecessary; module failure is
handled by primary. Refined: declared-provider requests need truthful missing coverage; no
0x10b success shortcut. Open:0x10b identity/provider coverage, native artifact init/unload route,
internal-ID variants, peak stats field semantics. Historical Windows487 remains unresolved/
unreproduced. M47 recommendation from these runs: filesystem startup for fopen, with the
already-known kernel munmap wrapper boundary assessed alongside it. Neither is implemented.


## Exact registration list

| Export | NID | Library/module | Status |
| --- | --- | --- | --- |
| sceSysmoduleLoadModule | `0x83C70CDFD11467AA` | libSceSysmodule/libSceSysmodule | Registered; unsupported-result dispatch observed |
| sceSysmoduleUnloadModule | `0x791D9B6450005344` | libSceSysmodule/libSceSysmodule | Registered; not runtime-confirmed |
| sceSysmoduleIsLoaded | `0x7CC3F934750E68C9` | libSceSysmodule/libSceSysmodule | Registered; not runtime-confirmed |
| sceSysmoduleLoadModuleInternal | `0xDFD895E44D47A029` | libSceSysmodule/libSceSysmodule | Registered; not runtime-confirmed |
| sceSysmoduleLoadModuleInternalWithArg | `0x847AC6A06A0D7FEB` | libSceSysmodule/libSceSysmodule | Registered; not runtime-confirmed |
| sceSysmoduleUnloadModuleInternal | `0xBD7661AED2719067` | libSceSysmodule/libSceSysmodule | Registered; not runtime-confirmed |
| sceSysmoduleIsLoadedInternal | `0xCA714A4396DF1A4B` | libSceSysmodule/libSceSysmodule | Registered; not runtime-confirmed |
| malloc_stats_fast | `0x2AE3AE0F9F21AA7E` | libc/libc | Registered; real return0; experimental layout semantics |
