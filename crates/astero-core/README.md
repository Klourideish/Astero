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

M15 input/acquisition is a thin application entry point to the unchanged loader acquisition API. It exposes immutable sources and structured failures without reading bytes itself, creating sessions or invoking parsing. See [frontend acquisition](../../knowledge/architecture/acquisition_frontend.md).

M16 input/inspection delegates an explicit source/budget request to loader header inspection without acquiring a file, attaching evidence or creating a session.

M17 input/dynamic delegates raw table observation with explicit budgets; no acquisition, session composition or linkage occurs.

M18 input/descriptors delegates selected descriptor metadata observation; it creates no session and performs no payload interpretation.

M19 input/string_references delegates explicit reference lookup without creating dependency/session state.
