# M16: explicit bounded header inspection

SourceArtifact -> explicit inspection request + InspectionLimits -> immutable evidence -> STOP.

## Chosen scope and ownership

Only existing ELF64 identification, file-header and program-header decoders are composed here.
Sections are not followed; dynamic/string/symbol/hash/relocation/candidate APIs are not invoked.
This is deliberately smaller than M11 linkage reporting. No new parser or format classifier exists.

loader/elf/inspect/bounded owns collection and report semantics. Core/input/inspection delegates;
CLI/inspection owns explicit request syntax and rendering. Core creates no session or SessionInputs.
GUI linkage observation and all synthetic linkage semantics remain unchanged. Dependencies, manifests
and lock remain unchanged: CLI -> loader is still dev-only; core delegates through its existing edge.

The public bounded inspection function is separate from M4's existing adaptation function.
Both use the same header::decode and ProgramHeaders machinery. The older adapter remains unchanged.
M16 retains raw header values; it does not create a ValidatedTarget, load plan or linkage report.

## Budget and completeness

InspectionLimits.max_program_headers is mandatory, measured in program-header entries of every type.
Zero is valid for a source declaring zero entries; an equal count succeeds. A larger count returns
HeaderBudget with declared and maximum counts before entry decoding or allocation. No prefix report
is returned. Header validity and whole-table source bounds are checked first, so malformed table
metadata can fail before the budget check. Unsupported extended counts remain existing ELF errors.

Identification (16 bytes) and file header (64 bytes) are intrinsically fixed-size. The existing
decoder proves the full table range through checked source access, without walking it. Ordinary ELF
count is u16 with 0xffff unsupported: at most 65,534 entries of 56 encoded bytes. Therefore a second
header-byte/step configuration is unnecessary. A fallible reservation requests only the proven,
budget-admitted entry count; each entry is decoded once and retained. Existing allocator/system
limitations remain; there is no promise of process-wide OOM recovery or exact allocator RSS.

InspectionOutcome is Complete(HeaderEvidence) or Failed(InspectionFailure). No Partial or Unavailable
variant is needed for this all-or-failure operation; unsupported forms are structured failures.
Complete means the requested header scope only, not whole-file validation. Segment payload ranges,
permissions, entry-point eligibility and runtime requirements are not admitted; an unvisited dynamic
or section pointer can be invalid while this raw header observation is complete.

## Immutable evidence and errors

InspectionReport fields are private. It retains the immutable source, requested limits and outcome
on success or failure. HeaderEvidence exposes borrowed header/program values only. Detached clones
of individual values cannot edit the report. A compile-fail doctest protects the outcome field.
SourceId/provenance/bytes are the same original source owner; repeated inspection does not mint a
new source or copy whole-file contents. Independent acquisitions still create independent IDs.

Failure variants distinguish header/table validation, header-budget refusal, allocation failure
and indexed program-header decoding. Existing ElfError values retain structure/offset/range context
and remain nested Error::source values. Source identity, provenance and limits remain on the report
even when decoding fails. CLI only formats these errors; it does not reinterpret them.

## Explicit CLI boundary

inspect requires --path, --max-bytes, --max-read-calls and --max-program-headers.
Acquisition uses M15 syntax and M14 mechanisms unchanged. Only the shared numeric option decoder
became crate-visible for the new parser-budget flag. Native args_os/PathBuf identity is preserved.
Acquisition alone still returns before all inspection code; inspect_acquired requires an explicit
call and refuses Ready/Failed acquisition selections. It never changes the acquisition selection.

The inspect command first reports the acquisition stage, then the explicit request and inspection
result. Both acquisition and inspection limits are printed. On inspection failure, acquisition may
still correctly say Acquired; the command exits nonzero. It states no guest loaded/execution/linkage.
There is no new session attachment or persistent registry and no automatic linkage after inspection.

## Fixtures, tools and validation

Existing M4 generated helpers moved unchanged into elf/inspect/synthetic for reuse. The original
test home re-exports them. This was required because the repository indexer deliberately rejects
cross-file #[path] imports; it was not broadened or bypassed. No production decoder changed.

The inspection_fixture Cargo example writes one 272-byte generated M4 executable-shaped fixture,
using create_new so existing files are not overwritten. It is a manual validation tool, not guest
execution or acquisition of a real binary. See [USAGE](../../USAGE.md) for exact tested commands.

Four new loader tests prove budget sweeps, equivalence with old decoders, immutable source sharing,
structured malformed/unsupported errors and unvisited dynamic/section pointers. Four CLI integration
tests prove explicit opt-in, exact/exhausted budgets, distinct acquisition/inspection failures,
native non-UTF-8 paths and deterministic repeated evidence. Existing M14/M15 and synthetic linkage
tests remain. No legacy/decrypted catalogue evidence was needed.

See [validation](validation.md) for full checks/manual exits. No admission, memory mapping, runtime
loading, NIDs, ABI, GPU work, linking or execution occurs. Later inspection scopes require their own
explicit request and suitable budgets; M16 grants no permission to invoke them automatically.
