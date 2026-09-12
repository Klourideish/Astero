# M45 kernel resource foundation

Astero now provides a runtime-owned direct-memory/VM family and counting semaphores.
The primary workload advanced from GetDirectMemorySize to sceSysmoduleLoadModule;
the second advanced from CreateSema through reservation/allocation/mapping to
malloc_stats_fast. These are controlled native observations, not successful game boots.
No M46 implementation, commit or push is part of this milestone.

## Evidence and migration baseline

Exact paths relative to the approved PS5Rust Knowledge Catalogue:

- `00_START_HERE/README.md` (routing).
- `04_MEMORY/INDEX.md`, `04_MEMORY/address_space_contract.md`.
- `04_MEMORY/source/ps5-libs__src__kernel__memory/INDEX.md` and
  `04_MEMORY/source/ps5-libs__src__kernel__memory/004.md`.
- `05_KERNEL_HLE/INDEX.md`, `05_KERNEL_HLE/completion_lifecycle.md`.
- `05_KERNEL_HLE/source/ps5-libs__src__kernel__sync/INDEX.md` and
  `05_KERNEL_HLE/source/ps5-libs__src__kernel__sync/007.md`.
- `05_KERNEL_HLE/source/ps5-libs__src__kernel__threading__semaphore/INDEX.md`
  (routing, not a separate implementation proof).
- `15_FIRMWARE_KNOWLEDGE/INDEX.md`,
  `15_FIRMWARE_KNOWLEDGE/contracts/munmap.md`,
  `15_FIRMWARE_KNOWLEDGE/document_guides/403_MEMORY.md.md`.
  The last is a MapDirectMemory2 signature lead, not full firmware proof.

Focused current PS5Rust source, relative to its read-only repository:
`crates/ps5-libs/src/kernel/memory.rs`: constants/size policy (158-198), CPU
protection conversion (430-441), size/allocation/query registrations (735-1023),
map/map2/named adapters (1050-1200), release/reserve (1278-1434), VirtualQuery
(1434-1515). `crates/ps5-libs/src/kernel/sync.rs`: corrected identities (1-38),
error mapping (~420), creation (609-665), wait/signal/poll/cancel/delete (731-904).
Line numbers describe the inspected source snapshot; symbols are durable routing.
For next-stop identity only: kernel/system.rs sceSysmoduleLoadModule constant
(~315) and kernel/libc/nids.rs MALLOC_STATS_FAST_NID (~347). Neither was migrated.

Astero records reviewed: M25 asynchronous timing, M28 Windows native VM, M29/M30
runtime preparation/closure, M31 first native entry/supervision, M33 owned sync,
M34 pthread lifecycle and M44 runtime observability, plus current owners.
No independent external emulator implementation was consulted. No broad firmware
or decrypted-catalogue ingestion occurred; current source-backed corpus execution
provided the additional artifact evidence.

Windows adaptation references:
[creating file views](https://learn.microsoft.com/en-us/windows/win32/memory/creating-a-file-view),
[MapViewOfFile](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-mapviewoffile),
[CreateFileMapping](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createfilemappinga).
These support section lifetime, address/offset granularity and executable view
mechanisms, not PS5 semantics.

## Ownership and dependency direction

Kernel `objects/memory` owns direct offsets and virtual mapping records. Kernel
`synchronization/semaphore` owns semaphore handles, counts and wait queues. Core's
existing runtime shares both owners among initial/worker providers. Libs owns
exact guest registration keys, argument/error conversion and checked output encoding.
The existing private Windows memory leaf owns sections, views, reservation and
protection APIs; no raw handles or host pointers become guest kernel identities.
No new crate or dependency was introduced. Unsafe additions stay in the previously
authorized memory backend leaf. Offline loading creates neither resource service.

Direct memory remains distinct from image mappings, libc heap, stacks and TLS.
The VM band is an emulator placement policy, not a title address. Native execution
remains direct x86-64; section views can be RX and no fetch/decode layer is added.
The Windows leaf was extended rather than creating another unsafe allocation owner.

## Direct memory and VM contract

- 16 GiB is the inherited **direct-offset namespace policy**, not eager commitment,
  physical hardware truth or an allocation guarantee. Live backing is capped at
  512 MiB per runtime; 256 resources, 256 views and 4096 retained observer records
  bound allocation/mapping history. Exhaustion returns an error.
- Physical/direct offsets and guest VA are separate. Checked deterministic first-fit
  searches the supplied offset interval. Size is nonzero, 16 KiB aligned; alignment
  is a power of two at least 16 KiB (zero selects that default). Nonnegative memory
  types are retained as opaque metadata; cache/GPU behavior is not implemented.
- Guest VA uses the reserved policy band `[0x400000000,0x500000000)`, with checked
  first-fit or exact placement. Windows section view addresses and relative offsets
  require 64 KiB alignment. Unsupported subgranular views refuse explicitly.
- Pagefile sections own committed bytes. Views retain shared backing, so aliases
  observe the same bytes and unmap/remap does not replace data with zeroes. Section
  creation permits executable views but no RWX view is created. Initial mappings
  and protection transitions are checked with VirtualQuery; RX flushes the cache.
- CPU low protection bits drive effective R/RW/RX/noaccess; write implies host read,
  as in the prototype. Higher requested bits are retained in query metadata only.
  CPU write+execute refuses. Requested `0xF2` is effective RW, not GPU coherency proof.
- ReserveVirtualRange is genuinely uncommitted/noaccess. A whole matching reserve
  can be replaced by a direct view. Under the owner lock the old reservation is
  released, a section view placed, and on failure the reservation is restored.
  This is **not atomic against unrelated host allocations**. Restoration failure
  removes the lost mapping record and reports native failure; it is never success.
- Unmap is whole-view only. Release is whole-resource logical release; existing
  views retain backing until last unmap, but no new views can use the released offset.
  Double release/unmap and overlap refuse. Partial unmap/protection/replacement are
  unsupported. POSIX munmap/mprotect return -1 with per-thread errno on failure;
  SCE adapters return distinct status codes.
- Queries cover this owner's real resources only; image/heap/stack/TLS query
  unification is deferred. Direct query writes exactly 24 bytes: start64/end64/type32,
  four zero reserved bytes. VM query writes exactly 72 bytes: start64/end64/offset64,
  protection32/type32/flags8/name32, remaining bytes zero. Direct flags use inherited
  `0x12`; reserve flags are neutral zero (limited policy, not fully proven firmware ABI).
  Outputs are byte encoded, not Rust-layout serialization, and fully preflighted.
- MapDirectMemory2's seventh argument comes from the checked guest stack. A changed
  memory type refuses rather than claiming to alter cache behavior. Named-map's
  optional name remains the prototype's ignored argument; SetVirtualRangeName stores
  a checked name up to 32 bytes. Flexible memory, general mmap, batch maps and partial
  view operations are not claimed by these registrations.

HLE checked access holds the resource mutex while inspecting/copying resource views.
No Rust reference is manufactured over guest bytes. Native guest accesses may race
unmapping like native memory operations; faults use the existing recovery boundary.
The owner serializes allocation/deletion/protection; it is not a process-global lock.
Outputs are preflighted; publication failure rolls back newly allocated resources.
Host copy/protection failures after mutation are structured errors, not atomicity claims.

## Semaphore contract

Runtime-owned non-reused u32 handles identify bounded objects. Runtime configuration
provides object/waiter caps; names are bounded to 31 bytes, including an allowed empty
name. Create writes the handle through argument zero and returns zero. The corrected
prototype WaitSema/SignalSema identities are preserved; host handles never escape.

Counts obey `0 <= initial <= maximum`, maximum positive, requested count positive.
Immediate wait consumes available count; poll returns Busy if unavailable. Blocking
wait queues an M33 guest Thread identity and a M25 ticket. FIFO counted grants are
explicit emulator policy, not firmware priority scheduling. Attr 0/1 is accepted;
priority mode/option structures refuse rather than pretending to schedule priorities.
A leading larger request is not bypassed by smaller queued requests.

Wait timeout is a checked relative u32 microsecond in/out value. Null means no guest
limit, still bounded by the armed execution deadline. There is no wall-clock or sleep
poll loop. Signal validates maximum-count overflow and grants only after ticket
cancellation wins against expiry; count is reserved before returning to a caller.
Delete wakes retained waiters as Deleted. Cancel resets the count (negative means
initial), reports waiter count and wakes Cancelled. Shutdown wakes Interrupted;
neither is normal success. Registry lock and ticket completion define race ordering.
Callers remove their own wait records after waking. Rust poison does not become guest
errno. Distinct Invalid/NotFound/Deleted/Busy/Overflow/Timeout/Cancelled/Interrupted/
Capacity results are retained. Overflow uses explicit SCE/FreeBSD error-number inference
rather than prototype generic invalid; full priority/fairness accuracy remains unproven.

Runtime shutdown prevents new work, stops semaphore and existing pthread waits, joins
workers, then releases resource mappings/backing and thread storage. No detached worker,
waiter or ticket is intentionally retained. M25 remains the only scheduler.

## Diagnostics and fingerprint

The owner snapshots feed M44 `kernel_resources`: direct allocation count/bytes,
active mappings, lifetime allocations/maps, semaphore count/waiters/waits/signals/
timeouts/cancellations. Semaphore waiting identities join thread-state aggregation.
The dashboard has one concise kernel-resource row; no per-call terminal output is added.
Kernel health follows real provider activity via existing M44 health classification.
JSON schema version 1 gains an additive defaulted object; old reports deserialize with
zero resource metrics. Stop snapshot describes resources before teardown; teardown
separately reports zero remaining reservations/waiters and release errors.

## Real experiments and refined hypotheses

Commands were the existing first-entry workflow, no sampling, explicit 250 ms wall,
15000 ms containment, 65536 HLE calls. `target/m45-primary-1.ps1` and
`target/m45-second-4.ps1` produced final evidence; exact reusable CLI is in USAGE.
Raw scripts/reports/logs are ignored. Both artifacts retain image bias 0x100000000.

Primary `primary_real_elf`: GetDirectMemorySize called once, returned 0x400000000.
No resource allocation/mapping or semaphore call occurred. It stopped at
sceSysmoduleLoadModule, NID **0x83C70CDFD11467AA**, libSceSysmodule/libSceSysmodule.
Overall 202.998 ms; supervised native 201.034 ms; 11000 HLE calls, 54 unique providers.
Boundary RIP **0x7ff6d020b356** is an Astero RuntimeBridge landing, RSP **0x200800808**;
separate guest continuation **0x1003f6f5e**, native thread ID 12428 (run-local).
Nine threads including main, eight workers joined. Eight pthread waits were frozen
at stop; zero waits/wakes/timeouts were semaphore activity. Heap peak 1055728 bytes.

Second `named_title_elf`: four successful CreateSema calls (attr1, initial0,
maximum0x7fffffff, null options), output slots 0x200800f40/0x200800f48, name pointer
0x10174b2ac. One 1 MiB ReserveVirtualRange with 256 KiB alignment selected
**0x400000000**; VirtualQuery consumed the 72-byte output. MainDirectMemory allocated
1 MiB, default alignment, type12, offset0. MapDirectMemory replaced the whole reserve
with effective RW (`0xF2` requested), exact flag0x10; return0. Snapshot: one backing
allocation, 1048576 bytes, one view, two lifetime mappings (reserve+map), four semaphores.
No actual semaphore wait/signal/cancel occurred; those paths have synthetic proof only.
It stopped at **malloc_stats_fast**, NID **0x2AE3AE0F9F21AA7E**, libc/libc.
Overall **5.191 ms**, native **4.588 ms**, 247 calls/28 providers, main only.
Boundary RIP **0x7ff7830cb236** is RuntimeBridge, RSP **0x200800ec8**;
guest continuation **0x100fedcbe**, native thread ID17860. Heap peak137616 bytes.

Intermediate second runs are retained, not rewritten as clean progression:
1. CreateSema passed; ReserveVirtualRange unresolved (4.192 ms overall), then migrated
   inside the approved VM family.
2. Type12 was incorrectly rejected by an overly narrow type0..3 validator. Guest then
   wrote the still-reserved address **0x4000fffe0**, controlled AV at guest RIP
   **0x100bc8a69** (7.566 ms). Rejected hypothesis: only types0..3 are valid.
3. Type12 accepted, but protection0xF2 was incorrectly masked out; map returned
   0xffffffff8002002d, same controlled AV (4.821 ms). Refined to prototype CPU low-bit
   conversion and opaque higher metadata. The final run passed both paths.

All runs returned controlled reports with clean containment. Final runs restored FS,
preserved GS, joined workers, left zero native resources/reservations, waiters or sleep
tickets, and no release errors. Source SHA256 before/after:
- primary: `A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A`
- second: `3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327`
No forced containment kill or supervisor intervention was needed in final runs.
Historical Windows error487 remains unresolved/unreproduced, not assigned a cause.

## Rejected prototype shortcuts and remaining limits

Rejected: global loose registries, raw host identities, scratch/fake allocations,
new-zero-backing on remap, uninterruptible host waits, wrapping handle reuse, creation
returning a handle instead of status, erroneous Wait/Signal NIDs, cancellation success,
and first-fit or size policy described as hardware truth. Inherited useful behavior:
offset/view separation, shared bytes/lifetime, coherent family identities and count APIs.

Unproven: priority waiter scheduling, exact cache/memory-type meaning, full VM query
flags/layout variants, rare cancellation races beyond synthetic arbitration, and real
semaphore blocking in these workloads. Unsupported: partial views, broad process VM
queries, flexible/general mmap services, cache/GPU protection semantics, sub-64KiB view
offsets. Fixed policy caps and the 4GiB VA band may require deliberate future expansion.
Reservation replacement has the documented host placement race. No title-specific path,
ID or address test exists. The next observed work is sysmodule startup for primary and
allocator statistics for second; neither was implemented during M45.

## Registered identities

All rows use exact libkernel/libkernel callable keys. Runtime-confirmed means the
specific identity succeeded in these runs, not every variant or argument is proven.

| Export | NID | M45 status |
| --- | --- | --- |
| sceKernelGetDirectMemorySize | `0xA4EF7A4F0CCE9B91` | Registered; runtime-confirmed |
| sceKernelAllocateDirectMemory | `0xAD35F0EB9C662C80` | Registered; unexercised in real runs |
| sceKernelAllocateMainDirectMemory | `0x07EBDCD803B666B7` | Registered; runtime-confirmed |
| sceKernelAvailableDirectMemorySize | `0x0B47FB4C971B7DA7` | Registered; unexercised in real runs |
| sceKernelDirectMemoryQuery | `0x047A2E2D0CE1D17D` | Registered; unexercised in real runs |
| sceKernelMapDirectMemory | `0x2FF4372C48C86E00` | Registered; runtime-confirmed |
| sceKernelReleaseDirectMemory | `0x301B88B6F6DAEB3F` | Registered; unexercised in real runs |
| sceKernelCheckedReleaseDirectMemory | `0x8705523C29A9E6D3` | Registered; unexercised in real runs |
| sceKernelVirtualQuery | `0xAD58D1BC72745FA7` | Registered; runtime-confirmed |
| getpagesize | `0x93E017AAEDBF7817` | Registered; unexercised in real runs |
| munmap | `0x52A0C68D7039C943` | Registered; unexercised in real runs |
| mprotect | `0x61039FC4BE107DE5` | Registered; unexercised in real runs |
| sceKernelMapNamedDirectMemory | `0x35C6965317CC3484` | Registered; unexercised in real runs |
| sceKernelMapDirectMemory2 | `0x0504278A8963F6D4` | Registered; unexercised in real runs |
| sceKernelSetVirtualRangeName | `0x0C6306DC9B21AD95` | Registered; unexercised in real runs |
| sceKernelReserveVirtualRange | `0xEE8C6FDCF3C2BA6A` | Registered; runtime-confirmed |
| sceKernelCreateSema | `0x840B48A18B097561` | Registered; unexercised in real runs |
| sceKernelCreateSema | `0xD7CF31E7B258A748` | Registered; runtime-confirmed |
| sceKernelWaitSema | `0x6716B45614154EC9` | Registered; unexercised in real runs |
| sceKernelSignalSema | `0xE1CCE9A47062AE2C` | Registered; unexercised in real runs |
| sceKernelPollSema | `0xD76C0E1E4F32C1BD` | Registered; unexercised in real runs |
| sceKernelCancelSema | `0xE03334E94D813446` | Registered; unexercised in real runs |
| sceKernelDeleteSema | `0x47526F9FC6D2096F` | Registered; unexercised in real runs |

23 identities (16 VM,7 semaphore) added to the runtime registration family; two already
observed identities promoted. Final NID state is 363 registered/4 observation-only
(367 total). The two new unresolved boundaries remain unregistered. Six ABI records
remain; no new standalone guest-layout contract is promoted beyond these scoped adapters.

## Validation and changed-file routing

30 new executable tests:10 direct-resource,10 semaphore,2 shared-backing,7 libs-adapter,
1 core JSON compatibility. Coverage includes aliases/remap persistence, invalid geometry,
RO/RX/noaccess, rollback of failed reservation replacement, allocation concurrency,
release-while-mapped, FIFO counts, cancellation/delete/shutdown/expiry, exact outputs,
and additive report compatibility. Full commands/counts are in validation.md.

Owners changed: memory Windows backend/re-export and tests; kernel objects/memory and
semaphore roots/mechanism/tests; libs kernel adapters/root/tests; core worker composition,
closure cleanup and observation model/collector/tests; CLI dashboard. Documentation:
three owner READMEs, this architecture record/navigation, USAGE, validation and index links/
generated human indexes. Local PROJECT_STATE retains M45 until cleanup approval.
