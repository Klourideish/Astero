# astero-cli

Creates a core-owned unloaded session in main and formats the same debugger report used by GUI/tests. Default run prints state; unsupported arguments fail. No separate emulator state.

## Module ownership

main: composition and entry point; lib: pure report formatting.

No guest mechanisms, fabricated results or toolkit types may enter core/debug contracts.
Integration tests live under this package's tests/. GUI also has pure swapchain-selection unit tests.

See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[session contract](../../knowledge/architecture/session_observation.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[GUI decisions](../../knowledge/architecture/gui_framework.md) and
[validation](../../knowledge/architecture/validation.md).
