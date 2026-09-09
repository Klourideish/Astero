# In-memory evidence composition (M12)

> Session attachment does not strengthen loader evidence.
>
> Immutable linkage evidence is observational input, not runtime-loaded module state.

## Dependency decision before implementation

M11 has 15 crates and eight declared internal edges. The report contains loader-owned bounded
source tokens, trusted hash extent provenance and nested symbol/relocation errors. These are not
currently neutral cross-subsystem values. We considered three alternatives:

- Move the complete report into astero-observe: this would also pull loader validation tokens and
  error taxonomy into a supposedly neutral owner, contradicting the narrow value-only purpose.
- Introduce parallel neutral DTOs: would duplicate classifications, source identities and a large
  structured error tree, require a second translation boundary and risk losing evidence. There is
  currently one evidence producer and one report family, not demonstrated cross-subsystem reuse.
- Retain the exact immutable report and deliberately activate the already allowed core -> loader
  edge: core can hold an Arc and check identity without knowing classification rules or parsing.
  This is downward application composition, not a runtime service depending on core.

We choose the third option for M12. No astero-observe crate is introduced. This is a deliberate
coupling to a read-only report API, not permission for core to inspect ELF internals. Reconsider a
neutral contract when another independently owned evidence family or versioned transport requires
it; do not move parser/trusted token construction into that crate. Loader remains dependency-free.

Debugger production stops depending directly on loader, consuming core's re-exported report/input
contract instead. Its loader dependency remains dev-only for independent generated fixtures.
CLI's loader fixture edge becomes normal because an explicit synthetic command generates bytes
and calls loader APIs at the executable composition boundary. It never classifies them itself.
No new external dependencies, upward edges or cycles are needed.

## Typed composition and identity

`SessionInputs::new(target, linkage)` accepts an optional `EvidenceTarget` and optional
`Arc<LinkageEvidenceReport>`. A report requires a target declaration with exactly the same
source and optional module ID. Missing target is `TargetRequired`; either mismatch is a structured
`IdentityMismatch`. No target/no evidence and target/no evidence are both explicit valid states.
The input's fields are private; no replacement method or mutable attachment registry exists.
`Session::with_inputs` consumes checked inputs; `Session::new` uses absent defaults.

Core never constructs or reclassifies report contents. The same source/module IDs, extent evidence,
names, counts, errors and Complete/Partial/Unavailable/Failed states survive. Failed reports are
allowed as observational input; attachment does not bless their underlying evidence. Identity
matching checks the declared report envelope, not the coherence of evidence rejected inside a
Failed report. Module IDs remain caller-supplied inspection context. This does not authenticate
an executable, runtime target, PS5 identity, content hash or a resolved module.

`loaded_target` remains None and `run()` still rejects with NoGuestLoaded after host initialization.
EvidenceTarget is deliberately separate from LoadedTarget. The synthetic CLI target is identified
by its actual loader source identity; its module ID is absent. Core tests also cover a known
synthetic module context and incompatible source/module declarations.

## Snapshot guarantee

`SessionSnapshot.inputs` is cloned under the existing session mutex together with lifecycle,
counters and diagnostics. Cloning inputs clones an Arc, not the report. The attached report has
no mutation API and core has no attachment replacement operation. Observations therefore capture
one session state with a fixed immutable report from one observation operation. Prior detached
snapshots retain their lifecycle and report even after the session stops/drops; weak observer
clones do not extend session lifetime, and subsequent reads return SessionClosed.

This narrow guarantee does not coordinate future mutable kernel, GPU, memory or thread snapshots.
Those will need explicit coordinated epochs/generations or another documented snapshot protocol.
No such system is implemented here. Report completeness and immutability are not proofs of
runtime consistency, admission, resolution or execution readiness.

## Debugger and CLI

`debug::snapshots::linkage::inspect_linkage(&impl ObserveSession)` performs one observation and
returns a session-bound LinkageSnapshot. Report access is derived from that snapshot's inputs;
there is no independent report argument to accidentally pair with a session ID. The old offline
route is renamed `inspect_standalone_report`, returns no session identity, and CLI presentation
labels it explicitly as standalone. Neither route classifies loader evidence.

`cargo run -p astero-cli -- --linkage --synthetic [--details]` generates a fixed 1,536-byte ELF-shaped
input, invokes the existing loader pipeline, creates checked inputs, constructs/initializes the
host session, observes it through debug and renders the resulting evidence. It prints an explicit
synthetic/no-loaded-PS5 banner. No files are read, no binaries migrated, and no guest target is
loaded. Source creation/inspection failures are propagated, not replaced by successful reports.

`--linkage` without `--synthetic` now observes an actual empty session and reports unavailable
evidence. Default invocation retains its M1 behavior. Unknown/repeated command options fail.
The library synthetic composition helper accepts a symbol budget so tests can exercise partial
reports through exactly the same session path. Presentation labels observed candidates exactly as
M11, including escaped raw byte names and explicit completeness. It does not own evidence truth.

GUI code is unchanged. Its existing Inspection includes SessionSnapshot.inputs, so a future pane
can access the shared report from the same snapshot without a separate loader route or a second
observation. No additional GUI dependency or framework/presentation work was needed.

## Dependency and policy result

15 crates remain. Nine unique declared internal edges exist (eight production, one dev-only):
core -> loader is newly activated; debug -> loader changes from normal to dev-only; CLI -> loader
changes from dev-only to normal for the explicit in-memory fixture composition entry point.
Other edges remain core/debug consumers and libs -> HLE. Loader still declares no dependencies.
Policy tests enforce downward composition and prohibit a production debugger -> loader edge.
No unused duplicate normal/dev edges remain, and no external packages/versions change.

## Validation and remaining pressure

Core fixtures cover empty inputs, complete/partial/unavailable/failed reports, target mismatch,
module/source preservation, snapshot determinism and shared ownership. Debugger tests capture
session identity/lifecycle/report together; CLI tests execute the synthetic command and preserve
partial metadata through the session path. A compile-fail doctest protects attachment fields.
See [validation](validation.md) for exact results and index counts.

The report remains loader-specific, an explicit coupling accepted here. A future neutral schema
must preserve structured errors and distinguish observation locations from validation authority;
just moving types is not sufficient. A bounded M13 option is a read-only GUI status/detail view of
these already composed synthetic session inputs, without adding files, linking, NIDs or execution.

## M13 consumer extension

The GUI now consumes this path. The synthetic generator moved into loader and typed composition into core; CLI -> loader is dev-only again. EvidenceOrigin records synthetic composition explicitly. See [GUI evidence](gui_linkage_evidence.md); no neutral crate or additional edge was needed.
