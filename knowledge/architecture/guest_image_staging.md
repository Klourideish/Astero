# M27: bounded real guest-image staging

Staged is not runnable. M27 executes no guest code.

## CPU direction and ownership

The primary CPU strategy is direct native execution of compatible decrypted PS5 x86-64 instructions
on x86-64 Windows, inherited as project direction from the experimentally validated PS5Rust prototype.
See [decisions](decisions.md). No interpreter/JIT or instruction-fetch abstraction is introduced.

Loader load_plan/staging consumes a private immutable M26 GuestLoadPlan. Its StagingBackend contract
requires fresh isolated storage, complete-image reservation, zeroed logical mappings, checked byte
writes, final protection and deterministic cleanup. Core input/staging adapts that interface to
memory/mapping::OwnedAddressSpace. Core -> memory is the sole new dependency edge, already allowed.
Loader stays dependency-free. No new crate, host library, timer, session or global registry is used.

Memory has its own typed GuestAddress; core translates the existing loader VirtualAddress intent.
Neither is a host pointer. Sparse logical regions allocate only their sizes, not holes or the full
VA envelope. Alignment preserves M26 bias and ELF source/address congruence; segment starts need not
be aligned to p_align (a real utility segment starts at 0xc030 with alignment 0x4000).
Byte storage has byte granularity; this is not an assertion about Windows reservation granularity.

## Native backend compatibility and limits

The backend sees the complete mapping list before writes so future Windows VM code can reserve a
whole image, account for allocation/page granularity, and merge shared-page protection requirements.
It must accept exact planned addresses or refuse. A changed bias needs replanning, not silent pointer
translation after relocations have embedded guest addresses. Direct instruction operands require
host/guest address correspondence for all reachable code/data, not just translation of the entry.
M27 byte vectors explicitly provide no such correspondence and expose no executable pointer.
They cannot be promoted to native execution by casting a buffer. Restaging into a proven native
reservation is required. No instruction decoding is part of memory access.

Intended R/W/X flags stay in the retained plan, including execute-only and no-access regions.
During isolated construction all regions are stageable. After copies and writes, the byte backend
freezes its write/map API and reports MetadataOnly protections. It does not pretend VirtualProtect,
NX or RELRO enforcement occurred. No executable host allocation is requested. A future native backend
must finalize page protections, resolve shared-page W/X conflicts, handle RELRO and failure/unmap,
and prove executable mappings before entry. Current unsafe policy would require an explicit narrow
backend policy decision; it is not relaxed in M27. Native entry/ABI bridges, TLS, exception recovery,
HLE boundaries, leases and teardown coordination remain kernel/execution work.

## Evidence reviewed

PS5Rust catalogue routes: 00_START_HERE/README.md, catalogue_map.md; 03_LOADER_LINKER/elf_self_protections.md,
relocation_subset.md, module_queries_unload.md and source/ps5-core__src__loader__self_loader/INDEX.md;
04_MEMORY/INDEX.md, address_space_contract.md, source/ps5-core__src__memory__address_space/001.md;
02_CORE_EXECUTION/INDEX.md, native_boundary.md, source/ps5-core__src__cpu__native_exec/001.md.

These preserve separate file/VA ranges, zero-fill, checked errors, final page permissions (shared
pages unioned and holes inaccessible), supported numeric writes, and retirement/TLS execution leases.
Late lifecycle debt includes stale TLS/dispatcher state and unsafe unmap while native work retains
references. Source tests in the catalogue were not run here. Native boundary records describe FS/TLS
switches, assembly entry, import trampolines and exception/stop scopes. No legacy raw-pointer access,
RWX shortcuts, fallback addresses, destructors or native invocation is copied into M27.

Decrypted catalogue route: 00_START_HERE/README.md then 01_IMAGES/PS5Util.elf/mapping.md. This shows
PT_LOAD/BSS and separate RELRO, TLS, SCE parameter and unknown 0x6fffff00/01 headers. It is a different
build, not an expected-count oracle. M26 guest_load_link_plan.md remains the plan semantics authority.
No external emulator or additional firmware source was needed.

## Admission, writes and transaction

Stage-blocking: missing evidence, unsupported machine/object type, malformed/overlapping segments,
unknown program semantics outside the explicit exception set, invalid target, arithmetic, overlapping
writes or nonzero RELATIVE symbol. Such failures return no image. Stage validation rechecks plan range
arithmetic, source tokens/identity, source/destination congruence, exact zero-fill extent and targets;
it does not parse ELF again. The plan and source remain shared and immutable.

Resolution-blocking: dependencies, references, provider evidence/residency, unavailable symbol values
and unsupported numeric relocation actions. These remain pending, not fabricated writes.
Execution-blocking: missing/nonexecutable entry, bootstrap, TLS, RELRO, SCE process/module parameters,
and 0x6fffff00/01 ancillary headers. Unknown ancillary interpretation remains experimental; staging
copies only established PT_LOAD bytes and does not claim those headers have been implemented.
DeferredDiagnostic is reserved; no current blocker is silently discarded into it.

Map zeroed regions, copy exactly source-backed prefixes, then apply supported 4/8-byte little-endian
concrete values from M26. x86-64 byte writes do not require natural alignment. M26 controls arithmetic;
staging does not recompute formulas. NONE performs no write. Unresolved/unsupported actions retain
original bytes. Even a selected provider with a concrete bias remains pending if it is not staged
here: r.resolution must be absent to permit application. Known-width targets are checked even when
pending. Original plan order/raw actions stay inspectable alongside Applied/Pending/NoWrite records.

All writes occur in a new owned image. Any allocation/copy/write/finalization error clears its mappings;
no partial image escapes. Error context includes operation/index and structured source/backend cause;
source provenance is in the input plan. Explicit release is idempotent and Drop releases mappings.
A per-image observer survives Drop and reports active region count, without retaining storage.

## Limits, status and diagnostics

Required max_mapped_bytes bounds sum of logical segment memory sizes, checked before mapping. Zero
refuses any nonempty image; exact size succeeds. Vec allocation uses checked conversion/try_reserve.
Existing M26 record budgets bound segment and relocation state counts. Reads borrow bounded slices;
there is no public mutable backing access on StagedGuestImage. No prefix is reported as success.

Staged and StagedWithPendingWork distinguish pending references/writes; Released has zero active
mapping count/bytes. All M27 images report ready_for_execution=false even with no M26 blockers.
Snapshots retain copied/zero-fill/applied/pending totals as historical staging facts after release.
The plan retains identity, intended permissions, bias, entry/bootstrap and every blocker.
CLI limits displayed detail to 16 ranges/blockers with explicit omitted counts; totals are exact.

## Experiments and outcomes

Real roles came exclusively from ignored LOCAL_TEST_CORPUS.json; no corpus files copied into Git.
All three inputs were SHA256-identical before/after (M26 validation records contain the same hashes).

| Role | Regions | Mapped bytes | Copied | Zero fill | Applied | Pending | Unresolved references |
|---|---:|---:|---:|---:|---:|---:|---:|
| linkage_sample | 5 | 3367 | 3336 | 31 | 2 | 9 | 8 |
| utility_build_comparison | 5 | 3343 | 3312 | 31 | 2 | 9 | 8 |
| primary_real_elf | 5 | 17424635 | 9164955 | 8259680 | 29791 | 1107 | 822 |

All returned StagedWithPendingWork, exit 0, MetadataOnly, no execution readiness and zero mappings
after teardown. Bias 0x100000000; both utilities entry absent; executable entry intent 0x100000070.
Utility ranges relative to bias: [0,0x3a2), [0x4000,0x41e5), [0x8000,0x80b0), [0xc000,0xc030),
[0xc030,0xc6f0) (comparison ends 0xc6d8). Permissions respectively X,R,RW,RW,none.
Executable ranges: [0,0x5242bc), [0x528000,0x703ed7), [0x704000,0x77c888),
[0x780000,0xfe5ae8), [0xfe5af0,0x10a56e8); same permission sequence.

Supported: unresolved references coexist with correct isolated staging; planned provider-independent
writes can be applied without provider closure; repeated teardown is safe. Refined: the catalogue
utility's large BSS does not describe these builds (31 bytes each). First real run blocked on
0x6fffff00; bounded policy now preserves 00/01 as execution blockers instead of interpreting them.
Unresolved hypothesis: ancillary tags/RELRO/SCE bootstrap need no additional PT_LOAD staging action;
real runs support structural copying only, not native readiness. Byte backing validates neither
Windows placement nor executable protection. Those constraints remain explicit for the native backend.

M28 recommendation: isolated Windows native VM backing and address-correspondence/protection proof,
consuming the same plans, before guest entry. Provider closure, ABI/TLS/exception setup and runtime
initialization still block execution; do not conflate native mapping success with permission to run.
