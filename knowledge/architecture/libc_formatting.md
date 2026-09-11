# M37 libc formatting and guest varargs

M37 adapts the validated PS5Rust printf family into Astero's checked native runtime.
Libs owns guest formatting semantics; ABI owns the byte layout; kernel captures native
arguments and owns bounded output; core composes lifetimes. Libs now has a normal ABI
dependency (previously dev-only). No host va_list, printf, FILE or guest Rust references.

## Focused migration evidence

PS5Rust Knowledge Catalogue routes: 00_START_HERE/README.md, 05_KERNEL_HLE/INDEX.md;
05_KERNEL_HLE/source/ps5-libs__src__kernel__printf/INDEX.md and 001.md, 002.md, 003.md;
05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__exports/INDEX.md.
Read-only implementation counterparts: crates/ps5-libs/src/kernel/printf.rs
(VaList/readers, parser, integer/decimal render, exported signatures),
kernel/libc/exports.rs (printf family wrappers) and kernel/libc/nids.rs (exact identities).
These supply migration behavior, not Astero ownership architecture. No external emulator
or new firmware/decrypted catalogue claim is needed for this inherited formatting family.
Real validation uses the existing primary_real_elf corpus role; no machine path is tracked.

## ABI and arguments

Explicit little-endian 24-byte va_list: gp_offset u32 at0, fp_offset u32 at4,
overflow_arg_area u64 at8, reg_save_area u64 at16. GP slots [0,48) stride8;
FP slots [48,176) stride16; overflow slots stride8 for promoted integers/pointers/double.
Offsets and alignment are checked, accessed slots use GuestMemory. The reader advances
its local cursor; guest va_list storage is preserved, matching the prototype wrappers.
No claim about va_arg aggregates, x87 long double or every SysV ABI feature.
Direct calls skip named GP parameters and use captured XMM0-7 independently. The kernel
adds the checked first overflow address to CallFrame; libs reads beyond the six-slot
snapshot through checked guest memory. Synthetic frames without an address retain the
bounded snapshot fallback. The bridge assembly is unchanged.

## Implemented family and semantics

Eight exact libc/libc identities: vsnprintf, snprintf, vsprintf, sprintf, printf,
vprintf, fprintf, vfprintf. One parser/renderer and separate direct/list readers.
Integers d/i/u/o/x/X: hh,h,l,ll,j,z,t; p,c,s,%%; flags, width, precision and i32 star
promotion. Negative star precision is omitted; negative width means left alignment.
Raw string/literal bytes are retained, no UTF-8 requirement. Decimal f/F/e/E/g/G uses
captured double bits and bounded pure rendering, locale-independent. No host variadic call.
Wide strings/chars, long double, hex floats, positional syntax, percent-n and unknown
conversions refuse explicitly. They never print a fallback literal or report success.

snprintf returns the full required byte count excluding NUL, writes at most capacity-1,
and writes NUL when capacity>0. Zero capacity performs sizing and allows null destination.
sprintf preflights its actual rendered extent including NUL; callers remain responsible
for the C object capacity, while Astero verifies guest mapping validity. Complete format,
arguments and destination are checked before output writes. Writes use64KiB chunks;
subsequent host-copy failure is structured but cannot roll back earlier chunks. No
atomicity guarantee against concurrent guest mutation/unmapping is claimed.

Policy limits: format scan1MiB, field width/precision1MiB,16384 conversions, rendered
output64MiB, existing checked-access64MiB operation/512MiB aggregate,4096 call observations,
shared console64KiB and existing4096 runtime HLE attempts. No silent truncation for a
policy limit. snprintf truncation alone is successful by its ABI. Bounded rendering uses
an owned buffer; it is not a streaming unlimited formatter. Output records are reserved
under a short lock, then formatting occurs without it. Console append is call-atomic.
No mutable guest references are manufactured; existing native memory backend performs copies.

Core retains two initialized read-only stream tokens in the owned foundation page at
base+16/base+32. fprintf/vfprintf accept only these exact identities. This is an Astero
stream contract, not a complete guest FILE layout or public standard-stream accessor.
Unknown FILE pointers refuse; no filesystem or fabricated success. printf/vprintf and
puts share the same process-owned output. Guest snprintf destinations remain independent.

Rejected prototype shortcuts: default4096-byte silent string truncation, lossy UTF-8,
unknown-conversion fallback, permissive arbitrary FILE handling, percent-n mid-format
writes, incorrect negative precision/unsigned sign behavior and unrounded hex-float output.

## Validation and outcome

Focused tests cover required-length/truncation, local cursor preservation, GP/FP/overflow,
integer/string/float behavior, malformed inputs, output preflight, exact conversion budget,
owned streams and concurrent output. A synthetic native two-worker test exercises actual
XMM/GP plus seven overflow arguments through the existing bridge, joins and releases mappings.
Real workload result and final full validation are recorded below after execution.


## Exact registrations

| Export | NID | Real M37 status |
|---|---|---|
| vsnprintf | 0x43657E8AABE3802D | Three calls returned36/47/38 |
| snprintf | 0x78B743C3A974FDB5 | Synthetic native workers only |
| vsprintf | 0x8DBCFD23DBE4AA49 | Synthetic tested, registered |
| sprintf | 0xB5C562E528AF17B4 | Synthetic tested, registered |
| printf | 0x85CB90803E775313 | Three calls returned36/47/38 |
| vprintf | 0x18CA6FC4F156F76E | Synthetic tested, registered |
| fprintf | 0x7DF7F010B5CD5450 | Synthetic owned-stream tested, registered |
| vfprintf | 0xA43043718EAE2D20 | Synthetic owned-stream tested, registered |

## Real continuation — 2026-09-11

One primary_real_elf run, exact command in USAGE M37; exit0, Clean containment,
250ms wall limit/15000ms outer containment. Elapsed12,797us overall,11,743us supervised
execution. Initial RIP0x100000070/RSP0x200800fb8/FS0x210000000; entry owns DT_INIT.
M36 stopped before vsnprintf. M37 completed three vsnprintf and three printf operations,
all on main, with one conversion each, no truncation/refusal. va_list at0x200800830;
destination0x100fc7940, capacity512. Required and written lengths36,47,38 excluding NUL.
Format0x10061e2db is `SCREAM: couldn't create mutex %s\n`; direct printf format0x1005f3aba
is `%s`. These two small strings were confirmed by read-only PT_LOAD source translation,
not a second execution. Console appended121 formatting bytes after existing15-byte puts:

```text
RezVR Start !!!
SCREAM: couldn't create mutex synth
SCREAM: couldn't create mutex synthClientBatch
SCREAM: couldn't create mutex effects
```

These are guest-reported mutex failures, not proof of their root cause. No synchronization
behavior changed to suppress the messages. No real float/truncation/FILE behavior is claimed.
The diagnostic output is bounded process-owned bytes; terminal output remains escaped/debug text.

Final stop UnresolvedFunction, ordinal138, exact NID0x836B558852288471,
library libSceAudioOut2 / module libSceAudioOut. No function name or implementation is
asserted. Captured boundary RIP0x7ff6c6c99136 (Astero landing, not guest instruction PC),
RSP0x2008006b8; separate guest import return address0x1003db88e. Arguments:
[0x20080088c,0x200800888,0x200800884,0x1008028c8,0xa0,0xa0]. No fault/object trap.
The source audio_out.rs exact-NID lookup did not establish a name; catalogue routing only
00_START_HERE/catalogue_map.md and11_AUDIO_MEDIA/INDEX.md was consulted for follow-up.
No audio implementation or M38 work was started.

Main309 recorded provider calls,workers16 (eight mutex locks and eight interrupted cond waits).
Eight workers retained M34 storage; all exited/joined. Eight waits,zero signalled wakes/timeouts,
waiters0 afterstop. Heap peak7104 bytes,87 live allocations before owner teardown in one4MiB arena.
Checked work1,408,260 bytes,one large operation,zero refusals.42 callbacks retained,not invoked.
Supervisor thread29792: no redirection,zero suspends/resumes. Host FS restored,GS preserved;
all native/runtime reservations0,releaseerrors[]. Artifact SHA256 before and after:
`A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A`.
Raw local log target/m37-real-1.log remains ignored. No game-boot/audio success claim.

Supported: inherited explicit va_list path and owned console formatting for this workload;
synthetic direct mixed-register/extended-stack worker support. Refined: prototype parser/error
and raw-byte semantics; no silent fallback. Deferred: locale/wide/x87/hexfloat/percent-n,
real guest FILE accessors/layout, rare ABI aggregates, exact rounding beyond tested cases.
Recommended M38: audio startup foundation migration driven by the exact unresolved AudioOut
identity, with the newly visible mutex messages investigated as related evidence. Do not
assume audio rendering works or that those messages explain the unresolved stop.
