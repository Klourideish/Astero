# M31 first controlled native entry

Direct x86-64 Windows guest execution remains primary. This capability is trusted-input
experimentation, not a security sandbox. Real execution is gated on synthetic supervision proof.

## Focused evidence

PS5Rust catalogue routes: 00_START_HERE/README.md; 02_CORE_EXECUTION/INDEX.md;
target_thread_operations.md; source/ps5-core__src__cpu__target_thread/INDEX.md and 002.md;
source/ps5-core__src__cpu__dispatcher/005.md and 008.md. Exact-target mailbox operations
explicitly do not establish arbitrary interruption. The dispatcher distinguishes actual
GetThreadContext RIP from an HLE return address, requires 16-byte CONTEXT alignment after
ERROR_NOACCESS experience, and pairs every successful suspension with resume. Its session
lifetime comments warn about retained dispatcher cycles. These are source evidence, not tests
newly run against PS5Rust. M29 ownership, M30 native_entry_closure and M25 asynchronous_timing
remain the Astero foundation. No external emulator or additional firmware semantics were needed.

Microsoft [SetThreadContext](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadcontext)
and [thread debugging](https://learn.microsoft.com/en-us/windows/win32/debug/thread-functions-for-debugging)
require context rights and suspension. [Suspension completion](https://devblogs.microsoft.com/oldnewthing/20150205-00/?p=44743)
explains why context capture follows SuspendThread. No arbitrary-thread suspension or allocator work
is performed while the owned thread is suspended.

## Controller and deadline

Core constructs the !Send EntryReadyGuest inside one named execution thread, consumes it through
private execute_first, joins and observes native resource release. There is no main-thread guest
transfer in the CLI. Kernel's lower-level checked native lease borrows live image/thread owners,
verifies the exact M30 supported context, return slot, TCB self pointer and executable coverage,
then uses the same M30 assembly. It does not accept an arbitrary host function pointer. The core
public execution factory must yield a genuine EntryReadyGuest; a blocked preparation cannot enter.

Kernel now depends downward on astero-timing; no loader timing dependency is introduced. A scoped
supervisor holds a handle to the exact still-live guest thread. M25 publishes a required 1..500 ms
deadline. On expiry the supervisor captures an aligned x64 CONTEXT (control/integer state), then
redirects only a PC inside the retained guest/probe/import-stub ranges to the restoration landing.
It captures actual RIP/GPRs before changing context. FS/stack restoration uses the existing frame on
the owning thread; the supervisor never races a Rust reference to that frame. There is no instruction
count claim. Observed expiry includes host scheduling latency; a requested 50 ms is not a promise
of exactly 50 ms CPU consumption.

Each successful suspension has one resume, including capture/set failures. No logging, allocation,
locks or consumer calls occur in that interval. Unexpected pre-existing suspension, OS failure or
resume imbalance aborts the worker process rather than claiming clean recovery. Host/HLE/bridge PCs
are resumed unchanged and retried using M25 deadlines, never redirected through a Rust call frame.
A hostile or stuck host call is therefore not guaranteed to recover in process. The CLI adds a
required 1..30 second outer containment deadline covering preparation too: timeout kills/waits the
worker process, classified TimeoutKilled, never successful native recovery. This is process-ending
last-resort containment, not TerminateThread. Child ownership cleanup also runs on errors.

The parent keeps stdout/stderr inherited, so bounded child reports cannot deadlock on an unread pipe.
An explicit internal worker mode owns the runtime; ordinary entry-readiness never executes artifacts.
The supervisor and timing workers join before the execution thread returns. A second M30 adapter
still refuses. An infinite Astero-owned loop with no calls/faults/returns is interrupted synthetically
before any real input is permitted; normal/HLE/fault/guard stops also cancel/join their watchdog.

## First-stop policy and evidence

Use unchanged primary_real_elf and ExperimentalEntryOwnedInit: transfer directly to its selected
entry, do not separately invoke DT_INIT. Permit only one _init_env and two atexit calls. Retain their
exact keys, six arguments and returned values. The next unknown provider, allowance exhaustion,
object trap, AV, illegal instruction, return or deadline stops. No callback/fini is executed and no
new provider is implemented to chase progress. Existing experimental environment/TCB assumptions
remain hypotheses until observed; reaching a call does not prove full semantics.

Reports distinguish actual captured fault/preemption PC from the import/return landing PC. The
separate return_address is a call-site continuation, never presented as the executing guest RIP.
Full GPR snapshots apply to faults/preemption; other argument lanes are boundary observations.
AV access kind comes from Windows ExceptionInformation, including read/write/execute where present.
Unknown exceptions retain Windows behavior. Parent crash/timeout is containment failure.

Native reports preserve source identity, initial context, elapsed duration, thread identity, stop
reason, startup calls, unresolved exact key/object identity, FS/GS restoration and balanced suspend
counts. Core observes image, stack, TLS, RX landings and object-trap release before reporting success.
No instruction-count instrumentation, secondary scheduler, JIT, interpreter or bulk NID migration.

## Experiment status

Synthetic infinite loop: 50 ms deadline, observed 55,683 microseconds, one suspend/resume,
actual loop RIP captured, host FS/GS restored, thread joined and adapter re-acquired. This proves
that an ordinary native loop can be interrupted on this host, not arbitrary hostile-code isolation.
This synthetic proof and full pre-entry checks passed before the single real attempt.

## First real result (one run only)

Selected primary_real_elf, source SHA256
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A before/after.
250 ms execution deadline, 15,000 ms additional whole-worker containment. The core execution duration
was 406 microseconds; supervisor armed interval 150 microseconds. Windows thread ID 25816.
Stop GuardedObject: continuable read AV 0xc0000005 at actual RIP 0x100332abf, RSP 0x200800f00,
fault address 0x210021000. That is the NOACCESS anchor for unresolved object symbol 7.
No asynchronous redirection was needed (zero suspends/resumes). Host FS restored, GS preserved,
execution thread joined, all five native/runtime reservation classes released with zero errors;
parent containment Clean, CLI exit 0. This establishes real native execution and regained control,
not a booted game or general stability.

Initial RIP 0x100000070, RSP 0x200800fb8, FS 0x210000000, RDI 0x200800fc0. One _init_env call
(ordinal 33) returned zero. Two atexit calls (ordinal 34): the host return-landing address was
rejected with 0xffffffff; guest fini 0x100524270 was accepted with zero. Exactly one callback was
retained, none executed. Other argument registers are preserved in the local raw report but have
no established meaning for a single-argument atexit call. Last import return address 0x10000009d
is explicitly not the fault PC. There is no initializer trace: passing these startup calls does
not by itself establish complete DT_INIT ordering. No early TLS fault proves a complete TCB.

A separate existing read-only ps5-identity command on the unchanged source correlates symbol 7
with raw name f7uOxY9mM1U#k#P, numeric 0x7FBB8EC58F663355, library ID 36 and module ID 15.
Metadata dynamic indexes 144/57 both name libkernel. The numeric NID is observed/unregistered;
provider contents/function-name meaning are deliberately not investigated or implemented here.
The report API retains associated raw relocation evidence from the unchanged load plan.

The real result refines M30's predicted two successful atexit registrations: one failed the guest
executable-range check. Do not fix that or fabricate symbol 7 contents in this milestone. M32
should investigate the observed object from catalogue/firmware evidence and account for the
return-landing registration mismatch before choosing another controlled run. No second real
execution occurred. Later report formatting/association retention changes were validated with
synthetic tests, not by replaying the real workload.


Captured Windows GPR order is RAX, RCX, RDX, RBX, RSP, RBP, RSI, RDI, R8..R15:

```text
210021000 c3480000 1005f9041 10077c720 200800f00 200800f20 1006106bf 100fc56b0
41c80000c1c80000 1005fae94 400000000 7f7fffff00000000 0 0 1 200800fc8
```

The faulting read had RAX equal to the guarded object's anchor. No instruction semantics or object
contents are inferred from that coincidence beyond the captured Windows read fault and trap identity.
The initial process-argument and synthetic native-boundary ABI records remain two distinct records;
this run does not certify full guest ABI/TLS semantics. NID inventory now has three unregistered
observations (including this object) and two existing startup registrations with scoped real-call evidence.
