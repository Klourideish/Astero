# Explicit string-reference observation (M19)

SourceArtifact -> explicit string_references::observe(source, limits) -> immutable
StringReferenceObservationReport -> STOP. No earlier frontend command invokes this operation.
DT_NEEDED -> bounded byte evidence does not create a dependency declaration or resolution action.

## Scope, prerequisites and reuse

Only DT_NEEDED is supported: it is the only existing named string-reference tag in Astero.
SONAME/RPATH/RUNPATH and unknown tags stay outside interpretation; M17 raw evidence is unchanged.
There is no API for enumerating every string, arbitrary caller-supplied offsets or resolving names.

M19 calls M18's existing selected descriptor observation. Its header/dynamic/descriptor budgets and
strict STRTAB/STRSZ and SYMTAB/SYMENT validation remain authoritative, including failure for an
invalid selected companion group even when there are no supported string references. M18 does not
call M19. A complete string descriptor is consumed by M6 DynamicStringTable::from_descriptor:
it accepts only a privately constructed M18 record and checks its token against the source.
M6 from_dynamic retains its semantics; both constructors share source-bound initialization.
The existing M6 lookup implementation is unchanged and remains the only byte scanner.

M19 uses M6's existing StringLimits value type (currently owned in dependencies/model) but does not
call dependencies::observe, construct DependencyName, enrich InspectedArtifact, or interpret imports.
The budget type's historical location does not grant dependency semantics. No symbol/hash/relocation
payload APIs, admission, linkage, session state or guest execution are called.

## Explicit budgets

All lower limits remain mandatory: max-bytes, max-read-calls, max-program-headers,
max-dynamic-entries and max-descriptors. New CLI limits align with M6 vocabulary:

| Limit | Meaning |
|---|---|
| max-string-references | Total supported source references attempted, including duplicates, empty and invalid references |
| max-scan-bytes-per-reference | Maximum bytes searched for one reference, including terminating NUL |
| max-total-scan-bytes | Aggregate searched-byte allowance for all references, including repeated offsets and NULs |

No defaults. The complete raw reference count is checked before any lookup. Zero references permits
only absence; exceeding it returns ReferenceBudget without a prefix. Every supported reference
counts, including one whose lookup would fail. Unsupported tags use raw-entry budget only.

The next lookup receives min(per-reference allowance, remaining total allowance). Successful M6
lookup charges content length + 1. Empty consumes one byte and one reference; raw non-UTF-8 bytes
are charged normally. Exact limits succeed if NUL is included. A zero scan allowance cannot even
establish an empty string. Duplicates are scanned/charged separately; there is no deduplication/cache.

M6 error precedence is retained: offset outside STRSZ fails before scanning; no NUL before a budget
shorter than the remaining table yields ScanLimit; exhausting the table without NUL yields
MissingTerminator. A failed lookup consumes its attempted slot and stops the request; no partial
string/prefix list is exposed. Error context includes originating entry, effective limit, completed
reference count, remaining aggregate allowance and the underlying table/range/offset diagnostic.

No additional output-byte knob is necessary: content bytes retained/emitted are bounded by the
aggregate scan allowance (excluding charged NULs). CLI escaping/hex adds a bounded expansion and
per-record overhead bounded by reference count. UTF-8 validation/presentation make bounded passes
over successful views; byte budgets charge lookup scans rather than literal CPU instructions.
The new record vector reserves fallibly only after counting the source-proven references. There
are no whole-table copies or untrusted-size string allocations. Existing lower-level allocation
policy remains unchanged.

## Evidence, encoding and status

Records preserve the original Needed tag, dynamic-entry index, raw offset, proven content token,
shared immutable source handle and encoding classification. as_bytes returns original bytes,
excluding NUL; source tokens retain identity. Order is dynamic-entry order. Equal byte values or
repeated offsets remain distinct source references. Unreferenced table contents are never scanned.

Encoding: Empty, Utf8, RawBytes. Empty is successful evidence, not absence. UTF-8 classification uses
M6's explicit validation and does not normalize bytes. Non-UTF-8 bytes are never replaced/lossily
converted. CLI uses quoted/escaped UTF-8, <empty>, and <non-UTF8: FF> style hex representation;
control bytes cannot become terminal actions. It never interprets a name as a library/path/NID.

- Complete: every supported reference succeeded; only byte evidence, not dependency availability.
- Unavailable: valid prerequisites contain no supported references (including no dynamic table).
- Failed Prerequisite: retains the entire failed M18 report with structured cause and raw audit data.
- Failed TableUnavailable: a supported reference exists without usable STRTAB/STRSZ metadata.
- Failed ReferenceBudget, Table, Lookup or Allocation: structured refusal, never partial success.

Conflicting/incomplete descriptors fail at M18; an empty table plus a reference fails M6 bounds.
The immutable report always retains source identity/provenance and all limits. Failed lookup records
retain source-bound range diagnostics. Nothing can attach mutable runtime state or change source
bytes. CLI exit 0 is Complete/Unavailable, exit 1 is Failed or selection/acquisition failure.

## Ownership and evidence

Loader elf/dynamic/string_references owns orchestration/value/error records; string_table owns byte
lookup. Core input/string_references delegates. CLI string_references selects native paths and
formats records; GUI is unchanged. No dependency edges or external packages were added.

Authority is the established [M6 lookup contract](dynamic_strings.md), [M18 descriptor proof](explicit_descriptor_observation.md)
and source-owned bytes. No external catalogue or real guest input was needed. Synthetic tests cover
reference ordering, duplicate/empty/raw views, independent earlier commands, source identity,
per/aggregate/reference budgets, boundary sweeps and native filesystem path preservation.

Potential later work may expose separately bounded additional references only when their semantics
are justified. This must not silently extend into symbol names, dependency resolution or linking.
