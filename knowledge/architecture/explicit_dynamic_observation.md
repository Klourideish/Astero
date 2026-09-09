# Explicit raw dynamic observation (M17)

SourceArtifact -> explicit dynamic::bounded::observe(source, limits) -> immutable
DynamicObservationReport -> STOP. Acquisition, M16 header inspection and this request are
independent operations. The CLI dynamic command explicitly acquires and then requests this report;
acquire and inspect never invoke it. No session is constructed or changed.

## Ownership and reuse

Loader elf/dynamic/observation/raw owns shared PT_DYNAMIC discovery, range proof and entry
traversal. Existing entries::decode remains the only Elf64_Dyn byte decoder. AddressTranslator
now accepts internal raw program-header observations as well as the existing ElfInspection API;
its mapping validation and translation policy are unchanged. This avoids invoking the M4 artifact
adapter or fabricating a module context. M16 header decoding supplies bounded discovery evidence,
not an implicit frontend operation. M16 gained only a crate-private consuming outcome accessor.

The existing M5 observe API delegates to the shared traversal, then runs its existing descriptor
pairing/translation. M17 does not call that descriptor step. Incomplete/duplicate STRTAB or other
recognized descriptors and invalid pointer values therefore remain raw observations in M17;
they still fail the separate M5 descriptor operation as before. No strings, symbols, hashes,
relocation contents, candidates, admission or linking are derived by M17.

Core input/dynamic delegates; CLI dynamic owns native-path arguments, explicit requests and
presentation. Existing loader dependency direction is unchanged; no new dependencies or GUI work.
The unchanged M5 byte fixture builder moved to dynamic/synthetic and is re-exported by its test
home so tests and the create-new-only dynamic_fixture example share one generator.

## Budgets and termination

Both max_program_headers and max_dynamic_entries are mandatory u64 caller inputs, with no defaults.
The former counts every program header, not only loads/dynamic headers. Existing fixed header
validation proves the table extent and refuses counts above budget before traversal/collection.
The ELF u16 count additionally bounds this discovery work; translation makes a bounded number of
linear passes over those headers. Reusing discovery internally does not grant a session state.

The latter counts attempted dynamic entries, including the terminal DT_NULL, which is retained.
Zero permits absence but refuses any nonempty table before its first entry. Exact budget succeeds
if the last allowed entry is DT_NULL. The next attempted entry beyond budget returns EntryLimit
(limit also equals the number successfully observed so far); no prefix is exposed as Complete.
No preallocation is based on the declared dynamic size. Fallible vector growth occurs only after
bounded decoding and before retention; allocation refusal is structured as Allocation { entries }.
Caller budgets bound work/retention, not a guarantee of available host memory.

Complete means the first DT_NULL was observed, not ELF admission or semantic validity. Its raw
value is retained. Trailing bytes, including odd padding, remain uninterpreted. Before DT_NULL,
a final short chunk is TruncatedEntry; exact extent exhaustion is MissingTerminator, including
an empty mapped dynamic segment. Budget refusal precedes decoding a disallowed short chunk.
No payload alignment requirement is invented: explicit byte decoding needs no host alignment.

## Discovery, status and errors

- No PT_DYNAMIC: Unavailable, even with zero entry budget; no translator is needed.
- Multiple PT_DYNAMIC headers: Failed MultipleTables, including identical headers.
- One table: require file_size <= memory_size, nonoverflowing virtual extent, actual source range,
  and identical source token from PT_LOAD translation and PT_DYNAMIC file offset.
- BSS, unmapped/conflicting ranges, source crossing, offset mismatch and overflow: existing
  structured DynamicError/TranslationError/SourceError preserved.
- Headers, discovery/ranges, decoding, budget, missing terminator or allocation failure: Failed;
  no complete entry list is retained. No Partial state or silent truncation.

The report owns a shared immutable SourceArtifact and private outcome/limits. Complete contains
RawTable with the program-header index, full declared source token, ordered entries (index, tag,
raw u64 value). Unknown(i64) preserves every numeric tag bit; no normalization or pointer inference.
Entry source offsets follow table offset + index * 16 within its validated range. Failure retains
the source/provenance and budgets in the enclosing report, with existing segment/entry/range/error
context and nested causes. Failed does not claim any completed observations.

CLI exit 0 covers Complete and Unavailable; Failed and acquisition/argument errors exit 1.
Source identity is process-local object identity; repeated observations share it and bytes.
Names/paths are native at acquisition, presentation is diagnostic only. No acquired or observed
artifact implies a loaded guest. No registry, resolution, memory mapping or execution exists.

## Evidence and remaining pressure

Uses the established [M5 policy and standard-format references](dynamic_elf_observation.md),
[M16 header boundary](explicit_inspection.md) and synthetic fixtures; no external catalogue was needed.
Tests exercise 168 budget/count pairs, 31 pre-termination extents, source/translation boundaries,
unknown raw values, independent acquisition/header operations and native path subprocesses.

Potential next work is a separately requested bounded descriptor observation surface, only after
its independent budgets and refusal policy are designed. It is not part of this capability.
