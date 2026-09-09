# M15: explicit acquisition-only CLI

frontend selection -> visible limits -> core input entry point -> M14 acquisition -> SourceArtifact -> STOP.

## Ownership and dependency decision

CLI is the narrowest frontend: native process arguments already support arbitrary paths and no file
picker, toolkit dependency or asynchronous GUI model is needed. GUI and all M14 implementation are
unchanged. The existing core -> loader dependency supplies a thin input/acquisition application entry
point. It delegates exactly once to artifact::filesystem::acquire, with no added validation or inspection.
Core exposes the immutable source, limits and structured acquisition errors through that boundary.
It does not construct a session. CLI's direct loader dependency remains test-only; all nine existing
internal edges and all manifests/lock versions are unchanged. No new crate is introduced.

CLI acquisition/arguments owns command syntax, selection owns one Request and synchronous State,
and presentation formats it. Native PathBuf identity is never recovered from a display string.
The loader remains the single owner of file access, bounds, races and SourceArtifact construction.

## Invocation and limits

astero-cli acquire --path <native-path> --max-bytes <u64> --max-read-calls <u64>

All three options are required exactly once, in any order. Limits are unsigned decimal u64 values;
there are no defaults, units conversion or hidden frontend caps. Zero is passed through honestly.
M14 still requires an EOF probe, so even empty-file success needs at least one read call.
The same chosen values are shown on success and acquisition failure. Syntax errors identify missing,
duplicate, unknown or invalid options and include usage. Mixing acquisition and linkage flags fails.

Arguments use args_os. Only numeric limits are decoded as text. The path value may be non-UTF-8 and
is passed unchanged through core into loader. Display uses escaped native-path Debug text; it is
not a serialization, canonicalization or identity mechanism.

## State and presentation

Selection::new(request) is Ready. acquire() synchronously becomes Acquired(SourceArtifact) or
Failed(AcquisitionError). There is no pretend Acquiring state or background work. Each acquisition
replaces the previous result; retained source clones keep their immutable bytes. Repeating unchanged
input preserves content/provenance but receives a new object ID under the existing source contract.

Selection owns no session or emulator state. The command returns before any existing session/linkage
branch. Success reports selected path, limits, acquired status, SourceId and observed byte length,
then explicitly states: No guest loaded. No parsing or linkage performed.
Acquisition failures retain native path, operation and nested loader error until the executable's
human presentation boundary. They print to stderr with nonzero exit status; success prints to stdout
and exits zero. Ready is testable/presentable through the library model but the live command is synchronous.

Arbitrary byte input, including an invalid file named .elf, succeeds when M14 acquisition permits it.
This does not establish ELF validity, admission, linkage availability or guest readiness. Neither core's
facade nor CLI selection calls a parser. No symbol/NID interpretation, filesystem search, guest memory,
execution or linkage attachment occurs. Existing synthetic linkage remains an independent mode.

## Tests and limitations

Six integration tests cover required/invalid/duplicate flags, numeric overflow, exact size limit,
byte and read-call failures (including EOF probe exhaustion), missing path with nested I/O error,
Ready/Acquired/Failed transitions, repeated acquisition/new identity, immutable retained bytes,
native non-UTF-8 model and subprocess arguments, arbitrary-byte executable success and error exits.
Existing synthetic linkage and default-session tests still run. All fixtures are small generated files
under target; no external binaries are used.

M14's limitations remain: synchronous I/O has no timeout, namespace checks are not a sandbox, and
size checks do not prove an atomic filesystem snapshot. CLI deliberately requires the operator to
choose budgets; a future convenience policy must be visible and explicitly named. No GUI work is
needed for M15. A later milestone can consider cancellation/namespace requirements before broadening
interactive input; no automatic handoff to parsing is implied.
