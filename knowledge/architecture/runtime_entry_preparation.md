# M29: owned native entry preparation, without entry

Native execution on x86-64 Windows remains primary. No interpreter/JIT or normal instruction-fetch
engine is introduced. PreparedBlocked is useful runtime preparation, not an EntryReadyGuest or a
permission to jump. **No guest code, constructor, import landing or entry trampoline is invoked.**

## Ownership and dependency direction

Core input/entry owns PreparedGuest: the M28 NativeBackedGuestImage and immutable source-bound plan,
kernel ThreadStorage, initial context, bounded HLE PreparedRegistry, RecoveryBoundary, optional
TimingEngine, and readiness diagnostics. It consumes the image; refusal drops acquired ownership.
No Session lifecycle is fabricated, no thread is created, and timing is opt-in only. Releasing stops
owned timing before storage; repeated release is safe. The native owner remains non-Send/non-Sync.
Future active execution needs a lease that prevents release until host control/FS are restored.

Kernel execution/preparation owns checked thread layout/storage and the recovery/import-encoding
contract. Memory owns OS mappings and protection. ABI layouts/entry owns byte layout and context
values, not assembly or host handles. HLE dispatch/prepared owns exact-key host-model registration.
Loader load_plan/bootstrap reuses existing observed PH and dynamic blocker evidence, supplies
source-bound PT_TLS templates, raw init/fini metadata, procparam and RELRO ranges. Core does not parse ELF.

Five already-allowed edges become real: core -> kernel, core -> HLE; kernel -> memory and ABI;
HLE -> ABI. No new crate/external dependency or allowlist expansion. Loader and timing remain
independent foundations. No global registry, VEH, thread-local singleton or new unsafe allowance.
The existing private Windows VM leaf adds write-removal/query support only.

## Focused evidence and limits

PS5Rust Knowledge Catalogue (all relative to its root), routed through 00_START_HERE/README.md,
catalogue_map.md and subsystem INDEX.md files:

- 02_CORE_EXECUTION/native_boundary.md: module lease, dispatcher/stop scopes, FS restore and import layout.
- 02_CORE_EXECUTION/assembly/001.md: host Windows saves, XMM6-XMM15, guest stack/return slot,
  RDI/RSI and explicit zeroing. Source evidence, not a rerun of prototype tests.
- 02_CORE_EXECUTION/assembly/003.md: Windows VEH arrives on guest stack; switch to host stack before
  compiled Rust to avoid a secondary __chkstk/STATUS_STACK_OVERFLOW fault.
- 02_CORE_EXECUTION/source/ps5-core__src__cpu__native_exec/007.md: variant-II TLS template before
  thread pointer, observed negative-offset failure, 0x200 TCB slack explicitly lacking a full reference.
- Same owner /010.md: fault routing, process-global legacy VEH and FS reassertion limits.
- 02_CORE_EXECUTION/source/ps5-core__src__cpu__dispatcher/001.md: 8-MiB main stack, 2-MiB worker stack.
- Same owner /014.md: guarded stack and explicit TLS release; /015.md: execute_process_entry,
  32-byte RDI EntryParams and RSI exit-handler contract; /016.md: parameter size/stack validation.
- 04_MEMORY/address_space_contract.md and source/ps5-core__src__memory__address_space/002.md:
  legacy addresses are implementation choices, not mandatory universal mappings.
- 03_LOADER_LINKER/provider_authority.md: function, data, direct-artifact and unresolved distinctions;
  stand-ins/scratch are not established semantics.
- 05_KERNEL_HLE/source/ps5-hle__src__thread/001.md: explicit thread stack/context/TLS ownership.
- 07_NIDS/by_sysmodule/SOURCE_OWNER_kernel/005.md: 0x6f3404c72d7cf592, _init_env; legacy registration
  was a zero-return **stub**, not a proven environment implementation.
- 07_NIDS/by_sysmodule/libSceLibcInternal.sprx/090.md: 0xf06d8b07e037af38, atexit; approved firmware
  numeric-name join and legacy callback collection, no Astero registration.
- 15_FIRMWARE_KNOWLEDGE/document_guides/403_THREADING_TLS.md.md: PT_TLS does not prove a complete
  TCB/destructor policy. /contracts/sceKernelGetProcParam.md: pointer-return ABI and procparam lead,
  distinguished from firmware-proven export identity. No guest API implemented from this alone.

Decrypted ELF catalogue: 00_START_HERE/{README,catalogue_map,key_findings}.md,
04_GLUE/PS5Util/{INDEX,0001}.md: utility callable glue differs from a process entry; DT_INIT can carry
constructors even without an init array. This catalogue is a different snapshot, not the selected
executable's execution trace. No external emulator implementation was read; comparative comments in
PS5Rust were retained as legacy leads, not independently corroborated claims.

## Workload and observed first calls

Selected LOCAL_TEST_CORPUS role primary_real_elf (smallest selected executable, PPSA09165 eboot).
PS5Util has raw entry zero and padding at VA zero; it was not forced into process entry.
The selected executable has raw entry 0x70, PT_TLS index 5 with filesz=memsz=0, and a 96-byte
PT_SCE_PROCPARAM at VA 0x77c828. A bounded, read-only Capstone experiment over 96 entry bytes found:

- 0x7a: mov r14d,[rdi]; 0x80: lea r15,[rdi+8], corroborating the parameter-record hypothesis.
- 0x84: call 0x521040; its PLT slot 0x77acc8 refers to symbol 1, bzQExy189ZI#l#l (_init_env).
- 0x8c and 0x98: call 0x521050; slot 0x77acd0 refers to symbol 2, 8G2LB+A3rzg#l#l (atexit).
- 0x9d: call 0x10 (DT_INIT), then main-shaped call 0x30ac60. This is bounded linear evidence,
  not a complete CFG or execution. Later symbol 3 XKRegsFpEpk remains unnamed here.

This experiment is not a parser/decoder in product code. No name/slot/artifact-specific branch is
hardcoded into runtime. Both earliest imports are catalogue-known and Astero-unimplemented; no guessed
_init_env success or fake callback registration was added. At least two immediate references are known;
all 1107 pending relocations conservatively remain entry blockers because startup reachability/closure
is not yet proven. There is no assertion that all 822 external references execute at startup.

## Stack and context

Caller explicitly supplies stack-base, stack-bytes, tls-base and max-runtime-bytes. The base is the
low guard page, aligned to native allocation granularity. Usable stack begins one host page above it.
The entire guard page is committed NOACCESS (persistent guard, not Windows one-shot PAGE_GUARD).
Stack is RW/NX. No fallback address; collisions refuse and roll back, including TLS collision after
successful stack reservation. Limits count stack + guard + page-rounded TLS/TCB storage. All additions,
rounding and allocations are checked. Oversize requests fail before large allocation. Zero/undersized
stack, malformed geometry, overlap and overflow fail structurally.

The validated run uses an 8-MiB stack, following the legacy main-stack size; this is an explicit caller
policy, not PS5 hardware truth. A 32-byte record is at top-64, argv0 points to the literal `guest` plus
NUL in the next 32 bytes (not a filesystem identity). argc=1, pad=0, argv/environment terminators zero.
RSP=record-8, RSP mod16=8. The return slot is zero and **not valid for execution** until a verified host
return landing is installed. A 128-byte red zone fits below RSP; no Windows shadow space is placed on
the guest stack. Windows shadow space belongs on the future host landing stack.

RIP is the existing biased entry, RDI points at parameters. Other GPRs are zero; RSI zero is an explicit
missing exit-handler blocker. XMM lanes are zero, MXCSR=0x1f80, x87 control=0x37f, RFLAGS=0x202 (DF clear).
These FP/flag initialization choices are conservative experimental policy, not a complete firmware CPU
reset specification. FS base is planned only; GS must retain the host TEB. The future bridge must save
host nonvolatile GPRs/XMM6-XMM15/control state and restore host stack/FS on every exit. M29 does not
load CPU registers. No reliance on incidental live host register contents exists in the context model.

## TLS and bootstrap

PT_TLS is read through the immutable source token. Malformed sizes, multiple TLS descriptors and invalid
alignment are explicit errors. Nonzero TLS first-byte alignment offsets are explicitly unsupported,
not silently laid out at the wrong negative offset. Template bytes are copied, memory tails/padding zero-filled, then a
self-pointer is stored at the planned thread pointer after aligned data (variant II). 0x200 bytes of
TCB slack are an **experimental legacy model**, page-rounded and budgeted; unknown errno/canary/DTV
layout is not fabricated. An empty/missing PT_TLS proves no main-module template only, not absence of
libc TLS. Therefore TcbLayoutUnproven and TlsActivationMissing remain entry blockers even in this run.
No Windows FS/GS change, module TLS activation, errno implementation or destructor occurs.

Raw DT_INIT/DT_FINI and other existing bootstrap blocker values remain retained, not executed. The
selected DT_INIT=0x10 and DT_FINI=0x524270 require an ordered startup/termination contract later.

## HLE and exception preparation

PreparedRegistry is immutable and bounded by caller capacity; keys require exact numeric NID plus
byte-faithful library/module context. Duplicates are ambiguous, no name-only fallback. Identity fields
are limited to 256 bytes; overlong declarations refuse rather than truncate. Kinds distinguish
SyntheticHostTest, HleImplementation and ArtifactDeclaration. Artifact declarations have no host
handler. Host-model closures can capture explicitly owned provider state, without global registries.
Host-model calls copy captured arguments and publish only return lanes after successful
handler completion; panic returns a structured failure without partial return publication.

This is not native registration: no handler address or fake trampoline is patched into an artifact.
The real run supplies an empty registry and reports zero installed providers. A tested 24-byte import
stub encoder preserves the legacy far-indirect layout, rejects sign-extending indices/zero landing;
its bytes are not materialized without a real landing. No arbitrary RIP is ever exposed as a Rust fn.

RecoveryBoundary owns Prepared/Stopped/Released state with first-stop-wins instrumentation. It is a
host-only contract, **not installed VEH/SEH recovery**. No catch_unwind claim is made for native faults.
The future native adapter must switch to a saved host stack before Rust, reject foreign/nested faults
outside its active lease, capture fault context, restore FS and nonvolatile state, and regain control
on return/stop/fault. Missing native bridge, return/exit landings and recovery adapter are immediate
blockers, not execution-deferred work. No process-global handler is installed merely to print readiness.

## RELRO and readiness

RELRO removal of write permission is permitted only for whole owned readable/non-executable pages,
with no pending relocation overlapping the interval. Unknown widths/locations conservatively prevent
sealing. Selected-but-unstaged provider values remain pending exactly as in M27. Unaligned ranges refuse
rather than accidentally protect neighbouring mutable bytes. Memory performs VirtualProtect to R and
VirtualQuery verification; a failure releases/quarantines the image rather than reporting a successful
partial transition. Real workload RELRO remains PendingWrites. Synthetic native tests verify exact R
sealing, idempotence and preservation of the adjacent RW page.

PreparedGuest exposes typed entry blockers and early-runtime blockers, plus preserved plan/diagnostics.
Unknown program semantics are entry blockers. No subsystem guesses move imports into GPU/audio/deferred
categories. Dependency declarations and unresolved-reference totals are also reported as early-runtime
pressure; unresolved relocation writes remain separately immediate blockers. EntryReady is always false
until an actual adapter can supply the missing authority; no public Boolean can fabricate a ready token.

## Real result and hypotheses

At image bias 0x100000000: NativeBackedWithPendingWork, RIP=0x100000070; stack guard
0x200000000..0x200001000, stack 0x200001000..0x200801000, RSP=0x200800fb8, RDI=0x200800fc0.
TLS/TCB mapping is 0x210000000..0x210001000, planned FS=0x210000000. Combined runtime bytes=8396800.
822 external references, 1107 pending relocations, 38 dependencies, zero provider registrations.
Two unknown program kinds 0x6fffff00/0x6fffff01 remain blockers; source SHA256 unchanged.
EntryReady=false, exit 0 means successful *preparation report*, not runnable guest. Teardown reports
zero image/stack/TLS reservations and byte mappings. No timing worker requested.

Supported: argument-record shape and stack alignment; isolated native stack/TCB lifecycle and cleanup.
Refined: empty PT_TLS is not sufficient to skip thread-local setup. Rejected: legacy _init_env stub is
not proof of valid environment setup. Unresolved: complete TCB/errno, early provider semantics,
RELRO closure, native bridge/fault adapter and startup ordering. These remain explicit M30 prerequisites.
M30 should close the selected startup path and validate host-only bridge/fault recovery before deciding
whether to perform the first controlled guest entry. M29 has not performed that entry.
