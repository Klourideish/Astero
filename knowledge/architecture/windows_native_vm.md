# M28: isolated Windows native VM realization

Executable host mapping is not guest execution. NativeBackedWithPendingWork is not runnable.

## Ownership and narrow safety exception

Primary CPU strategy remains native x86-64 Windows execution, not an interpreter/JIT.
Memory/mapping/windows_native owns safe layout/model and a private platform.rs FFI leaf. Core
input/native explicitly consumes an existing M27 StagedGuestImage, not raw ELF bytes. It retains the
same immutable plan and original staging statistics. Loader, timing and dependency edges are unchanged.
CLI native-map composes acquisition/planning/staging/realization explicitly and tears both images down.
No new crate or dependency. Windows x86-64 gates all FFI; other hosts explicitly refuse the CLI path.
Non-Windows cross compilation was not performed; generic layout tests do not require Windows.

M28's requested Windows allocation/protection requires a deliberate exception to the earlier
GUI-only unsafe boundary. astero-memory now denies unsafe, with allowance only on its private
Windows platform module. Workspace forbid remains, GUI exception unchanged, and a policy test pins
this narrow opt-in. Local AGENTS routing was updated consistently and stays untracked. Execution
entry/ABI/exception host integration remains kernel/execution; VM mechanisms stay in memory.

All raw pointers, Windows flags, pseudo-handles and FFI declarations are private. x64 SYSTEM_INFO
and MEMORY_BASIC_INFORMATION layouts have compile-time size checks. Reservation ownership is installed
immediately after successful VirtualAlloc, before commit can fail. Copy uses the returned allocation
pointer plus a checked offset. Every copied range is in prevalidated committed RW pages and disjoint
from the live source allocation. No externally writable pointer, callback or entry address API exists.
The native owner is deliberately not Send/Sync; future leases/thread safety require explicit design.

## Focused evidence

PS5Rust catalogue: 00_START_HERE/README.md; 04_MEMORY/INDEX.md;
source/ps5-core__src__memory__address_space/INDEX.md then 002.md (dynamic-module base/collision),
004.md (reservation guard), 007.md (translate_vaddr); 02_CORE_EXECUTION/native_boundary.md.
M27 guest_image_staging.md and M26 guest_load_link_plan.md supply current plan/staging authority.
The earlier decrypted PS5Util mapping record remains corroboration of a different build; no new
corpus/catalogue bulk ingestion or external emulator comparison was needed.

Legacy source comments record a 96-TB placement failure with Windows error 487 and an empirical base
change. This is documentary workload evidence, not a universal address rule. Astero preserves the
actual Windows error, never substitutes a magic legacy base. The reservation guard lesson is essential:
commit failure after reservation must not leak an unpublished range. Native boundary evidence requires
TLS/FS, stack/context, exception recovery and lifetime coordination before entry; none is implemented.
Legacy checked host translation for host services does not prove arbitrary host placement is compatible
with unmodified native instructions containing absolute addresses.

Primary Windows API references reviewed:
- [VirtualAlloc](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc)
- [VirtualProtect](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualprotect)
- [VirtualFree](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualfree)
- [FlushInstructionCache](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-flushinstructioncache)
- [SYSTEM_INFO](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-system_info)
- [MEMORY_BASIC_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-memory_basic_information)

## Exact address model and bounded reservation

Host address equals the already-biased guest address in M26/M27. Source offsets remain independent.
No extra host bias, arbitrary-placement fallback, ASLR retry loop or relocation recomputation occurs.
A conflicting reservation fails atomically with OS code/address/extent. Explicitly choosing a different
image bias requires a new plan and staged bytes. It is not a repair applied to existing pointers.

GetNativeSystemInfo supplies page size and allocation granularity (observed 4096 and 65536). Complete logical
ranges are checked for size/source agreement, overlap and overflow before any VM mutation. Reservation
base rounds down to allocation granularity, end rounds up to page size. Logical segment boundaries and
permissions remain in the retained plan. One bounded contiguous envelope is reserved, without backing
for holes. Only pages intersecting logical regions are committed. Existing gaps and envelope padding
remain reserved/inaccessible where no page is committed; no speculative giant guard arena is allocated.
Partial-page padding necessarily follows the page protection; it is marked as coarsening/widening.

NativeLimits separately bounds reservation envelope and committed page bytes; CLI's required
--max-native-bytes supplies the same explicit maximum to both. M27 max-mapped-bytes still bounds logical
backing. Page-table work/allocation is bounded by envelope/page count. Zero and one-byte-short budgets
refuse, exact boundaries succeed. Allocation conversions/arithmetic and Vec reservations are checked.
The original M27 byte image coexists during realization, so peak storage includes both backends.

## Materialization and protection proof

Reserve MEM_RESERVE, commit occupied pages RW (not executable), copy the complete M27 staged ranges,
including BSS and applied relocation bytes. M28 applies no relocations, reads no new ELF payload and
makes no provider assumptions. Compare all materialized bytes before final protections. Compute each
page's permission union; Windows RW entails read access. Any union containing write+execute is refused,
even if a logical input asks for it. No automatic RWX fallback exists.

Final Windows requests cover NOACCESS, READONLY, READWRITE, EXECUTE and EXECUTE_READ. Effective
protections remain generic R/W/X in diagnostics; Windows constants do not cross the backend boundary.
VirtualProtect runs only on owned aligned committed pages. FlushInstructionCache runs for every
executable page after writes/protection; x86 coherence is not treated as an API exemption. VirtualQuery
checks allocation owner, committed state and expected protection for each occupied page. Readable
portions are additionally checked with ReadProcessMemory after transition. Execute-only/no-access
bytes are never dereferenced after finalization; they were compared during RW construction.
This is an OS protection-state proof, not a claim of hardware execute-only isolation or guest execution.

Shared-page permissions may exceed an individual segment's intent. Both utility builds have RW data
sharing their last page with a logically no-access metadata segment; report widening instead of hiding
it. Widened pages also include subpage padding. This is acceptable for this non-running realization,
not blanket native-execution admission. Logical protections remain separately inspectable.

## RELRO and pending work

Known PT_GNU_RELRO ranges remain in source-bound plan headers and are printed. M28 deliberately defers
RELRO finalization while final provider-dependent relocation writes are pending. It does not silently
advertise full RELRO or runtime readiness. All tested real inputs have pending writes. Future final
link/protect must validate RELRO coverage/page edges and freeze those pages after closure; the current
backend is immutable once published and has no later patch API. This deferral does not block enforcing
ordinary executable/read/data page protections now.

M27 applied/pending counts and source identity are retained exactly. Native lifecycle distinguishes
NativeBacked, NativeBackedWithPendingWork and Released. Even no pending relocation is not entry
permission; provider residency, bootstrap/TLS, exceptions and execution lifetime remain absent.

## Failure, introspection and teardown

Structured failures distinguish unsupported host, geometry, empty/malformed ranges, overlap, overflow,
reservation/commit budgets, allocation, writable-executable conflict, OS reservation/collision,
commit/protect/query/read/cache flush/release failures and lifecycle misuse. OS errors retain operation,
address, size and code. No partial image is published. Private deterministic fault checkpoints test
rollback after commit, before protect and after protect; actual Windows reservations are released.

Reservation RAII owns both reserved and committed pages. VirtualFree releases the original base with
zero size and MEM_RELEASE. Explicit release is idempotent; errors keep ownership for retry. Drop attempts
release and retains a failure code in the observer rather than claiming zero mappings. A surviving
per-owner observer contains no pointer and records active reservation count/release error. No global
mapping registry or orphan worker exists. Native snapshots retain historical verified facts after
release but mark inactive and committed bytes zero. Tests verify retrying the same address after rollback.

## Real-artifact experiment

All paths came from LOCAL_TEST_CORPUS.json. All three SHA256 hashes matched M26/M27 before/after.
No input copied into the repository. Exact validated commands are in USAGE.md.

| Role | Identity envelope | Committed | Materialized | File / zero-fill | Applied / pending | Widened pages |
|---|---|---:|---:|---|---|---:|
| linkage_sample | 0x100000000..0x10000d000 | 16384 | 3367 | 3336 / 31 | 2 / 9 | 4 |
| utility_build_comparison | 0x100000000..0x10000d000 | 16384 | 3343 | 3312 / 31 | 2 / 9 | 4 |
| primary_real_elf | 0x100000000..0x1010a6000 | 17432576 | 17424635 | 9164955 / 8259680 | 29791 / 1107 | 5 |

All three returned NativeBackedWithPendingWork, exit 0; zero native reservations and byte mappings after
teardown, release error zero. Source IDs and plans remain process-local immutable evidence. Utilities
have 8 unresolved references each; executable has 822. Effective utility pages: X at +0, R at +0x4000,
RW at +0x8000 and +0xc000. Executable has 4256 committed pages, never RWX. Complete page diagnostics are
retained; CLI shows at most 16 entries with omission counts. No guest entry, instructions, RIP/RSP,
threads, TLS setup, constructors, HLE, dispatcher or timing workers occurred.

Supported: exact identity placement at the selected bias works on this host; copies/protections/query
and readback succeed; collisions preserve the first owner; all normal/fault cleanup releases ownership.
Refined: logical no-access and RW cannot coexist exactly on one page; widening remains observable.
Unresolved: other host address layouts, final RELRO, executable-page subrange isolation, exception/ABI/
TLS and provider closure. No inference of general workload execution compatibility follows from this.
M29 should establish a deliberately chosen workload's provider/bootstrap and execution-lifetime
prerequisites, with a separate native-entry review. It has not begun.
