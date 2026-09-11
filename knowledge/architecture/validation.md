# Validation

Run from the workspace root:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python tools/check_policy.py
python -m unittest discover -s tools -p "test_*.py" -v
git diff --check
# If Git is unavailable:
python tools/check_whitespace.py
```

Python 3.9+ is needed for the standard-library-only policy checks. Cargo metadata inspects
all declared dependency kinds and target conditions, including renamed packages. The checker
enforces the allowlist, core/debug direction, HLE direction, member identity and actual cycles.
Integration tests belong under owning crates; root tests/ is only navigational.

## M0 results â€” 2026-09-08

Environment: Windows; rustc 1.97.0 (2d8144b78), Cargo 1.97.0 (c980f4866), Python 3.14.6.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed; 15 members |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | passed: 2 debugger rejection integration tests; 0 unit tests; 0 doctests |
| python tools/check_policy.py | passed: 15 members, 3 internal edges, valid active state |
| policy unittest suite | passed: 8 tests |
| CLI, GUI and GPU smoke cargo run | passed; each reports scaffold limitations |
| git status --short; git diff --check | unavailable: C:\Astero has no .git repository |
| supplemental source whitespace scan | passed; does not replace Git diff validation |

The initial policy unittest run found a helper parameter collision with Cargo's target field;
the helper was corrected and all 8 tests rerun successfully. No Rust validation failures occurred.

There are 12 library targets and 3 executable targets. Actual dependencies are CLI â†’ core,
GUI â†’ core and libs â†’ HLE; there are no external Rust dependencies.
The debugger tests validate explicit unsupported responses only. There are **zero emulator tests**;
these results establish buildability and scoped policy behavior, not emulator correctness or regression safety.
No prototype code, external catalogue or real binary was copied or executed. No commit or push occurred.


## M1 results — 2026-09-08

M0 cleanup was explicitly approved: its active item was removed, while the M0 evidence above remains.
M1 uses the same Windows/Rust/Cargo/Python environment reported above.

| Required check | Actual result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | passed: 15 tests, 0 failures, 0 ignored; 0 doctests |
| python tools/check_policy.py | passed: 15 members, 6 internal edges; active-state schema valid |
| python -m unittest discover -s tools -p test_policy.py -v | passed: 16 tests |
| git status --short | unavailable: not a Git repository; git diff validation remains unavailable |
| python tools/check_whitespace.py | passed; supplemental only |

Rust breakdown: 6 core session integration tests; 3 debugger integration tests; 2 CLI integration tests
(including a real executable invocation); 2 GUI adapter integration tests; 2 pure GUI surface-extent
unit tests. These verify lifecycle rejection, truthful unloaded state, unique process-local identity,
coherent detached observations, observer expiry, frontend derivation and unsupported capabilities.
The policy suite adds checks for debug/core direction, GUI isolation, rejected frameworks, approved
cleanup, schema and evidence references. Cargo metadata checks declared normal/dev/build/target edges.
The final lockfile also contains none of eframe, egui, Glow, glutin or wgpu.

## GUI runtime evidence

Launched target/debug/astero-gui.exe on the visible Windows desktop. Runtime output:

```text
GUI Vulkan device: NVIDIA GeForce RTX 4070
GUI Vulkan: first frame presented
```

Computer-use visual inspection confirmed a real window with session-1, Ready, No guest loaded,
one host lifecycle change, all 8 subsystem entries, and the complete debugger capability inventory.
Clicked Diagnostics and observed it collapse. Maximized the window and observed a correctly redrawn
layout; restored it and observed the original layout. Closed with Alt+F4; process exited with code 0.
This validates host GUI initialization, draw/present, basic input, resize/recreation and clean shutdown
on this host. It does not establish Vulkan validation-layer cleanliness, driver-loss recovery,
other-platform support, all DPI configurations or emulator correctness.

Initial implementation checks caught a missing mutable closure binding in the Winit callback; it was
corrected before the passing check/build/Clippy/test runs. The rejected eframe path was replaced,
not runtime-validated or retained as a fallback.

There are **zero emulator tests**. No guest binaries, PS5Rust implementation or catalogues were migrated
or executed. No commit, push or project licence selection occurred.

## Approved cleanup and Git baseline

M1 cleanup and Git initialization were explicitly approved after the validation above.
The active list is now empty; durable M0/M1 records remain. Policy negative tests use a
synthetic fixture independent of active work. All 16 policy tests, live state/dependency
validation and supplemental whitespace checks passed. Git staged whitespace validation
now passes for the baseline. Build outputs and Python caches are excluded.

## Bounded structural scaffolding — 2026-09-08

Started from clean main at baseline 304e600. Declared 148 directory module homes across 13 crates;
ABI/CLI retain their small existing structure. Existing M1 code moved into narrower physical owners,
with compatibility re-exports preserving public paths. See [module structure](module_structure.md)
for the complete ownership/move inventory. No M2 work or emulator functionality was added.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including the real GUI executable |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 15 passed, 0 failed, 0 ignored; 0 doctests |
| python tools/check_policy.py | passed: 15 members, 6 internal edges, 148 module homes, valid active state |
| python -m unittest discover -s tools -p "test_*.py" -v | 25 passed: 16 existing policy tests + 9 structural tests |
| python tools/check_whitespace.py | passed across tracked and new source/config/docs |
| git diff --check | passed; supplemental check also covers untracked new files |

The unchanged Rust tests comprise 6 core session, 3 debugger, 2 CLI, 2 GUI adapter integration tests,
and 2 pure Vulkan extent unit tests. No new emulator tests or functionality were introduced.
Source comparison of 14 relocated/split M1 implementation/type files matched the baseline after
excluding imports, comments and whitespace. Cargo metadata dependency declarations match the
pre-pass snapshot exactly. Manifests, Cargo.lock and the dependency allowlist are unchanged.

No new GUI runtime launch was performed for this structural pass: GUI compilation and the unchanged
M1 vertical-slice tests passed; the earlier visual runtime evidence remains scoped to the M1 run.
No real binary parsing/execution, PS5Rust migration, admission, HLE, AGC, PM4, register, shader,
VideoOut, audio or expanded debugger behavior was added. No commit or push was performed.

## Final structural gap audit (2026-09-08)

Against d60f7d5: four documentation-only module roots, declarations, bounded ownership notes and
inventory/tracking changes. No M1 implementation changes or new tests were needed for these empty roots.

- cargo fmt --all -- --check: passed after formatting the new declaration order.
- cargo check --workspace --all-targets: passed, including GUI targets.
- cargo clippy --workspace --all-targets -- -D warnings: passed.
- cargo test --workspace: 15 passed, 0 failed (core 6, debug 3, CLI 2, GUI 4); 0 doctests.
- python tools/check_policy.py: passed; 15 crates, 6 internal edges, 152 module homes, valid active state.
- python -m unittest discover -s tools -p "test_*.py" -v: 25 passed (16 policy, 9 structure).
- python tools/check_whitespace.py and git diff --check: passed. Git emitted only line-ending conversion notices.

Manifests, lockfile and dependency policy are unchanged. No guest implementation, parsing, execution
or migration was introduced; these results do not establish emulator correctness. GUI compilation and
its four tests passed; GUI runtime launch was not repeated for this structural-only audit. No commit or push.
See [audit classifications and evidence](structural_gap_audit.md).

## M2 synthetic admission/load-plan contracts — 2026-09-08

Started from clean main at e444215. M2 changes only loader implementation/tests, focused architecture
and README records, active state, and dependency-policy tooling. ELF/SELF homes and every other
crate's implementation are unchanged. No source bytes or prototype material were inspected/migrated.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 34 passed total: 31 executable tests + 3 compile-fail doctests; 0 failed/ignored |
| python tools/check_policy.py | passed: 15 packages, 6 internal edges, valid active state |
| python tools/check_structure.py | passed: unchanged 152 declared module homes |
| python -m unittest discover -s tools -p "test_*.py" -v | 27 passed: 18 dependency/state + 9 structure tests |
| python tools/check_whitespace.py | passed |
| git diff --check | passed |

Rust additions: 16 loader integration tests and 3 compile-fail doctests. The integration tests cover
all rejection families, deterministic multi-region plans, deferred descriptors, source/alignment/
permission preservation, immutable observations and a 1,056-case file/memory-size partition sweep
(counted as one test). The unchanged M1 suite contributes 15 tests: core 6, debug 3, CLI 2, GUI 4.
Policy additions reject loader runtime/convenience dependencies and speculative dependency permissions.
The focused and full validation runs passed; formatting was applied before the final format check.

No Cargo manifests or Cargo.lock changes, no external dependencies and no actual internal-edge
changes. Policy now requires loader to remain dependency-free, removing its previous unused ABI/memory
permissions. Direct review confirms planning only creates owned values: no I/O, session references,
guest allocations, mappings, relocation evaluation, symbol resolution or runtime callbacks exist.
GUI runtime launch was not repeated; its build and existing tests passed. These are synthetic contract
checks, not proof of PS5/emulator correctness. No commit, push or M3 work occurred.

See [loader pipeline](loader_pipeline.md) for ownership, exact invariants and deferred design pressure.

## M3 immutable source binding — 2026-09-08

Started from clean main at 19f8e6d. Scope: loader source/inspection/admission/plan contracts, loader tests,
README/architecture records and active state. No other crate implementation, Cargo manifest, lockfile,
dependency policy or structural inventory changed. No real file or prototype material was accessed.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 49 passed: 41 executable tests + 8 compile-fail doctests; 0 failed/ignored |
| python tools/check_policy.py | passed: 15 packages, 6 internal edges, valid active state |
| python tools/check_structure.py | passed: unchanged 152 module homes |
| python -m unittest discover -s tools -p "test_*.py" -v | 27 passed: 18 dependency/state + 9 structure |
| python tools/check_whitespace.py | passed |
| git diff --check | passed; only Windows line-ending conversion notices |

Preserved: all 16 M2 integration tests and three M2 compile-fail doctests, adapted to actual source
buffers and nested source errors; all 15 M1 tests remain unchanged. Added: nine source integration tests,
one isolated identity-exhaustion unit test and five compile-fail doctests. New coverage includes
shared ownership, source retention after other handles drop, token mismatch, immutable public APIs,
non-spoofable input identity/length, truncation, exact/empty/overflow boundaries, concurrent identity
allocation/reads and deterministic plans. A 6,137-case range sweep compares to native slice bounds;
the retained M2 size sweep covers 1,056 combinations. Sweeps count as one test each, not thousands.

The source object owns private immutable boxed bytes through Arc and IDs cannot wrap. Review confirms
all admission source ranges use SourceArtifact::checked_range; plans retain a matching source and copy
tokens, with zero-fill separate. No I/O, parsing, guest application, relocation evaluation, resolution
or session/runtime mutation occurs. Source creation alone advances a process-local ID counter.
No dependencies or actual internal edges were added. GUI runtime launch was not repeated.

These checks establish in-memory source-binding contracts, not PS5/emulator correctness, recoverable
allocator exhaustion, persistent identity or a complete hostile-input resource policy. See
[source binding](source_binding.md) for those explicit limits. No commit, push or M4 work occurred.

## Indexing foundation before M4 — 2026-09-08

Started from clean main at a28b04c with M3 already committed/cleaned up. Changes are limited to indexing
and policy tooling, generated knowledge/indexes, repository navigation instructions and active state.
No crate Rust source, Cargo manifest, lockfile, dependency policy or module-home inventory changed.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 49 passed: 41 executable tests + 8 compile-fail doctests; 0 failed/ignored |
| python tools/generate_indexes.py --check | passed; all generated indexes current |
| python tools/check_policy.py | passed: index freshness, 15 crates, 6 internal edges, valid active state |
| python tools/check_structure.py | passed: 152 declared homes |
| python -m unittest discover -s tools -p "test_*.py" -v | 47 passed: 27 existing policy/structure + 20 new index/extraction tests |
| python tools/check_whitespace.py | passed |
| git diff --check | passed |

Two consecutive generator executions produced byte-identical output across all 16 generated files
(eight data JSON files, seven Markdown views and INDEX_SCHEMA.json). Aggregate SHA-256, calculated
from sorted filename + NUL + raw file bytes:
4b345d6395b9c4fba8237a9181adf289f8dfd2e58d3da1dd40404fe07db10ece.
README.md is the maintained indexing contract and is not a generated output.

Record counts: implementation 141; subsystems 113; modules 215; NIDs 0; ABI 0; diagnostics 5;
tests 96 (49 Rust declarations/doctests + 47 Python unittest declarations); sources 212 (204 Rust +
8 Python tools). All 152 required homes are represented. Current sources produce zero unsupported or
unlinked extraction notes. Implementation statuses are 135 implemented, 2 scaffolded method declarations,
3 tested and 1 runtime_validated. The four promotions come from explicitly scoped, fingerprint-pinned
M1/M3 evidence, not from indexing or the existence of tests. No GUI runtime launch occurred in this pass.

Search checks: BoundSourceRange/checked_range returns four implementation Markdown rows; the artifact
subsystem query returns 32 implementation records; implemented returns 135 records. The NID field query
returns zero matches (expected rg exit 1). The documented PowerShell JSON filter resolves checked_range.
jq examples are documented as optional and were not executed. Passing results above are separate from
TEST_INDEX's declaration inventory and do not establish guest correctness.

Index tests cover deterministic output, source movement with stable symbol IDs, stale content/hashes,
paths/ranges, empty homes, duplicate IDs, statuses, source ownership, missing homes, line endings,
unlinked sources, zero semantic categories, stale reviews, read-only generated locations, repository
freshness, comments/literals, impl/trait/inline modules, opaque macros, unsupported path overrides and
doctest fences. An initial CRLF test fixture accidentally doubled existing CR characters; it was corrected
and all 20 index tests and the combined 47-test suite passed. Initial diagnostic links also exposed a
duplicate related-test list; generation now deduplicates those relations before validation.

No dependencies were added. No external catalogue/NID/ABI data, emulator functionality, real binary
parsing, migration, guest execution, commit, push or M4 work was introduced. See
[index contract and limits](../indexes/README.md).

## M4: bounded ELF64 header/program-header inspection — 2026-09-08

Started from clean main at fedd9e2. Only loader Rust code/tests, architecture/navigation records,
semantic index links, generated indexes and active state changed. No dependencies, manifests,
lockfile or internal edges changed: loader remains dependency-free; workspace has 15 crates/6 edges.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including existing GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 56 executable tests + 8 compile-fail doctests passed; 0 failed/ignored |
| python -m unittest discover -s tools -p "test_*.py" -v | 47 passed: 27 policy/structure + 20 index/extraction |
| python tools/check_policy.py | passed: dependency/state/structure and index freshness |
| python tools/check_whitespace.py | passed, 297 files |
| git diff --check | passed |
| python tools/generate_indexes.py --check | passed |

All 16 M2 and 9 M3 integration tests, the source-ID unit test and 8 doctests remain. M2's obsolete
Elf-is-always-unsupported expectation was removed for the deliberate format-gate extension; all other
admission checks are unchanged. Fifteen new ELF integration tests cover generated header-only,
executable/module, RX/RW, BSS and PT_NULL cases; identification/header/table errors; unsupported
machine/role; source/size/alignment/overlap/entry admission failures; and explicit deferred semantics.
Sweeps cover 129 source lengths, 72 table offset/count combinations and 49 segment extents; a separate
fixture iterates the maximum 65,534 ordinary PT_NULL entries. No real input or guest runtime is tested.

Two consecutive generator runs produced byte-identical output across 16 generated files. Aggregate
SHA256 of sorted filename + NUL + bytes:
198a982ba65c95fba694c9bd232500f8715b4eab7c5806ce42bd2970f59a6aa7.
Index counts (before -> after): implementation 141 -> 165; subsystems 113 -> 114; modules 215 -> 228;
diagnostics 5 -> 6; tests 96 -> 111; sources 212 -> 225; NIDs and ABI remain zero. Total 849 records.
Five new required ELF child homes bring structural policy to 157. All 24 ELF symbols resolve to
loader/elf; rg ELF/subsystem searches each found 24 implementation rows. Extraction notes remain empty.
Reviewed semantic links connect parser/admission/planning symbols to the new tests and ElfError to
its inspection consumer. No new runtime_validated claim is made; indexed tests are not execution evidence.

During development, initial private re-export visibility failed compilation and was corrected to
ELF-local visibility. The original M2 rejection fixture then exposed the intentional format-gate
change described above. Subsequent loader and complete workspace runs passed. These results establish
bounded byte inspection and synthetic contract behavior, not complete ELF/PS5 support or emulator
correctness. GUI runtime launch was not repeated. See [ELF scope and pressures](elf_inspection.md).
No SELF, dynamic parsing, filesystem adapter, memory application, migration, guest execution, commit
or push occurred. Recommended M5 remains a proposal only.

## M5: dynamic ELF observations and address translation — 2026-09-08

Started from clean main at 557a7c1. Added dedicated loader/elf/address_translation and dynamic nested
owners, generated fixtures/tests, focused architecture and regenerated indexes. Source binding,
M4 header/program/adapter code, admission and load-plan implementations are unchanged. No Cargo
manifest/lockfile/dependency-policy changes: 15 crates, six internal edges, dependency-free loader.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 79 executable tests + 8 compile-fail doctests passed; 0 failed/ignored |
| python -m unittest discover -s tools -p "test_*.py" -v | 47 passed: 27 policy/structure + 20 index/extraction |
| python tools/check_policy.py | passed: dependency/state/structure and index freshness |
| python tools/check_structure.py | passed: 163 declared module homes |
| python tools/check_whitespace.py | passed: 314 files |
| git diff --check | passed |
| python tools/generate_indexes.py --check | passed after each of two consecutive regeneration runs |

All prior tests are preserved unchanged. Twenty-three new integration tests exercise translation
source identity, start/interior/end, BSS/pure BSS, overflow, conflicting and adjacent maps, malformed
load extents, absent/multiple/source-inconsistent dynamic tables, unknown tags and repeated NEEDED,
termination/entry budgets, duplicate/incomplete groups, descriptor widths/extents, initialization
observations, source lifetime and continued admission rejection. Sweep oracles cover 5,445 load/range
combinations, 50 dynamic table sizes and 25 descriptor pointer/size combinations. No names, symbols
or relocations are semantically parsed and no bytes/runtime state are patched or executed.

Two consecutive generator executions and checks produced byte-identical output across 16 generated
files. Aggregate SHA256 of sorted filename + NUL + raw bytes:
85549063e05a27c567001ae4772e131ff94fcf3120f226fefb9e9f5e2523317d.
Index counts before -> after: implementation 165 -> 198; subsystems 114 -> 115; modules 228 -> 244;
diagnostics 6 -> 8; tests 111 -> 134; sources 225 -> 241; NIDs and ABI remain zero. Total 940 records.
All 33 M5 symbols resolve to astero-loader/elf and their narrow module paths; the Markdown grep query
found those 33 rows. Extraction notes remain empty. Reviewed links associate translator/observer/
decoder/descriptor symbols with the new tests and expose separate translation/dynamic error records.
Indexed declarations are not passing-test or runtime-validation claims.

The first focused 23-test run passed; Clippy then flagged a collapsible conditional and the large
present/absent report enum. The conditional was collapsed and Present now boxes its fixed-size report.
The final workspace tests and all checks above passed after those changes; no warnings were suppressed.
GUI still builds; GUI runtime launch was not repeated. Evidence proves generated-byte observation
contracts, not ELF/PS5 linking completeness or emulator correctness. Existing alignment and quadratic
admission pressures remain; dynamic scan budgets are explicit caller input. See
[dynamic observation scope](dynamic_elf_observation.md).

No new dependencies, linking, strings/symbols/relocations interpretation, NIDs, Sony tag interpretation,
SELF, filesystem/mmap, guest memory, HLE, runtime application, PS5Rust migration, guest execution,
commit or push occurred. M6 has not started.

## M6: dynamic strings and dependency declarations — 2026-09-08

Started from clean main at e102a2f. Added loader/elf/dynamic/string_table and dependencies owners,
generic DependencyName, corrected Dependency variants, generated tests and focused architecture/index
records. Existing source binding, address translation, M5 descriptor observation and load-plan code
are unchanged. Admission retains the module-ID checks and permits ordered/repeated constructor-valid
Named declarations; no ELF admission requirement is removed. M2 fixtures use the new Module spelling.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 97 executable tests + 9 compile-fail doctests passed; 0 failed/ignored |
| python -m unittest discover -s tools -p "test_*.py" -v | 47 passed: 27 policy/structure + 20 index/extraction |
| python tools/check_policy.py | passed: dependency/state/structure and index freshness |
| python tools/check_structure.py | passed: 165 declared module homes |
| python tools/check_whitespace.py | passed: 324 files |
| git diff --check | passed |
| python tools/generate_indexes.py --check | passed after each of two consecutive regeneration runs |

Eighteen new integration tests and one name-immutability doctest cover absent/empty/NUL-only tables,
bounded suffixes, exact terminal NUL, non-UTF-8 identity, duplicate/order/provenance preservation,
missing termination beyond DT_STRSZ, empty-name rejection, descriptor conflicts and prior M5 failures,
scan budgets including repeated references, lifetime and generic admission/plan metadata preservation.
Sweeps exercise 146 offset/budget combinations across six tables, 13 repeated dependency counts and
12 source-boundary pointer/size pairs. All prior tests remain; only three M2 fixture constructions were
adapted from the old struct to Dependency::Module. The generic pipeline test uses a separate synthetic
fixture and confirms named declarations cannot satisfy ModuleId imports; ELF guards stay intact.

Two consecutive generator/check cycles produced byte-identical output across 16 generated files.
Aggregate SHA256 of sorted filename + NUL + raw bytes:
2f0e1832388d5b0d59472d676275ca1c477d905f2bb58300f62c41f8c8c5ec61.
Index counts before -> after: implementation 198 -> 228; subsystems 115 -> 115; modules 244 -> 255;
diagnostics 8 -> 11; tests 134 -> 153; sources 241 -> 250; NIDs and ABI remain zero. Total 1,012 records.
All 30 added implementation symbols resolve to astero-loader and the ELF/dependencies owners. The
string_table/dynamic::dependencies/dependencies::name Markdown search returned those 30 rows.
Extraction notes remain empty. Reviewed test/diagnostic links locate byte names, string lookup and
NEEDED adaptation without hand-maintained spans. No new runtime-validation claim is made.

The initial focused 18-test run passed with two unused test imports; Clippy correctly rejected those
imports under -D warnings. They were removed without lint suppression, and the final full validation
above passed. GUI runtime launch was not repeated. Tests establish generated-byte/name contracts, not
ELF linking completeness, dependency availability or emulator correctness. No full-table conformance
claim is made for unreferenced strings. See [string/name scope](dynamic_strings.md).

No dependencies, Cargo manifests/lockfile or internal edges changed: dependency-free loader, 15 crates,
six internal edges. No dependency loading/search, HLE/provider/sysmodule lookup, NIDs, SONAME/search-path
interpretation, symbol/relocation parsing, GOT/PLT behavior, SELF, filesystem/mmap, guest memory, runtime
modules, PS5Rust migration, guest execution, commit or push occurred. M7 has not started.

## M7 validation: bounded dynamic symbol candidates (2026-09-08)

All requested Cargo checks passed: fmt --all -- --check, check/build --workspace --all-targets,
clippy --workspace --all-targets -- -D warnings, and test --workspace. Build includes GUI; no GUI
runtime launch occurred. Exact results: 106 executable tests and 9 compile-fail doctests passed,
zero failed/ignored. Nine new generated-fixture tests; previous 97 tests and all doctests retained.

Python unittest discovery passed 47 tests: 27 policy/structure plus 20 index/extraction. Policy/state
validation passed: 15 crates, six internal edges, 166 module homes. Supplemental whitespace passed
330 files; git diff --check passed. No manifest, lockfile, dependency or edge changes.

Two consecutive index generation/check cycles passed with 16 byte-identical generated files.
Aggregate SHA256 (sorted filename + NUL + raw bytes):
1263e24bc6189eb040c506d784388a4fe13794c48ddb4c4f0f13d54f367a07e3.
Counts before -> after: implementation 228 -> 239; subsystems 115 -> 115; modules 255 -> 261;
diagnostics 11 -> 12; tests 153 -> 162; sources 250 -> 255; NIDs and ABI zero. Total 1,044 records.
All 11 symbol_table implementation rows appear under loader/ELF; reviewed diagnostic/test links
connect candidate access and count refusal. Initial generic-impl link spelling was rejected by the
generator, then corrected to the discovered ID before successful validation.

Sweeps cover 1,024 binding/type/other combinations, five special section designations and 26 source
extents. Tests establish bounded candidate/name contracts, not table membership, linking or emulator
correctness. Source/translation/admission/load-plan mechanisms remain unchanged.
No hash-table interpretation, relocation parsing/application, import/export derivation, NIDs, HLE,
SELF, guest memory, runtime linking, migration or execution occurred. M8 has not started. See
[dynamic symbols](dynamic_symbols.md) for count dependency and scope.

## M8 validation: hash metadata and trusted symbol extents (2026-09-08)

Scope: nested hash observation/sysv/gnu/extent/error owners, optional M7 enumeration integration,
tags, generated fixtures, architecture and indexes. No manifests, Cargo.lock, dependency policy
or internal edges changed: 15 packages, six edges, dependency-free loader.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 122 executable tests + 10 compile-fail doctests passed; zero failed/ignored |
| python -m unittest discover -s tools -p "test_*.py" | 47 passed: 27 policy/structure + 20 index/extraction |
| python tools/check_policy.py | passed, active state valid, fresh indexes |
| python tools/check_structure.py | passed, 172 module homes |
| python tools/check_whitespace.py | passed, 348 files |
| git diff --check | passed |
| two generate_indexes.py / --check cycles | passed; 16 files byte-identical |

New coverage: 16 executable tests and one extent-privacy doctest. Prior M2-M7 tests remain intact.
Sweeps include 64 SysV count pairs, 32 GNU termination/source-boundary cases and exact-end/one-byte
short symbol and hash boundaries. Maximum u32 counts are widened rather than wrapped; actual source
bounds reject oversized arrays. Fixtures cover source identity, absent/partial/agreed/conflicting
evidence, cycle/termination failures, budgets and fused iteration errors. An initial test punctuation
error was fixed before successful compilation; no lint or validation rule was weakened.

Index counts before -> after: implementation 239 -> 282; subsystems 115 -> 115; modules 261 -> 279;
diagnostics 12 -> 13; tests 162 -> 179; sources 255 -> 272. NIDs/ABI remain zero. Total 1,140 records.
All 43 hash/enumeration implementation rows are searchable under loader/ELF; reviewed test and
diagnostic relationships are recorded without manual spans. Aggregate SHA256 of sorted filename +
NUL + raw bytes for the 16 generated files:
a9d0b672ec2f37a23bfcbd39f637565c02689af9f8b069f0fdd74dc798a73b9b.

No GUI runtime launch was performed; this evidence is generated-input contract validation, not
linker/hash-name correctness or emulator correctness. Empty GNU buckets remain partial evidence,
and the original M7 constructor retains enumeration refusal. Source bounds, admission guards and
immutable planning are preserved. No import/export derivation, dependency resolution, symbol lookup,
NIDs, relocations, PLT/GOT, HLE, SELF, guest memory, migration or execution occurred. M9 not started.
See [hash contract](elf_hash_extents.md) for conservative support and remaining resource pressure.

## M9 validation: bounded relocation observations (2026-09-09)

Scope: seven nested ELF relocation homes, module export, generated fixtures, architecture/ownership
records and indexes. Earlier source, translation, dynamic descriptor, hash and symbol implementations
remain unchanged. No manifest, lockfile, dependency or internal-edge changes: 15 packages, six edges,
dependency-free loader. No commits or pushes were made during milestone implementation.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 137 executable tests + 11 compile-fail doctests passed; zero failed/ignored |
| python -m unittest discover -s tools -p "test_*.py" | 47 passed: 27 policy/structure + 20 index/extraction |
| python tools/check_policy.py | passed; active state valid and indexes fresh |
| python tools/check_structure.py | passed, 179 declared module homes |
| python tools/check_whitespace.py | passed, 365 files |
| git diff --check | passed |
| two generate_indexes.py / --check cycles | passed; 16 generated files byte-identical |

New coverage: 15 executable tests and one trusted-relocation-extent privacy doctest. All prior tests
remain intact. Sweeps exercise 74 table sizes, six indices and 100 symbol/type/signed-addend tuples.
Fixtures cover source identity, missing evidence, null/highest-valid/out-of-range references, name
failures, PLT/REL deferral, exact-end/truncated/overflow descriptors, aliases/conflicts and budgets.
Aliases enumerate once with both provenance labels; failed reads/iteration cannot claim completion.

Index records before -> after: implementation 282 -> 320; subsystems 115 -> 115; modules 279 -> 296;
diagnostics 13 -> 14; tests 179 -> 195; sources 272 -> 288. NIDs/ABI remain zero. Total 1,228.
All 38 ELF relocation implementation records are searchable with reviewed test/diagnostic links.
Aggregate SHA256 of sorted filename + NUL + raw bytes across 16 generated files:
723ccd0652cfce077e9d118d858231187c623271880b6f1d492691421b733ec5.

No GUI runtime launch was performed. Tests establish generated-byte structural/reference contracts,
not relocation applicability, import/export semantics, linking or emulator correctness. No application,
symbol/dependency resolution, NIDs, PLT/GOT patching, runtime addresses, guest memory, HLE, SELF,
PS5Rust migration or execution occurred. M10 has not started. See
[relocation contract](elf_relocation_observation.md) for support boundaries and remaining pressure.

## M10 results — 2026-09-09

Bounded linkage candidate derivation only; no resolution, runtime registration, NIDs,
relocation application, external binaries or emulator migration. Prior M2-M9 APIs/tests remain
unchanged. Working tree started clean at e711de6; no commit or push performed.

| Required check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 147 executable Rust tests, 12 compile-fail doctests passed; zero failures/ignored |
| python tools/check_policy.py | passed: 15 members, 6 internal edges, active state valid |
| structure policy | passed: 185 declared module homes |
| python -m unittest discover -s tools -p "test_*.py" -v | 47 passed: 18 policy + 9 structure + 20 index/extraction |
| index generation/freshness | passed; two regenerations byte-identical across all 17 index-directory files |
| git diff --check | passed |
| python tools/check_whitespace.py | passed; supplemental source/document whitespace scan |

M10 adds 10 executable integration tests and one compile-fail doctest. This includes a
192-image deterministic attribute/section/relocation sweep (four name states each), named and
unnamed candidates, byte names, duplicate identity, trusted membership, source mismatch, null
references, aliases, nested error boundaries and explicit budgets. Existing 137 executable tests
and 11 doctests pass unchanged. No runtime validation or emulator correctness claim follows.

Indexes: implementation 349 (+29), subsystems 115 (+0), modules 307 (+11), source files 298 (+10),
diagnostics 15 (+1), tests 206 (+11), NIDs 0 and ABI 0. Total 1,290 navigation records (+62).
Reviewed links connect the candidate tests to M6 string lookup, M7 symbols, M8 evidence and M9
relocation enumeration. Source spans remain generated. No index schema/tool dependency change.
The index test inventory includes Python and doctest records; it is not an executable Rust count.

Scope: new loader candidate modules/test, loader wiring/README, focused architecture and module-home
records, active state, reviewed index links and generated indexes. No Cargo manifest, Cargo.lock,
dependency policy or unrelated crate implementation changed. Loader remains dependency-free.
See [candidate decisions and pressure](linkage_candidates.md). M10 cleanup was approved and its active item removed; durable evidence remains here.
M11 has not begun.

## M11 results — 2026-09-09

Bounded linkage evidence reporting through loader, debugger and CLI, using generated in-memory
ELF fixtures only. No resolution, NIDs, runtime mutation, guest memory, relocation application,
PS5Rust migration or execution was introduced. Started clean at 3eb0593; no commit or push.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 157 executable Rust tests + 13 compile-fail doctests passed; zero failed/ignored |
| python tools/check_policy.py | passed: 15 packages, 8 internal edges, active state valid |
| structure policy | passed: 188 module homes |
| python -m unittest discover -s tools -p "test_*.py" | 48 passed: 19 policy + 9 structure + 20 index/extraction |
| index generation twice / --check | byte-identical across 17 index-directory files; fresh |
| git diff --check | passed |
| supplemental whitespace | passed: 386 files |

New Rust coverage: five loader report tests, two debugger snapshot tests, three CLI tests and one
compile-fail doctest. Existing capability inventory assertions were updated for the implemented
offline LinkageEvidence operation; guest operations remain unsupported. Tests cover complete/partial/
unavailable/failed states, symbol/detail/name/relocation budgets, mixed counts, byte names, duplicates,
null symbols, unknown attributes, source mismatch, failure prefixes, owned lifetimes, deterministic
ordering, identical debugger report identity and accurate CLI labels. A bounded symbol-budget sweep
checks thresholds 0 through 9. The executable command honestly reports no supplied report; actual
nonempty evidence is exercised by renderer integration tests. No interactive GUI validation claimed.

Indexes: implementation 371 (+22), subsystems 116 (+1), modules 317 (+10), sources 307 (+9),
diagnostics 16 (+1), tests 218 (+12), NIDs 0, ABI 0; total 1,345 (+55). Test index counts include
Python/doctests and do not equal executable Rust test counts. Reviewed links connect report collection,
completeness/budgets, structured diagnostics, debugger snapshot inspection and CLI rendering.

No external dependency/version changes. Added debug -> loader (normal, previously allowed) and
CLI -> loader (dev-only fixture edge, explicitly enforced by policy). Cargo.lock reflects those two
internal edges. Loader remains dependency-free. Scope is reporting/consumers/tests/docs/policy/indexes;
core and GUI source code remain unchanged. See [report architecture](linkage_reporting.md).
M11 cleanup was approved and its active item removed; durable evidence remains here. M12 has not begun.

## M12 results — 2026-09-09

In-memory composition only. Started clean at 05d0111. Loader report generation/classification is
unchanged; no neutral crate, file input, linking, NIDs, runtime-loaded module, guest memory or
execution was added. No commit or push. See [dependency/composition decision](evidence_composition.md).

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including unchanged GUI source |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 164 executable tests and 14 compile-fail doctests passed; zero failures/ignored |
| python tools/check_policy.py | passed: 15 crates, 9 internal edges, active state valid |
| structure policy | passed: 190 declared module homes |
| python -m unittest discover -s tools -p "test_*.py" | 49 passed: 20 dependency/state + 9 structure + 20 index/extraction |
| index regeneration twice and --check | byte-identical across 17 index-directory files; fresh |
| git diff --check | passed |
| supplemental whitespace | passed: 391 files |
| cargo run -p astero-cli -- --linkage --synthetic | passed: synthetic banner, Ready session, no guest loaded, one import candidate, one export candidate |

Added seven executable tests (four core composition, one debugger session path, two CLI live/path)
and one compile-fail input-immutability doctest. Existing standalone report tests now use the
explicitly renamed standalone helper; they remain independent of session claims. A new policy test
checks composition direction, and the previous dev-only fixture-edge test now protects debug rather
than CLI. Tests cover all completeness states, mismatch, identity, Arc sharing, deterministic
observations, dropped-session behavior and partial frontend labels. No GUI runtime validation or
emulator correctness claim is made.

Actual graph changes: core -> loader added; debug -> loader becomes dev-only; CLI -> loader becomes
normal for explicit synthetic input composition. Nine unique declared edges include one dev-only
edge (eight production edges). No external package/version changes; Cargo.lock adds core's loader
edge. Loader remains dependency-free and GUI/core do not gain debugger-facing runtime mechanisms.

Index records: implementation 382 (+11), subsystems 116 (+0), modules 321 (+4), source files 311 (+4),
diagnostics 17 (+1), tests 227 (+9), NIDs 0, ABI 0; total 1,374 (+29). Reviewed links cover input
validation, session construction, session-bound/standalone debugger routes and the CLI synthetic
composition path. No manual source-range edits. Test inventory includes Python/doctest records.

Scope: core typed input/session observation, debugger transport, CLI synthetic composition/rendering,
corresponding tests, manifests/lock, policy, documentation and generated indexes. No loader parser,
classification, relocation or evidence-generation implementation changed. M12 cleanup was approved and its active item removed; durable evidence remains here.
M13 results follow below.

## M13 results — 2026-09-09

Read-only GUI evidence only. No commit/push, file input, new ELF parsing, linking, NIDs, relocation
application or guest execution. Shared generated demo code moved from CLI into loader with typed
core composition; report generation/classification algorithms are unchanged.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed after collapsing one nested UI conditional |
| cargo test --workspace | 169 executable tests + 14 compile-fail doctests; zero failures/ignored |
| python tools/check_policy.py | passed: 15 crates, 9 unique internal edges, active state valid |
| structure policy | passed: 193 declared module homes |
| Python unittest discovery | 49 passed: 20 dependency/state + 9 structure + 20 index/extraction |
| index generation twice and freshness | byte-identical across 17 index-directory files; fresh |
| git diff --check | passed |
| supplemental whitespace | passed: 399 files |
| real GUI --synthetic-linkage | Vulkan first frame, correct pane/counts, selection, maximize/restore, exit 0 |

Five new GUI integration tests cover absence and all report completeness states, typed synthetic
provenance, no guest loaded, exact borrowed report/count/detail identity, candidate selection,
duplicate/absent/empty/UTF-8/non-UTF-8 names, unknown attributes, ordinary/PLT references and partial
budgets. Existing CLI live synthetic tests pass through the shared core composer. No new policy
test count; the dev-only edge test now covers both debug and CLI. GUI rendering is not unit-tested
as ImGui internals; the Astero view model is.

Runtime used the NVIDIA GeForce RTX 4070. Complete demo counts: eight observed symbols, one import,
one export, one internal, four unclassified, one null; two unique references, one ordinary and one
PLT/JMPREL. Symbol 1 selection showed its raw FF name and both reference classes. Synthetic and
No guest loaded labels were visible. Maximize/restore reflowed the real window without losing
selection; closing returned code 0. Partial/unavailable/failed UI rules are validated by model
tests, not claimed as interactive runtime coverage. This is host GUI evidence, not emulator correctness.

No external dependency changes; Cargo.lock unchanged. Nine unique internal edges remain, now seven
normal and two dev-only (CLI/debug -> loader). GUI retains core/debug only; loader is dependency-free.
Three new declared homes: loader report/synthetic, core inputs/synthetic, GUI model/linkage.

Indexes: implementation 401 (+19), subsystems 116 (+0), modules 328 (+7), sources 318 (+7),
diagnostics 17 (+0), tests 232 (+5), NID 0, ABI 0; total 1,412 (+38). Reviewed links connect GUI
names/status/selection and shared synthetic composition with tests; the existing report diagnostic
now includes the GUI surface. Generated locations were not hand-edited.

See [GUI evidence architecture](gui_linkage_evidence.md). M13 cleanup was approved and its active item removed. Durable evidence remains here; M14 has not begun.

## M14 results — 2026-09-09

Started clean at f4ce32b. Host file acquisition only; existing SourceArtifact and all M13 runtime
code unchanged. No parsing, admission, loading, linkage, NIDs, guest mechanisms or GUI/CLI wiring.
No commit or push. See [filesystem boundary](filesystem_input.md).

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including existing GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 178 executable tests + 14 compile-fail doctests passed; zero failures/ignored |
| policy/state | passed: 15 crates, 9 unique internal edges |
| structure policy | passed: 194 declared module homes |
| Python unittest discovery | 49 passed: 20 dependency/state + 9 structure + 20 index/extraction |
| index generation twice and --check | byte-identical across 17 files; fresh |
| git diff --check | passed |
| supplemental whitespace | passed: 406 files |

Nine newly executed tests: four Windows integration tests and five private-reader unit tests.
Coverage includes arbitrary non-ELF bytes, empty and exact-limit input, excess size, missing/invalid/
directory paths, exclusive Windows open failure, non-UTF-8 Windows path, immutable retention,
new object identity per acquisition, 256 extent combinations, fragmented/interrupted reads,
call budgets, I/O error source, unrepresentable length, bounded chunks and final size changes.
A tenth new indexed test is Unix-only symlink rejection; not executed or claimed validated here.
Injected reader/final-size tests are deterministic race evidence, not a concurrent filesystem stress test.
No real input corpus or downstream parser was invoked.

No dependency/manifests/lock changes; loader remains dependency-free. Nine existing internal edges
remain seven normal and two dev-only. New home: loader artifact/filesystem. No M13 reinterpretation.
No GUI runtime revalidation was needed because no GUI/runtime code changed.

Indexes: implementation 413 (+12), subsystems 116 (+0), modules 334 (+6), sources 324 (+6),
diagnostics 18 (+1), tests 242 (+10), NID 0, ABI 0; total 1,447 (+35).
Reviewed links connect acquisition, bounded reads, resource/race fixtures and structured errors.
Locations are generated. Test index includes inactive platform branches and is not an execution count.

M14 cleanup was approved and its active item removed; durable records remain. Durable pressure: synchronous I/O latency, adversarial namespace
replacement and atomic snapshot guarantees need explicit platform work if required; this adapter
claims none of them. A future milestone may expose acquisition-only selection/size policy through
a frontend, retaining the STOP boundary. No M15 work has begun.

## M15 results — 2026-09-09

Started clean at 7d92593. CLI acquisition-only selection with required native path/byte/read budgets;
thin core application entry point. M14 loader source and GUI remain unchanged. No parsing, loading,
linkage, NIDs, guest state or execution is introduced. No commit or push.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including unchanged GUI |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 184 executable tests + 14 compile-fail doctests; zero failures/ignored |
| policy/state | passed: 15 crates, 9 unique internal edges |
| structure | passed: 197 declared module homes |
| Python discovery | 49 passed: 20 dependency/state + 9 structure + 20 index/extraction |
| regeneration twice and freshness | byte-identical across 17 index-directory files; fresh |
| git diff --check | passed |
| supplemental whitespace | passed: 414 files |

Six new CLI integration tests cover mandatory/invalid/duplicate limits, exact-size success,
byte-limit/read-budget failure, nonexistent path retaining I/O cause, synchronous state transitions,
unchanged-input repeated acquisition/new identity, retained immutable bytes, native non-UTF-8 path
through model and real executable, success/status/length/budget output and nonzero error exits.
Actual CLI subprocesses acquire generated invalid-ELF bytes without a parser or session branch.
The five linkage tests, including live synthetic mode, and two default-presentation tests still pass.
No GUI behavior changed; no new interactive GUI smoke is claimed.

No dependencies, manifests or lock changes. Existing nine edges remain seven normal/two dev-only;
CLI -> loader remains dev-only and loader remains dependency-free. New homes are core/input,
core/input/acquisition and CLI/acquisition. There is no file-reading implementation outside loader.

Indexes: implementation 428 (+15), subsystems 118 (+2), modules 341 (+7), sources 331 (+7),
diagnostics 19 (+1), tests 248 (+6), NID 0, ABI 0; total 1,485 (+38). Reviewed links cover native
argument syntax, selection/delegation, presentation and diagnostics. M14 acquisition diagnostics
now identify the application/CLI consumer path; source locations remain generated.

See [frontend boundary](acquisition_frontend.md). M14 synchronous I/O/namespace/snapshot limitations
remain unchanged. No M16 work has begun.

M15 cleanup was approved and its active item removed. All durable records remain; M16 has not begun.

## M16 results — 2026-09-09

Started at 85f2f5a with only the user's intentional AGENTS.md catalogue-path edit; preserved unchanged.
Explicit CLI header inspection only. No new parser logic, loading, admission, dynamic/linkage calls,
NIDs, ABI, GPU work or execution. No commit/push. See [inspection boundary](explicit_inspection.md).

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI and fixture example |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 192 executable tests + 15 compile-fail doctests; zero failures/ignored |
| CLI integration coverage | 17 passed: 4 inspection + 6 acquisition + 5 linkage + 2 presentation |
| policy/state | passed: 15 crates, 9 unique internal edges |
| structure policy | passed: 201 declared homes |
| Python discovery | 49 passed: 20 dependency/state + 9 structure + 20 index/extraction |
| index regeneration twice / --check | byte-identical across 17 files; fresh |
| git diff --check | passed |
| supplemental whitespace | passed: 428 files |

17 executable CLI tests (4 inspection + 6 acquisition + 5 linkage +
2 presentation). The workspace executable count above includes all owning crates. Eight new
executable tests (four loader, four CLI) plus one immutability compile-fail doctest. Tests cover
a 56-case header-count/budget sweep, zero/exact/excess budgets, reused decoder equivalence,
immutable shared source identity/bytes/provenance, malformed/unsupported forms, uninterpreted
dynamic/section pointers, explicit opt-in, unchanged acquisition state, native non-UTF-8 process
arguments, acquisition versus inspection errors and deterministic repeated inspection.
Existing synthetic linkage/default session modes still pass.

The indexer rejected initial cross-file #[path] fixture imports. The established M4 generated fixture
helpers were moved unchanged into loader elf/inspect/synthetic and re-exported by their old test home.
CLI tests and the small create-new-only example share that generator. No indexer workaround, copied
parser or production-decoder change was introduced. No external catalogue was consulted.

### Manual smoke evidence

Run from repository root in PowerShell. The example refuses overwrites; reuse an existing generated
fixture or choose a fresh output path.

~~~powershell
cargo run -p astero-loader --example inspection_fixture -- .\target\m16-header.elf
cargo run -p astero-cli -- inspect --path ".\target\m16-header.elf" --max-bytes 1024 --max-read-calls 4 --max-program-headers 1
cargo run -p astero-cli -- inspect --path ".\target\m16-header.elf" --max-bytes 1024 --max-read-calls 4 --max-program-headers 0
cargo run -p astero-cli -- inspect --path ".\README.md" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 8
~~~

| Input / operation | Expected and actual result | Exit |
|---|---|---|
| Generated M4 fixture writer | Wrote 272 bytes, no real binary used | 0 |
| Generated file, one-header budget | Acquired 272 bytes; Complete (header scope only); one program header | 0 |
| Same file, zero-header budget | Acquired; Failed HeaderBudget, declared 1 / maximum 0 | 1 |
| Repository README text | Acquired; Failed Header(InvalidMagic) | 1 |

Every inspection result printed the source ID/provenance, explicit parser budget and no guest loaded,
execution or linkage. Acquisition status was printed separately. GUI was unchanged and not relaunched.
USAGE.md includes these exact commands and separates real acquisition, explicit inspection and synthetic
linkage. The fixture is ignored under target; no binary payload is added to Git.

No dependencies/manifests/lock changes. Nine edges remain seven normal/two dev-only; loader remains
dependency-free. Counts: implementation 459 (+31), subsystems 119 (+1), modules 354 (+13), sources
343 (+12), diagnostics 21 (+2), tests 257 (+9), NID 0, ABI 0; total 1,553 (+68). Test inventory includes
doctests/platform branches. Reviewed links connect inspection budgets/failures and frontend consumers
to tests. Generated locations were never hand-edited.

Remaining pressure: Complete describes headers, not payload/admission validity. Future optional
inspection scopes require distinct explicit budgets; M14 I/O/namespace limitations remain. M17 has
not begun. M16 cleanup was approved and its active item removed. Durable records are preserved.

## M17 results — 2026-09-09

Started clean on main at fabd470; HEAD and origin/main remain unchanged. No commit/push/history
operation. Independent raw dynamic observation only; no GUI or guest runtime change. See
[capability contract](explicit_dynamic_observation.md). Shared raw traversal is extracted once;
the M5 descriptor operation retains its existing step and all prior tests pass.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI and fixture examples |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 202 executable Rust tests + 16 compile-fail doctests; no failures/ignored |
| CLI integration tests within workspace run | 21 passed, including 4 new dynamic tests; M16 and synthetic linkage retained |
| policy/state/structure | passed: 15 crates, 9 unique edges, 205 module homes |
| Python tests | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| index generation twice | byte-identical across 17 index-directory files |
| index freshness | passed: 1,621 navigation records |
| whitespace / git diff --check | passed; supplemental scan covers 442 files |

Ten new executable tests: six loader and four CLI; one new compile-fail immutable-report doctest.
Coverage includes 168 budget/count pairs, 31 pre-terminal extents, no/multiple/empty tables,
retained DT_NULL, odd trailing bytes, unknown tags/raw values, source identity/immutability,
translation/source/overflow and exact-end bounds, independent acquisition/header requests,
required limits, malformed input, deterministic output and native non-UTF-8 Windows paths.
Unix-specific behavior is not claimed validated. Allocation failure is structured but no allocator
exhaustion experiment was performed. Existing synthetic linkage tests pass; GUI was not relaunched.

### Manual CLI evidence

Exact commands also appear in USAGE.md. Synthetic fixture only, no real guest input. The earlier
272-byte M16 header fixture was reused. Example writer refuses overwrite; choose a fresh path or
reuse the generated file when repeating these checks.

~~~powershell
cargo run -p astero-loader --example dynamic_fixture -- .\target\m17-dynamic.elf
cargo run -p astero-cli -- dynamic --path .\target\m17-dynamic.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 3
cargo run -p astero-cli -- dynamic --path .\target\m17-dynamic.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 2
cargo run -p astero-cli -- dynamic --path .\target\m16-header.elf --max-bytes 1024 --max-read-calls 4 --max-program-headers 1 --max-dynamic-entries 0
~~~

| Operation | Expected and actual result | Exit |
|---|---|---|
| Fixture writer | Created 1536 synthetic bytes | 0 |
| Entry budget 3 | Acquired; Complete; 3 entries; STRTAB MAX pointer not dereferenced; Unknown(-42) preserved; NULL value 7 retained | 0 |
| Entry budget 2 | Acquired; Failed EntryLimit { limit: 2 }; no partial successful list | 1 |
| Header fixture | Acquired; Unavailable (no PT_DYNAMIC) | 0 |

Each CLI observation labels the explicit request, source/provenance, both parser budgets and no
loaded guest/linkage/execution. Acquisition remains separately labeled. No external catalogue was
needed; no PS5Rust/decrypted input, NID/ABI implementation, string/symbol/hash/relocation semantics,
linkage or guest state was introduced.

No manifest, lockfile or dependency-policy changes. Loader is dependency-free. Existing 9 edges
remain 7 normal and 2 dev-only. Shared M5 byte fixture construction moved unchanged to loader
elf/dynamic/synthetic; the owning test module re-exports it. No general fixture framework was added.

Indexes: implementation 488 (+29), subsystems 120 (+1), modules 367 (+13), sources 356 (+13),
diagnostics 22 (+1), tests 268 (+11), NID 0, ABI 0; total 1,621 (+68). Reviewed links connect raw
observation, CLI requests, diagnostic context and tests. Machine-readable outputs remain ignored;
Markdown navigation remains tracked. Source ranges are generated, not manually edited.

M17 cleanup was approved and its active item removed; durable records remain. M18 has not started.
Remaining pressure is future separately budgeted descriptor observation and existing synchronous
acquisition/platform snapshot limitations. Complete means raw-table scope, not target validity.

## M18 results — 2026-09-09

Started clean on main at 45d80a1; HEAD and origin/main remain unchanged. No commit/push/history
operation. Selected STRTAB/STRSZ and SYMTAB/SYMENT metadata only, using the existing M5 collector.
See [capability contract](explicit_descriptor_observation.md). M17 private discovery is shared,
without changing its public outcome or introducing automatic descriptor calls.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI and fixture examples |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 212 executable Rust tests + 17 compile-fail doctests; no failures/ignored |
| CLI integration tests within workspace run | 25 passed, including 4 new descriptor tests; prior inspection/dynamic/linkage tests retained |
| policy/state/structure | passed: 15 crates, 9 unique edges, 208 module homes |
| Python tests | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| index generation twice | byte-identical across 17 index-directory files |
| index freshness | passed: 1,685 navigation records |
| whitespace / git diff --check | passed; supplemental scan covers 454 files |

Ten new executable tests: six loader and four CLI; one immutable-report compile-fail doctest.
Coverage: 16 family/budget combinations, 81 string-boundary combinations, zero/exact/excess budgets,
equal/conflicting duplicates, missing companions, symbol-width rejection, first-entry bounds,
BSS/unmapped/overflow/crossing errors, independently valid overlapping ranges, unavailable/discovery
failure distinction, raw audit values, source identity/immutability, shared M5 semantic equivalence,
invalid payloads not interpreted, all prior explicit commands, and native Windows non-UTF-8 paths.
Unix-specific behavior is not claimed validated. Allocation refusal is structured, but allocator
exhaustion was not induced. Existing synthetic linkage tests passed; GUI was not relaunched.

### Manual CLI evidence

Exact commands also appear in USAGE.md. Three 1536-byte synthetic fixtures use the existing M5
builder. Their payload bytes are intentionally unsuitable for string/symbol interpretation. The
writer refuses existing outputs; reuse files or choose fresh names when repeating these commands.

~~~powershell
cargo run -p astero-loader --example descriptor_fixture -- .\target\m18-valid.elf valid
cargo run -p astero-loader --example descriptor_fixture -- .\target\m18-none.elf none
cargo run -p astero-loader --example descriptor_fixture -- .\target\m18-conflict.elf conflict
cargo run -p astero-cli -- descriptors --path .\target\m18-valid.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 2
cargo run -p astero-cli -- descriptors --path .\target\m18-valid.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1
cargo run -p astero-cli -- descriptors --path .\target\m18-none.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 4 --max-descriptors 0
cargo run -p astero-cli -- descriptors --path .\target\m18-conflict.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 4 --max-descriptors 1
~~~

| Operation | Expected and actual result | Exit |
|---|---|---|
| Fixture generation, valid/none/conflict | Created each 1536-byte synthetic layout | 0 each |
| Valid, descriptor budget 2 | Complete; Strings full 16-byte range; Symbols first 24-byte entry only; raw original fields retained | 0 |
| Valid, descriptor budget 1 | Failed Budget { attempted_families: 2, maximum: 1 }; no partial list | 1 |
| Unsupported-only tags, budget 0 | Unavailable NoSupportedDescriptors; no hash/relocation pointer traversal | 0 |
| Conflicting STRTAB, budget 1 | Failed DuplicateTag { tag: StrTab, first: 0, second: 1 } | 1 |

Each observation labels its explicit request, source/provenance, all observation budgets, no payload
traversal and no guest/linkage/execution. Acquisition output is a separately labeled stage.
No external catalogue was needed. No string reads, symbol enumeration/count derivation, hash walks,
relocation decoding, import/export/linkage/NID/ABI interpretation, guest admission or runtime state.

No dependency/manifests/lock changes; loader remains dependency-free. The 9 edges remain 7 normal and
2 dev-only. New nested homes: loader elf/dynamic/descriptors, core input/descriptors, CLI descriptors.
M5 collector visibility/iterator input is generalized for filtered reuse; its pairing implementation
is unchanged. M17 extraction avoids duplicate header decoding; its prior tests pass unchanged.

Indexes: implementation 517 (+29), subsystems 121 (+1), modules 378 (+11), sources 367 (+11),
diagnostics 23 (+1), tests 279 (+11), NID 0, ABI 0; total 1,685 (+64). Reviewed links connect selected
observation, source/budget evidence, CLI and tests. Generated JSON remains local/ignored; Markdown
navigation remains tracked. Source locations are regenerated, not manually repaired.

M18 cleanup was approved and its active item removed; durable records remain. M19 has not started.
Remaining pressure: Complete certifies only selected metadata; future payload observation needs
separate caller-visible budgets and must not automatically become dependency resolution/linking.

## M19 results — 2026-09-09

Started clean on main at 7148724; HEAD and origin/main remain unchanged. No commit/push/history
operation. Explicit DT_NEEDED byte references only, using M18 descriptors and unchanged M6 lookup.
See [capability contract](explicit_string_references.md). The only M6 implementation change is a
source-checked constructor for private M18 descriptor records, sharing existing initialization.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI and fixture examples |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 221 executable Rust tests + 18 compile-fail doctests; no failures/ignored |
| CLI integration tests within workspace run | 29 passed, including 4 new reference tests; earlier commands and synthetic linkage retained |
| policy/state/structure | passed: 15 crates, 9 unique edges, 211 module homes |
| Python tests | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| index generation twice | byte-identical across 17 index-directory files |
| index freshness | passed: 1,749 navigation records |
| whitespace / git diff --check | passed; supplemental scan covers 467 files |

Nine new executable tests: five loader and four CLI; one immutable-report compile-fail doctest.
Coverage: 170 offset/scan-window combinations, 24 per/aggregate-budget combinations, reference
budget zero/exact/excess, repeated offsets, empty/UTF-8/raw byte distinctions, final NUL boundaries,
unterminated and out-of-range references, absent/malformed prerequisites, no unreferenced scanning,
shared-source determinism, mismatched descriptor/source rejection and native Windows non-UTF-8 paths.
Earlier frontend commands succeed even when M19 lookup fails. Existing synthetic linkage tests pass.
Unix-specific behavior is not claimed validated; allocator exhaustion was not induced. GUI unchanged
and not relaunched. These are bounded structural/byte contract tests, not guest correctness evidence.

### Manual CLI evidence

Exact commands also appear in USAGE.md. Three synthetic 1536-byte files contain valid, raw and
unsupported-only reference cases; no external input. Writers refuse overwrites. Reuse generated
files or choose fresh output names to repeat the checks.

~~~powershell
cargo run -p astero-loader --example string_reference_fixture -- .\target\m19-valid.elf valid
cargo run -p astero-loader --example string_reference_fixture -- .\target\m19-raw.elf raw
cargo run -p astero-loader --example string_reference_fixture -- .\target\m19-none.elf none
cargo run -p astero-cli -- string-references --path .\target\m19-valid.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 1 --max-scan-bytes-per-reference 11 --max-total-scan-bytes 11
cargo run -p astero-cli -- string-references --path .\target\m19-raw.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 1 --max-scan-bytes-per-reference 2 --max-total-scan-bytes 2
cargo run -p astero-cli -- string-references --path .\target\m19-valid.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 1 --max-scan-bytes-per-reference 10 --max-total-scan-bytes 11
cargo run -p astero-cli -- string-references --path .\target\m19-valid.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 5 --max-descriptors 1 --max-string-references 0 --max-scan-bytes-per-reference 11 --max-total-scan-bytes 11
cargo run -p astero-cli -- string-references --path .\target\m19-none.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 4 --max-descriptors 1 --max-string-references 0 --max-scan-bytes-per-reference 0 --max-total-scan-bytes 0
~~~

| Operation | Expected and actual result | Exit |
|---|---|---|
| valid/raw/none fixture writers | Each created 1536 synthetic bytes | 0 each |
| Valid reference, scan/total 11 | Complete; UTF-8 "libdemo.so", content 10 bytes, charged 11 including NUL | 0 |
| Raw reference, scan/total 2 | Complete; <non-UTF8: FF>, exact byte FF retained | 0 |
| Valid reference, per scan 10 | Failed ScanLimit; no truncated prefix | 1 |
| Valid reference, reference budget 0 | Failed ReferenceBudget { observed_references: 1, maximum: 0 } | 1 |
| No supported references, zero lookup limits | Unavailable; unterminated unreferenced contents not scanned | 0 |

Every invocation labels acquisition separately, explicit reference observation, source/provenance,
all budgets and no enumeration/dependency resolution/linkage/guest execution. No external catalogue
was needed. No SONAME/RPATH/RUNPATH support, symbol names/counts, hash walks, relocation decoding,
import/export/NID/ABI semantics, dependency declarations, admission or guest runtime was introduced.

No dependency/manifests/lock changes; loader remains dependency-free. Existing 9 edges remain 7 normal
and 2 dev-only. M6 StringLimits is reused as a pure value contract; dependencies::observe is not called.
New homes: loader elf/dynamic/string_references, core input/string_references, CLI string_references.

Indexes: implementation 545 (+28), subsystems 122 (+1), modules 390 (+12), sources 379 (+12),
diagnostics 24 (+1), tests 289 (+10), NID 0, ABI 0; total 1,749 (+64). Reviewed relationships connect
M6/M18 proof reuse, lookup budgets, CLI presentation and tests. Generated JSON remains ignored;
Markdown navigation remains tracked. Source locations are regenerated rather than hand-maintained.

M19 is ready_for_cleanup pending approval; its local active item remains. M20 has not started.
Remaining pressure: new reference kinds or payload scopes need independent decisions and explicit
budgets. A possible M20 is explicit bounded hash-metadata/trusted-symbol-extent evidence using
existing contracts, stopping before symbol enumeration or linkage. No such work was performed here.

## M20: explicit bounded hash metadata — 2026-09-09

M20 adds an independent loader/core/CLI request; shared M8 decoders/proof rules remain unchanged.
No symbol enumeration, names, strings, relocations, linkage, NID/ABI or runtime work is called by
this request. M16-M19 retain their own boundaries. No dependencies, manifests or lockfile changed.

| Check | Actual result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI and examples |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 231 executable tests + 19 compile-fail doctests; no failures/ignored |
| CLI integration tests included above | 33 passed; 4 new, earlier commands/synthetic paths retained |
| dependency/state/structure policy | passed: 15 crates, 9 unique internal edges, 215 module homes |
| Python tests | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| index regeneration twice | byte-identical across 17 index-directory files |
| freshness | passed: 1,808 navigation records |
| whitespace / git diff --check | passed; supplemental scan covers 480 files |

Ten new executable tests (six loader, four CLI) and one immutable-report compile-fail doctest.
Coverage includes 16 SysV count combinations, 25 header/array source-end boundaries, eight GNU
terminator/count boundaries, every insufficient budget below exact SysV/GNU/both work, zero budgets,
positive/zero/maximum fields, invalid bloom/buckets, absent/duplicate/conflicting descriptors,
GNU lower-bound-only evidence, corroboration, inadequate symbol backing, immutable source identity,
M7 candidate-only refusal after independent proof and native Windows non-UTF-8 paths. Invalid symbol
bytes and unrelated malformed Needed/Rela metadata remain unread. Old hash/enumeration tests pass.
An initial sweep expected a valid two-bucket layout to fail; the test expectation was corrected,
then focused tests and full validation passed. No production semantic workaround was made.
Unix-only behavior is not claimed validated. Allocation failure was not injected. GUI remains
unchanged, builds, and was not relaunched. These checks are not evidence of guest correctness.

### Manual CLI smoke tests

All inputs below are generated 1536-byte synthetic fixtures; writers refuse overwrites. Reuse
existing files or choose fresh names when repeating. Exact commands are also in USAGE.md.

~~~powershell
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-sysv.elf sysv
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-both.elf both
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-conflict.elf conflict
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-none.elf none
cargo run -p astero-loader --example hash_metadata_fixture -- .\target\m20-lower.elf lower
cargo run -p astero-cli -- hash-metadata --path .\target\m20-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 9
cargo run -p astero-cli -- hash-metadata --path .\target\m20-both.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 18
cargo run -p astero-cli -- hash-metadata --path .\target\m20-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 8
cargo run -p astero-cli -- hash-metadata --path .\target\m20-none.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 0
cargo run -p astero-cli -- hash-metadata --path .\target\m20-conflict.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 18
cargo run -p astero-cli -- hash-metadata --path .\target\m20-lower.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 7
~~~

| Observation | Expected and actual result | Exit |
|---|---|---|
| five fixture writers | 1536 bytes created each | 0 each |
| SysV, 9 words | Complete, nchain=3, trusted count 3, 72 symbol backing bytes | 0 |
| Both, 18 words | Complete, SysV/GNU exact 3 corroborated, both proof ranges retained | 0 |
| SysV, 8 words | Failed WorkLimit, no trusted count | 1 |
| None, 0 words | Unavailable / no supported hash descriptor | 0 |
| Conflict, 18 words | Failed ConflictingEvidence, SysV 3 vs GNU 2 | 1 |
| Lower-bound GNU, 7 words | Complete metadata, LowerBound(1), exact count Unavailable | 0 |

Output labels acquisition separately from explicit hash observation, original selected tag indices
and values, budgets, source identity/provenance, raw hash fields, proof/range and the no-enumeration /
no-names / no-linkage / no-guest stop boundary. No catalogue was consulted: existing M8 contracts
were sufficient. No real guest binaries were used or executed.

Indexes: implementation 567 (+22), subsystems 123 (+1), modules 402 (+12), sources 391 (+12),
diagnostics 25 (+1), tests 300 (+11), NID 0, ABI 0; total 1,808 (+59). Generated JSON remains ignored.
New homes: loader hash/bounded and hash/synthetic, core input/hash_metadata and CLI hash_metadata.
Only the M8 collector was extracted for reuse; SysV/GNU decoding and extent rules are unchanged.

M20 is ready_for_cleanup pending approval; the local active item remains. No commit/push/history
change occurred; HEAD and origin/main remain 6950e014965f55c2d6bd9634b77a48811520a448. M21 has not started.
Remaining pressure: M8's conservative GNU layout support and budgeted repeated SysV paths remain
explicit. A possible M21 is separately requested bounded symbol observation from trusted extents,
with explicit entry/name budgets and no classification/linking. No such consumer was added here.

## M21: explicit trusted symbol observation — 2026-09-09

M21 explicitly consumes immutable exact M20 evidence, preserving same-source identity and [0,N)
membership. Core delegates; CLI's separate symbols mode requests and supplies proof. M7 decoding
was extracted once for shared use; its candidate-only count refusal and name behavior remain.
M6 lookup, M20, earlier commands and GUI are unchanged. No dependency/manifests/lockfile changes.

| Check | Actual result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI and fixtures |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 240 executable tests + 20 compile-fail doctests; no failures/ignored |
| CLI integration tests included above | 37 passed, including 4 new; old synthetic modes retained |
| dependency/state/structure | passed: 15 crates, 9 unique edges, 219 module homes |
| Python tests | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| regeneration twice | byte-identical across 17 index-directory files |
| index freshness | passed: 1,876 navigation records |
| whitespace / git diff --check | passed, supplemental scan covers 495 files |

Nine new executable tests (5 loader, 4 CLI), one immutable-report compile-fail doctest. Coverage:
273 lookup-count/per-scan/aggregate-budget combinations; exact/insufficient entry limits; count one;
zero count refused upstream; exact-end/short symbol extent; symbol zero; SysV/GNU/corroborated proof;
lower-bound/absent/conflicting/mismatched-source evidence; index ordering and duplicates; all raw
fields; unknown attributes; unnamed/empty/UTF-8/raw bytes; missing/out-of-range/unterminated names;
shared Arc identity and deterministic immutable results. Native Windows non-UTF-8 paths validated.
Earlier acquire/inspect/dynamic/descriptors/string-references/hash-metadata commands succeed on a
fixture where explicit M21 rejects invalid symbol zero. Old M7/M8 tests pass after decoder extraction.
An unused test import was removed before full warnings-denied validation. Unix-only behavior is not
claimed validated. Allocation failure was not injected. GUI built but was not relaunched. This is
bounded structural/name observation evidence, not emulator correctness or runnable-guest evidence.

### Manual CLI evidence

Three generated synthetic 1536-byte fixtures, no guest binaries. Writers refuse existing outputs.
Reuse fixtures or choose fresh names when repeating. Exact commands also appear in USAGE.md:

~~~powershell
cargo run -p astero-loader --example symbol_fixture -- .\target\m21-sysv.elf sysv
cargo run -p astero-loader --example symbol_fixture -- .\target\m21-raw.elf raw
cargo run -p astero-loader --example symbol_fixture -- .\target\m21-lower.elf lower
cargo run -p astero-cli -- symbols --path .\target\m21-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 2 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-sysv.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 5 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-lower.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12
cargo run -p astero-cli -- symbols --path .\target\m21-raw.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 8
~~~

| Operation | Expected and actual result | Exit |
|---|---|---|
| three fixture writers | each created 1536 synthetic bytes | 0 each |
| SysV, symbols 3 / lookups 2 / scan 6 / total 12 | Complete indices 0,1,2; unnamed zero; duplicate alpha names; exact proof 3 | 0 |
| SysV, symbols 2 | Failed EntryBudget before decoding; no successful prefix | 1 |
| SysV, scan 5 | Failed ScanLimit at symbol 1, no partial symbol list | 1 |
| GNU lower-bound-only | Unavailable, no enumeration | 0 |
| Raw, symbols 3 / lookups 2 / scan 6 / total 8 | Complete; raw FF preserved/displayed <non-UTF8: FF> | 0 |

Output includes original source/provenance, explicit budgets, expected count and proof, raw fields,
structural views and no import/export/linkage/dependency/NID/guest execution claims. A symbol marked
Undefined remains only that structural fact. No classification or relocation association was called.
No external catalogue was consulted; existing Astero contracts resolved the scope and semantics.

Indexes: implementation 595 (+28), subsystems 124 (+1), modules 416 (+14), sources 405 (+14),
diagnostics 26 (+1), tests 310 (+10), NID 0, ABI 0; total 1,876 (+68). Generated JSON remains ignored.
New nested homes are symbol_table/bounded and symbol_table/synthetic, core input/symbols, CLI symbols;
shared symbol_table/decode replaces the prior inline field decoder. No new parser or string reader.

M21 is ready_for_cleanup pending approval; its local active item remains. No commit/push/history
change occurred; HEAD and origin/main remain 3ab7ad5b7c4e195703cb8d834eef187cb87f4515. M22 has not started.
Remaining pressure: repeated bounded M18 discovery after M20 is explicit; immutable proof removes
coherence risk, not that fixed extra work. Future classification must be separately requested and
budgeted. A possible M22 is bounded candidate classification using established evidence, stopping
before linkage/dependency/NID resolution. No such work is included here.

## M22: explicit structural symbol classification — 2026-09-09

Loader candidates/structural consumes a shared complete M21 report. Core delegates and the separate
CLI command explicitly composes prerequisites. M10 import/export eligibility and relocation
association are not invoked. No dependency, manifest, lockfile or GUI changes. Prior CLI commands
and synthetic linkage paths remain intact. No catalogue or real guest input was used.

| Check | Actual result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI/examples |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 247 executable tests + 21 compile-fail doctests; zero failed/ignored |
| CLI integration subset | 40 passed, including 3 new |
| dependency/state/structure | 15 crates, 9 unchanged edges, 222 declared homes |
| Python suite | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| index regeneration twice / freshness | byte-identical across 17 files; 1,929 records current |
| whitespace / git diff --check | passed; supplemental scan covers 507 files |

Seven new executable tests (4 loader, 3 CLI) and one immutable-report compile-fail test. A bounded
150-combination sweep varies section, binding/type and visibility, with raw non-UTF8 names.
Coverage includes null, undefined, ordinary, ABS, COMMON, reserved/extended; local/global/weak and
unknown values; duplicate names; unnamed/empty/UTF8/raw; shared original records/proof; deterministic
ordering and repeated classification; exact/zero/insufficient budget with no prefix; unavailable
and failed prerequisites. Prior acquire and M16-M21 commands never emit classification. Native
Windows non-UTF8 paths pass; Unix-only behavior is not claimed validated. Allocation failure was
not injected. A test-only unused import was removed before warnings-denied full validation.
GUI built and its existing tests passed; no GUI runtime smoke was needed for this CLI-only change.

### Manual CLI evidence

These exact commands are documented in USAGE.md. Fixture writers are overwrite-safe; reuse their
outputs or select fresh names for another run. Only repository-generated 1536-byte fixtures used.

~~~powershell
cargo run -p astero-loader --example classification_fixture -- .\target\m22-ordinary.elf ordinary
cargo run -p astero-loader --example classification_fixture -- .\target\m22-special.elf special
cargo run -p astero-cli -- classify-symbols --path .\target\m22-ordinary.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12 --max-classifications 3
cargo run -p astero-cli -- classify-symbols --path .\target\m22-ordinary.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12 --max-classifications 2
cargo run -p astero-cli -- classify-symbols --path .\target\m22-special.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 8 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 2 --max-name-scan-bytes 6 --max-total-name-scan-bytes 12 --max-classifications 3
~~~

| Operation | Expected and actual result | Exit |
|---|---|---|
| ordinary and special fixture writers | each created 1536 bytes | 0 each |
| ordinary / classifications 3 | Complete: null, UndefinedCandidate, DefinitionCandidate | 0 |
| ordinary / classifications 2 | Failed Budget count=3 maximum=2, no successful prefix | 1 |
| special / classifications 3 | Complete: SpecialCandidate retains Reserved(65312), Unknown(14/15/5) | 0 |

Ordinary symbols 1 and 2 are both named alpha, GLOBAL/FUNC; section alone distinguishes their roles.
Neither is labeled import/export. Output retains source/provenance and prerequisite status, and
states no resolution/linkage/NID/guest loading/execution. Classification reads no source bytes,
relocations, PLT/GOT or other artifacts; it pairs roles with the existing immutable M21 records.

Indexes: implementation 616 (+21), subsystems 125 (+1), modules 427 (+11), sources 416 (+11),
diagnostics 27 (+1), tests 318 (+8), NID 0, ABI 0; total 1,929 (+53). Three required nested homes
added: loader candidates/structural, core input/classification, CLI classification. Generated JSON,
AGENTS.md and PROJECT_STATE.json remain ignored/local-only.

M22 is ready_for_cleanup pending approval. HEAD and origin/main remain
51628cfaa6b1626b041b0db02ca541feeddf4fcc; no commit/push/history change. M23 has not started.
Remaining pressure: structural roles deliberately do not prove import/export eligibility or use.
A possible M23 is separately requested bounded relocation report exposure using M9 mechanisms,
with explicit budgets and trusted reference validation; no application, association or resolution.

## M23: real ELF linkage evidence — 2026-09-09

Capability: one explicit request composes bounded immutable source, exact hash proof, M21 symbols,
M22 roles, M9 RELA/alias handling and M6 dependency-name lookup. Core delegates; CLI presents.
See [M23 evidence/rules](real_linkage_evidence.md) for focused catalogue routes and hypothesis outcomes.
No manifests, lockfile, dependencies, GUI, runtime state or external corpus files were changed.

| Check | Actual result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed, including GUI/examples |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 260 executable tests + 22 compile-fail doctests; none failed/ignored |
| CLI integration subset | 43 passed, including 3 new |
| dependency/state/structure | 15 crates, 9 unchanged edges, 226 declared homes |
| all Python tests | 49 passed: 20 dependency/state, 9 structure, 20 index/extraction |
| regeneration twice / freshness | 17 files byte-identical; 2,008 navigation records current |
| whitespace / git diff --check | passed; supplemental scan covers 523 files |

Thirteen new executable tests (10 loader, 3 CLI), one immutable-report compile-fail test. Tests cover
combined raw fields/index association, unknown relocation numbers, signed addends, null/nonzero,
undefined-with/without references, global/local/unknown/hidden/unnamed/empty/raw names, no export
promotion, source identity/immutability, duplicate needed names, shared name budgets, exact/insufficient
relocation count, canonical aliases and conflicts, malformed descriptors, REL/RELR refusal, out-of-range
symbol references, missing trusted count and bad symbol prerequisites. Existing M9's 15 tests passed
after extracting canonical raw iteration; full M10-M22 and synthetic GUI/session tests also passed.
Native Windows non-UTF8 path integration passed. Unix-only behavior and allocator-failure injection
are not claimed. GUI built/tests passed but was not relaunched because its code/path is unchanged.
No runtime correctness, binary compatibility or resolved-linkage claim follows from these tests.

### Real CLI experiments

All paths selected from ignored LOCAL_TEST_CORPUS.json. No real files copied into Astero. Before/after
SHA-256 matched for both selected files; only source bytes were inspected, never executed. The
following exact commands were run (full output retained locally under ignored target/m23-validation).

~~~powershell
$corpus = Get-Content -LiteralPath .\LOCAL_TEST_CORPUS.json -Raw | ConvertFrom-Json
$sample = $corpus.artifacts.linkage_sample.path
cargo run -p astero-cli -- linkage-evidence --path "$sample" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 1024
cargo run -p astero-cli -- linkage-evidence --path "$sample" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 10
$other = $corpus.artifacts.utility_build_comparison.path
cargo run -p astero-cli -- linkage-evidence --path "$other" --max-bytes 1048576 --max-read-calls 64 --max-program-headers 64 --max-dynamic-entries 256 --max-hash-words 4096 --max-descriptors 2 --max-symbols 1024 --max-name-lookups 1024 --max-name-scan-bytes 256 --max-total-name-scan-bytes 262144 --max-relocations 1024
cargo run -p astero-loader --example linkage_fixture -- .\target\m23-synthetic.elf
cargo run -p astero-cli -- linkage-evidence --path .\target\m23-synthetic.elf --max-bytes 2048 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 16 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 4 --max-name-scan-bytes 6 --max-total-name-scan-bytes 24 --max-relocations 4
~~~

| Input / request | Expected and actual outcome | Exit |
|---|---|---|
| linkage_sample, sufficient limits | Complete supported observation, 14 symbols / 11 relocations | 0 |
| linkage_sample, max-relocations 10 | Failed EntryBudget { count: 11, limit: 10 }, no linkage prefix | 1 |
| utility_build_comparison | Complete, same symbol/relocation/candidate totals | 0 |
| fixture writer | 1536 generated bytes, overwrite-safe | 0 |
| synthetic fixture | Complete: 3 symbols, 4 relocations, 1 external candidate, 1 definition | 0 |

Real primary (67,668 bytes): SHA-256 2f824c2a233c540d7220a4b1bc3b9c76e1a314267e47fe31740db5b93f582a3f.
Comparison (67,628 bytes): daa15b69a637b29a34f93d5e9dd28c112e15f82f18ca12a9858b2eab55acef0b.
Both have 9 nonzero symbol-associated records, 2 null records, 8 external candidates, 5 definitions,
zero ambiguous references, libkernel.prx/libc.prx declarations. Primary histogram: type 1 x1, 6 x1,
7 x7, 8 x2. All are known in the legacy numeric subset; all application semantics and providers
remain unsupported/unresolved in Astero. Ordinary and PLT associations are evidence, not calls.
The 68,080-byte catalogue PS5Util is a different hash/build with 18 symbols/13 relocations. Matching
mapping/dependency/reference shape supports reuse; differing counts are not treated as regression.
No substitution, real NID resolution, module ownership inference or guest loading occurred.

Indexes: implementation 651 (+35), subsystems 126 (+1), modules 441 (+14), sources 430 (+14),
diagnostics 28 (+1), tests 332 (+14), NID 0, ABI 0; total 2,008 (+79). Generated machine-readable
files, AGENTS.md, PROJECT_STATE.json and LOCAL_TEST_CORPUS.json remain ignored/local-only.

M23 is ready_for_cleanup pending approval; active hypotheses were refined into the focused record.
Remaining pressure and a possible M24 (bounded PS5 module/library-name correlation without binding)
are recorded there. M24 has not started. No commit/push/history change: local HEAD remains
c26077f1728b7662ce4568551553444a8a964b71; origin/main remains
5e7f450b10e3ef1091105d69fba5fd15e83d7d15 (pre-existing unpushed housekeeping commit preserved).

## M24 results - 2026-09-09

[PS5 identity evidence](ps5_identity_evidence.md) records sources, hypotheses and real outcomes.
No guest binary was executed; real files were inspected as data. No dependency changes.

| Validation | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 276 executable tests, 22 compile-fail doctests, all passed |
| CLI integration tests | 45, included above, all passed |
| Python unittest discovery | 50 passed: 20 policy/state, 9 structure, 21 index/extraction |
| dependency/state/structure policy | 15 crates, 9 actual internal edges, 230 declared homes; passed |
| index regeneration | two byte-identical generations, 16 generated files; freshness passed |
| whitespace | git diff --check and supplemental scan passed |

New coverage: 14 loader, 2 CLI and 1 Python index test. Codec vectors are documentary
corroboration, not runtime/ABI proof. No HLE or relocation application tests are claimed.
Detailed logs are local under ignored target/m24-validation.

Manual commands are exactly those in USAGE.md's PS5 identity section. The comparison also ran
with --max-identity-records 1024. No input paths are embedded here.

| Input/request | Expected and actual result | Exit |
|---|---|---|
| linkage_sample / 1024 records | Complete, 13 NIDs, 7 descriptor hypotheses + 7 unsupported records | 0 |
| utility_build_comparison / 1024 | Complete, same totals; differing library IDs/symbol order | 0 |
| utility_build_comparison / 27 | Failed Budget { count: 28, maximum: 27 }, no successful prefix | 1 |
| identity_fixture writer | 2560 synthetic bytes, create-new/overwrite-safe | 0 |
| synthetic / exact 6 | Complete, two NIDs, null symbol, two descriptors + one unsupported | 0 |
| synthetic metadata offset 0xffffffff | Failed String, dynamic_index 14, OffsetOutOfBounds | 1 |

For the final negative smoke, a new target/m24-malformed.elf was created with Python create-new
mode from target/m24-identity.elf; struct.pack_into('<Q', bytes, 0x6e8, 0xffffffff) changed only
its synthetic descriptor offset. Exact command:

```powershell
cargo run -p astero-cli -- ps5-identity --path .\target\m24-malformed.elf --max-bytes 4096 --max-read-calls 4 --max-program-headers 2 --max-dynamic-entries 32 --max-hash-words 64 --max-descriptors 2 --max-symbols 3 --max-name-lookups 16 --max-name-scan-bytes 64 --max-total-name-scan-bytes 256 --max-relocations 4 --max-identity-records 6
```

Both real before/after SHA256 values match the capability record. All 13 numeric NIDs overlap;
eight context suffixes and symbol order differ. All 26 symbol context pairs match artifact-local
declarations under the labelled hypothesis. Version semantics/attribute tags remain unresolved.
No provider, HLE or dependency is resolved and no relocation is applied.

Indexes: implementation 686 (+35), subsystems 127 (+1), modules 454 (+13), sources 443 (+13),
diagnostics 29 (+1), tests 349 (+17), NIDs 2 (+2 observed/unregistered), ABI 0; total 2090 (+82).
NID Markdown exposes unregistered provenance; owner status describes the decoder, not an HLE.
Generated JSON, AGENTS.md, PROJECT_STATE.json and LOCAL_TEST_CORPUS.json remain local-only/ignored.

No commit/push/history change. HEAD and origin/main remain 0766acf8017a3dfc866c94cef13140c4f211acd6.
M24 remains active pending cleanup approval; M25 has not started.


## M25 results - 2026-09-09

[Asynchronous timing](asynchronous_timing.md) records exact catalogue evidence, ownership,
race semantics, measured host limitations and future consumer boundaries. No guest code ran.

| Validation | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 292 executable tests and 22 compile-fail doctests passed |
| Existing CLI integration tests | 45 included above, passed |
| Python unittest discovery | 51 passed: 21 policy/state, 9 structure, 21 index/extraction |
| dependency/state/structure policy | 16 crates, 10 actual internal edges, 235 declared homes; passed |
| deterministic index regeneration | 16 generated files byte-identical; freshness passed |
| whitespace | git diff --check and supplemental scan passed |

Added 14 timing and 2 core tests plus one dependency-policy test. Most timing semantics use
explicit manual advancement; real tests enforce never-early delivery and a tolerant 5-second
liveness guard. One measured run: 15 ms request took 19.3608 ms, 5.333333 ms took 18.5654 ms,
and 16.666667 ms took 32.0294 ms. These are noisy observations, not latency guarantees.
No default host timer-resolution change is claimed. Focused tests passed before full validation.

Manual command, exit 0: `cargo run -p astero-timing --example deadlines`.
Expected and actual: Cancelled; events 1 and 2 dispatched in order at manual 5000000 ns with
zero lateness; pending=0 fired=2 cancelled=1; worker joined. This exercises a real worker,
not a synchronous mock. USAGE.md contains the copy/pasteable command and output.
Full Cargo/Python logs are local under ignored target/m25-validation.

Indexes: implementation 764 (+78), subsystems 132 (+5), modules 468 (+14), sources 457 (+14),
diagnostics 30 (+1), tests 366 (+17), NIDs 2 (unchanged, observation-only/unregistered), ABI 0.
Total 2219 (+129). Generated JSON and local configuration/tracker files remain ignored.
Only new dependency edge is core -> timing; timing is dependency-free and loader remains
independent. No external dependency added. Queue work is bounded O(n), with caller-owned terminal
tickets; per-ticket Arc allocation follows the process allocator's ordinary failure policy.

M25 remains active ready_for_cleanup pending user approval. No commit/push/history changes:
HEAD and origin/main remain 332b39e8a1ff518c1f3e4807666a9512b6b3e05b. M26 has not started.


## M26 results - 2026-09-09

[Guest load/link planning](guest_load_link_plan.md) records catalogue evidence, provider rules,
real experiments and explicit blockers. No guest binary was executed; files were data inputs only.

| Validation | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 313 executable tests and 23 compile-fail doctests passed |
| CLI integration tests | 48 included above, passed |
| Python unittest discovery | 51 passed: 21 policy/state, 9 structure, 21 index/extraction |
| dependency/state/structure | 16 crates, 10 internal edges, 238 declared homes; passed |
| deterministic indexes | two byte-identical generations, 16 files; freshness passed |
| whitespace | git diff --check and supplemental scan passed |

New executed coverage: 18 loader tests, 3 CLI tests and 1 compile-fail doctest. An additional Unix
native-path test is indexed but was not executed on Windows. Tests cover positive/mismatched/ambiguous
provider matching, alias independence, catalogue-known/unregistered distinction, segment/protection/
zero-fill/source proof, action arithmetic, TLS-value refusal, empty TLS, budgets and immutable order.
Earlier CLI and synthetic linkage paths passed unchanged. Clippy found three style warnings during
implementation; they were fixed before the final full validation. Logs: ignored target/m26-validation.

Manual commands are exactly USAGE.md's load-plan block, including its explicit PowerShell limit
array and local corpus role selection. No absolute input paths are committed. Expected and actual:

| Command/input | Actual result | Exit |
|---|---|---|
| load-plan linkage_sample + utility_build_comparison provider | 5 segments, 2 dependencies, 8 references, 5 candidates, 0 selected, 11 actions, 2 concrete values, 26 blockers | 1 |
| load-plan utility_build_comparison | same segment/dependency/reference/action totals; no supplied candidates; 26 blockers | 1 |
| load-plan primary_real_elf | 5 segments, entry intent 0x100000070, 38 dependencies, 822 references, 30898 actions, 29791 concrete values, 1973 blockers | 1 |
| first command with final plan limit replaced by 1 | Plan(Budget { required: 2, maximum: 1 }); no successful prefix | 1 |

Blocked is an explicit planning result, not a failed experiment. No unsupported relocation numeric
types appeared in these inputs. Missing providers plus bootstrap/RELRO/SCE requirements prevent
load readiness. Empty TLS was found to require no work and is no longer an unnecessary blocker.
No provider or guest memory was loaded. Real provider placement/dependency closure remains required.

Before/after SHA256 matched for all inputs:
- linkage_sample: 2f824c2a233c540d7220a4b1bc3b9c76e1a314267e47fe31740db5b93f582a3f
- utility_build_comparison: daa15b69a637b29a34f93d5e9dd28c112e15f82f18ca12a9858b2eab55acef0b
- primary_real_elf: a15e44caf80be1d86b72c2c1a6d1f93a171128962390cb28080507202f6e914a

Indexes: implementation 815 (+51), subsystems 133 (+1), modules 481 (+13), sources 470 (+13),
diagnostics 31 (+1), tests 389 (+23), NIDs 2 unchanged observation-only/unregistered, ABI 0.
Total 2321 (+102). No dependency changes. Machine-readable indexes and local tracker/corpus files
remain ignored; no machine paths were added to tracked source or documentation.

M26 remains active ready_for_cleanup pending approval. No commit/push/history changes; HEAD and
origin/main remain f15a7b633cdf64b5eb2a05722127587a5c8184ef. M27 has not started.

## M27 results - 2026-09-09

[Guest staging](guest_image_staging.md) records focused catalogue evidence, native CPU direction,
blocker policy, real results and pending execution requirements. No guest instructions executed.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 329 executable Rust tests; 23 doctests passed |
| CLI integration tests | 50 included above; passed |
| Python discovery | 51 passed: 21 policy/state, 9 structure, 21 index/extraction |
| policy/structure | 16 crates, 11 internal edges, 241 homes; passed |
| indexes | 16 generated files byte-identical across two generations; fresh |
| whitespace | supplemental 578 files and git diff --check passed |

Added 16 tests: 10 core composition/transaction, 4 memory backend and 2 CLI tests. Coverage includes
copy/provenance, BSS, exact/refused budgets, alignment/overlap, known writes, unchanged pending bytes,
selected-but-unstaged provider values, no execution readiness, repeated release/drop, deterministic
staging and an injected post-copy write failure with zero surviving mappings. One early test used
an unknown-width relocation to try to prove write overlap; corrected to two known-width writes.
All final tests passed. No GUI/timing/native-entry changes or guest execution tests were introduced.

Manual commands exactly match USAGE.md's stage-image block, reusing its explicit corpus/plan limits:
`cargo build -p astero-cli`, then `.\target\debug\astero-cli.exe stage-image --path <role.path>
@planLimits --max-mapped-bytes 67108864` (PowerShell call operator). Roles and observed results:

| Role | Regions | Bytes copied / zero-fill | Applied / pending | Unresolved | Exit / teardown |
|---|---:|---|---|---:|---|
| linkage_sample | 5 | 3336 / 31 | 2 / 9 | 8 | 0 / zero mappings |
| utility_build_comparison | 5 | 3312 / 31 | 2 / 9 | 8 | 0 / zero mappings |
| primary_real_elf | 5 | 9164955 / 8259680 | 29791 / 1107 | 822 | 0 / zero mappings |

All StagedWithPendingWork, MetadataOnly, ready_for_execution=false. SHA256 before/after matched all
three M26 hashes; inputs unchanged. Initial run refused unknown PH 0x6fffff00 before allocation;
its narrow execution-blocker treatment is an explicit structural-staging hypothesis, not established
payload semantics. Latest runs reproduced all counts after final backend-reservation wiring.
A max-mapped-bytes=1 utility run returned Budget { required: 3367, maximum: 1 }, exit 1, zero mappings.
Full local logs are ignored target/m27-validation; no native input paths entered tracked records.

Indexes: implementation 870 (+55), subsystems 134 (+1), modules 490 (+9), sources 479 (+9),
tests 405 (+16), diagnostics 32 (+1), NIDs 2 unchanged observation-only/unregistered, ABI 0.
Total 2412 (+91). No external dependency; only core -> memory activated. Existing loader and timing
remain dependency-free. No history mutation/commit/push: HEAD and origin/main remain a9f3e77.
M27 awaits approval for tracker removal; M28 has not started.

## M28 results - 2026-09-09

[Windows native VM](windows_native_vm.md) records the narrow unsafe exception, exact placement,
page-sharing constraints, catalogue/Microsoft evidence, real results and remaining native-entry work.
No guest instructions were executed.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 345 executable Rust tests and 23 doctests passed |
| CLI integration tests | 52 included above; passed |
| Python discovery | 52 passed: 22 policy/state/safety, 9 structure, 21 index/extraction |
| policy/structure | 16 crates, 11 internal edges, 244 homes; passed |
| indexes | two generations byte-identical across 16 files; fresh |
| whitespace | supplemental scan and git diff --check passed |

Added 16 Rust tests: 12 memory layout/Windows ownership tests, two core composition tests and two
CLI tests. Coverage includes exact/refused budgets, page union/padding and W+X refusal, holes,
collision ownership, query/readback, protected reads, repeat release/drop and separate owners.
Private fault checkpoints after actual commit, before protection and after protection prove rollback
and address reuse; these simulate transaction failures, not forced Windows API failures. A new
Python test pins the memory unsafe exception to the private Windows leaf. Final full validation
followed zero-initialization of FFI output storage, including reserved fields.

Manual commands exactly match USAGE.md's native-map block, reusing its corpus and plan limits:
`cargo build -p astero-cli`, then `.\target\debug\astero-cli.exe native-map --path <role.path>
@planLimits --max-mapped-bytes 67108864 --max-native-bytes 67108864` with PowerShell call operator.

| Role | Reserved / committed bytes | Pending relocations | Exit / teardown |
|---|---|---:|---|
| linkage_sample | 53248 / 16384 | 9 | 0 / zero native and byte owners |
| utility_build_comparison | 53248 / 16384 | 9 | 0 / zero native and byte owners |
| primary_real_elf | 17457152 / 17432576 | 1107 | 0 / zero native and byte owners |

Expected and actual: exact placement at 0x100000000, OS protections verified, readback verified,
instruction cache flushed, NativeBackedWithPendingWork, no execution. All three input SHA256 hashes
were unchanged. Replacing --max-native-bytes with 1 for linkage_sample returned reservation Budget
(required 53248, maximum 1), exit 1 and zero reservations/release errors. Collision and injected
rollback checks use synthetic OS-backed tests. RELRO remains explicitly deferred until pending
writes/provider closure; ordinary page protections are enforced now. Logs remain ignored under target.

Indexes: implementation 920 (+50), subsystems 136 (+2), modules 502 (+12), sources 489 (+10),
tests 422 (+17), diagnostics 33 (+1), NIDs 2 unchanged observation-only/unregistered, ABI 0.
Total 2504 (+92). No dependency/edge additions. Local AGENTS policy exception remains ignored.
HEAD and origin/main remain 0179c2093b3e9e8a12a29916bf6c158a05fde76b; no commit/push/history changes.
M28 awaits approval for active-item removal. M29 has not started.

## M29 results - 2026-09-10

[Runtime entry preparation](runtime_entry_preparation.md) records the exact catalogue/firmware routes,
selected workload, argument/TCB hypotheses, ownership, real results and remaining entry blockers.
No guest instructions, constructors, native import landing or runtime entry trampoline were executed.

| Check | Result |
|---|---|
| cargo fmt --all -- --check | passed |
| cargo check --workspace --all-targets | passed |
| cargo build --workspace --all-targets | passed |
| cargo clippy --workspace --all-targets -- -D warnings | passed |
| cargo test --workspace | 365 executable Rust tests and 23 doctests passed |
| CLI integration tests | 54 included above; passed |
| Python discovery | 52 passed: 22 policy/state/safety, 9 structure, 21 index/extraction |
| policy/structure | 16 crates, 16 internal edges, 250 module homes; passed |
| indexes | fresh; two byte-identical generations across 16 files |
| whitespace | supplemental scan and git diff --check passed |

20 new Rust tests: ABI 2, HLE 4, kernel 8, core 3, memory 1, CLI 2. Tests exercise explicit argument
encoding/context, host-only provider return/ambiguity/panic rules, bounded stack/TLS layout, native
NOACCESS guard and template/self-pointer readback, TLS-collision rollback, recovery first-stop contract,
far-indirect stub *encoding only*, runtime ownership, consumed-image refusal, exact-page RELRO query,
and CLI budget propagation. No native-entry success or actual fault-recovery claim comes from these.
One overflow test initially failed because its chosen extent did not overflow; corrected to cross the
boundary. A CLI fixture initially failed at an earlier invalid relocation target, then was corrected
using the existing staging fixture addresses to reach the intended runtime-budget boundary.

Manual command: the exact USAGE.md entry-readiness block with primary_real_elf and the existing
explicit plan budgets, --max-mapped-bytes 67108864 --max-native-bytes 67108864 --stack-base 8589934592
--stack-bytes 8388608 --tls-base 8858370048 --max-runtime-bytes 16777216. Expected and actual:
exit 0, PreparedBlocked, EntryReady=false; RIP 0x100000070, RSP 0x200800fb8 (mod16=8), RDI 0x200800fc0,
empty PT_TLS with experimental TCB at 0x210000000. 822 external references, 1107 pending relocations,
38 dependencies, zero HLE registrations/native matches. RELRO PendingWrites; recovery model Prepared,
native adapter not installed. All image/stack/TLS reservations and byte mappings were zero at teardown.
Repeated preparation produced the same report. With --max-runtime-bytes 1: exit 1, structured Budget
(required 8396800, maximum 1); ownership rollback is covered by the core test. Selected input SHA256
before/after: A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Ignored target/m29-validation contains logs; no machine-specific path enters tracked documentation.

A separate bounded read-only 96-byte Capstone experiment corroborated RDI record consumption and
first PLT references _init_env/atexit through slots 0x77acc8/0x77acd0. These names remain documentary
catalogue correlations, not registered HLE. _init_env's legacy zero-return stub was explicitly rejected
as proof of environment closure. PT_TLS emptiness was refined to mean no main-module template, not
no libc TCB/errno requirement. Native bridge/return/exit landing, FS activation, fault adapter, pending
provider writes and initializer ordering remain immediate blockers; no ready token is fabricated.

Indexes: implementation 993 (+73), subsystems 137 (+1), modules 515 (+13), sources 502 (+13),
tests 442 (+20), diagnostics 34 (+1), NIDs 2 unchanged observation-only/unregistered, ABI 1 (+1).
Total 2626 (+122). ABI record is the legacy-correlated process-argument encoder, not complete native ABI.
Five already-allowed edges activated; no external dependency, new crate or unsafe-policy relaxation.
HEAD and origin/main remain 0b19a5fc18c0920dcaf678838ef4ec70b8554989. No commit/push/history changes.
M29 awaits approval for active-item removal; M30 has not started.


## M30 native entry closure validation

On x86-64 Windows: cargo fmt --all -- --check; cargo check --workspace --all-targets;
cargo build --workspace --all-targets; cargo clippy --workspace --all-targets -- -D warnings;
cargo test --workspace. All pass. 381 executable Rust tests and 23 doctests pass;
56 CLI tests are included (also run separately). 16 Rust tests added: kernel 9 (including
one internal classifier test), libs 2, core 2, memory 1 and CLI 2. Direct synthetic assembly
checks eight Windows nonvolatile GPRs and XMM6-XMM15 across return/import/fault paths, FS/GS
restoration, illegal instruction/access violation and NOACCESS recovery, provider panic/refusal,
ordinal dispatch, exclusive handler lifetime, callback limits/duplicates and token refusal.
No timing tolerances or real guest execution are involved. Core tests prepare, never execute,
synthetic ELF fixtures. RW writes preflight an entire range before modifying memory.

All 53 Python tests pass: policy 21, structure 9, native unsafe ownership 2, index tests 21.
Policy reports 16 crates, 18 internal dependency edges, 250 declared module homes. Existing
allowed core-to-libs and libs-to-kernel edges activated; no external dependency or new crate.
Unsafe is isolated to the new private kernel native leaf, enforced by the added policy test.
Initial Clippy caught two context-pointer test assignments; explicit refreshed pointer handoffs
fixed them and warnings-denied validation passed. No warnings were suppressed.

Manual commands are the exact USAGE.md closure and bridge_smoke blocks. With local corpus role
primary_real_elf, existing plan limits, --close-entry, and --max-runtime-bytes 16777216:
expected/actual exit 0, experimental EntryReady=true and token issued then dropped. RIP
0x100000070, RSP 0x200800fb8, FS 0x210000000. 1107 writes treated: 836 function landings
(two exact startup matches; 834 unresolved) and 271 object writes to 17 guarded identities.
20,480 RX landing bytes plus 69,632 NOACCESS bytes; existing runtime storage 8,396,800 bytes.
RELRO finalized on committed pages, 12,288-byte inaccessible gap preserved, zero untreated writes.
Image/stack/TLS/byte ownership counters and release-error counts are zero after teardown.
Unknown object contents, unknown function implementations, TCB/environment completeness and
initializer policy remain explicit early-runtime risks, not silently resolved providers.

Repeat with --max-runtime-bytes 8396800: expected/actual structured Closure: Budget, exit 1.
The cap fits M29 but refuses M30 additional mappings. `cargo run -p astero-kernel --example
bridge_smoke` returns exit 0 and SYNTHETIC BRIDGE VALIDATED, including controlled codes
0xc000001d and 0xc0000005 with host_fs_restored=true and host_gs_preserved=true.
No file argument exists for the synthetic bridge. Source SHA256 before/after both real
preparations: A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Logs are local under ignored target/m30-validation. No source bytes are modified or executed.

Index counts: implementation 1057, subsystems 137, modules 525, source files 511, tests 459,
diagnostics 35, NIDs 4, ABI 2; total 2730. Two old NIDs stay observation-only/unregistered;
two new startup registrations describe their narrow implementations (experimental _init_env,
registration-only atexit). The new ABI record covers synthetic native transitions, not a complete
PS5 ABI. The process-argument record remains unchanged. Index discovery is not runtime proof.

Two regenerations were byte-identical across all 17 index-directory files; freshness passed.
Policy/state/structure and supplemental whitespace (613 files), plus git diff --check, pass.
HEAD and origin/main remain 542006b5d0efc0da981be0fa453c913131dc38a8. No commit, push or history
change. M30 is ready_for_cleanup awaiting approval; M31 has not started. Local corpus, generated
JSON, AGENTS.md and PROJECT_STATE.json remain ignored/local-only.


## M31 first controlled native entry validation

Before real entry: formatting, workspace check/build, warnings-denied Clippy, all 386 then-current
Rust tests and 23 doctests passed; policy/state/structure and fresh indexes passed. Synthetic
supervisor_smoke returned exit 0 under independent 5-second process containment: 50 ms deadline,
51,607 us observed, actual RIP inside the exact infinite-loop range, one SuspendThread and one
ResumeThread, host FS restored/GS preserved, execution thread joined and bridge re-acquired.
This was the mandatory gate. No real corpus bytes ran in that proof.

ONE real execution followed, using the exact USAGE.md first-entry command: primary_real_elf,
existing explicit plan/native/runtime limits, --wall-ms 250 --containment-ms 15000. Expected: first
controlled boundary and joined teardown. Actual: exit 0, GuardedObject read AV 0xc0000005 at RIP
0x100332abf, RSP 0x200800f00, address 0x210021000, symbol 7. Execution thread 25816, core interval
406 us, armed supervisor interval 150 us, no intervention/suspension. _init_env returned 0; two
atexit calls returned 0xffffffff for host landing and 0 for guest fini 0x100524270. One callback
retained, none invoked. All image/stack/TLS/landing/trap reservations zero, no release errors,
thread joined and parent process containment Clean. This is real guest execution and controlled
recovery, not game boot. Full context and assumptions are in first_native_entry.md.

Source SHA256 before/after:
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
A separate existing ps5-identity read-only command (same acquisition/observation limits, no execution)
correlated symbol 7 with f7uOxY9mM1U#k#P, 0x7fbb8ec58f663355, libkernel metadata indexes 144/57.
No missing object or startup behavior was implemented after observing it. No second real run.

Post-run: formatting, workspace check/build, Clippy -D warnings and workspace tests pass:
388 executable Rust tests, 23 doctests. 58 CLI tests are included and also pass separately.
Seven tests added: kernel 3 (loop/preemption, supervised stop paths, refusal), core 2 (authority
and process containment), CLI 2 (explicit limits/refusal). Existing concurrent-adapter test now
checks another thread too. Synthetic cases cover normal return, HLE, unknown import, AV, illegal
instruction, guarded read and deadline. Child tests distinguish success, nonzero failure and a
killed/joined hung worker; no TerminateThread. Compact report formatting and associated relocation
retention were added after the one real run and validated synthetically, not by replaying the guest.

53 Python tests pass: policy 21, structure 9, native ownership 2, indexes 21. 16 crates, 19 internal
edges and 250 declared homes. Only new dependency edge is kernel -> timing; no external dependency.
An initial anonymous-const logical-ID collision was resolved by one combined layout assertion;
no index-generator exception or weakened validation was introduced. An initial clone-on-Copy
Clippy failure was corrected without lint suppression. Index discovery is not execution proof.

Indexes: implementation 1080, subsystems 137, modules 527, sources 513, tests 466, diagnostics 36,
NIDs 5, ABI 2; total 2766. Three NIDs are observation-only/unregistered, two are the existing narrow
startup registrations. ABI contracts unchanged in scope. Local logs under target/m31-validation
retain raw first-stop output; paths and generated JSON remain local-only.

Two index regenerations were byte-identical across all 17 index-directory files. Freshness,
policy/state/structure, supplemental whitespace (616 files) and git diff --check pass.
M31 is ready_for_cleanup awaiting user approval; M32 has not started. HEAD and origin/main remain
41421b88bc961eb95f923d651ca4f59960ec0be5; no commit/push/history changes. Source paths, active state,
AGENTS.md, corpus configuration and generated JSON remain local-only/ignored.


## M32 startup foundation migration — 2026-09-10

Full cargo fmt/check/build/Clippy -D warnings/workspace tests passed: 402 executable Rust tests,
23 doctests. 14 added tests (libs 11, memory 2, core 1). Existing CLI integration coverage remains
included. Python: 53 passing tests, comprising policy 21, structure 9, native policy 2 and indexes 21.
Policy: 16 workspace members, 21 internal dependency edges, 250 module homes. Two added edges
are libs test-only ABI/memory; no new runtime/external dependency. Whitespace: 626 files and
Git diff --check pass. Source-backed indexes: 1154 implementation, 137 subsystems, 536 modules,
522 source records, 480 tests, 36 diagnostics, 34 NIDs and 2 ABI; total 2901.

Before real execution: complete Rust checks and synthetic supervision passed; 50 ms loop observed
52,204 us with one suspend/resume, restored FS/GS and joined thread. Three controlled real runs,
all on primary_real_elf with required wall-ms 250 and containment-ms 15000, were authorized by
M32's within-cluster adaptation rule. First stopped at __cxa_atexit (306 us), second at operator
new (241 us); existing prototype behavior was adapted and focused checks rerun before continuing.
Third stopped at scePthreadRwlockInit (947 us), outside this startup wave. 52 completed HLE calls;
15 live allocations/1664 aligned bytes before teardown, 16 retained callbacks, none invoked.
All runs exit 0 with Clean containment, joined thread, zero reservations/release errors and unchanged
SHA256 A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
No fourth runtime run. Later ENOMEM propagation refinement is synthetic-tested only.
Full context, evidence, limits and hypothesis outcomes: startup_foundation.md. Local raw logs
remain ignored under target/m32-validation. M33 is not started; no commit/push/history changes.


## M33 pthread synchronization migration - 2026-09-10

Full workspace fmt/check/build, warnings-denied Clippy and cargo test passed: 426 executable Rust
and 23 doctests. Added 24 tests: 19 synchronization, 4 guest-provider integration, 1 synthetic native
blocked-HLE wait. 53 Python policy/state/structure/native/index tests passed. 16 crates, 22 internal
edges and 250 homes. Only new dependency is core test-only ABI. Whitespace: 634 files and Git diff
check pass. Index counts: implementation 1193, subsystems 137, modules 543, sources 529, tests 504,
diagnostics 37, NIDs 116, ABI 2; total 3061. Registered NIDs 113, unregistered 3.

Preflight passed before ONE real primary_real_elf execution, exact existing first-entry command
with wall-ms 250 and containment-ms 15000. scePthreadRwlockInit, scePthreadCondInit and
scePthreadMutexInit each succeeded once; three retained objects, zero waits/wakes/timeouts.
74 total completed provider calls. New stop scePthreadAttrInit 0x9ec628351cb0c0d8, ordinal 119.
Elapsed 1406 us, RSP 0x200800e68, host landing RIP 0x7ff72df56d06, separate guest continuation
0x1001c2814. Clean parent containment/exit 0, FS/GS preserved/restored, joined thread, zero native
reservations and no release errors. Hash unchanged:
A15E44CAF80BE1D86B72C2C1A6D1F93A171128962390CB28080507202F6E914A.
Full source/evidence/limits/outcomes: pthread_foundation.md. Logs under ignored target/m33-*.log.
Synthetic 50 ms infinite loop stopped after 55,525 us with balanced suspend/resume. Post-run synthetic
native HLE blocked-mutex test also passed; no second real run. No M34 implementation or commit/push.

Final M33 index regeneration was byte-identical across all 17 index-directory files; freshness,
policy/state/structure, all 53 Python tests and whitespace checks passed after the shutdown fix.


## M34 pthread lifecycle migration - 2026-09-11

Final formatting/check/build (workspace all-targets), warnings-denied all-targets Clippy and full
workspace tests passed: **453 executable Rust tests and 23 doctests**, no failures/ignored tests.
27 added tests: kernel lifecycle 12, native bridge workers 3, core composed workers 7, guest adapter
contracts 5. Existing executable CLI integration, M31 supervision and M33 synchronization remain
included. Focused tests cover OS workers with synthetic native bytes, private stacks/TLS/errno,
return/exit/join/detach, exact provider keys, attr errors, publication rollback, condition wait/relock
and shutdown interruption. Synthetic ordinary infinite-loop worker at 0x740000000 was interrupted
by the existing supervisor's 25 ms deadline; actual captured RIP in loop, balanced suspend/resume,
restored FS/GS, joined host and zero storage reservations are asserted. The test makes no hard host
scheduling latency or instruction-count claim.

Python: 53 tests passed (policy/state/structure/native-policy/index), 16 workspace members,
22 internal edges, 250 module homes. No dependency/crate changes. Supplemental whitespace 642
files plus git diff --check pass. Index counts: implementation 1266, subsystems 137, modules 551,
sources 536, tests 531, diagnostics 38, NIDs 176, ABI 2; total 3237. 174 registered NID identities and
2 observation-only/unregistered. ABI records retain process arguments and synthetic native boundary.

One real run occurred only after preflight (full Rust and synthetic supervisor tests, check/build/
Clippy and policy). Exact command and expected/actual outcome: USAGE.md M34; complete evidence:
pthread_lifecycle.md. Primary corpus role only, wall-ms 250/containment-ms 15000, process exit 0,
Clean containment, 10,121 us report duration. Eight native workers, seven real condition waits,
zero signal wakes/timeouts. All waits interrupted on shutdown. First outside-lifecycle stop:
libc memset 1,352,000-byte request explicitly refused by existing 1 MiB per-operation limit.
No further real execution or libc-limit modification. Main and all workers restored FS/preserved GS,
joined, mappings/reservations 0, release errors [], source SHA unchanged (full hash in capability doc).

Post-run refinements added typed interrupted/faulted outcomes and explicit call dispositions,
main-thread layout/outcome, output-pair preflight and bounded trace preallocation; synthetic tests
and full final suite passed afterward. The raw ignored log is target/m34-real-1.log; full final Rust,
Python and policy logs are target/m34-final-{rust,python,policy}.log. No claim that cancelled waits
returned successfully or that unexercised lifecycle exports have runtime proof.

Final regeneration is verified byte-identical across all 17 index files; freshness and policy checks
are rerun after durable documentation and ready_for_cleanup tracker updates. M34 stays active pending
approval; no M35 work, commit, push or history change.

## M35 libc large-memory continuation - 2026-09-11

Final workspace formatting, all-target check/build, warnings-denied all-target Clippy and workspace
Rust tests passed: 474 executable tests and 23 doctests. Added 21 tests: 16 libc checked-memory/string/
allocation/output tests, three native cross-mapping tests, one concurrent heap test and one composed
synthetic native worker large-memory test. Existing CLI, M31 supervisor, M33 synchronization and M34
worker lifecycle regressions remain included. No real corpus bytes are used by synthetic tests.

Python: all 53 tests passed, including policy/state/structure and index checks. Ownership remains
16 crates, 22 internal edges and 250 declared module homes. Index counts: implementation 1319,
subsystems 137, modules 560, sources 545, tests 552, diagnostics 39, NIDs 201, ABI 2; total 3355.
NIDs: 198 registered identities and 3 observation-only/unregistered. ABI records unchanged.

Three authorized real runs continued only within the approved libc cluster, then stopped at the
outside-cluster sceUserServiceInitialize boundary. Exact command: USAGE.md M35. Full evidence,
identities and per-run PCs: libc_large_memory.md. All three exits 0, Clean containment and unchanged
source SHA256. Final duration 13,256 us (native supervisor 12,059 us), 250 ms deadline, no asynchronous
intervention. The 1,352,000-byte memset succeeded at [0x100906e68,0x100a50fa8). Final heap peak 7104
bytes supports retaining one bounded 4 MiB arena rather than speculative growth. Eight workers
joined; eight condition waits were interrupted by shutdown, with no signal wakes or timeouts.
Final native/runtime reservations 0, release errors [], host FS restored and GS preserved.

Logs remain ignored under target/m35-real-{1,2,3}.log and target/m35-final-{rust,python}.log.
Post-run synthetic refinements preserve structured OS copy failure, alignment errno and worker access;
the final suite passed without additional real execution. Repeated index generation is byte-identical
across all 17 index files; freshness, policy and whitespace checks are repeated after these records.
M35 remains ready_for_cleanup pending approval. No M36 work, commit, push or history change.


## M36 UserService startup migration - 2026-09-11

Formatting, all-target workspace check/build, warnings-denied Clippy and full Rust suite pass:
489 executable tests and23 doctests. Fifteen added adapter tests cover lifecycle, exact identity,
outputs, invalid ranges, queue ordering/retry, settings, concurrency and runtime ownership.
Existing M31 supervision, M33 synchronization, M34 workers and M35 large-memory regressions pass.
First preflight exposed the256-entry registry cap; extending it by the15 M36 entries fixed worker
creation. Initial failure poisoned the serial test lock, producing seven secondary failures; fresh
rerun passed all8 composed worker tests, followed by two full passing suites. No real execution
occurred before passing full preflight/policy. A missing tracker completion list was also corrected.

One primary_real_elf run: exact USAGE M36 command,250ms/15000ms limits, exit0/Clean,
17,851us overall /16,206us native. Initialize returned0; user0x10000000 logged in,one pending login.
No other UserService API observed. Next unresolved libc/libc vsnprintf NID0x43657E8AABE3802D,
ordinal328; no formatting implementation added. All8 workers joined,8 waits interrupted,0 wakes/
timeouts, native/runtime reservations0,releaseerrors[]. Host FS restored,GS preserved,sourceSHA
unchanged; exact hash/context/arguments in user_service.md.

All53 Python tests pass. Policy:16 crates,22 internal edges,251 module homes; tracker valid.
Index total3419: implementation1343,subsystems138,modules564,sources549,tests567,diagnostics40,
NIDs216 (213 registered,3 unregistered), ABI2. All17 index files regenerate byte-identically.
Freshness, policy/state/structure, supplemental whitespace and git diff --check pass after final docs.
Local logs target/m36-{preflight-rust,final-rust,final-python,worker-regression,real-1}.log stay ignored.
M36 ready_for_cleanup pending approval; no M37, commit, push or history rewrite.


## M37 libc formatting/guest-varargs migration — 2026-09-11

All-target workspace check/build, formatting and warnings-denied Clippy pass. Final full
Rust suite: 514 executable tests and 23 doctests, no failures/ignored. Added 24 formatting
integration tests plus one composed native two-worker variadic test. Existing CLI, native
supervisor, synchronization, lifecycle and checked-memory suites included. All 53 Python
tests pass, including policy/state/structure and index tests. Policy: 16 crates, 22 internal
edges, 252 module homes. No extra crate or external dependency.

First synthetic worker probe refused because its old fixture supplied libkernel identity
for libc snprintf; fixture corrected to exact libc/libc, with no NID fallback. Fresh test and
both subsequent full suites pass. Clippy caught one inherited collapsible conditional, fixed.
No real execution happened before full passing preflight and policy/index freshness.

One real primary_real_elf run (exact USAGE M37 command): exit 0, Clean containment,
250 ms wall/15 s outer bound; elapsed 12,797 us overall /11,743 us native. vsnprintf and
printf each called three times, returned 36/47/38, no truncation or formatting refusal.
Output exposes three guest mutex failure messages. Next boundary is outside formatting:
NID 0x836B558852288471, libSceAudioOut2/libSceAudioOut, unresolved ordinal 138.
Full pointers, format strings, context distinctions, source SHA and teardown are in
libc_formatting.md. Eight workers joined, waits interrupted, FS restored/GS preserved,
zero reservations, no release errors; source SHA unchanged. No additional run or audio fix.

Index counts: implementation 1388, subsystems 138, modules 571, sources 556, tests 592,
diagnostics 41, NIDs 224 (221 registered, 3 unregistered), ABI 3; total 3513.
One ABI record adds explicit scalar va_list; unobserved formatting conversions/export paths
remain synthetically tested, not runtime-confirmed. Generated JSON stays ignored/local-only.
Repeated regeneration checks all 17 index files for byte identity; final freshness, policy,
supplemental whitespace and git diff checks are recorded at completion.
Logs remain ignored under target/m37-{preflight-rust,final-rust,final-python,real-1}.log.
M37 remains active ready_for_cleanup pending approval; no M38, commit, push or history rewrite.

## M38 AudioOut/media-startup migration — 2026-09-11

Full preflight passed before real execution. Final fmt/check/build (all targets), warnings-denied
Clippy and workspace Rust suite pass:541 executable tests,23 doctests,zero failures/ignored.
Added27 tests (16 service,10 core adapters,one mutex regression); refined existing copied-mutex
identity test. CLI, M31 supervisor, M33 waits, M34 native workers, M35 memory and M37 formatting
regressions included. Audio manual-clock/cancellation/batch tests pass; actual audio wait/output
behavior remains unobserved in the real workload. All53 Python tests pass, including index and
policy/state/structure. Policy:16 crates,25 internal edges,252 module homes.

One primary_real_elf run used the exact M38 USAGE command:250ms wall/15000ms containment,
exit0/Clean,16489us overall/15283us native. Six AudioOut2 startup calls succeeded; three owned
objects created/closed; no buffers/waits/events. Four mutex creations now succeed; the three
failure messages disappear. Next unknown ABI: GetSpeakerInfo0x0C89B3D85B7D1368. No PS5Rust
implementation or complete firmware output contract; no fabricated response or unrelated fix.
Eight workers joined, condition waits interrupted, FS restored/GS preserved, zero reservations,
zero audio objects/tickets, releaseerrors[], identical source SHA. Full context/evidence in
[audio startup](audio_startup.md); raw logs target/m38-real-1.log remain local/ignored.

Indexes:implementation1434,subsystems138,modules575,sources560,tests619,diagnostics42,
NIDs249 (246 registered/3 unregistered),ABI3,total3620. Audio family25 registered,only six
runtime-confirmed; new SpeakerInfo observation unregistered. No new ABI claim.
Final freshness and repeated generation cover all17 files for byte identity; policy, supplemental
whitespace and Git diff checks must remain clean. M38 ready_for_cleanup pending approval;
no M39,commit,push or rewritten history. Logs target/m38-final-{check,build,clippy,rust,policy,python}.log.

## M39 GetSpeakerInfo ABI and AudioOut continuation — 2026-09-11

Preflight all-target check/build,Clippy and full Rust tests passed before real execution,with
policy/index checks. Final formatting,workspace all-target check/build,warnings-denied Clippy
and Rust suite pass:547 executable tests,23 doctests,zero failures/ignored. Six new tests cover
speaker layout/global semantics/errors/ranges/selector width/shutdown. Existing CLI,native
supervisor,M33 mutex/waits,M34 workers,M35 memory,M37 formatting,M38 AudioOut tests included.
All53 Python tests pass; policy16 crates/25 edges/252 module homes; active state valid.

One real run: exact M39 USAGE command,250ms wall/15000ms containment,exit0/Clean.
13195us overall/12154us native. GetSpeakerInfo once returned0,PortGetState once returned0;
next unresolved sceKernelUsleep0xD637D72D15738AC7(argument1000),outside AudioOut.
No audio buffers/waits; eight workers interrupted/joined; zero audio/native/runtime resources,
sourceSHA unchanged. See audio_speaker_info.md for firmware/hash/call-site evidence,field
confidence and raw context. No second-title run. No physical playback validation claim.
Final refinements explicitly stored Stereo in service state and tested u32 selector width;
these preserve selector0 bytes from the real run and did not warrant another depth-chasing run.

Index counts:implementation1441,subsystems138,modules576,sources561,tests625,diagnostics42,
NIDs250(247registered,3unregistered),ABI4,total3637. One new ABI; one provider promoted;
new kernel sleep observation unregistered. Repeated generation checks all17 index files for
byte identity; freshness,policy/state/structure,supplemental whitespace and git diff checks
required clean at completion. Raw validation logs target/m39-{pre*,final-*,real-1}.log are ignored.
M39 ready_for_cleanup pending approval; no M40 implementation,commit,push or rewritten history.

## M40 kernel clocks and sleep migration - 2026-09-11

Preflight formatting, all-target check/build, warnings-denied Clippy, workspace Rust tests and
policy/index validation passed before the real run. Final Rust suite passes: 572 executable tests,
23 doctests, zero failures/ignored. Added 25 tests: 15 kernel conversion/wait tests, eight checked
provider tests, two synthetic native worker tests. Existing M25 core timing test is preserved in
its original file; new adapters use guest_timing.rs. M31 supervisor, M33 synchronization, M34
worker lifecycle, M35 memory, M37 formatting and M38/M39 audio regressions all run in this suite.
Real-time tests use tolerant host guards; manual clocks prove no early completion without sleeps.

One real primary_real_elf run, exact M40 USAGE command: 250 ms wall/15000 ms containment.
sceKernelUsleep(1000) returns 0; 1 ms requested, 12.582 ms observed, 11.582 ms lateness.
No clock query is runtime-confirmed. Next boundary: unresolved powf, 0xD43D07D8A363B211,
libc/libc. Total 29017 us, native supervision 27757 us; no redirection/suspend/resume.
Eight workers joined, eight condition waits interrupted, zero remaining waiters/timing tickets,
zero native/runtime reservations, FS restored/GS preserved, release errors empty, containment Clean.
SHA256 unchanged; full boundary context and evidence distinctions are in kernel_timing.md.
No second-title run or math migration. Raw logs remain ignored under target/m40-*.

Indexes: implementation 1483, subsystems 138, modules 582, sources 567, tests 650,
diagnostics 43, NIDs 268 (265 registered / 3 observation-only), ABI 5; total 3736.
18 timing registrations include scoped explicit CPU-clock refusals; only Usleep is runtime-confirmed.
One new timespec representation ABI, not a full kernel clock contract. No new crates/dependencies:
16 crates, 25 internal edges, 252 module homes. All 53 Python tests passed. Final freshness and policy/state/structure passed; all 17 index files regenerate byte-identically.
Supplemental whitespace and git diff --check pass. M40 remains active ready_for_cleanup pending approval.

## M41 scalar math and cross-title smoke - 2026-09-11

Final formatting, workspace all-target check/build, warnings-denied Clippy and full Rust suite pass:
594 executable tests and 23 doctests, zero failures/ignored. Added 22 tests: 20 scalar contract tests,
one native float/double worker probe, one call-budget lifecycle test; extended CLI limit rejection
coverage. Existing supervisor, synchronization, workers, checked memory, formatting, AudioOut and
timing regressions are included. No primary rerun followed Phase B because no runtime fix followed it.

Primary first run: powf succeeded; 3737 sincosf calls then 4096-call allowance exhaustion.
After adding an explicit bounded --max-hle-calls option, full preflight passed again. Continuation
with 65536 calls and unchanged 250 ms/15000 ms bounds recorded powf once and sincosf5086 times;
next unresolved libc/libc __cxa_guard_acquire 0xDC63E98D0740313C. Duration96841 us overall,
95381 us native; no supervisor redirection. Eight workers joined, zero reservations/waiters,
FS restored/GS preserved, empty release errors, Clean containment, unchanged source SHA.

One distinct named_title_elf attempt followed the passed Phase A gate. Acquisition/plan and byte
staging succeeded; native reservation failed at0x100000000,size29134848,Windows487. No EntryReady,
no second guest code, no second HLE calls/guest workers or runtime-duration claim. Parent reaped
child exit1 (WorkerFailure), not reported as clean native recovery. No successful failed-branch
reservation was published; no numeric runtime mapping snapshot is claimed for this failure path.
Separate offline stage-image commands confirm both byte stages and zero mappings after release.
The second source SHA is unchanged. Exact plans, hashes, context and evidence limits are in
libc_scalar_math.md; commands are in USAGE. Logs target/m41-* remain ignored/local-only.

NIDs321:318 registered/3 observation-only;53 scalar registrations,only powf/sincosf runtime-confirmed.
ABI5 unchanged. One new libs/libc/math home;16 crates/25 edges/253 module homes. All 53 Python tests pass, including policy/state/structure and index freshness.
All 17 index files regenerate byte-identically. Supplemental whitespace and git diff --check pass.
Indexes: implementation1497, subsystems138, modules586, sources571, tests672, diagnostics43,
NIDs321, ABI5; total3833. M41 active ready_for_cleanup pending approval. No commit/push or M42 implementation.
