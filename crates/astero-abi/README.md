# astero-abi

Guest ABI layouts, identifiers and constants.

## Interfaces and status

Concrete guest layouts include process-entry arguments, native call-frame evidence and the
M37 scalar va_list view. This crate owns representations, never runtime mechanisms.

## Module ownership

layouts, identifiers. Modules belong to this crate's scope.

Forbidden: Host-side application types or service mechanisms.

Allowed dependencies are permissions, not a requirement to add dependencies.
See [boundaries](../../knowledge/architecture/crate_boundaries.md),
[dependency policy](../../knowledge/architecture/dependency_policy.json),
and [repository contract](../../AGENTS.md).

M29 layouts/entry provides explicit 32-byte process argument encoding and a planned CPU context/captured call frame. The legacy-correlated guest convention is separate from Windows host ABI; no native bridge is implemented.

M37 adds the explicit 24-byte guest va_list layout in `layouts::varargs`; it is not a host va_list.
See [formatting contract](../../knowledge/architecture/libc_formatting.md).

M39 adds byte-exact 80-byte GetSpeakerInfo encoding under layouts/audio; device field meanings
remain comparative-correlated. See [M39](../../knowledge/architecture/audio_speaker_info.md).

M40 layouts/time encodes the 16-byte guest timespec explicitly; it does not reuse host layout.

M46 layouts/allocator contains explicit experimental version-one managed-size encoding; field confidence is recorded in sysmodule_allocator.md.

M47 layouts/stat encodes the prototype-corroborated 120-byte stat representation; metadata fields remain compatibility policy.
