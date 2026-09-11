# M41 scalar math and cross-title executable validation

## Ownership and migration baseline

Scalar guest contracts live in the new libs/libc/math home: exports owns exact identity,
XMM/GP extraction and checked guest outputs; values owns bounded numerical adaptations.
There is no service state, allocator, timer or new dependency. Core adds the family to every
existing main/worker registry. Existing bounded StartupCall observations now retain the first
two XMM inputs and XMM0 output so float-returning calls are not falsely described by RAX alone.
The default 4096-call bound remains; retained scalar context adds 48 bytes per retained call.
The new module home prevents putting a growing family into a loose libc root file.

Catalogue routing (relative to PS5Rust Knowledge Catalogue):
- 00_START_HERE/README.md and catalogue_map.md.
- 05_KERNEL_HLE/INDEX.md.
- 05_KERNEL_HLE/source/ps5-libs__src__kernel__libc/INDEX.md and 004.md.
- 05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__exports/INDEX.md and 001.md.
- 05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__nids/INDEX.md and 006.md.
- 05_KERNEL_HLE/source/ps5-libs__src__kernel__libc__numeric_parse/INDEX.md and 001.md.

Compact catalogue partitions do not retain every registration closure. Read-only current
PS5Rust source supplies the detailed migration baseline:
crates/ps5-libs/src/kernel/libc/exports.rs:8-42 (frexp), 738-1135 (scalar registrations);
kernel/libc/nids.rs:62-182 and 307-318 (exact identities and historical corrections);
kernel/libc/numeric_parse.rs:139-198 (logb and nextafter);
kernel/libc.rs:3031-3104 (native lane, rounding, decomposition and classification tests).
No external emulator was consulted. Prototype comments about firmware and old workloads are
attributed legacy evidence, not new firmware disassembly or a new cross-title observation.

The reviewed prototype exports support 53 scalar identities. sqrt/sqrtf, cbrt/cbrtf,
floor/ceil/trunc, fabs/copysign, double fmin/fmax, rint and remainder/remquo were searched
but have no registration/constants in this prototype source. They are not invented from
function-name hashes. They remain unmigrated pending actual export evidence. Nearbyintf,
round/roundf, fminf/fmaxf, frexp/frexpf, ldexp/ldexpf, scalbn, sincos/sincosf,
classification, logb, nextafter and hypotf are included with the transcendental family.

## ABI, numerical behavior and limits

Inputs use the low 32 or 64 bits of captured XMM0/XMM1; return uses XMM0 at exactly that
precision. Upper return bytes are cleared. Float math uses f32 operations rather than silently
using double math. Classification returns an integer in RAX. Integer exponents use GP lane0,
not XMM1; pointer outputs use GP lane0 (and lane1 for sincos). Existing call-frame/native bridge
mechanisms are reused. A synthetic native worker probe proves both float and double dispatch,
return, host FS/GS restoration and teardown. No new ABI record is needed for this existing boundary.

Rust scalar methods implement powers, trig, exponentials, logs, rounding and hypot. fmod uses
truncating remainder (%), not IEEE remainder. NaN/infinity behavior follows those methods;
NaN payload identity and cross-host transcendental last-bit equality are not guaranteed.
The inherited handlers do not set guest errno for domain/range; Astero preserves that limited
policy and never writes host CRT errno into guest TLS. Negative log/domain inputs produce NaN,
zero log produces negative infinity and overflow can produce infinity. Full C math_errhandling,
floating exceptions and non-default fenv are not claimed. Initial MXCSR/x87 policy is unchanged;
providers do not change rounding mode. Nearbyintf models default ties-to-even only.

Adaptations deliberately reject silent prototype errors:
- modf/modff return signed zero for integral/infinite input, not inf-inf NaN or lost minus zero.
- ldexp/scalbn scale in bounded exact binary chunks; intermediate 2.powi cannot overflow a
  representable final answer. Extreme exponents return signed infinity/zero; zero/NaN/inf preserved.
- fminf/fmaxf explicitly choose the negative/positive zero tie rather than rely on Rust tie choice.
- sincos validates both complete outputs before either mutation; failed outputs stop structurally
  instead of returning an integer failure ignored by a floating-point caller.

Frexp normalizes subnormals with an exact binary scale and emits a four-byte signed exponent.
Modf writes four/eight bytes; sincos writes two four/eight-byte scalars. Byte encoding is explicit.
Every destination is charged and fully validated before writing. No arbitrary guest references,
host pointers or host CRT varargs. A later OS copy failure remains structured; rollback is not
claimed. Pure operations allocate no guest memory; scale takes at most eight chunks plus a factor.
Math handlers are stateless, shared-memory writes use the existing checked access policy.

## Tests and experiment gates

20 focused integration tests cover exact identity/capacity, normal family values, separate lanes,
precision, NaN/infinity/signed zero, domain/range and unchanged errno storage, rounding, integer
exponents, extreme scaling, subnormal decomposition, exact output widths, invalid/RO output,
preflight of both sincos outputs, classification and nextafter/logb. One native worker test runs
Astero-owned float and double probes. Real corpus bytes are not used by synthetic tests.

Primary: primary_real_elf, same prepared context and 250 ms wall / 15000 ms containment.
Second: named_title_elf, selected from existing local configuration as a distinct named executable
(27,908,888 bytes), without selecting by ease of imports or expected outcome. Acquisition capacity
must cover its actual size; execution time is not enlarged. Phase B waits for primary powf progress,
clean source/teardown and preflight validation. No second-title-specific fix or default CLI change.

## Exact registered identities

Only actual real calls may later be marked runtime-confirmed. Known identity is not firmware
numerical conformance. All keys below use libc/libc.

| Name | NID | Precision |
|---|---|---|
| powf | 0xD43D07D8A363B211 | F32 |
| __powisf2 | 0x122324810B0E7D4D | F32 |
| pow | 0xF4B0A3A56C90E597 | F64 |
| frexp | 0x900FD3762382B1A6 | F64 |
| frexpf | 0x69A0CC186917171A | F32 |
| ldexp | 0x26BC0520CCCA36BD | F64 |
| ldexpf | 0x927D32898784C600 | F32 |
| scalbn | 0x28628179572A2637 | F64 |
| sincos | 0x8CC07B105CAEDF46 | F64 |
| sincosf | 0xA73B55E00175F222 | F32 |
| modf | 0xD163070DBE43B7DE | F64 |
| modff | 0xDFE50F33FF44EB16 | F32 |
| nearbyintf | 0x73EE2BFD3FED1087 | F32 |
| fminf | 0xB9545C336C8574FE | F32 |
| fmaxf | 0x2F2C760F350BECB7 | F32 |
| hypotf | 0x8B3DAC8401852317 | F32 |
| logb | 0xA302AE7A0654E1EC | F64 |
| nextafter | 0x87E27AD114657E79 | F64 |
| nextafterf | 0xDE6DABA3E0E2F829 | F32 |
| exp | 0x35569D7E7CD08474 | F64 |
| expf | 0xF33B2ED385CDB19E | F32 |
| exp2 | 0x76769E1976E33FA1 | F64 |
| exp2f | 0xC2E010B7F8FEA78A | F32 |
| log | 0xAED57BFE3582E988 | F64 |
| logf | 0x4505CB6DD4F695CE | F32 |
| log2 | 0x6390E1B832869674 | F64 |
| log2f | 0x86C8BD76BCC74769 | F32 |
| log10 | 0x5AE31B3C128DD535 | F64 |
| log10f | 0x961A5DE9693A71CB | F32 |
| sin | 0x1FCC9AD87D348DB2 | F64 |
| sinf | 0x438AD12F7E0211E1 | F32 |
| cos | 0xD96137053615C0A3 | F64 |
| cosf | 0xFCFE8534CCE4D8A7 | F32 |
| tan | 0x4FBBB236A3FBBD00 | F64 |
| tanf | 0x644E9134BF9E2DB9 | F32 |
| asin | 0xECBCB9DB368BE384 | F64 |
| asinf | 0x1995A317F6081459 | F32 |
| acos | 0x24172062E5BC94F5 | F64 |
| acosf | 0x408FF1D122FC8E1C | F32 |
| atan | 0x39799AB8B750F246 | F64 |
| atanf | 0xC1E0EE83C403FE51 | F32 |
| atan2 | 0x1D46D998E9D3FC38 | F64 |
| atan2f | 0x107FF1EF5DC0F7D7 | F32 |
| round | 0x9E56A88CBF610ED0 | F64 |
| roundf | 0x0C31C6D5AEBEDEAD | F32 |
| fmod | 0xA4AC2C96C3149929 | F64 |
| fmodf | 0xF3C56FFC0CC7563F | F32 |
| __isnan | 0x19FC40A7D5F28AAB | F64 |
| __isnanf | 0x940F786604FEBCC3 | F32 |
| __isfinite | 0x7612B5E822B08508 | F64 |
| __isfinitef | 0x43CA6F2629945A2B | F32 |
| __isinf | 0x574DA816FFBF2730 | F64 |
| __isinff | 0xAC333201FD4986E8 | F32 |

## Evidence-driven call-budget refinement

The first primary run passed powf(10.0f, 0.4f), returned about 2.5118864f, then performed
3737 successful sincosf calls before the shared 4096-call allowance stopped execution.
This was not an unresolved function or a numerical failure. Teardown/source integrity were clean.
It justifies an explicit --max-hle-calls option, default 4096, accepted range 1..=65536.
The EntryReadyGuest owner accepts the budget before entry; main/worker admission shares one
atomic counter and bounded retained records. Worker records are reserved before execution;
changes after admission refuse. Wall/containment limits remain independent and unchanged.
No unbounded call loop or math exemption. Existing default/concurrent-budget tests remain;
new upper/zero/after-admission checks and CLI duplicate/range checks cover the option.
The continuation uses 65536 calls with the same 250 ms / 15000 ms limits.

## Primary continuation result

Preflight formatting, all-target check/build, warnings-denied Clippy, workspace Rust tests and
policy/index checks passed before execution, and again after adding the explicit call budget.
Primary continuation: 65536 HLE calls, 250 ms wall, 15000 ms containment; source selected by role.
Two primary runs total: one established the call-budget pressure, one captured the final boundary.
The final run retained 5445 total main/worker provider records. Math: powf once, sincosf 5086 times.
Powf inputs are 0x41200000 (10.0f) and 0x3ecccccd (approximately 0.4f); output 0x4020c2bf
(2.5118863582611084f). The call uses XMM0/XMM1 and returns through XMM0, not RAX.
Sincosf writes two checked four-byte outputs; no claim of exhaustive trigonometric conformance.
All other migrated math identities remain unexercised by this run.

Final stop: UnresolvedFunction, __cxa_guard_acquire, NID 0xDC63E98D0740313C, libc/libc,
ordinal 66. Read-only name corroboration: PS5Rust kernel/libc/nids.rs:354. No guard implementation
was added; C++ static initialization is outside scalar math.
Boundary RIP 0x7ff62b38b6a6 is an Astero import landing; RSP 0x2007fcae8.
Separate guest continuation/return address 0x1003e4337. No fault or guarded data trap.
GP arguments [0x100fe0d40,0x2007fcd50,0x2007fcdf0,0x2007fe140,0x18,0xc].
Duration 96841 us overall, 95381 us native supervised interval. No asynchronous redirection,
zero suspend/resume operations. Eight workers joined from condition waits; host FS restored,
GS preserved; zero native/runtime reservations, empty release errors, containment Clean.
M32 startup, M33 sync, M34 workers, UserService, AudioOut and M40 sleep worked unchanged.
Retained callbacks were not invoked; audio submitted no buffers. Source SHA before/after:
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.

## Second executable smoke and limits of the result

Selected named_title_elf is 27,908,888 bytes. The role names a distinct configured eboot and
its plan declares PS5Player_IL2CPP plus Il2CppUserAssemblies.prx and PS5Util.prx dependencies.
It was not selected for easy imports. Acquisition budget 32 MiB covers its measured size;
source read count and semantic budgets remain bounded. Wall/containment stay 250 ms / 15000 ms.
One first-entry command was attempted for this second executable; no alternate title or bias retry.

Acquisition and planning succeeded. The offline plan is Blocked with missing explicit providers
and preserved program semantics, as the primary's offline plan also is before runtime closure.
Owned-byte staging succeeds: five segments, 29,101,924 mapped bytes, 27,849,252 copied bytes,
1,252,672 zero-fill bytes, 27,865 applied structural relocations, 1,704 pending relocations.
A separate offline stage-image diagnostic confirms zero active byte mappings after release.

The contained preparation attempt stopped at native allocation:
Os { operation: reserve_exact_or_collision, address: 4294967296, size: 29134848, code: 487 }.
The existing Windows backend rejects failed VirtualAlloc exact reservation; it never replaces
an occupied range. Requested envelope is 0x100000000..0x101bc9000.
The actual competing allocation/cause was not sampled, so this is an exact-placement refusal,
not proof that a particular host allocator, image or title-specific assumption caused it.
No bias workaround or host VM modification was made to force EntryReady.

Native realization did not complete; EntryReady was not issued; entry RIP was never transferred.
There are zero observed second-title HLE/startup/math calls or guest threads. Stack/TLS, startup
order, timing, platform services and exception recovery for this title remain untested.
No final guest RIP/RSP, NID or execution duration exists. Raw ELF entry is 0x70; planned biased
entry is 0x100000070 only. Child exit 1; parent reports WorkerFailure(Some(1)), not clean guest
recovery. The parent reaped the child. No successful reservation was published by the failed
VirtualAlloc branch; process termination releases child-owned host allocations. The first-entry
failure path does not emit a numeric runtime reservation snapshot, so none is fabricated.
Second SHA before/after: 3354D079F8E62BF47B2FBB057D828FE6C2124BC444A1C74AC9AA6D2438FB7327.

## Cross-title comparison

| Observation | Primary | named_title_elf |
|---|---:|---:|
| File bytes | 9,225,340 | 27,908,888 |
| Loadable segments | 5 | 5 |
| External references | 822 | 579 |
| Dependencies | 38 | 36 |
| Encoded NIDs | 822 | 581 |
| Relocations | 30,898 | 29,569 |
| Pending after byte staging | 1,107 | 1,704 |
| Byte-staging mapped bytes | 17,424,635 | 29,101,924 |
| EntryReady | yes | no; native reservation refused |
| Guest workers | 8 | none started |
| Timing/services/math | exercised as recorded above | not reached |
| Final boundary | C++ guard import | Windows exact VM placement |

Unchanged acquisition, SCE metadata parsing, dependency/reference evidence and byte staging work
for both layouts. Native runtime generalization is NOT established by the blocked second attempt.
No title ID/hash/path/RIP conditional was introduced. The configured fixed 4 GiB image bias is a
shared placement policy whose robustness is now an explicit cross-title pressure; the observed
failure does not yet prove a title-dependent code bug. A different relocation volume and module
relationships are preserved rather than assumed equivalent to the primary.

## Hypotheses and next-wave rationale

Supported: existing FP lanes and native bridge support real powf/sincosf; stateless scalar adapters
preserve worker teardown. The 4096-call default was too small for this ordinary math initialization;
explicit bounded allowance enables progress without changing wall supervision.
Refined: prototype modf/scalbn edge shortcuts are corrected and tested, not blindly inherited.
Unresolved: full fenv/errno fidelity and cross-host numerical last bits; exact native-reservation
failure cause; second-title runtime/TLS/HLE behavior, which was never reached.
Recommend M42 first investigate and harden generic native address-placement/preparation across
both layouts, then resume the coherent C++ static-initialization cluster identified by the primary.
Do not mistake the second attempt for a successful game boot or a clean native recovery result.
No M42 implementation, commit or push occurred in M41.

## Completion validation

Formatting, all-target check/build and warnings-denied Clippy pass. Full suite:594 executable
tests and23 doctests, no failures/ignored;53 Python tests pass. Index freshness, policy/state/
structure and both whitespace checks pass;17 index files regenerate byte-identically.
Indexes:1497 implementations,138 subsystems,586 modules,571 sources,672 test records,
43 diagnostics,321 NIDs(318registered/3observation-only),5 ABI records;3833 total.
The worker composition file remains around1200 lines, predominantly existing lifecycle tests;
reviewed additions only wire stateless providers, bound calls and extend synthetic evidence.
No new execution bridge, runtime singleton or crate dependency was introduced.
M41 remains ready_for_cleanup pending user approval; source/remote HEAD remains post-M40.
