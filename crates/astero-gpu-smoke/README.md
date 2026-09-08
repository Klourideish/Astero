# astero-gpu-smoke

Isolated GPU/shader validation executable.

## Interfaces and status

M0 scaffold only. Executable reports that no GPU validation has occurred.

## Module ownership

main: executable entry point. Modules belong to this crate's scope.

Forbidden: Session composition or reporting unperformed GPU validation.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).
