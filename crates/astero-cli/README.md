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

M13 moves the shared demo generator to loader and composition to core; CLI calls core and its direct loader edge is dev-only again. No file input or resolution is added.

M15 acquisition/ owns native argument selection, Ready/Acquired/Failed state and human presentation. Both --max-bytes and --max-read-calls are mandatory. It delegates through core input, without session creation or parsing. See [input selection](../../knowledge/architecture/acquisition_frontend.md).

M16 inspect requires all acquisition arguments plus --max-program-headers. It prints acquisition separately from immutable header-only inspection. acquire and synthetic linkage remain separate. See [usage](../../USAGE.md) and [inspection](../../knowledge/architecture/explicit_inspection.md).

M17 dynamic requires acquisition limits, --max-program-headers and --max-dynamic-entries. It is distinct from inspect and linkage. See [usage](../../USAGE.md).

M18 descriptors requires all dynamic/acquisition limits plus --max-descriptors. It reports selected structural metadata, never payload contents. See [usage](../../USAGE.md).

M19 string-references requires all descriptor limits plus explicit reference/per-scan/total-scan budgets. It presents referenced bytes only. See [usage](../../USAGE.md).

M20 hash-metadata requires acquisition/header/dynamic limits plus --max-hash-words. It prints counts/proof only; see [usage](../../USAGE.md).

M21 symbols explicitly requests count proof then bounded symbol/name observations. All entry/lookup/scan limits are required; see [usage](../../USAGE.md).

`classify-symbols` explicitly composes bounded symbol observations and structural roles. All budgets are required; see [USAGE](../../USAGE.md).

M23 `linkage-evidence` explicitly observes selected real or synthetic ELF input with visible budgets. Existing synthetic `--linkage` modes remain separate; see [USAGE](../../USAGE.md).
