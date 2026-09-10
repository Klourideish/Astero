# Astero navigation indexes

Start here to locate implementation, its narrow owner and relevant evidence before scanning source.
These indexes describe Astero only. No PS5Rust, firmware catalogue, NID database or guest ABI data is
imported. M4 ELF inspection is indexed as Astero source. Index generation itself never parses ELF/SELF
input artifacts or changes emulator code.

## Generate and validate

```text
python tools/generate_indexes.py
python tools/generate_indexes.py --check
python tools/check_policy.py
python -m unittest discover -s tools -p "test_*.py" -v
```

check_policy includes index freshness. Regenerate after source, test, ownership, tool or semantic-link
changes. Commit regenerated files with the underlying change only when a commit is authorized.
Do not edit generated JSON/Markdown or manually repair line numbers. README.md is the human-maintained
contract; INDEX_SCHEMA.json and the other indexes are generated. Python standard library only; Cargo
metadata supplies workspace membership/targets. No database, daemon, network or parser dependency.

## Focused views

| View | JSON | Markdown / purpose |
|---|---|---|
| Implementation | implementation.json | [IMPLEMENTATION_INDEX.md](IMPLEMENTATION_INDEX.md): Rust declarations and methods, source spans, logical owners, reviewed relations |
| Subsystems | subsystems.json | [SUBSYSTEM_INDEX.md](SUBSYSTEM_INDEX.md): crate/category -> subsystem -> modules and definitions |
| Modules | modules.json | [MODULE_INDEX.md](MODULE_INDEX.md): logical modules, source owner, implementation home, documented responsibility/forbidden responsibility |
| NIDs | nids.json | [NID_INDEX.md](NID_INDEX.md): observed correlations and explicitly scoped startup registrations; unregistered does not mean HLE implementation |
| ABI | abi.json | [ABI_INDEX.md](ABI_INDEX.md): process-entry arguments and tested synthetic native boundary; neither is a complete PS5 ABI |
| Diagnostics | diagnostics.json | [DIAGNOSTIC_INDEX.md](DIAGNOSTIC_INDEX.md): deliberate host snapshot, support inventory and loader error surfaces |
| Tests | tests.json | [TEST_INDEX.md](TEST_INDEX.md): Rust test functions/doctests and Python unittest declarations; never a passing-test report |
| Source files | SOURCE_INDEX.json | Complete crate Rust inventory plus top-level Python tooling: logical module paths, line counts, declarations, hashes, extraction notes |

Each JSON envelope has schema_version, generator_version, kind, record_count and records. The generated
[JSON Schema](INDEX_SCHEMA.json) describes all eight record families, including empty NID/ABI schemas.
The repository validator also enforces path existence, ranges, hashes, ownership, references, unique
logical IDs, known module homes, status vocabulary and byte-for-byte generated content (normalizing
checkout line endings when checking). No timestamp, absolute host path or Cargo target-directory path
is serialized. Hashes use normalized UTF-8/LF text; generation writes LF with deterministic sorting.

## Identity, ownership and relationships

Rust logical IDs combine crate, Cargo target, physical module path, impl/trait owner and symbol name.
They do not contain line numbers. Moving a leaf file into a same-name mod.rs home preserves its ID;
changing the actual Rust module/type/symbol name changes identity and requires updating reviewed links.
Public re-export aliases are not alternative definition IDs. Impl headers distinguish trait methods.
Source-file IDs are paths; they intentionally change when a file moves. Modules shared by multiple
Cargo targets retain each context, and SOURCE_INDEX lists all module IDs/paths for that physical file.
Inline modules share their enclosing file and have their own ranges. Tooling uses Python AST names.
Doctest IDs use the attached declaration plus fence ordinal; reordering fences can change those IDs.

Category is the crate family (or tooling). Subsystem is the first logical module component; root
entry points use root. Detailed module paths, responsibilities and implementation_home select the
narrowest existing owner. Responsibility comes from module documentation and crate boundaries;
forbidden responsibility comes from the owning crate's architecture table. Follow linked focused
records for semantic boundaries such as libs AGC APIs versus GPU mechanisms; identical names do not
merge owners. Known empty homes remain scaffolded even when a different subsystem is implemented.

Test lexical_identifiers are token observations, not name resolution, call graphs or proof of coverage.
They deliberately do not create semantic links. related_tests/related_implementations contain reviewed
relationships or an attached doctest. Most symbols have no reviewed test association yet: an empty list
means unknown linkage, not proof that the symbol is untested. Tool implementation definitions appear
in SOURCE_INDEX; implementation.json is limited to Rust product contracts and functions.

## Status and evidence

Controlled vocabulary: planned, scaffolded, implemented, tested, runtime_validated. Baseline discovery
marks concrete declarations/bodies implemented and declaration-only methods scaffolded. This means
source exists, not semantic completeness or guest capability. Module/subsystem implemented status
means at least one discovered implementation below that owner; it does not claim all children work.
An executable that only prints a scaffold message still has a real main function, not emulator support.

Test records say test_declared_not_executed. Passing counts belong in
[validation](../architecture/validation.md), never inferred from discovery. Conditional attributes are
not evaluated; generated inventories cover source declarations, not one host's compiled configuration.

[tools/index_links.json](../../tools/index_links.json) is the small reviewed semantic supplement. It
contains symbol IDs, test/diagnostic relationships and evidence links, never source locations. The
initial diagnostic links describe actual M1 observations and M2/M3 errors; loader errors have no GUI/CLI
consumer yet. NID/ABI entries require actual Astero symbols and local provenance. Their source locations
and status are always derived from the implementation record, not supplied by the supplement.

Three tested source reviews and one runtime_validated GUI entry are explicitly scoped to recorded
M3 tests or M1 host-window evidence. Reviews pin symbol and relevant source/test fingerprints. Changed
source automatically removes the promotion and records stale_source, retaining historical evidence.
A renewed tested/runtime claim requires reviewed validation evidence and updated fingerprints; simply
indexing a test never grants it. No guest functionality is promoted by the GUI runtime record.

## Extraction limits

The Rust extractor is a bounded lexer plus item/delimiter walker. It skips nested comments, ordinary/
raw strings and character literals, recognizes ordinary functions/types/traits/constants/statics/type
aliases, tracks impl owners and inline/out-of-line modules, and does not descend into function bodies.
Recorded spans start at declaration visibility/keyword (after attributes/docs) and end at its balanced
body or semicolon. They are navigation metadata, not full compiler syntax spans.

It does not expand macros, resolve imports/re-exports, infer calls, evaluate cfg, index fields/enum
variants/local functions, or model every Rust grammar extension. Opaque/unsupported items get source
extraction notes rather than invented symbols. Module status is scoped to discovered declarations when
notes exist; inspect those notes before judging coverage. #[path]/cfg_attr path overrides, malformed
lexing/delimiters, missing/ambiguous modules and duplicate IDs fail explicitly instead of guessing.
Unlinked Rust files remain in SOURCE_INDEX with discovered names and no fabricated module path.

Rust doctests currently support triple-slash/inner line-doc fences with rust, compile_fail, no_run or
no language tag. Other fences, block-doc tests, doc attributes, macro-generated tests and build-script
outputs are outside this extractor. Python uses ast for exact declaration spans and unittest names;
custom discovery/decorators are not executed. These are future parser/tooling decisions if needed.

## Search examples

```powershell
rg -n 'BoundSourceRange|checked_range' knowledge/indexes/IMPLEMENTATION_INDEX.md
rg -n '"subsystem": "artifact"' knowledge/indexes/implementation.json
rg -n '"status": "implemented"' knowledge/indexes/implementation.json
rg -n '"nid":' knowledge/indexes/nids.json
(Get-Content knowledge/indexes/implementation.json -Raw | ConvertFrom-Json).records |
  Where-Object symbol -eq 'checked_range' | Select-Object id,file,lines,related_tests
```

The NID query deliberately returns no matches (rg exit 1) while its record_count is zero.
The actual bounded source type is BoundSourceRange; there is no CheckedSourceRange type to invent.
If jq is available:

```sh
jq '.records[] | select(.category == "loader" and .symbol == "checked_range") | {id,file,lines,related_tests}' knowledge/indexes/implementation.json
jq '.records[] | select(.subsystem == "artifact") | {id,module_ids}' knowledge/indexes/subsystems.json
jq '.record_count' knowledge/indexes/nids.json
```
