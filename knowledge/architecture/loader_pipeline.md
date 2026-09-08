# Loader inspection, admission and load plans (M2/M3/M4)

SourceArtifact -> InputArtifact -> inspect -> InspectedArtifact -> admit -> ValidatedTarget -> plan -> LoadPlan.
Application to memory/runtime is future work. Admission success means a supported description of
synthetic or bounded ELF-derived work, never permission or ability to execute guest code.

## Ownership and entry points

[astero-loader](../../crates/astero-loader/README.md) owns admission and planning. This supersedes
the old M1 recommendation that core own admission. Core remains session composition and is not
changed or connected to the loader in M2. Neither runtime services nor frontends are changed.

- artifact/source.rs: immutable source ownership, generated identity and checked bound ranges.
- artifact/inspection.rs: source-bound synthetic input, classification, immutable inspected view.
- metadata/regions.rs: loader-local address, source-range, permission and region intent.
- modules/identity.rs: synthetic module identity/metadata and observed/admitted roles.
- dependencies/requirement.rs, imports/symbol.rs, exports/symbol.rs, relocations/work.rs:
  format-independent work descriptors, with no symbol/NID resolver or relocation evaluator.
- admission/rejection.rs, validation.rs, validated.rs: policy checks, taxonomy and admission authority.
- load_plan/plan.rs: immutable mappings, copy extents and explicit zero-fill extents.

The loader has no dependencies. M2 removes the speculative loader -> ABI/memory permissions and
adds it to dependency_free_packages in the policy; normal/dev/build/target declarations are checked.
No actual dependency edge changes. Address/permission intent is local, not guest ABI, a host pointer,
a mapped-memory capability or justification for trivial-type reuse dependencies.

## Inspection describes; admission accepts or rejects

> A target being syntactically inspectable does not mean Astero accepts it for loading or execution.

InputArtifact contains a required immutable SourceArtifact and synthetic ArtifactDescription.
inspect preserves supplied metadata, including unsupported classifications; it performs no parsing.
Identity, actual source length and provenance come only from the source. M3 intentionally removes
M2's caller-assigned ArtifactId and declared size fields; there is no unbound compatibility path.
See [immutable source binding](source_binding.md) for ownership, checked reads and error semantics.

SourceId identifies one immutable process-local source object; it is not a hash or persistent identity.
ModuleId identifies a module within the caller's synthetic dependency graph; it is not a kernel handle.
Labels and module/symbol names are text; bounded classifications and failure categories are enums.
No PS5 identifiers, page sizes, relocation numbers or firmware constants are invented.

Observed families are Synthetic, Elf, SelfFormat and Unknown; architectures are X86_64, Aarch64 and
Unknown; roles are Executable, Module and Unknown. M2 accepted only Synthetic/X86_64/executable-or-module; M4 also permits Elf at this format gate.
This is an explicit synthetic policy, not a claim that x86-64 or a PS5 binary is supported at runtime.
All ranges use u64 sizes and half-open extents; requested virtual addresses are fixed intents.

Future format-specific parsing -> InspectedArtifact -> format-independent admission/planning.
ELF/SELF adapters must produce these loader contracts, not expose parser structures to the application.
M4 implements the ELF adapter; SELF remains scaffolded. See [ELF inspection](elf_inspection.md).

## Admission invariants and rejection taxonomy

Checks run in the following order, returning the first failure. Region/descriptor indices always
refer to the original observed lists, even though admitted regions are subsequently sorted by address.
The caller retains the inspected artifact identity alongside a structured rejection.

| Boundary | Checks and failure categories |
|---|---|
| Classification | UnsupportedFormat, UnsupportedArchitecture, UnsupportedRole |
| Explicit requirements | UnsupportedRequirement for TLS, initializer callbacks or dynamic placement; never silently dropped |
| Required metadata | MissingMetadata identifies module metadata, nonblank module name or nonempty regions |
| Region sizes | EmptyRegion; InvalidSize when source size exceeds memory size |
| Extents | VirtualRangeOverflow identifies the region; SourceRange nests a SourceError for overflow, invalid offset or excessive length against actual bytes |
| Alignment | InvalidAlignment for zero/non-power-of-two alignment or a misaligned requested address |
| Overlap | OverlappingRegions identifies both original indices; even identical virtual overlaps are rejected conservatively |
| Entry | InvalidEntryPoint retains optional address; executable requires an entry, module may omit it; any entry must be within source-backed executable bytes |
| Linking metadata | MalformedMetadata identifies descriptor kind, index and bounded MetadataProblem |
| Relocation vocabulary | UnsupportedRelocation identifies a descriptor outside the sole synthetic work kind |

Adjacent virtual regions are valid. Source extents may overlap (shared input bytes); source offsets
need not be aligned. This is intentionally not ELF congruence or host-page mapping policy. Empty source
extents are allowed, including offset at actual EOF, and produce pure zero fill. Virtual/source end
overflow is rejected, including an otherwise tempting extent ending at 2^64.

Dependencies are unique and cannot refer to the current module. Imports require a nonblank symbol,
a declared dependency and a unique dependency/symbol pair. Exports have unique nonblank names and
nonzero extents contained within one admitted region; function exports require executable permission.
Export metadata is not proof of a callable implementation. No dependency availability, transitive
cycle resolution, version matching, symbol binding or initialization order is inferred.

SyntheticAbsolute64 is an eight-byte deferred patch descriptor, not an ELF relocation. Its patch must
fit within one region, cannot overlap another patch, and references an existing import index or a local
address within a region. Addends are retained without arithmetic or evaluation, including negative and
extreme values. Final writable permission is not required: staged write/protection policy is deferred.
No endianness, final relocation value, overflow behavior or execution support is claimed.

Rejection implements std::error::Error and Display without an external crate. Structured variants,
not Display text, are the diagnostic contract; GUI/debug integration and presentation are deferred.

## Validated target and immutable plan

Only admission can construct ValidatedTarget. Its private storage contains normalized TargetMetadata
and address-sorted ValidatedRegion values, not InputArtifact or a raw parser wrapper. Getters return
shared references/slices. TargetRole excludes Unknown and all values held in a validated target have
passed admission. Detached metadata/region copies can be edited but cannot mint admission authority.

Only plan(&ValidatedTarget) constructs LoadPlan. It clones the admitted metadata and emits mappings
with requested range, alignment and permissions. An initialized prefix has a CopyIntent from its
validated source-identity/range token to mapping start; the plan retains one shared immutable source. A remaining tail has an explicit zero-fill AddressRange; zero-length operations
are omitted. Admission proofs make range arithmetic infallible. Metadata retains identity, family,
architecture, role, module, source provenance/size, entry, dependencies, imports, exports and relocations.
Descriptor order is preserved because relocation import indices refer to that order. No initializer
order exists: inputs declaring initialization callbacks are rejected instead.

Repeated planning of the same bound source/input yields equal plans; region permutations yield the
same canonical mappings. A reconstructed source has a new identity. No runtime handles or callbacks
enter these contracts. Source creation alone advances a private identity counter; inspection, admission
and planning do not. Planning allocates ordinary host-owned result vectors and clones source handles only. Tests compare observations and targets before/after repeated
planning; dependency policy and direct source review establish that no session/runtime interface or I/O
is involved. This is not a memory application, import resolver or relocation executor.

## Evidence, limits and next pressure points

[Integration fixtures/tests](../../crates/astero-loader/tests/contracts.rs) are handwritten synthetic
metadata, never derived from Sony binaries. They cover valid executable/module cases, every rejection
family, metadata consistency, canonical planning, copy/BSS intent and a 1,056-case size sweep.
Compile-fail doctests cover private target construction and immutable target/plan views. See
[validation](validation.md) for exact executed counts; synthetic tests establish no emulator correctness.

M3 closes the declared-size/identity trust gap with immutable source ownership. Future work still needs
resource limits and potentially faster overlap checks for large hostile metadata. Accepted permission
combinations are descriptions, not host protection guarantees. Cross-module linking, richer symbol/
relocation vocabulary, relocation evaluation and dynamic placement require separate contracts.
Networking, platform/input, video decoding, playback, compatibility, firmware runtime interfaces,
caches and global configuration remain untouched decision gates. See [source binding](source_binding.md)
for remaining lifetime, persistent-identity and byte-acquisition design pressure and recommended M4.

## M4 extension

elf::inspect decodes immutable bytes and returns an ELF-specific report with the same InspectedArtifact.
The only admission algorithm change is allowing ArtifactFamily::Elf; its region/linking checks and
validated target/plan representation remain unchanged. PlatformSemantics and UninspectedProgramSemantics
requirements prevent ignored ABI flags or non-load descriptors from silently yielding a complete plan.
This gate accepts bounded work descriptions, not PS5 platform support or fully conforming ELF images.
See [ELF inspection](elf_inspection.md) for the conservative alignment and ET_DYN limitations.
