# astero-debug

Inspects core through ObserveSession; returns the common SessionSnapshot and capability inventory. Guest operations remain explicitly unsupported. Core never imports debug.

## Module ownership

[breakpoints/](src/breakpoints/mod.rs), [context/](src/context/mod.rs), [faults/](src/faults/mod.rs), [gpu/](src/gpu/mod.rs), [memory/](src/memory/mod.rs), [modules/](src/modules/mod.rs), [nids/](src/nids/mod.rs), [sampling/](src/sampling/mod.rs), [session/](src/session/mod.rs), [snapshots/](src/snapshots/mod.rs), [symbols/](src/symbols/mod.rs), [threads/](src/threads/mod.rs), [tracing/](src/tracing/mod.rs), [watchpoints/](src/watchpoints/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

No guest mechanisms, fabricated results or toolkit types may enter core/debug contracts.
Integration tests live under this package's tests/. GUI also has pure swapchain-selection unit tests.

See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[session contract](../../knowledge/architecture/session_observation.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[GUI decisions](../../knowledge/architecture/gui_framework.md) and
[validation](../../knowledge/architecture/validation.md).

M11 snapshots/linkage consumes an Arc of the loader-owned linkage report without classification.
The new loader dependency serves read-only reporting; see [report contract](../../knowledge/architecture/linkage_reporting.md).

M12 inspect_linkage takes a session observer; offline access is inspect_standalone_report.
Production imports core contracts; direct loader use is dev-only. See [composition](../../knowledge/architecture/evidence_composition.md).
