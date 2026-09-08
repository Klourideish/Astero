# astero-gpu-smoke

Isolated GPU/shader validation executable.

## Interfaces and status

M0 scaffold only. Executable reports that no GPU validation has occurred.

## Module ownership

[capture/](src/capture/mod.rs), [fixtures/](src/fixtures/mod.rs), [replay/](src/replay/mod.rs), [scenarios/](src/scenarios/mod.rs), [validation/](src/validation/mod.rs).

Nested child ownership follows the [module inventory](../../knowledge/architecture/module_structure.md).
New functionality belongs in the narrowest declared owner; these roots do not add capabilities.

Forbidden: Session composition or reporting unperformed GPU validation.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
