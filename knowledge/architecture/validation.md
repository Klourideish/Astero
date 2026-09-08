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
