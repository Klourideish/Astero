# M13: read-only GUI linkage evidence

The GUI presents loader-derived evidence; it does not derive linkage semantics.
UI actions are added only after their underlying emulator semantics exist.

## Ownership and transport

The existing core/session contract is sufficient. model/linkage borrows the report and counts
from one debugger-provided session snapshot. ui/modules/linkage renders that view and retains
only a selected symbol index. It does not inspect ELF, classify symbols or recount candidates.
The immutable Arc report remains owned by SessionInputs; synthetic attachment does not load a guest.
No neutral crate or additional dependency edge is needed.

The generated demonstration moved from CLI into loader's narrow
elf/dynamic/candidates/report/synthetic owner. Core's session/inputs/synthetic calls that
factory and constructs checked SessionInputs. CLI and GUI share this composition path.
CLI -> loader is now dev-only for its existing fixture tests; GUI still depends only on core/debug
internally. Core -> loader remains the deliberate M12 coupling. Loader stays dependency-free.
The factory uses established inspection/classification/report collection, without changing them.

EvidenceOrigin::Synthetic is set by the core synthetic composer and captured in the same immutable
inputs as the report. General inputs default to Unspecified. This is composition provenance, not
cryptographic source authentication, a target identity or a claim about runtime-loaded state.
Source/module matching remains the existing checked input contract.

## Presentation contract

The pane always shows availability, provenance, no-guest state and Complete / Partial / Unavailable /
Failed explicitly. Partial status retains budget reasons and remaining-symbol information. Counts
are labelled observed; retained detail length is not a candidate total. Trusted extent evidence,
ordinary/PLT reference counts and structured failures remain loader observations.

Selection uses symbol index, never name identity. Details retain classification, binding, type,
visibility, section, source/name tokens, raw attributes and relocation provenance. Duplicate names,
unknown attributes, absent names and empty names remain distinct. UTF-8 is validated and escaped
for display; non-UTF-8 uses uppercase byte hex, for example <non-UTF8: FF>. Underlying names are
unchanged. Names are text content, not ImGui widget IDs or action labels.

## Synthetic launch and scope

cargo run -p astero-gui -- --synthetic-linkage opens the real Winit/Ash Vulkan/ImGui shell with
shared synthetic inputs. Default launch has no report. The demo has eight observed symbols:
one import candidate, one export candidate, one internal, four unclassified and one null; two
unique relocation references (one ordinary, one PLT). It includes duplicate, absent, empty and
non-UTF-8 names. No guest is loaded. Core's typed demo cases also exercise partial/unavailable/failed
reports without GUI-only injection. CLI keeps --linkage --synthetic.

The session snapshot and immutable attached evidence are coherent under M12's narrow guarantee.
This does not solve coordinated snapshots of future mutable kernel/GPU/memory services. No file
input, linking, NIDs, relocation application or runtime controls were added. A future pane can
extend presentation only when its underlying observation contract exists.

## Evidence

Five GUI integration tests cover all completeness states, provenance/no-guest, borrowed report
identity and counts, selection, duplicate names, unknown attributes, ordinary/PLT labels, name
formatting and partial detail budgets. Runtime on 2026-09-09 used NVIDIA GeForce RTX 4070 Vulkan:
first frame presented, synthetic/no-guest and Complete labels and fixture counts were visible;
selecting symbol 1 displayed its FF byte name and both reference classes. Maximize/restore
reflowed the window and preserved selection; native close exited with code 0. This validates
host presentation only, not emulator correctness. See [validation](validation.md) for full checks.
