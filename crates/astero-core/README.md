# astero-core

Owns a real host Session, checked lifecycle and coherent detached snapshots. SessionObserver is weak/read-only; ObserveSession is the application observation contract. No guest loading or execution exists.

## Module ownership

[session/](src/session/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

No guest mechanisms, fabricated results or toolkit types may enter core/debug contracts.
Integration tests live under this package's tests/. GUI also has pure swapchain-selection unit tests.

See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[session contract](../../knowledge/architecture/session_observation.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
[GUI decisions](../../knowledge/architecture/gui_framework.md) and
[validation](../../knowledge/architecture/validation.md).

M12 session/inputs carries checked immutable linkage evidence. Core depends on loader only for
that report/identity contract; it never derives evidence. See [composition](../../knowledge/architecture/evidence_composition.md).

M13 session/inputs/synthetic composes loader-generated demonstration reports and records typed synthetic provenance. It performs no parsing/classification itself; normal inputs remain explicitly unspecified in origin.
