# M7: bounded dynamic symbol candidates

> Observing and naming a dynamic symbol does not mean Astero has resolved, imported, exported, relocated or executed it.

## Scope and evidence

[SymbolTable](../../crates/astero-loader/src/elf/dynamic/symbol_table/read.rs) owns explicit individual
candidate access under ELF/dynamic/symbol_table. No format-independent import/export contract is
populated, admission guard cleared, source mutated or load plan applied. The source, translation,
M5 descriptor and M6 string-table contracts remain unchanged.

Standard ELF evidence: [generic ABI symbol table](https://gabi.xinuos.com/elf/05-symtab.html), read
2026-09-08. ELF64 entry layout is 24 bytes: u32 name offset, u8 info, u8 other, u16 section index,
u64 value and u64 size. Binding is the high info nibble and type the low nibble. The cited generic
ABI uses three visibility bits; Astero names values 0-3 and preserves higher values as Unknown.
The complete other byte is retained, so processor-specific interpretation can be added later without
losing bits. This is ordinary ELF observation, not evidence of PS5-specific symbol semantics.

## Source binding and count dependency

SymbolTable::new consumes an immutable ElfInspection reference and explicit M5 dynamic-entry budget.
It requires a coherent SYMTAB/SYMENT descriptor; M5 proves only the first 24 bytes are source-backed.
The translator and string-table view derive from that same inspection; callers cannot replace the
address, source identity or table extent independently. The handle borrows the original ELF report;
name views borrow the table, with no whole-file/table or owned-name copy.

read_candidate(index, max_name_scan_bytes) checks index * 24 and base + offset, translates all 24
bytes with M5, and reads only the resulting SourceArtifact token before fixed little-endian decoding.
It performs constant entry work plus explicitly budgeted name scanning. Each call is independent;
there is no aggregate batch API, cache or internally unbounded iteration.

**Candidate access is not symbol-table membership proof.** A caller-requested index may point at
unrelated but source-backed bytes. No count can currently distinguish that situation. symbol_count()
returns CountUnavailable, even after successful reads and even for a null entry. There is no iterator,
caller-supplied trusted count, inferred EOF count or neighbour-address heuristic. Null symbols are not
terminators. Only index zero is structurally required to contain all zeros when requested.

Future bounded hash-table inspection must establish count/extent evidence before enumeration can
exist. DT_HASH and GNU_HASH are not parsed in M7. Their semantics/resource bounds need their own
milestone; nothing here silently treats either raw tag as trusted count evidence.

## Observation and name semantics

[DynamicSymbolObservation](../../crates/astero-loader/src/elf/dynamic/symbol_table/model.rs) retains
index, source token, raw name offset/info/other, binding/type/visibility, section designation, value
and size. Zero/nonzero size and value are uninterpreted observations, not code pointers or allocations.
Local/global/weak and ordinary notype/object/function/section/file/common/TLS kinds are named;
unknown numeric nibbles are preserved. No binding algorithm is implemented.

Section zero is Undefined; ordinary indices are Index (a structural definition reference only).
Absolute, Common, Extended and reserved numeric designations are distinct. Section existence and
extended-index payloads are not validated because section parsing is deferred. These observations
never imply that an undefined symbol is a PS5 import or that a definition is an eligible export.

Name offset zero means None without reading string bytes. Nonzero offsets use M6 lookup within the
complete declared STRSZ and the explicit scan budget. Empty referenced strings are Some(empty),
not absent. Non-UTF-8 bytes remain unchanged; as_utf8 is explicit/fallible. Names are borrowed with
checked source tokens and cannot outlive their table. No DependencyName nonempty constraint is
imposed on symbol names and no normalization, NID interpretation or lossy conversion occurs.

## Failure boundaries

[SymbolError](../../crates/astero-loader/src/elf/dynamic/symbol_table/error.rs) retains existing errors:

- Dynamic: incomplete descriptor, unsupported SYMENT (anything except 24), duplicate descriptor,
  initial source translation and other M5 observation failures retain their original tag context.
- TableUnavailable: absent dynamic/symbol table; CountUnavailable: no trusted enumeration extent.
- IndexOverflow: multiplication or address addition failure with requested index.
- Translation: per-index unmapped/BSS/crossing/overflow/source errors. A truncated candidate fails
  here before decoding; no new duplicate truncation taxonomy conceals the original boundary.
- Source: checked source token read failure.
- StringsUnavailable or Name: symbol index and missing table/name offset or nested M6 diagnostic.
- InvalidNullSymbol: requested index zero has nonzero fields.

Unknown extensions are observations, not malformed failures. No global table ordering, section
consistency, psABI combinations or linker conformance is asserted by individual access.

## Tests and deferred work

[Generated fixtures](../../crates/astero-loader/tests/dynamic_symbols.rs) reuse repository-owned M5
bytes only. Nine tests cover null/unnamed/empty/byte names, defined and undefined symbols, exact field
values, immutable identity and repeated access, error boundaries, index arithmetic, missing/incomplete
metadata and 24-byte truncation. Sweeps cover 1,024 binding/type/other combinations plus five special
section designations and 26 source extents. Existing M2-M6 tests remain unchanged.

See [validation](validation.md) for executed results. No runtime linking, relocation parsing/application,
GOT/PLT, HLE, NIDs, SELF, guest memory, filesystem input, PS5Rust migration or execution is introduced.
Existing alignment/overlap and future load-bias pressures remain open. Recommended M8: bounded hash
metadata and trustworthy symbol-count/extent evidence, before enumeration or import/export derivation.
