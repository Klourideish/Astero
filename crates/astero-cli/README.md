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

M11 linkage::render consumes debugger linkage snapshots. --linkage [--details] reports absence
until an input adapter exists; tests exercise nonempty generated reports. The loader edge is
dev-only. See [report contract](../../knowledge/architecture/linkage_reporting.md).

M12 --linkage --synthetic [--details] composes generated loader evidence into a real host session.
The loader edge now supports that production demo; rendering still consumes debug snapshots.
No file input is supported. See [composition](../../knowledge/architecture/evidence_composition.md).
