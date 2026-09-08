# astero-debug

Inspects core through ObserveSession; returns the common SessionSnapshot and capability inventory. Guest operations remain explicitly unsupported. Core never imports debug.

## Module ownership

inspection: report; capabilities: discovery; control: pause rejection.

No guest mechanisms, fabricated results or toolkit types may enter core/debug contracts.
Integration tests live under this package's tests/. GUI also has pure swapchain-selection unit tests.

See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[session contract](../../knowledge/architecture/session_observation.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[GUI decisions](../../knowledge/architecture/gui_framework.md) and
[validation](../../knowledge/architecture/validation.md).
