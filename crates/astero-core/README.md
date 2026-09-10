# astero-core

Owns a real host Session, checked lifecycle and coherent detached snapshots. SessionObserver is weak/read-only; ObserveSession is the application observation contract. Session state remains separate from explicit M27 staging; no guest execution exists.

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

M20 input/hash_metadata delegates immutable source/count evidence requests; no session or symbol consumer is created.

M21 input/symbols delegates explicit same-source hash-proof consumption; no session or classification is created.

`input/classification` delegates immutable M21 report classification to loader; it creates no session or guest state.

M23 input/linkage_evidence delegates the loader capability over immutable source input; no session/guest state is created.

M24 input/ps5_identity delegates immutable M23 report plus identity limit to loader, without session construction. See [PS5 identity evidence](../../knowledge/architecture/ps5_identity_evidence.md).

M25 Session::with_timing explicitly receives a TimingEngine. Ordinary sessions and offline input APIs remain worker-free; stop/drop joins the opted-in engine. The only new dependency is core -> timing.

M26 input/load_plan delegates offline load/link planning without constructing a session or timing worker.

M27 input/staging composes loader plan application with astero-memory byte regions. The new core -> memory edge is already policy-permitted. It creates no Session/timer or execution state.

M28 input/native realizes an existing M27 staged image and retains its plan/diagnostics. It neither restages nor reapplies relocations; no session/timer/entry is created.

M29 input/entry owns NativeBackedGuestImage, kernel thread storage/context, host-model HLE registry, recovery state and optional timing. EntryReady remains false until native adapter/provider prerequisites are met.

M30 input/entry/closure owns validated bridge, RX import landings and NOACCESS unresolved-object traps alongside PreparedGuest. Explicit experimental closure can issue EntryReadyGuest; no artifact transfer method exists. See [native closure](../../knowledge/architecture/native_entry_closure.md).

M31 input/entry owns a dedicated first-entry thread and additional worker-process containment; preparation stays independent. See [first entry](../../knowledge/architecture/first_native_entry.md).
