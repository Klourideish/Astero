# M44 runtime observability

## Scope and evidence

M44 adds observation and terminal presentation to the existing native execution controller.
It changes no provider implementation, guest ABI, relocation, execution allowance, placement
policy or kernel-resource semantics. M45 resumes migration at the M43 frontiers.

Authoritative Astero evidence reviewed: `first_native_entry.md` (M31 execution lease,
actual-PC capture, paired SuspendThread/GetThreadContext/ResumeThread and containment),
`native_entry_closure.md` (M30 native bridge), `asynchronous_timing.md` and `kernel_timing.md` (M25/M40 clock integration),
`pthread_foundation.md`, `pthread_lifecycle.md`, and `ajm_c11_startup.md`.
Implementation review used kernel `execution/host/platform.rs`, `threading/thread/lifecycle.rs`,
`synchronization/owned.rs`, `timing/sleep.rs`; core `input/entry/closure.rs` and `workers.rs`;
HLE `dispatch/prepared.rs`; memory `allocation/heap.rs`; audio `output/service.rs` and
`codecs/ajm.rs`. No new PS5 semantics or catalogue migration was needed. External emulator
implementations were not consulted. Historical Windows error 487 remains unresolved/unreproduced.

## Ownership and dependencies

Core `input/entry/observability` owns the versioned snapshot and weak Observer. HLE owns
bounded atomic per-import metrics; kernel owns thread/wait counters and actual-PC sampling;
memory owns allocation peaks; audio owns port/AJM snapshots. CLI only renders, selects modes
and writes files. Existing core composition remains authoritative; loader-only APIs do not
instantiate an observer, worker, terminal or timing engine.

The observer never grants execution authority. `execute_first_entry_observed` retains the
same EntryReadyGuest factory/owned-thread path as `execute_first_entry`. No guest resource
can be obtained from a snapshot. The observer upgrades a Weak runtime only while holding its
own observation lock; it finishes all owner reads and drops that Arc before unlocking.
`detach` takes this same lock before worker/image teardown, preventing a frontend snapshot
from extending native mapping lifetime past the release audit. No runtime hot path takes the
observer lock. Owner locks are acquired separately, never nested for the dashboard.

No new workspace crate or internal dependency edge. Core adds serde/serde_json for one typed
versioned serializer and sha2 for the immutable source digest; CLI adds crossterm for safe
terminal size/alternate-buffer/cursor handling on Windows. Hand-written JSON, hashing and
additional unsafe Win32 terminal code were deliberately avoided. APIs checked against
[serde_json](https://docs.rs/serde_json/latest/serde_json/),
[sha2](https://docs.rs/sha2/latest/sha2/), and
[crossterm terminal documentation](https://docs.rs/crossterm/latest/crossterm/terminal/index.html).
Crossterm default event features are disabled; only its Windows support is selected.
No new Astero unsafe location; suspension remains inside the existing private native leaf.

## Snapshot / fingerprint contract

Schema version 1 is `observability::Snapshot`, shared by live UI and final JSON. It contains
artifact display name, SHA256 of retained source bytes, source size, entry/RSP/load bias,
elapsed/native interval, provider counters and exact keys, thread record/live/peak counts,
heap and allocation peaks, image/runtime byte counts, synchronization counters, audio state,
subsystem health, PC aggregates, stop context and a separate post-join teardown record.
Guest source digest is recomputed after the run; manual validation additionally hashes the
source file before and after execution. This is an integrity observation, not a security sandbox.

Addresses/NIDs use hex strings, with null for unavailable context. Library/module display
strings also retain exact byte hex to avoid merging distinct non-UTF8 identities. Providers
are deterministically sorted and merged by numeric NID plus raw library/module bytes; no
NID-only fallback. Counts include entered calls (including a blocked HLE wait), observed
returns, unknown registry lookups and refusals. A return may itself contain a guest error:
`returned_providers` is not proof of full semantic support. It never updates durable NID status.
Last ordinal/thread are published as one atomic pair. Completion counters publish before the
snapshot reads calls, avoiding a completed count greater than started calls under valid use.

Live snapshots span an observation window across independently synchronized owners; they are
not a globally atomic emulator checkpoint. Object/thread fields freeze immediately after the
main native stop, before teardown. `teardown` separately reports final joined workers, FS/GS,
remaining waiters/sleep tickets, reservations and release errors. Thus eight frozen waiters and
zero teardown waiters describe two different times. Peak thread records include starting
reservations; failed creation can leave a historical record, as in the existing table.

Runtime byte count includes image, foundation and live thread storage, explicitly excluding
import landings/object traps; image bytes are also separate. Stack/TLS counts are live storage
owners, not historical layouts. Heap live/peak bytes and allocation peak come from the allocator.
Wait identity covers pthread/C11 synchronization and guest sleep tickets; it does not claim
all possible Windows/HLE blocked states. Running-or-host includes native and host transition work.
Fault counts are captured exits, not an instrumentation of every Windows mapping attempt.

The observed runtime registry has **385 exact callable keys plus one data export**. The
unchanged NID index has **340 registered records and four observation-only records**: some
records explicitly group libc/libkernel aliases and are not a one-record-per-runtime-key census.
There are still six guest ABI records. JSON schema is a diagnostic contract, not a new guest ABI.

## Subsystem health

NotReached means no observed use; Present means a call has entered without an observed return;
Initialized means an owner reports initialization; Active means returned provider activity or
buffer submission, never code presence alone. Partial denotes a reached but limited mechanism
(AJM context/module bookkeeping without decode). Blocked denotes unknown/refused dispatch.
Failed and Unavailable are available for explicit owner failures/unsupported mechanisms; they
are not inferred from absent calls. Loader is Initialized only after native mapping/preparation.

Exact migrated pthread and C11 export sets disambiguate those families from libkernel/libc.
Module context routes AudioOut, AJM, VideoOut, AGC, sysmodules and UserService. Unclassified
providers have their own row rather than being attributed to Kernel. Filesystem/Shader remain
NotReached until a routed activity mechanism exists. This is explicit partial coverage, not
proof those guest concepts are absent. Initialization success is taken from owner state, not
merely an HLE return lane. AJM details include contexts, registered modules and instances;
Audio reports timed-null backend, ports, buffers and pending work. No decode/playback claim.

## PC sampling and bounds

Disabled by default. `--pc-sample-ms 5..1000` enables a per-execution-thread interval; 20 ms was
validated. The existing watchdog schedules the earlier of the next sample and execution deadline
through M25. It suspends only its owned thread, obtains the real integer/control context,
resumes exactly once, checks the outcomes, then aggregates the observation. No allocation,
logging, callback or mutex acquisition occurs while the target is suspended. Expired guest-range
PCs still redirect through the existing recovery landing; sampling never redirects before expiry.
Return stops the timing engine's current wait before joining the watchdog. Existing process
containment handles unrecoverable API failures; those are not reported as clean recovery.

All workers share a bounded sampler through the existing adapter Arc/OnceLock. There is no
second suspending controller and no new process-global map. Maximum 4096 retained samples and
65536 classified ranges; capacity overflow increments dropped count. Unique RIP/page sets and
distribution describe retained samples only. Guest pages are explicitly 4 KiB diagnostic buckets.
Source 0 is the prepared image identified by the fingerprint hash; landings are classified
separately. Assembly bridge PCs are RuntimeBridge; all remaining PCs are HostOrHle, not guessed
from a stack return address. Last PC, per-native-thread counts and guest RIP/page sets are retained.
Sampling cadence is requested, not guaranteed; suspension perturbs scheduling and is not an
instruction count, cycle count or coverage percentage. Final captured stop RIP and guest import
continuation remain separate; HLE boundary register arrays unavailable in M30/M31 are null.

Metrics allow at most 65536 import slots and 256-byte identity components; provider call limits
remain unchanged (up to 65536 per controlled run). Owner capacities remain authoritative. UI
redraw is at most 5 Hz and skips unchanged frames, with no per-call writes. JSON/provider/sample
aggregation occurs outside dispatch; existing deep call records remain bounded by the call budget.

## Terminal and logging

Interactive TTYs supporting an at-least-80x20 terminal use alternate screen, hidden cursor,
clipped rows and paired subsystem states. Resize is bounded to current dimensions; there is no
per-event scrolling. Drop restores the main screen/cursor and joins the renderer, then prints a
persistent full final report with stop and teardown at the end. It never waits for user input.
TERM=dumb, non-TTY, small/unsupported terminals and --no-dashboard use plain output. --trace
selects the existing full forensic scrolling report after execution. --verbose adds a bounded
provider summary and live returned-provider detail. --log-file writes the fingerprint plus all
retained forensic diagnostics without terminal spam. --report-json writes only schema-v1 JSON.
Output paths must be new: no existing source/evidence file is overwritten. Abrupt child-process
abort/kill cannot promise a final file; containment remains the parent diagnostic. Preparation
failure yields Failed, not a fabricated Stopped/clean teardown record.

## Real experiments

All runs used the existing primary_real_elf / named_title_elf corpus paths, 250 ms execution,
15000 ms containment and 65536 HLE calls. No provider was added between runs.

- First primary smoke found a diagnostic-only missing-row unwrap after native return. The
  child failed; no clean recovery was claimed, source hash unchanged. Fixed with explicit
  unknown-subsystem insertion, regression tests and failed-factory/panic status handling.
- TERM=dumb primary retry: 183.851 ms, 10998 calls, same direct-memory query boundary, clean.
- First interactive sampled primary: 184.375 ms, 45 balanced samples (one GuestImage, 44
  HostOrHle), same boundary, clean. This proved live redraw and alternate-buffer restoration.
- Final compact interactive primary: **182.344 ms overall / 180.660 ms native interval**,
  10998 calls, 52 unique used / 50 with observed returns, one unknown, zero refused/faults or
  supervisor redirections. Nine thread records/peak, eight pre-teardown waiters, eight joined
  workers. Heap peak 1055728 bytes, allocation peak90; image17432576 bytes, runtime excluding
  landings30224384 bytes. AJM one context/four modules/no instances; Audio initialized/null sink,
  no ports/submissions. **45/45 suspend/resume**, all45 HostOrHle this run; zero guest samples
  does not mean zero guest execution. Stop UnresolvedFunction, NID **0xA4EF7A4F0CCE9B91**,
  libkernel/libkernel, captured bridge RIP **0x7ff671812cf6**, RSP **0x200800778**, separate guest
  continuation **0x1003c8006**, native thread30300.
- Interactive second: **1.117 ms**, C11 activity and same semaphore boundary; too short to
  promise a Running frame at5Hz. Final explicit --no-dashboard --verbose second:
  **1.198 ms overall /0.749 ms native**, 42 calls,17 used/16 returned, one unknown, zero refusals;
  one main/no workers, three mutexes/one cond, no waits. NID **0xD7CF31E7B258A748**,
  libkernel/libkernel, bridge RIP **0x7ff671812cf6**, RSP **0x200800f38**, guest continuation
  **0x1014e5936**, native thread25048. Image29110272 bytes; runtime excluding landings41705472.

Successful runs: FS restored, GS preserved, all threads joined, zero native reservations,
release errors, remaining synchronization waiters or sleep tickets; audio/AJM stopped.
Primary source **A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A**;
second **3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327**. Both file
hashes and retained-source before/after digest are unchanged. Historical487 is not reclassified.

The 183.851 ms unsampled and184.375/182.344 ms sampled runs establish no measurable overhead
bound: they are a small uncontrolled sample. Sampling stays opt-in. Owner snapshot locks and
terminal scheduling may perturb execution; runtime frontiers/teardown remained the same.

## Validation and handoff

Focused tests cover atomic counters/pairs, exact provider identity, unknown-subsystem insertion,
Present/Active/Blocked transitions, typed JSON round-trip and unavailable fields, failed factory,
shared observer, bounded sampling/classification, actual infinite-loop sampling with paired
suspend/resume and FS/GS restoration, return/import/fault regressions, terminal fallback,
compact layout and no-overwrite output. Existing wait/worker/heap tests protect owner semantics.
Final post-review contained verification (same limits, primary --no-dashboard with sampling, second --trace) also reached the same frontiers: primary 178.179 ms /176.442 ms native,45 balanced HostOrHle samples; second 1.224 ms /0.794 ms native. Both retained 10998/42 calls, clean teardown and unchanged hashes. Shared boundary RIP 0x7ff73c8b3546 was a bridge landing; RSP/guest continuations stayed 0x200800778/0x1003c8006 and0x200800f38/0x1014e5936. Native thread IDs 20720/816 are run-local. Full commands/counts are recorded in `validation.md`. Raw target/m44-* files remain ignored.

M45 resumes the coherent kernel-resource migration: primary direct-memory-size query and
second sceKernelCreateSema. M44 neither implements those providers nor defines fake progress.

## Changed-file manifest

50 tracked-worktree paths changed/added (plus ignored active PROJECT_STATE.json):

- `Cargo.lock`
- `README.md`
- `USAGE.md`
- `crates/astero-audio/README.md`
- `crates/astero-audio/src/codecs/ajm.rs`
- `crates/astero-cli/Cargo.toml`
- `crates/astero-cli/README.md`
- `crates/astero-cli/src/entry/dashboard.rs`
- `crates/astero-cli/src/entry/first.rs`
- `crates/astero-cli/src/entry/mod.rs`
- `crates/astero-core/Cargo.toml`
- `crates/astero-core/README.md`
- `crates/astero-core/src/input/entry/closure.rs`
- `crates/astero-core/src/input/entry/mod.rs`
- `crates/astero-core/src/input/entry/observability/collect.rs`
- `crates/astero-core/src/input/entry/observability/mod.rs`
- `crates/astero-core/src/input/entry/observability/model.rs`
- `crates/astero-core/src/input/entry/workers.rs`
- `crates/astero-core/tests/observability.rs`
- `crates/astero-hle/README.md`
- `crates/astero-hle/src/calls/metrics.rs`
- `crates/astero-hle/src/calls/mod.rs`
- `crates/astero-hle/src/dispatch/prepared.rs`
- `crates/astero-hle/tests/metrics.rs`
- `crates/astero-kernel/README.md`
- `crates/astero-kernel/src/execution/host/mod.rs`
- `crates/astero-kernel/src/execution/host/platform.rs`
- `crates/astero-kernel/src/execution/host/sampling.rs`
- `crates/astero-kernel/src/execution/preparation/storage.rs`
- `crates/astero-kernel/src/synchronization/condvar/operations.rs`
- `crates/astero-kernel/src/synchronization/mutex/operations.rs`
- `crates/astero-kernel/src/synchronization/owned.rs`
- `crates/astero-kernel/src/synchronization/rwlock/operations.rs`
- `crates/astero-kernel/src/threading/thread/lifecycle.rs`
- `crates/astero-kernel/tests/bridge.rs`
- `crates/astero-memory/README.md`
- `crates/astero-memory/src/allocation/heap.rs`
- `knowledge/architecture/README.md`
- `knowledge/architecture/crate_boundaries.md`
- `knowledge/architecture/module_structure.json`
- `knowledge/architecture/module_structure.md`
- `knowledge/architecture/runtime_observability.md`
- `knowledge/architecture/validation.md`
- `knowledge/indexes/ABI_INDEX.md`
- `knowledge/indexes/DIAGNOSTIC_INDEX.md`
- `knowledge/indexes/IMPLEMENTATION_INDEX.md`
- `knowledge/indexes/MODULE_INDEX.md`
- `knowledge/indexes/SUBSYSTEM_INDEX.md`
- `knowledge/indexes/TEST_INDEX.md`
- `tools/index_links.json`
