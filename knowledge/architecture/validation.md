# Validation

Run from the workspace root:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python tools/check_policy.py
python -m unittest discover -s tools -p test_policy.py -v
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
