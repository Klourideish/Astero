# M35 libc large-memory/runtime continuation

## Baseline and ownership

PS5Rust migration evidence routed through 00_START_HERE/README.md and catalogue_map.md,
05_KERNEL_HLE/source/ps5-libs__src__kernel__libc/INDEX.md,
05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__exports/INDEX.md and 001.md,
04_MEMORY/source/ps5-core__src__memory__allocator/INDEX.md and 001.md.
Focused original PS5Rust sources: crates/ps5-libs/src/kernel/libc/{exports,nids}.rs (memory,
ASCII strings, tokenization, alignment/usable-size and delete handlers), and
crates/ps5-core/src/memory/allocator.rs (bounded 2 GiB arena, ownership/coalescing and observed
free-list performance pressure). Prototype comments are implementation evidence, not independent
firmware proof. No external emulator or new firmware investigation was needed.

M32's 1 MiB constant was an explicit development copy/scan and scratch-allocation bound. It was
also checked in core's access adapter: policy and validity had become coupled. It was not an ELF,
Windows or libc semantic. Memory owns native range traversal; HLE owns the checked-copy interface
and generic byte budget; libs owns libc semantics; core supplies retained runtime owners. No new
crate dependencies. Unsafe remains confined to the existing private memory/native bridge leaves.

## Checked work

Full source/destination coverage and permissions are preflighted before bulk mutation, including
adjacent explicit mapping owners. Gaps, guards, read-only writes and checked-add overflow refuse.
Write access remains RW/NX only. Mapping page lookup is binary search, not quadratic page rescanning.
OS checked copies never manufacture guest Rust references. Each read/write scratch copy is at most
64 KiB. Memmove chooses backwards chunks for destination-over-source overlap; memcpy explicitly
refuses overlap rather than claiming memmove semantics. Memset fills chunks and memcmp compares
chunks. Invalid tails refuse before the first write. A later OS failure/concurrent guest lifetime
change may leave a partial operation: no rollback or atomic-copy guarantee is made. HostCopy failures retain OS operation, failing chunk address/size and error code across the HLE boundary; no completed-byte count is invented.

Named runtime policy is 64 MiB per logical primitive and 512 MiB aggregate byte work per execution.
AccessLimits/AccessBudget accept explicit limits; current runtime composition selects the named policy above. No new CLI knob per helper was added.
Charges count bytes attempted, including scan windows and failed range preflight; copy charges its
logical extent once (not separate source and destination bytes). Allocation and copy scratch are
separate budgets. Strings scan mapped page windows, stop at NUL, remain byte-oriented and bounded.
strstr uses linear KMP with a separate 1 MiB pattern bound; case folding is ASCII, not guest locale.

M34 already protected process heap/environment/callback state with owned locks; per-thread errno
remains independent. Shared aggregate charging is atomic. Guest races are not transformed into
Rust aliasing; a storage lock only spans checked access, never a guest synchronization wait.
The existing 4096 provider-attempt budget remains explicit and process-wide; M34 used only 212
records, so no evidence justified raising it before continuation.

## Heap and migrated surface

The reusable 4 MiB heap remains bounded and mutex-owned; M34 had only 3392 live bytes. No evidence
yet justifies copying the prototype's 2 GiB arena. Alignment-aware splitting/coalescing now supports
memalign/aligned_alloc/posix_memalign and owned usable-size queries; peak bytes are recorded.
Allocation failure retains ownership, and invalid POSIX alignment leaves the output slot unchanged
rather than copying the prototype's eager zero publication. No arbitrary host-pointer fallback.

Added exact identities: memchr, strcpy, strncpy, strcat, strncat, strstr, strcasecmp, strncasecmp,
strdup, strtok_r, strspn, strcspn, memalign, aligned_alloc, posix_memalign, malloc_usable_size and
aligned operator delete. Existing malloc/calloc/realloc/new/delete, environment/errno/process and
controlled __stack_chk_fail/exit behavior remain shared. Full C++ unwind, filesystem, scheduling
and TLS destructors are out of scope. Prototype read failures returning fabricated equality/null
are replaced with explicit checked failures.

## Checked CRT and process output continuation

First continuation reached puts; second reached strcpy_s. Both belong to this approved wave, so
puts and the complete focused memcpy_s/memset_s/strcpy_s/strncpy_s/strcat_s/strncat_s family were
adapted before the third run. PS5Rust exports.rs lines 427-541, 2959-3138 and 3936-3943 establish
this baseline. Checked CRT uses raw EINVAL=22/ERANGE=34, including deliberate destination clearing
on constraint failure. Those defined error effects are not an accidental partial successful copy.
Policy/allocation refusal remains explicit AccessFailure. Normal operations still preflight the
full affected range. POSIX alignment failures preserve the output slot. Added allocation helpers
set only the caller's errno (EINVAL for invalid alignment; ENOMEM on exhaustion).

puts appends raw bytes and newline to an owned process diagnostic sink (64 KiB total, 8192-byte
terminated input window) and returns nonnegative success. It neither opens files nor assumes host
FILE layout; invalid/capacity failures are explicit, never silently dropped output. It shares only
its own short sink lock. The selected workload printed `RezVR Start !!!`.

## Real workload observations (2026-09-11)

All runs used primary_real_elf through LOCAL_TEST_CORPUS.json, exact unchanged first-entry command
in USAGE.md, 250 ms native wall bound and 15000 ms parent containment. First preflight included full
Rust tests, all-targets build, Clippy, policy and synthetic M31/M33/M34 tests. Added puts/checked CRT
were tested and rebuilt before the next permitted within-cluster continuation. Three runs total;
no fourth run and no UserService implementation. Each exited 0 with Clean containment.

| Run | Stop | NID | Duration | Boundary RIP | RSP | Separate guest continuation |
|---|---|---|---|---|---|---|
| 1 | Unresolved puts | 0x610d276afa7e6087 | 13,786 us | 0x7ff6fcb7ad96 | 0x200800c58 | 0x10030ac9b |
| 2 | Unresolved strcpy_s | 0xe576b600234409da | 13,142 us | 0x7ff7c018c5f6 | 0x200800c58 | 0x10030acb3 |
| 3 | Unresolved sceUserServiceInitialize | 0x8f760cbb531534da | 13,256 us | 0x7ff74bd2d4e6 | 0x200800c58 | 0x10030ae13 |

These RIPs are captured **host import landing PCs**, not executing guest PCs. Initial context stayed
RIP=0x100000070, RSP=0x200800fb8, FS=0x210000000. Final unresolved exact key:
libSceUserService/libSceUserService, ordinal400; arguments
`[0x200800c88, 0x1005ebb47, 0, 0, 0x78, 0x80]`. No fault/exception occurred.
Final native interval=12,059 us, Windows thread8540, no supervisor redirection or suspensions.
Host FS restored and GS preserved for main and all eight workers.

The original 1,352,000-byte memset returned its destination 0x100906e68 successfully in all three
runs. Its affected half-open range is [0x100906e68, 0x100a50fa8). The final run also completed a
9,580-byte memset at 0x1009048f8. Exactly one operation exceeded 1 MiB. Aggregate charged primitive
work was 1,380,184 bytes with zero access-budget refusals; 64 MiB/512 MiB policy remained far away.

Final main report: 280 dispatched provider records, including 97 operator-new calls, 47 memcpy,
10 delete, 9 strlen, 4 memcmp, 2 memset, 1 puts, 1 strcpy_s and 1 strstr. Scheduling attr behavior
remained eight explicit ENOTSUP returns; no scheduling migration was attempted. Sixteen worker
provider records: eight MutexLock returned and eight CondWait interrupted at owner shutdown.
No worker libc memory operation was observed in the corpus run; a separate synthetic two-native-worker
1,352,000-byte memset test proves that path and shared heap/budget behavior.

All eight M34 workers ran. Nine synchronization objects, eight waits, zero signalled wakes, zero
timeouts; final waiters0/stopped=true. No false successful-wake claim. Peak live heap=7104 bytes,
87 live allocations before teardown, 97 allocations/10 frees, one 4 MiB arena, no growth. This current
observation rejects speculative arena enlargement: 4 MiB remains explicit temporary runtime policy.
GuestHeap accepts a size parameter; future growth must retain ownership/bounds and is not claimed here.

All guest host threads joined, native/runtime reservations0, releaseerrors[], no orphan/suspended
workers. Source SHA256 before/after EACH run:
`A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A`.
Raw local audit logs: target/m35-real-{1,2,3}.log (ignored). Post-run refinements to preparation-only
access, alignment errno, typed OS copy failures, free-list spare capacity and synthetic worker tests did not rerun the corpus.

The final name was corroborated narrowly through 07_NIDS/MASTER_NID_INDEX.md ->
by_sysmodule/libSceUserService.sprx/INDEX.md -> 019.md, exact record j3YMu1MVNNo. That record joins
approved firmware numeric identity; no new firmware disassembly or UserService behavior was inferred.
It is one observation-only/unregistered index record, not an HLE implementation.

## Findings, limits and validation

Supported: valid large memset and chunked native access; shared process services remain usable with
real workers; bounded diagnostic output and checked strcpy_s advance startup. Refined: the old 1 MiB
cap was policy/scratch coupling, not mapping validity. Rejected: copying a fixed 2 GiB heap reservation
or returning fabricated success on bad guest addresses. Remaining limits: no atomic rollback for OS
copy failure/concurrent invalid lifetime; 64 MiB per primitive, 512 MiB aggregate work, 64 KiB scratch,
1 MiB strstr pattern, 4 MiB heap/4096 allocations, 4096 provider attempts, bounded output. Long operations
are O(bytes plus page lookup), not instruction-counted. Guest races have no stronger atomicity promise.
Full locale formatting, filesystem, dynamic TLS, scheduler policies and C++ unwinding remain outside scope.

Recommended M36, based on the final boundary only: UserService/startup platform-service foundation
migration beginning with sceUserServiceInitialize. No M36 implementation has started.
ABI remains two records (process arguments, synthetic native boundary); memory-copy details add no ABI.
Final full validation/counts are recorded in validation.md.

## Exact migrated identities

Each primitive has separate exact libc/libc and libkernel/libkernel registrations following the
existing composition policy; puts is libc/libc only. A numeric identity is counted once by the index.
Unexercised migrated functions are registered, not runtime-confirmed.

| New export | NID | Real M35 use |
|---|---|---|
| strcpy | 0x9226525C859DF6F8 | Not observed |
| strncpy | 0xEAC256896491BAA9 | Not observed |
| strcat | 0x2ECE2DCF38629AA4 | Not observed |
| strncat | 0x907838E6A3C2E9FD | Not observed |
| strstr | 0xBE28B014C68D6A60 | 1 successful call in final run |
| strcasecmp | 0x015EA2A4235AE11C | Not observed |
| strncasecmp | 0xA57BDB0DF721BBA9 | Not observed |
| strdup | 0x83BCF3CCB0D81B0D | Not observed |
| strspn | 0xFE453A6C1E0CFFE9 | Not observed |
| strcspn | 0xAB417AC92FEB0A6B | Not observed |
| strtok_r | 0x7A7A8F18B7E654D5 | Not observed |
| memalign | 0x5237F72B332F4662 | Not observed |
| aligned_alloc | 0xD81B6483C936E198 | Not observed |
| posix_memalign | 0x7154A4F72F1445B7 | Not observed |
| malloc_usable_size | 0x3437127DC619442F | Not observed |
| operator delete aligned | 0x6D9C7E1454A59143 | Not observed |
| memchr | 0xF2EF253F3504ABE5 | Not observed |
| memcpy_s | 0x3452ECF9D44918D8 | Not observed |
| memset_s | 0x87C1B0A8F15BBBA2 | Not observed |
| strcpy_s | 0xE576B600234409DA | 1 successful call in final run |
| strncpy_s | 0x60DCCD909CD8A848 | Not observed |
| strcat_s | 0x2BE81C9C51492957 | Not observed |
| strncat_s | 0x342E0C481F814508 | Not observed |
| puts | 0x610D276AFA7E6087 | 1 successful call in final run |

Index outcome: 201 NID records, 198 registered and 3 observation-only/unregistered. Twenty-four new registered numeric identities and one new runtime observation; ABI remains 2.
