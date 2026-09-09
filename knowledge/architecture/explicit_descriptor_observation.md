# Explicit dynamic descriptor observation (M18)

SourceArtifact -> explicit elf::dynamic::descriptors::observe(source, limits) -> immutable
DescriptorObservationReport -> STOP. This is independent of acquisition, header inspection,
raw dynamic observation and linkage. No prior frontend operation or report is required.

## Selected scope and authority

Two existing families prove the bounded metadata boundary without exposing every loader facility:

| Family | Required tags | Established extent |
|---|---|---|
| Strings | STRTAB + STRSZ | Entire declared byte extent translates to immutable source |
| Symbols | SYMTAB + SYMENT | SYMENT must be 24; only the first entry range is proven |

No string terminator/encoding/name is checked. No symbol is decoded or counted. Hash, REL/RELA,
JMPREL, init/fini, NEEDED and unknown tags are outside this selected operation, even if another
Astero API recognizes them. They remain ordered raw evidence; their companions/pointers are not
validated here. Adding other families requires a later explicit scope decision, not an automatic
extension whenever the existing collector gains functionality.

The existing observation/descriptors::collect remains the sole pairing/entry-size/translation
implementation. It now accepts a borrowed iterator; M18 supplies only the four selected tags.
M5 still supplies its complete entry slice, preserving its semantics. M17's bounded discovery was
factored into a private shared helper returning headers/raw table; its public operation still stops
before descriptor collection. No decoder, ELF artifact adapter or payload parser was duplicated.

Loader owns semantics. Core input/descriptors delegates with typed limits/results. CLI descriptors
selects a native path, uses existing acquisition, explicitly requests the operation, then formats the
report. It creates no Session, registry, runtime module or mutable guest state. GUI and dependency
edges are unchanged; loader stays dependency-free.

## Budgets and error precedence

All CLI limits are mandatory, without defaults: max-bytes, max-read-calls, max-program-headers,
max-dynamic-entries and max-descriptors. The first two retain acquisition semantics. Header and raw
entry discovery retain [M17](explicit_dynamic_observation.md) semantics, including budgeted DT_NULL.

max-descriptors counts distinct supported families attempted, not raw tags or payload elements.
Presence of either companion counts the family, even if incomplete or conflicting. At most two
families exist in this selected set; larger caller limits are legal but do not expand the set.
Zero permits absence; any supported family causes Budget failure. An exact limit permits pairing.
All raw entries (including unsupported tags and DT_NULL) consume the raw traversal budget;
unsupported tags consume no descriptor budget. Duplicate tags count one family and then fail the
shared strict pairing rule if the family budget permits interpretation.

Precedence: complete bounded raw discovery, count supported families, refuse insufficient family
budget, run shared pairing/extent checks, retain all records or fail. No prefix is reported Complete.
A family budget refusal does not claim that the groups would otherwise have been valid.

No extra range/traversal knob is needed: selected interpretation performs at most two translated
ranges and a fixed number of linear scans over already bounded headers/raw entries. Selected fields
are at most four unique tags; grouping vectors are at most two values. The new output vector uses
fallible reservation for at most two records and reports Allocation refusal. Existing bounded
header/raw collection policies and fixed-small-map allocation behavior remain as before.

## Pairing, ranges and conflicts

The shared collector rejects duplicate singleton tags, including equal duplicates, with the tag
and both original entry indices. Missing companions fail with present/missing tags. SYMENT other
than 24 fails UnsupportedEntrySize. STRSZ zero is allowed if the empty range translates under the
existing closed-endpoint rule. A zero symbol width is not supported.

Pointers are virtual addresses, never source offsets. Existing AddressTranslator validates checked
address+size arithmetic, PT_LOAD backing, BSS/crossing/unmapped/conflicting mapping cases and source
identity. A translated symbol starting range proves only 24 bytes, not table length or membership.

Distinct family extents may alias or overlap. M5 does not assert cross-family payload exclusivity;
M18 preserves both independently validated ranges without merging or interpreting them. This is
neither a memory mapping conflict nor proof that both payload interpretations would succeed.

## Immutable evidence and statuses

The report privately owns source, budgets, optional raw table and outcome. SourceArtifact sharing
preserves bytes, process-local identity and provenance. Each successful record retains a typed
Strings/TableDescriptor or Symbols/SymbolTableDescriptor value plus its two original dynamic entries
(tag, raw u64 value, original index). Records use fixed Strings-then-Symbols order; raw order remains
available. No raw normalization, UTF-8 name assumptions or payload copies occur.

- Complete: all selected groups passed the existing structural rules; not payload/admission validity.
- Unavailable(NoDynamicTable): bounded discovery found no table.
- Unavailable(NoSupportedDescriptors): a complete raw table has none of the four selected tags.
- Failed: discovery, family budget, shared interpretation/translation, or output allocation failure.

No Partial state. Failed pairing/budget results retain complete raw evidence for auditing original
indices/values; a failed raw traversal has no complete raw table. The report retains source/budgets
in every outcome. Nested causes distinguish headers/raw traversal from DynamicError pairing and
TranslationError/SourceError. Raw audit evidence does not strengthen failed descriptor evidence.

CLI uses exit 0 for Complete/Unavailable and exit 1 for Failed or acquisition/argument errors.
Its output labels the explicit operation, selected families, budgets, source identity/provenance,
original fields and proven ranges, first-symbol-only uncertainty and the stop boundary.

## Validation and future pressure

Tests use generated in-memory M5 fixtures, including nonterminated 0xff string payloads and invalid
symbol bytes that this operation must not interpret. Tests compare selected values against the
existing M5 collector, exercise duplicate/companion failures, 16 family/budget combinations,
81 string-boundary combinations, overlap, raw-request independence and native CLI paths.

No external catalogue was needed; authority is the established [M5 descriptor policy](dynamic_elf_observation.md).
The fixture example has only valid, none and conflict modes and refuses overwrites. No real input,
PS5/NID/ABI behavior, admission, guest memory, resolution or execution was introduced.

A potential M19 is separately requested bounded string-reference observation, with its own explicit
lookup/output limits. It must not become automatic dependency resolution or symbol enumeration.
