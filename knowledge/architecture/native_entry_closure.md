# M30 native entry closure

Astero retains direct native x86-64 Windows execution as its primary CPU strategy.
M30 prepares the selected M29 `primary_real_elf` and validates the bridge using only
statically linked Astero-owned assembly probes. **No real guest artifact code executes.**
`EntryReadyGuest` authorizes a future controlled experiment, not successful guest operation.

## Evidence and limits of inference

Catalogue-relative references, reached through each catalogue's START_HERE indexes:

- PS5Rust `02_CORE_EXECUTION/native_boundary.md`, `assembly/INDEX.md`, `assembly/001.md`
  through `004.md`: separate stacks, Windows nonvolatile GPR/XMM preservation, import capture,
  and switching out of the guest stack before Rust fault handling.
- `02_CORE_EXECUTION/source/ps5-core__src__cpu__native_exec/007.md` and `010.md`:
  experimental TCB slack, variant-II negative offsets, FS decay, dispatch and recovery pitfalls.
- `03_LOADER_LINKER/provider_authority.md` and
  `source/ps5-core__src__loader__elf/001.md`: object providers are not function pointers.
  The catalogue source reference was followed read-only to
  `crates/ps5-core/src/loader/elf.rs` (program-kind conversion, lines 79-99), which labels
  `0x6fffff00` SceComment and `0x6fffff01` SceVersion.
- `07_NIDS/by_sysmodule/SOURCE_OWNER_kernel/005.md`: `_init_env`,
  `bzQExy189ZI`, numeric `0x6f3404c72d7cf592`, is a legacy zero-return stub. This is not
  firmware proof that environment initialization is unnecessary.
- `07_NIDS/by_sysmodule/libSceLibcInternal.sprx/090.md`: `atexit`, `8G2LB+A3rzg`,
  numeric `0xf06d8b07e037af38`; firmware-backed identity and legacy callback collection.
- `15_FIRMWARE_KNOWLEDGE/document_guides/403_THREADING_TLS.md.md`: does not establish a
  complete TCB or destructor ordering contract.
- Decrypted catalogue `04_GLUE/PS5Util/INDEX.md` and `0001.md`: utility glue from a different
  snapshot does not prove the selected executable's process startup.

The existing M29 bounded startup disassembly supplies the selected workload's entry-owned
`DT_INIT` hypothesis: `_init_env`, two `atexit` calls, then init/main-shaped calls.
No external emulator was consulted. Windows CONTEXT layout and host preservation were checked
against the installed SDK and [Microsoft CONTEXT](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-context)
and [x64 convention](https://github.com/MicrosoftDocs/cpp-docs/blob/main/docs/build/x64-calling-convention.md)
documentation. The published VEH sample is explicitly unsuitable for x64 and was not copied.

## Ownership and bridge

Core composes the M29 image, stack, TLS, context, optional existing timing slot, startup registry,
RX import landings, NOACCESS object traps and bridge. Loader planning remains immutable and offline.
Kernel `execution/host/platform.rs` is the sole new private unsafe leaf, including assembly.
Memory retains host VM placement, checked writes and protections. Libs owns startup behavior;
kernel owns bounded callback storage. Existing allowed core-to-libs and libs-to-kernel edges activate;
there are no external dependencies or new crates.

The bridge is thread-affine and process-exclusive: a second owner explicitly refuses. Its TLS slot
and VEH have deterministic lifetime; unsuccessful removal quarantines ownership instead of allowing
a replacement over a possibly live handler. Active frame ownership is scoped to the invocation.
M30 exposes only fixed synthetic probes; no public arbitrary RIP/function-pointer transfer exists.
Real executable page ranges are retained and validated for M31's future invocation lease.

Assembly saves Windows nonvolatile GPRs and FXSAVE state (including XMM6-XMM15), establishes known
guest GPR/vector/FP defaults and flags, switches stacks and activates guest FS. Host GS is unchanged.
The return landing restores FS, stack and saved host state and clears DF. Synthetic checks seed all
eight Windows nonvolatile GPRs and ten nonvolatile vector registers and verify them after transitions.
This establishes ABI preservation, not restoration of every arithmetic flag or full PS5 CPU state.

The import landing captures six argument GPRs, eight vector lanes and six checked stack words,
saves guest nonvolatile GPRs, restores host FS/stack for Rust dispatch, then restores the guest path.
RAX/XMM0 are return lanes. Exact NID/library/module keys select providers. Missing keys, unsupported
results and panics stop through the controlled landing; they never manufacture a successful return.
`NativeExit` retains reason, RIP/RSP, exception/address, ordinal and captured registers. A normal RET
has an installed return landing. Full process exit and callback invocation are deferred.

VEH accepts only continuable access violations/illegal instructions in the active frame's ranges.
It switches to the saved host stack and restores FS before minimal Rust context handling, then
redirects to the return landing. Inactive, foreign-range and noncontinuable exceptions continue search.
No allocation/logging occurs in the classifier. This is fault recovery, not a security sandbox,
arbitrary host-fault suppression, asynchronous preemption, CET certification or complete unwinding.
FSGSBASE host support is checked; unsupported hosts refuse. Multiple simultaneous runtimes are deferred.

## Startup closure and experimental policy

`close_entry` consumes M29 preparation. Existing nonempty provider registries explicitly refuse
rather than being silently replaced. The selected empty-registry path installs two exact libc/libc
registrations. `_init_env` counts every call and returns zero under a documented experimental no-op
hypothesis. `atexit` retains at most 256 executable callback addresses, including duplicates, in
registration order; reverse-order observation is available. Null, non-executable or over-capacity
registration fails. No callback executes and no full libc shutdown semantics are claimed.

`PreserveUnknowns` retains TCB/initializer blockers. The CLI's explicit `--close-entry` selects and
prints `ExperimentalEntryOwnedInit`: M29's self-pointer/0x200-byte slack TCB is activated experimentally,
despite empty PT_TLS, and entry owns DT_INIT (0x10); DT_FINI (0x524270) remains metadata. No init is called.
This policy permits a controlled first attempt, not a claim that libc TLS/errno/environment is complete.
SceComment/SceVersion are retained nonblocking metadata; other unknown program kinds remain blockers.

Supported unresolved FUNC writes receive 24-byte ordinal far-indirect landings. Width/action/addend
eligibility is checked; there is no RWX mapping or relocation replay over RX pages. Unknown functions
stop with ordinal/key evidence. Unresolved OBJECT references do not receive executable stubs or fake
storage: experimental NOACCESS traps group references by symbol, preserving checked S+A relationships
within page-rounded spans. Access stops for diagnosis. These are unresolved identities, not providers;
arbitrary pointer arithmetic is not guaranteed to stay in a trap. This policy is useful for a bounded
first-entry experiment but must not be presented as general guest-memory isolation.

All required writes finish before RELRO. Present RELRO pages become read-only and are queried;
unmapped holes stay inaccessible. Untreated writes or invalid protection ranges retain blockers.
The byte cap includes existing stack/TLS, RX landings and object trap reservations. Upstream record
budgets bound planning, keys and patches; allocation refusal is explicit. Recovery range retention
has a fixed 65,536-range cap. No timing worker is created by this operation.

## Readiness and experiment outcome

Only a closure with no MustClose records can issue the privately constructed `EntryReadyGuest`.
It owns the live resources; M29's blocked type is not execution authority. Entry executable coverage,
stack/alignment, return landing, validated bridge, TLS policy, startup providers, safe unresolved
landings, treated writes and RELRO state must all hold. M31 still must arm a real-artifact active frame
and consume the token to transfer control; M30 intentionally has no such public operation.

The same `primary_real_elf` as M29 is used; paths remain in local configuration. RIP `0x100000070`,
RSP `0x200800fb8` (mod16=8), FS `0x210000000`; the 8 MiB stack and empty-template TCB are unchanged.
1,107 pending writes become 836 function landings and 271 guarded-object writes for 17 identities.
Two startup keys match registered providers; other function calls and all guarded object accesses
remain early-runtime stop conditions. RX landings occupy 20,480 bytes; object traps 69,632 bytes.
One RELRO range is sealed on committed pages, with a 12,288-byte inaccessible hole.
Experimental `EntryReady=true`; no MustClose blockers remain. Source bytes are not modified.

Hypotheses: host preservation/recovery is supported by direct synthetic execution; the original
function-only closure was rejected as insufficient for 271 object writes. Object trap closure is
an explicit refined experiment, not data-provider implementation. TCB/environment completeness and
entry-owned initializer semantics remain experimental until actual workload observation. Early-runtime
unknown imports, objects, errno/TLS, callback execution, thread creation and subsystems remain deferred.

M31 should consume this token for a supervised first native attempt, arm exact ranges, capture the
first return/fault/unresolved key and stop. No bulk provider migration is justified by readiness.
Validation results and actual command outcomes belong in [validation](validation.md).
