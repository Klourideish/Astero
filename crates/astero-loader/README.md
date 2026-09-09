# astero-loader

Binary parsing, load plans, relocations and import metadata.

## Interfaces and status

M2 implements synthetic metadata inspection, explicit admission and immutable load planning.
M3 requires immutable source bytes via artifact::SourceArtifact; identity and length are source-owned.
Copy work retains validated source-identity/range tokens and the plan retains the source lifetime.
See [source binding](../../knowledge/architecture/source_binding.md).
Use artifact::inspect, admission::admit and load_plan::plan in order.
M4 adds elf::inspect(source, module_context): bounded ELF64 little-endian header/program-header inspection.
Use report.artifact() with existing admission/planning. Synthetic and generic ELF x86-64 descriptions
may be admitted; no target is applied or executed. See [ELF scope](../../knowledge/architecture/elf_inspection.md).
M5 adds elf::address_translation::AddressTranslator and elf::dynamic::observe(&report, limits).
Dynamic reports retain source-bound descriptors; they neither interpret tables nor clear admission
requirements. See [dynamic observation](../../knowledge/architecture/dynamic_elf_observation.md).
M6 adds elf::dynamic::dependencies::observe and reusable string_table views. DT_NEEDED becomes
Dependency::Named with exact owned bytes; original module-ID declarations use Dependency::Module.
No names are resolved and no ELF admission guard is cleared. See [dynamic strings](../../knowledge/architecture/dynamic_strings.md).
See the [loader pipeline](../../knowledge/architecture/loader_pipeline.md) for invariants, errors and limits.

## Module ownership

[admission/](src/admission/mod.rs), [artifact/](src/artifact/mod.rs), [dependencies/](src/dependencies/mod.rs), [elf/](src/elf/mod.rs), [exports/](src/exports/mod.rs), [imports/](src/imports/mod.rs), [load_plan/](src/load_plan/mod.rs), [metadata/](src/metadata/mod.rs), [modules/](src/modules/mod.rs), [relocations/](src/relocations/mod.rs), [self_format/](src/self_format/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner. ELF has dedicated header, translation and dynamic child owners; SELF remains an unused scaffold.
Implementation lives in focused child files; module roots only declare/re-export contracts.

Forbidden: Executing guest code or owning session lifecycle.

The loader is dependency-free, enforced by repository policy. Run `cargo test -p astero-loader`
for synthetic invariant tests and immutable-boundary compile-fail doctests.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M7 adds elf::dynamic::symbol_table::SymbolTable for bounded candidate reads and byte names.
Enumeration requires future trusted count evidence; see [dynamic symbols](../../knowledge/architecture/dynamic_symbols.md).

M8 adds hash::observe and SymbolTable::with_hash/enumerate; see [hash extents](../../knowledge/architecture/elf_hash_extents.md).

M9 adds relocations::RelocationTables for bounded RELA records and trusted symbol references.
See [relocation observation](../../knowledge/architecture/elf_relocation_observation.md).

M10 adds elf::dynamic::candidates::enumerate for trusted, budgeted import/export candidate classification.
See [candidate policy](../../knowledge/architecture/linkage_candidates.md); no runtime resolution.

M11 adds candidates::report::collect with owned bounded evidence and explicit completeness.
See [linkage reporting](../../knowledge/architecture/linkage_reporting.md).
