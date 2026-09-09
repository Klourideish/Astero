# M14: bounded filesystem input

filesystem path -> bounded acquisition -> SourceArtifact -> STOP.

## Ownership

astero-loader::artifact::filesystem owns this host acquisition adapter. It is format-independent
and separate from kernel/filesystem (guest VFS, guest descriptors and host I/O on behalf of guests).
No new crate, dependency or internal edge is needed. SourceArtifact itself is unchanged; it still
owns immutable bytes, a generated object identity and optional display provenance.

acquire(path, AcquisitionLimits) is a library API. No CLI/GUI wiring, file picker, automatic parsing,
admission, loading, linkage, session composition, NIDs or execution was added. Future callers may
explicitly hand the returned artifact to another stage; acquisition never chooses that stage.

## Exact resource policy

The caller must supply max_bytes and max_read_calls; there is no implicit default or PS5 size rule.
Zero bytes permits an empty file. At least one read call is needed even for an empty file because
EOF must be probed. Opened-handle length above max_bytes fails before allocation/reading.
Lengths must fit usize and isize::MAX before a fallible try_reserve_exact request.

The adapter requests storage for only that checked candidate length, initializes it, then reads in
chunks of at most 64 KiB. There is no read_to_end or buffer growth. A one-byte stack probe detects
extra data and is never appended. Every read attempt, including Interrupted and EOF probing,
consumes the explicit call budget. Exhaustion returns failure, never a prefix artifact.
Positive short reads continue; EOF before expected length is ShortRead.

The byte limit bounds requested payload storage, not exact allocator RSS. The allocator may round
capacity; existing Vec-to-Box conversion can transiently require another bounded payload allocation.
Arc, path and provenance overhead remain ordinary standard-library allocations. Payload reservation
failure is structured; process-wide allocator exhaustion is not promised recoverable by SourceArtifact.
Synchronous OS open/metadata/read latency has no timeout guarantee. Read-call budget bounds attempts,
not wall-clock duration. See [Vec reservation](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.try_reserve_exact).

## File evidence and races

Initial symlink_metadata rejects an observed final symlink, directory or other non-regular object.
The opened handle is checked again for regular-file type; only its metadata supplies candidate length.
All bytes are read from that one handle. EOF probing and final handle-length recheck detect observed
growth/truncation. Failure precedence is: path/type, open/handle metadata/type, size/conversion/reserve,
read budget/I/O/short read, excess-byte probe, final metadata/size, SourceArtifact construction.

This is not a secure path sandbox or filesystem transaction. Parent symlinks/native path resolution
remain OS behavior. Namespace replacement between precheck and open is possible; the initial final-
symlink check is not a race-proof no-follow guarantee. A replacement non-regular object can also
block open before the handle check on some hosts. Use selected regular files in a stable namespace;
adversarial namespace hardening and asynchronous cancellation require later platform-specific work.
No writable handle or mutable alias is retained after construction.

Same-size writes, grow-then-shrink races or mutations after the final check can evade detection.
Success means immutable bytes acquired through bounded reads with consistent observed length,
not an atomic snapshot or authenticity claim about a path. A File handle alone does not prevent
other processes modifying content ([Rust File contract](https://doc.rust-lang.org/std/fs/struct.File.html)).
No retry loop attempts to obtain a supposedly stable version.

## Identity, provenance and errors

Independent successful acquisitions create different process-local SourceIds even for identical
bytes; clones share identity/storage. Changing the file later cannot mutate an acquired artifact.
Native Path inputs do not require UTF-8. Error.path retains the requested PathBuf, operation identifies
the failing stage, and Failure preserves structured sizes/counts and nested I/O, reservation or source
errors through Error::source. Success provenance is an escaped diagnostic path label, not a path
serialization or reopening token. No lossy conversion is used to select the file.

Failure families: I/O, non-regular object, byte limit, unrepresentable length, allocation, read-call
budget, short read, detected growth, final size change and SourceArtifact error. I/O kind and OS code
remain accessible. Relative paths retain the requested spelling; no canonicalization is implied.

## Tests and scope

Small files under the workspace target test-fixture directory prove empty/exact-limit/arbitrary-byte
success, over-limit rejection, immutable retention after file edits, repeated content with distinct
identities, missing/invalid/directory paths, Windows exclusive-open access failure and native non-UTF-8
path acquisition. A Unix-only final-symlink test is included; it is not runtime-validated on Windows.
Private reader tests inject short/interrupted/denied reads and changed lengths without timing races.
A 16 x 16 extent sweep checks exact/short/growing outcomes; chunk/call tests verify 64 KiB requests
and EOF budget accounting. No real binary corpus or downstream parser is used.

See [validation](validation.md) for exact executed results. Future handoff stays explicit.
