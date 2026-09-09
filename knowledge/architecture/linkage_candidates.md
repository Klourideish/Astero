# Import and export candidates (M10)

> ImportCandidate does not mean resolved import.
>
> ExportCandidate does not mean registered/exported runtime symbol.
>
> Candidate identity is evidence-based, not name-based.

## Ownership and entry point

`astero-loader::elf::dynamic::candidates::enumerate(&symbols, &relocations, limits)`
consumes M8 `SymbolTable::with_hash` and M9 `RelocationTables`. It returns one observation
per trusted symbol in index order, including non-candidates. Import/export views expose
private, read-only evidence; callers cannot inject a detached symbol, edit classification,
or turn an arbitrary M7 indexed read into a candidate. No new dependency is needed.

The narrow owners are `classification/`, `evidence/`, `imports/`, `exports/`, `error/`
and the bounded enumeration wiring. Existing format-independent synthetic `Import` and
`Export` contracts remain unchanged: their module/name/address requirements are not evidence
of resolved ELF relationships. Candidates do not populate them, alter admission requirements,
or mutate inspected artifacts, load plans or runtime state.

## Classification policy

The ordinary ELF basis is the [generic ABI symbol table specification](https://gabi.xinuos.com/elf/05-symtab.html):
local/global/weak binding, undefined and special section indices, and default/hidden/internal/protected
visibility. This is generic ELF evidence, not Sony/PS5 semantics. Protected definitions can be
externally visible; visibility alone does not establish resolution. M10's narrower eligibility
rules below are conservative Astero policy, not claims that other combinations cannot exist.

| Evidence | Classification |
|---|---|
| Index zero, validated by M7 | Null, even when referenced |
| Absolute / common section | Absolute / Common, never automatically exported |
| Reserved / extended section index | Unclassified: special section |
| Unknown binding/visibility or other uninterpreted `st_other` bits | Unclassified: unknown attributes |
| Unknown type, section/file/common type | Unclassified: unsupported type |
| Local plus protected visibility | Unclassified: conflicting attributes |
| Undefined local | UndefinedUnclassified: local binding |
| Undefined non-default visibility | UndefinedUnclassified: non-external visibility |
| Undefined global/weak, default visibility, no-type/object/function/TLS type | ImportCandidate, named or unnamed, with or without relocations |
| Defined ordinary section, local | InternalDefined: local binding |
| Defined ordinary section, hidden/internal | InternalDefined: non-external visibility |
| Defined ordinary section, global/weak, default/protected, eligible type and nonempty name | ExportCandidate |
| Otherwise eligible defined symbol without a name or with empty name | Unclassified: missing/empty name |

Rules run in table order; special section classifications retain all raw attributes rather
than claiming attribute validity. For undefined symbols, ambiguity is `UndefinedUnclassified`;
other uncertainty is `Unclassified`, each with a controlled reason. Ordinary section indices
are observed designations, not proof that an unparsed section exists. Function/TLS kinds are
attributes, not implemented call/TLS behavior. Unknown numeric values remain in M7 observations.

## Evidence and names

Each observation retains the trusted extent (including hash provenance), complete symbol
observation, source token, index, attributes, byte name view and associated raw relocations.
Identity is `(SourceId, symbol_index)` within M3's source-object identity model. Duplicate name
bytes do not merge entries or choose a winner. A candidate's evidence and classification explain
the result without a runtime symbol registry.

M6 string references remain borrowed from immutable source bytes. Name offset zero is `Absent`;
a nonzero offset at a NUL is `Empty`; nonempty bytes are distinguished as `Utf8` or `Bytes` using
explicit validation. All raw bytes remain authoritative. Invalid name offsets/termination fail
through the existing structured string/symbol errors. No normalization, lossy identity, NID
recognition, name hashing, copying of the source or whole string table is introduced.

## Relocation association and limits

The complete M9 canonical relocation inventory is validated against the same trusted symbol
table before candidate iteration starts. A bounded `BTreeMap` groups raw records by symbol index,
not name; within a group M9 ordinary-prefix/PLT order is preserved. M9 alias records occur once
with both labels. Null records remain with the null observation. Ordinary, PLT, both and no
observed use are explicit; they say nothing about calls, GOT ownership or lazy binding.

`CandidateLimits` contains the existing M8 symbol and M9 relocation budgets. Exact counts are
preflighted, then associations grow only from successfully validated records, bounded by the
caller's relocation budget. No allocation reserves an unvalidated count. Candidate iteration
is lazy and fail-stop. Entry/name budget errors are errors, not complete prefixes. A caller
who stops iteration early has not completed enumeration. Successful collection requires reaching
the trusted end without errors. Symbol and relocation name-scan budgets are independent: repeated
relocation uses are charged repeatedly, in addition to the symbol pass. Construction failure
discards associations and returns no candidate stream.

Errors preserve `SymbolError` and `RelocationError` context; a distinct source mismatch reports
both identities. Missing extent, out-of-range relocation references, unavailable REL observation,
and malformed names cannot silently omit evidence. Observational uncertainty is a classification,
not a fabricated hard parsing error.

## Validation and deferred work

[Generated fixture tests](../../crates/astero-loader/tests/linkage_candidates.rs) exercise imports,
exports, duplicates, null/absent/empty/raw names, uncertain attributes, both relocation classes,
aliases, membership, source mismatch, fail-stop errors and budgets. A bounded 192-image sweep
combines attributes, sections and relocation classes with four name states per image.
A compile-fail doctest checks that public consumers cannot assign classification.
See [validation](validation.md) for actual results; indexing alone proves neither tests nor runtime behavior.

No M6-M9 API correction was needed. Future pressure: the association inventory uses memory
proportional to the explicit relocation budget; streaming large inputs may need a different
association contract. Generic classification of absolute/common and extension attributes remains
deferred, as do section validation and machine-specific semantics. Existing conservative alignment,
quadratic admission overlaps and parser resource-policy decisions remain unchanged.

A bounded M11 option is an immutable candidate evidence report for CLI/debugger consumption,
with explicit completeness and resource status. It should preserve this evidence model and keep
resolution separate. NIDs, firmware matching, dependency/symbol resolution, registration, relocations,
load bias, SELF, guest memory and execution require their own later authorization and evidence.
