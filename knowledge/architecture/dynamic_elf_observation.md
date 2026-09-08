# M5: dynamic ELF observation and address translation

M6 adds [bounded string and dependency interpretation](dynamic_strings.md) after this unchanged stage.
The statements below describe M5 descriptor observation, not the added M6 interpretation API.

> Observing a dynamic-table descriptor does not mean Astero has resolved or applied it.

SourceArtifact -> elf::inspect -> elf::dynamic::observe(report, ObservationLimits) -> dynamic report.
The report is format-specific and separate from InspectedArtifact. Existing admission/planning remains
available through report.artifact(); a PT_DYNAMIC image still fails admission with the existing
UninspectedProgramSemantics requirement. Even a DT_NULL-only table does not clear that guard.

## Evidence and scope

Primary standard-format evidence, read 2026-09-08:
[gABI dynamic structures/tags](https://gabi.xinuos.com/elf/08-dynamic.html),
[symbol entry layout](https://gabi.xinuos.com/elf/05-symtab.html), and
[relocation entry layouts](https://gabi.xinuos.com/elf/06-reloc.html).
These inform the ordinary signed tag/value encoding and fixed ELF64 descriptor widths. All limits,
error precedence, duplicate/group handling and conservative mapping rules below are Astero policy,
not asserted PS5 semantics. No Sony/prototype/catalogue input or implementation was used.

## Dedicated ownership

- elf/address_translation/{translate,error}.rs: declared virtual ranges, load/source extent validation,
  unambiguous source translation and BSS/crossing diagnostics. No runtime addresses or load bias.
- elf/dynamic/tags/kind.rs: recognized tags plus lossless Unknown(i64).
- elf/dynamic/entries/decode.rs: bounded 16-byte little-endian entry decoding.
- elf/dynamic/observation/inspect.rs: PT_DYNAMIC discovery, checked source binding and budgeted scanning.
- elf/dynamic/observation/descriptors.rs: singleton/group consistency and pointer translation.
- elf/dynamic/observation/model.rs: immutable report with read-only access; detached descriptor values.
- elf/dynamic/error/failure.rs: dynamic diagnostics, nesting source/ELF/translation causes.

No new crate, dependency, unsafe code, generic helper home or runtime interface is introduced.
Existing source/header/region/admission/load-plan implementations are unchanged. Six nested module
homes are added to structural policy; generated indexes carry current spans and reviewed test links.

## Translation contract

AddressTranslator::new(&ElfInspection) records PT_LOAD mappings with original program-header indices.
It checks every load's file_size <= memory_size, virtual-memory end without overflow, and full source
prefix against actual immutable bytes. A malformed unrelated load therefore also prevents translator
construction. This validation is required to use unadmitted observations safely; it is not target admission.
It intentionally does not validate alignment or execution permissions.

translate(VirtualAddress, size) first checks request addition. It chooses the unique segment containing
the start, rejects any competing mapping intersecting that owner's requested extent, then requires
the range to remain in one memory extent and its source prefix. Identical overlapping maps are also
ambiguous. Adjacent segments cannot be stitched into one source token even if their file bytes abut.
Query work is linear in load count; no all-pairs overlap preprocessing is needed.

Positive ranges are half-open. Empty requests use closed endpoints: file-end/zero is a valid empty
source token; memory-end/zero in a nonempty BSS tail is ZeroFill. A shared adjacent endpoint for an
empty request is conservatively Ambiguous. A zero-size load may authorize only an empty range at its
start, with a checked empty source extent. Positive reads in pure BSS never yield source bytes.

Result categories: checked BoundSourceRange; ZeroFill for requests beginning wholly beyond the file
prefix within memory; CrossesSourceBoundary for file-to-BSS spans; CrossesMapping for spans leaving
the owning memory extent; Unmapped start; Ambiguous owners; request/load arithmetic overflow;
InvalidLoadSize; and nested Source failures. A request with an unmapped start stays Unmapped even if
later bytes intersect a load. No BSS bytes are synthesized and no source buffers are copied.

The source offset is file_offset + (address - virtual_start). Validated prefix/request bounds prove
this arithmetic safe; SourceArtifact::checked_range constructs the actual identity-bound token.
Translation uses only the ELF-declared image coordinates. ET_DYN base/load-bias and relocated runtime
address attribution require future explicit models; no guessed base or guest-memory dependency.

## Dynamic discovery and termination

observe takes explicit ObservationLimits { max_entries }; there is no default implicit PS5 ceiling.
The budget includes DT_NULL, bounds scanning and retained entries, and may be zero. The caller chooses
an appropriate resource budget; library errors report EntryLimit rather than partial successful results.
No vector is preallocated from an untrusted size/count. Collection grows only as validated entries are
read, bounded by both source extent and caller budget. A caller granting an excessive budget still
accepts ordinary host allocation/work costs; standard allocator OOM behavior is unchanged.

No PT_DYNAMIC returns Absent without requiring a translator. More than one returns MultipleTables,
even for identical descriptors. One table requires file_size <= memory_size, nonoverflowing virtual
memory extent, an actual source range, and translation of its entire declared file range through
PT_LOAD. That token must agree with PT_DYNAMIC's own p_offset; mismatch is a structured failure.
A memory-only table or a range crossing BSS cannot be read. This is source-binding consistency, not
runtime module creation or PT_DYNAMIC payload semantics.

Entries carry index, DynamicTag and u64 value. Unknown signed tags preserve all original bits and
repetitions; their pointers/values are not interpreted by number parity or vendor guesses. Decoding
stops at the first DT_NULL and includes it in the report. Its value is ignored semantically but retained.
Trailing bytes (including a partial padding tail) are not decoded after termination. Before termination,
a short final entry is TruncatedEntry; exact table exhaustion, including a mapped zero-size table,
is MissingTerminator. If the next chunk exceeds budget, EntryLimit wins before reading that chunk.

DynamicTable owns a cloned immutable source handle, full table token, program index, entry list and
validated descriptors. Present boxes the report so Absent does not carry a large inline payload.
Borrowed views are read-only; detached descriptors cannot mutate the report/source or grant admission.
Raw bytes, offsets, and zero-filled test strings are not reinterpreted as names or executable content.

## Descriptor policy

Recognized tags: NULL, NEEDED, STRTAB/STRSZ, SYMTAB/SYMENT, RELA/RELASZ/RELAENT,
JMPREL/PLTRELSZ/PLTREL, INIT/FINI, INIT_ARRAY/INIT_ARRAYSZ and FINI_ARRAY/FINI_ARRAYSZ.
All other tags, including ordinary tags not yet implemented and OS/processor/Sony extensions, remain
Unknown. Unknown preservation means unsupported semantics are visible, not supported linking.

NEEDED repeats preserve order, including repeated equal offsets. Other recognized non-NULL tags are
singletons; any repetition, even equal values, is rejected before descriptor construction. This avoids
silent last-value wins. Absent descriptor groups are allowed; a partly present group is rejected with
present/missing tag context. No values are inferred to fill missing tags. In particular RELAENT alone
(even beside PLT metadata) is conservatively incomplete under M5's RELA triple policy.

| Group | Validation and observation |
|---|---|
| STRTAB + STRSZ | Whole declared byte extent translates to one source prefix; no string decoding |
| SYMTAB + SYMENT | Entry size must be 24; only the first entry's source range is proven. Total count/extent is unknown |
| RELA + RELASZ + RELAENT | Entry size 24; total size divisible by it; whole extent source-backed; no relocation decoding |
| JMPREL + PLTRELSZ + PLTREL | PLTREL selects ordinary REL (17, width 16) or RELA (7, width 24); size divisible by width, extent source-backed |
| INIT_ARRAY/FINI_ARRAY + respective size | Size divisible by ELF64 pointer width 8; extent translated, pointers not read |
| INIT / FINI | Optional observed addresses, including zero; not dereferenced, validated as code or invoked |
| NEEDED | Requires the validated string descriptor and offset < size; no name, encoding or termination claim |

Zero-sized tables/arrays still require a valid empty source range. Symbol observation requires one
whole entry and cannot use an empty extent. Descriptor size/address overflow remains a nested
TranslationError with tag context. Relocation-width checks are format vocabulary only; they perform
no relocation arithmetic or writes. All descriptor pointers use the dedicated translator; fixed
recognized groups keep pointer-validation work bounded independently of repeated/unknown entry count.

## Error boundaries

ElfError remains raw header/program inspection. TranslationError owns mapping/source relationships.
DynamicError covers multiple/invalid/mismatched tables, source/translation causes, budget exhaustion,
missing terminator/partial entry, duplicate tags, incomplete groups, unsupported entry width/PLT kind,
nondivisible sizes and invalid NEEDED offsets. Error::source exposes nested causes; table or tag context
identifies the relevant boundary. No generic success fallback, silent truncation or skipped descriptor.
Rejection remains the existing later admission taxonomy, unchanged by observing a dynamic report.

## Validation, pressure and deferred work

[Generated fixtures](../../crates/astero-loader/tests/dynamic_fixtures/mod.rs) and
[23 integration tests](../../crates/astero-loader/tests/dynamic_observation.rs) exercise translation,
BSS, ambiguous mappings, no/one/multiple tables, ordinary/unknown tags, budgets/termination,
source lifetime, descriptor pairs and boundaries. Sweeps cover 5,445 load/range combinations,
50 dynamic sizes and 25 descriptor pointer/size combinations. See [validation](validation.md).

M4 conservative alignment and quadratic admission overlap checks remain unchanged. Translation adds
no requirement to rewrite them. Future work needs full symbol-table extent evidence, dynamic metadata
completeness and runtime load-bias rules; source/work ceilings remain an explicit caller/system policy.
No string/symbol/relocation parsing, dependency-name resolution, NIDs, Sony interpretation, linker,
SELF, filesystem/mmap, guest mappings, runtime application, migration or guest execution occurs.
Recommended M6: bounded dynamic string observations with explicit termination/encoding/work rules,
using generated bytes only, before names are used for resolution. M6 has not started.
