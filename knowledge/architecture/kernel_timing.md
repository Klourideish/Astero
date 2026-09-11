# M40 kernel timing, clocks and sleeps

Kernel/timing owns guest clock/deadline interpretation and bounded wait ownership; libs/kernel/timing
owns exact NIDs, guest memory and SCE/POSIX result shapes. ABI/layouts/time owns explicit 16-byte
signed 64 seconds/nanoseconds encoding. Core owns GuestTiming and injects the existing M25 scheduler;
no new crate, dependency, unsafe leaf, timer worker or process-global owner.

## Focused migration evidence

Catalogue routes:00_START_HERE/README.md and catalogue_map.md;05_KERNEL_HLE/INDEX.md;
05_KERNEL_HLE/clocks_deadlines.md; 05_KERNEL_HLE/source/ps5-core__src__timing/INDEX.md and 004.md (worker/resolution),
05_KERNEL_HLE/source/ps5-libs__src__kernel__timer/INDEX.md and 004.md/005.md (diagnostics, sleep, nanosleep).
Read-only current counterparts:kernel/timer.rs constants 19-30, owned event registry/schedule,
sleep_duration 541-567, nanosleep 569-582, registrations 584-631;kernel/system.rs constants 19-32,
monotonic_micros 374-390, write_timeval/timespec/clockres 392-437, registrations 593-685.
System source routed through 05_KERNEL_HLE/source/ps5-libs__src__kernel__system/INDEX.md; current source is
more complete than compact catalogue excerpts. Corrected process/TSC NIDs retained, not old guesses.
15_FIRMWARE_KNOWLEDGE/contracts/clock_gettime.md and nanosleep.md corroborate veneers/identities;
clock_gettime IDs/normalization and nanosleep restart behavior are NOT proven by veneer alone.
10_VIDEOOUT/flips_and_vblank.md and M25 durable evidence reviewed as consumer pressure only;
no video/event-queue migration. M33 absolute conversion and M34/M39 runtime ownership inspected.
No new external emulator source or raw firmware execution. Historical references inside prototype
comments remain attributed, not independent re-verification.

Rejected shortcuts: SystemTime-derived monotonic counter, unchecked microsecond multiplication,
saturating unrepresentable durations, uninterruptible std::thread::sleep, global timing registry,
process-lifetime timeBeginPeriod, elapsed-time masquerading as POSIX CPU consumption, one-nanosecond
hardware resolution claims and generic incorrect errno 3. M25 records coarse wake latency and 60Hz
presentation policy; neither is guest hardware precision proof.

## Domains and checked conversion

Host monotonic is M25 Tick origin. Guest monotonic/uptime uses that same explicit session engine
epoch (not wall time, not paused guest CPU time). IDs 4/5/7/11 use microsecond quantization;
8/12 fast use millisecond quantization. Guest realtime 0/9 follows host Unix wall clock at each
query;10 fast uses millisecond quantization;13 whole seconds. Tests inject Fixed realtime.
These quantizations are modeled guest policy, not measured host clock_getres or wake precision.
CPU clock IDs 1/2/14/15 and POSIX clock() explicitly refuse ENOTSUP; never return wall elapsed
as CPU usage. Other unknown IDs EINVAL. No timezone/calendar, clock adjustment or suspend model.
GetProcessTime/Counter preserve prototype elapsed microseconds at 1MHz, explicitly NOT CPU time.
ReadTsc uses checked u128 scaling to a virtual 3.5GHz counter; frequency is prototype policy.
Untrapped native RDTSC is NOT claimed calibrated to that virtual counter; future native-clock
correspondence remains pressure. No instruction counting or real hardware frequency claim.

Timespec two LE i64 fields, 16 bytes; nanos must be 0..999999999, seconds nonnegative; overflow
refuses. Integer ns serialization normalizes, invalid guest input is rejected rather than silently
carried. Seconds/microseconds conversions checked; zero is immediately due but still a ticket.
Shared absolute conversion handles clock 0 host-wall delta or clock 4 engine ticks; past is due now.
Wall conversion snapshots the relation once; later wall jumps do not retime an admitted wait.
M33 now calls this common helper without changing mutex/condvar wake or reacquisition semantics.
Timeval is the inherited pair of 64-bit fields with microseconds in the second field; nonnull
legacy timezone output unsupported. No host struct padding or arbitrary guest references.

## Sleep and lifetime

Every calling guest thread waits on its own M25 completion ticket, with its M34 Thread identity.
No controller-thread sleep and no extra per-timer host thread. Thread lifecycle may remain Running;
GuestTiming snapshot overlays explicit sleeping(thread, sequence) identity. Central lock is released
before Ticket.wait. Only Fired at/after requested deadline completes successfully. Runtime deadline
caps long sleeps, classified Interrupted rather than a completed shorter sleep. Cancel/Stopped is
not success. Zero/past behavior and equal-deadline ordering inherit M25. No hard precision promise.
Arm occurs before execution; shutdown prevents admission, cancels tickets, wakes callers, then core
joins workers and releases storage. Abnormal worker exits also stop timing. No orphan/ticket leak.
Per-service 64 pending sleeps, first 64 detailed sleep/clock observations, cumulative counters; existing
4096 HLE-attempt policy and execution wall budget remain. Requested/elapsed/lateness are separate;
elapsed includes caller scheduling after publication. No raw payload logging.

Relative sleep/usleep/nanosleep aliases return 0 on completion. Runtime cancellation returns the
existing controlled StopRequested boundary; no guest signal/restart semantics invented. Optional
nanosleep remainder is preflighted before waiting, written only on interruption, untouched on success.
POSIX invalid/unsupported/fault returns-1 with caller-owned errno 22/45/14; SCE returns corresponding
0x8002xxxx without changing errno. Resource refusal 12; policy/host-copy failures stay structured.
Full output preflight prevents discovery of invalid tail after a successful partial write; no
transactional rollback claim for later OS copy failure. Unknown clocks never produce output.

## Tests

15 mechanism/conversion tests, 8 adapter tests, two synthetic native worker tests. Manual 1us/1ms
sleeps have no early completion; cancellation, deadline cap, concurrent waits, owner shutdown, query
aliases, CPU refusal, overflow, serialization, absolute pthread conversion and invalid outputs covered.
Synthetic native workers use existing bridge to sleep/return or stop while sleeping; hostFS/GS,
joined storage and ticket cleanup checked. The real observation below records wake latency separately from the requested duration.

## Exact registrations

Registered does not imply every operation/domain succeeds: POSIX clock() explicitly refuses
unsupported CPU accounting. Other clock IDs are scoped above. Runtime-confirmation recorded later.

| Name | NID | Exact library/module |
|---|---|---|
| sceKernelUsleep | 0xD637D72D15738AC7 | libkernel/libkernel |
| sceKernelSleep | 0xFD947E846EDA0C7C | libkernel/libkernel |
| sceKernelNanosleep | 0x42FB19C689AF507B | libkernel/libkernel |
| sleep | 0xD30BB7DE1BA735D1 | libkernel/libkernel |
| usleep | 0x41CB5E4706EC9D5D | libkernel/libkernel |
| nanosleep | 0xC92F14D931827B50 | libkernel/libkernel |
| _nanosleep | 0x361A6CA7176310A5 | libkernel/libkernel |
| sceKernelClockGettime | 0x4018BB1C22B4DE1C | libkernel/libkernel |
| clock_gettime | 0x94B313F6F240724D | libkernel/libkernel |
| clock_getres | 0xB26223EDEAB3644F | libkernel/libkernel |
| sceKernelGettimeofday | 0x7A37A471A35036AD | libkernel/libkernel |
| gettimeofday | 0x9FCF2FC770B99D6F | libkernel/libkernel |
| sceKernelGetProcessTime | 0xE09DAC5099AE1D94 | libkernel/libkernel |
| sceKernelGetProcessTimeCounter | 0x7E0C6731E4CD52D6 | libkernel/libkernel |
| sceKernelGetProcessTimeCounterFrequency | 0x04DA30C76979F3C1 | libkernel/libkernel |
| sceKernelReadTsc | 0xFF62115023BFFCF3 | libkernel/libkernel |
| sceKernelGetTscFrequency | 0xD63DD2DE7FED4D6E | libkernel/libkernel |
| clock | 0x4193FA23D659C691 | libc/libc |

## Real workload observation and hypothesis outcomes

One primary_real_elf run used the exact M40 USAGE command: 250 ms wall limit,
15000 ms outer containment. No second title and no depth-chasing repeat.
M39's unresolved sceKernelUsleep was replaced by one successful main-thread call:
argument 1000 microseconds, return 0; M25 ticket sequence 9, requested 1,000,000 ns,
deadline tick 20,481,200 ns, elapsed 12,582,000 ns, lateness 11,582,000 ns.
Thread(1) completed; no early completion. Host latency is empirically coarse in this run;
no timeBeginPeriod change or precise 1 ms wake claim. No guest clock/tick query occurred.

The next stop is UnresolvedFunction, powf, NID 0xD43D07D8A363B211, libc/libc,
ordinal 89. Exact-name corroboration: read-only PS5Rust source kernel/libc/nids.rs:73
(POWF_NID). This is a different math subsystem; no math implementation was started.
Captured boundary RIP 0x7ff66c426a26 is the Astero landing, NOT an executing guest PC.
Guest import return address is separately 0x1003e0096; RSP 0x200800808.
Captured GP arguments [0, 0xa, 0xffffffffffffffff, 0xa, 0, 0xfffffffffffffff5]
are not claimed to be powf's floating-point operands. No fault or guarded object.
Total report duration 29,017 us; supervised native interval 27,757 us; no redirection,
zero suspends/resumes. Initial RIP/RSP/FS remain M30's prepared context.

Eight workers stopped from condition waits and joined; synchronization reports eight waits,
zero wakes/timeouts and zero remaining waiters. Guest timing stopped with zero pending tickets,
one completion and zero interruptions. Audio has zero objects/tickets/buffers; no playback claim.
Host FS restored and GS preserved on main/worker exits. Native/runtime reservations zero,
release errors empty, containment Clean. Retained callbacks were not invoked.
Source SHA256 before/after: A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Local raw log: target/m40-real-1.log (ignored); source remains outside the repository.

Supported: M25 relative sleep migration advances this workload and cleanly preserves workers.
Refined: a 1 ms requested duration does not imply 1 ms observed completion on Windows.
Synthetic-only: other aliases, clock queries, cancellation and absolute conversions.
Unresolved: realtime epoch policy under clock adjustments, guest CPU accounting, virtual-counter
correspondence with native RDTSC, fine/fast hardware precision and full signal restart semantics.
These limitations are explicit; CPU accounting is refused rather than fabricated.

Recommend M41: coherent libc scalar-math migration driven by powf, with a second-eboot smoke
once that wave's primary run is stable. Cross-title validation is now useful; none ran in M40.
No new runtime ABI confirmation beyond the observed sleep call. The new timespec ABI records
legacy-correlated byte representation and synthetic validation, not full kernel clock semantics.
