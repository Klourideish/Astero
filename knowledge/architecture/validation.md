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
