# M25: foundational asynchronous timing

One timing owner; explicit runtime lifetime. Host monotonic deadlines are not guest clock semantics.

## Evidence and lessons

Focused PS5Rust catalogue routes (relative to the local AGENTS.md catalogue root):
- `00_START_HERE/README.md`, `catalogue_map.md`, `05_KERNEL_HLE/INDEX.md`.
- `05_KERNEL_HLE/clocks_deadlines.md` and `source/ps5-core__src__timing/INDEX.md`;
  focused parts `001.md`/`002.md` (typed time, clock/manual wake model), `004.md` (worker and Windows
  resolution comment), `005.md` (schedule/cancel/generation/shutdown), `006.md` (worker loop),
  `007.md` (process runtime singleton and integration).
- `05_KERNEL_HLE/owned_address_waits.md`: a control wake is not guest predicate success; queue
  lifetime matters. Its indefinite helper does not establish timed-wait semantics.
- `05_KERNEL_HLE/source/ps5-libs__src__kernel__timer/INDEX.md`, focused `003.md` registration/reset
  routes, `004.md` call-site timing diagnostics and `005.md` sleep-duration adapter.
- `10_VIDEOOUT/INDEX.md`, `flips_and_vblank.md`, source video_out/mod `INDEX.md` and `009.md`:
  driver epochs, coalesced missed vblanks, and a documented 60 Hz presenter ceiling.
- `16_DIAGNOSTICS/INDEX.md` routes and timer call-site diagnostics above.
- `15_FIRMWARE_KNOWLEDGE/contracts/clock_gettime.md`: documented syscall veneer does NOT establish
  supported clock IDs or process CPU-time behavior. The datetime/process source routes were checked;
  no focused process-clock implementation proof was found. M25 does not infer CPU time from elapsed
  host time, or implement clock_gettime from the wrapper alone.

These are source/documentary observations, not newly executed prototype workload evidence. The
user's history says timing was introduced late; the catalogue does not independently establish a
complete chronology or every causal bug. Concrete debt visible in the records: global runtime and
reset generations, remaining direct thread sleeps in timer adapters, owner/control-wake coupling,
and added call-site diagnostics because aggregate NID counts could not distinguish one stuck wait
from many progressing waits. Audio comments report a 5.33 ms period waking around 15.8 ms on Windows.
The 60 Hz record is a presenter policy/capability statement, not proof of physical presentation or
new Astero frame-loop performance. Preserve request/publication/presentation distinctions.

Preserved lessons: absolute monotonic deadlines, explicit wake notification on manual advances,
equal-deadline order, cancellation truth, teardown ownership, lateness and pending-work diagnostics.
Deliberately not copied: process-global OnceLock runtime, wrapping event IDs, saturating deadline
arithmetic, callbacks on the worker, process-lifetime timeBeginPeriod without restoration, guest
address registries, periodic coalescing policy or any HLE/NID/ABI implementation. No external emulator
implementation was needed, and no raw firmware or guest program was executed.

## Ownership

The dependency-free `astero-timing` crate owns `time/`, `clock/`, `scheduler/`, `diagnostics/`.
An existing kernel/timing scaffold was considered: generic scheduling there would couple future
video/GPU/audio mechanisms to a consumer kernel service. It remains the future guest timing-policy
adapter home. Timing must never depend on core, kernel, loader, debug, frontends or consumers.
Only core gains a current normal dependency. Future concrete service adapters may add deliberate
downward edges; no speculative dependency edges or global event bus were added.

`Session::with_timing(inputs, TimingEngine)` transfers explicit ownership. Ordinary `new` and
`with_inputs` remain worker-free, as do all loader/offline input APIs. `Session::timing()` returns
an optional weak producer. Session stop shuts down and joins; dropping the session also joins.
No timing fields are inserted into immutable loader reports or GUI snapshots. Timing snapshots are
separate observations; they do not claim cross-service snapshot epochs or guest-loaded state.
Host time continues through any future emulator pause; M25 implements no guest pause/CPU clock.
A future guest domain can be layered deliberately without changing the host clock's meaning.

## Time and clocks

`Tick`, `Span`, `Deadline` use nanoseconds, with u64 bounds and checked conversions/addition/difference.
Zero duration is due immediately; equality is due; past deadlines are eligible with measured
lateness. Overflow/refusal is explicit, not a saturated future deadline. Underflow is BeforeOrigin.
Tick/deadline values are relative to the receiving engine's origin; mixing engine epochs is a
caller error, not a supported conversion. Tickets additionally enforce originating engine identity.

Private `Source` separates real `std::time::Instant` elapsed time from manual atomic ticks; no wall
clock drives scheduling. Real conversion overflow fails/stops explicitly. `ManualClock` is an
independently clonable weak control handle from `TimingEngine::manual`; advance is forward-only,
checked, and notifies under the same mutex as the worker's predicate/wait transition. It never
sleeps or executes callbacks; the actual asynchronous worker publishes due completions. Tests wait
for completion acknowledgement, not a race-prone assumption that advance returns after publication.
No arbitrary clock/plugin callbacks can block or panic while a scheduler lock is held.

## Scheduling and expiry

One named joined worker uses Mutex/Condvar. A pre-reserved vector is maintained in
(deadline, registration sequence) order. Registration sequence linearizes under the queue lock;
concurrent producer arrival order is not predetermined. Already fired work cannot be retroactively
reordered by a new past deadline. Equal queued deadlines publish in sequence order, with a dispatch
ordinal; consumer thread execution order is outside this guarantee.

Every insertion/cancellation notifies the worker. Empty/manual queues wait indefinitely; real queues
use their nearest remaining deadline. Very distant host waits are capped at one hour per wait to
avoid platform timeout conversion limits, then rechecked. This is not a spin/polling scheduler.
Queue operations are O(n); capacity bounds their work. A different indexed structure may be justified
by measured larger workloads, not introduced as speculative complexity now.

Expiry uses a one-shot completion ticket, not a callback. Outcomes are Fired(Expiry), Cancelled or
Stopped(reason). Clones observe the same terminal value; multiple reads are observations, not repeat
delivery. Publication/notification occurs under the queue lock and a private short-lived ticket lock
for a clear linearization point. No consumer code, channel backpressure or callback runs there.
Wait releases the ticket lock; consumers cannot borrow internal locks. An unread/slow ticket cannot
stall later expiry. A consumer can wait/poll then adapt its own guest predicate or work queue.
A wake alone must never become guest synchronization success.

Cancellation before worker publication wins even if the deadline has become due; after publication
it returns AlreadyFired. Repeated cancellation returns AlreadyCancelled; shutdown returns Stopped.
Foreign tickets fail. Sequence IDs never wrap/reuse: exhaustion refuses scheduling. Terminal tickets
are not retained in a central registry; clients retain their completion cells as long as needed.

## Shutdown and failure

Explicit shutdown closes registration, releases all pending tickets as Stopped, notifies the worker
and joins it. Concurrent/repeated shutdowns serialize joining: return means worker exit. There are
no callbacks to self-join or wait for. Worker panic is caught, closes pending work and is reported;
clock overflow similarly stops rather than inventing time. Unknown diagnostic timestamps are None.
Drop performs the same stop/join. Weak clients/manual handles do not keep a worker alive; after owner
drop operations report Closed, while retained tickets still expose terminal truth. No detach/reset
API silently starts a new generation. A fresh engine has a fresh ownership identity.

## Resources and diagnostics

Config requires `max_pending` and `max_snapshot_entries`; there are no hidden capacity defaults.
Queue storage is fallibly reserved before spawning. At capacity registration fails with maximum;
zero capacity intentionally refuses all events. Cancellation frees capacity immediately. Snapshots
retain min(request, configured maximum, pending count) and explicitly report omitted entries.
Completion cells are bounded by accepted pending registrations; consumer-retained terminal tickets
are caller-owned memory, not an ever-growing engine queue. Standard Arc allocation still follows
Rust's process allocator failure behavior; queue/snapshot reservation failures are structured.

Generic UTF-8 labels are at most 48 bytes; overlength is refused, never truncated. Snapshots contain
clock mode/current tick, stopped reason, exact pending count/next deadline, ordered retained events,
sequence/label/scheduled-at/deadline, scheduled/fired/cancelled/released/rejected/wait totals, peak
pending, worker-wait state, maximum lateness and the last terminal event. Saturating diagnostic
counters expose a saturation flag. Last-event storage is bounded, not an unlimited tracing log.
Scheduling/expiry counters and retained event data allow future tools to identify current waits
without introducing subsystem-specific event enums or noisy normal logging.

## Windows and precision

M25 uses the default host timer policy and changes no system/process timer resolution, so there is
no host timer state to restore. No Windows unsafe FFI or convenience dependency was added. Nanosecond
representation/monotonic precision does not promise nanosecond wake accuracy. Every wake rechecks the
clock; actual lateness is reported. One local test observed 15 ms -> 19.36 ms, 5.33 ms -> 18.57 ms,
16.67 ms -> 32.03 ms. These noisy host observations support the catalogue warning, not a fixed
resolution model or performance guarantee. Timing tests require never-early publication and a
5-second liveness guard, not submillisecond precision. Most tests advance manual time without sleep.

Before latency-sensitive audio/VBlank integration, investigate an explicit scoped host-resolution
backend with safe ownership/restoration and measured benefit. The current unsafe boundary is not
weakened by hiding FFI in an unrelated crate. Default-policy scheduling is fully asynchronous now;
precision policy, guest clocks and periodic consumer semantics remain separate decisions.

## Validation and future use

Fourteen timing tests cover arithmetic, exact/past/equal/different deadlines, capacity/labels,
manual overflow, cancellation/expiry races, concurrent producers/cancellers, shutdown races, slow
consumers, bounded diagnostics, owner drop and real earlier-deadline wakeups. Two core integration
tests verify explicit session ownership and stop/drop. A policy test protects timing's leaf status
and loader independence. `cargo run -p astero-timing --example deadlines` demonstrates deterministic
ordering, cancellation and joined shutdown; USAGE.md records the exact output. Full validation and
counts are recorded in validation.md, not inferred from generated indexes.

Future kernel waits, sleep/semaphore/condvar/event adapters, video frame scheduling, GPU fence/wait
instrumentation, audio pacing, timed work and diagnostic timeouts can receive Scheduler handles.
They must implement their own predicate/periodic/guest policy after ticket completion. No guest
clock HLE, VBlank, GPU/audio loop, NID/ABI registration, execution or loader timing coupling exists.
Recommended M26: scoped host wake-precision experiments plus one bounded consumer wait adapter,
validated against this engine, before broad PS5 timer HLE or frame/audio scheduling.
