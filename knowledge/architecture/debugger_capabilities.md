# Debugger capability inventory

Implemented means real behavior exists; validated means named checks exercise it.
The runtime inventory reports support; this record distinguishes validation scope.

| Capability | Owner / debugger responsibility | Implementation | Validation |
|---|---|---|---|
| Session identity/lifecycle/target | core host state; debug reads ObserveSession | implemented | core/debug/CLI/GUI adapter tests; real window |
| Subsystem availability/statistics | core availability and host lifecycle changes | implemented | common snapshot/frontend tests |
| Structured host snapshot/diagnostics | core coherent snapshot; debug inspection | implemented | detached snapshot and concurrent consistency tests |
| Capability inventory | debug support discovery | implemented | inventory/support and frontend tests |
| Thread/context inspection | kernel observations; debug view | planned / unsupported | rejection only |
| Memory/maps | memory checked observations | planned / unsupported | rejection only |
| Module/NID attribution | loader, libs, HLE; debug correlation | planned / unsupported | rejection only |
| Guest fault capture | kernel execution and faulting service | planned / unsupported | rejection only |
| Tracing/filtering | emitting services; debug filters | planned / unsupported | rejection only |
| Actual-PC sampling | kernel execution backend; debug provenance | planned / unsupported | rejection only |
| Structured guest snapshots | services; core consistency; debug view | planned / unsupported | rejection only |
| Guest execution control | kernel mechanism; core lifecycle; debug requests | planned / unsupported | pause rejection only |
| Breakpoints/watchpoints | kernel execution and memory | planned / unsupported | rejection only |
| GPU visibility | gpu/shader/video observations | planned / unsupported | rejection only |

inspect_session(&impl ObserveSession) returns a real detached snapshot or SessionClosed/StateUnavailable.
pause() returns Unsupported(ExecutionControl). require_support is a support query, not an operation.
Other guest operation APIs do not exist. No guessed threads, registers, maps, PC, NID calls, GPU work
or breakpoints are present. HostFault records orchestration failure, not guest fault capture.
Runtime services import neither core nor debug. See [session contract](session_observation.md)
and [validation](validation.md).

Physical scaffolding does not change this inventory. Existing session inspection is under src/session/;
the new threads/context/memory/modules/symbols/nids/tracing/sampling/breakpoints/watchpoints/faults/gpu/
snapshots roots contain ownership documentation and declarations only.

M11: LinkageEvidence is implemented and tested for offline immutable report inspection under
snapshots/linkage (loader owns semantics). Tests cover identity, ownership and all completeness states.
This does not enable guest module/NID attribution, resolution, relocation application or guest snapshots.
The snapshots root is now implemented for this bounded host report; other scaffold status is unchanged.

M12 LinkageEvidence inspection obtains the report from one session snapshot. Standalone reports
have an explicit separate helper and no session claim. Composition/CLI tests validate this path;
no guest inspection/control capability is added.
