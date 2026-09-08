# M4: bounded ELF64 header and program-header inspection

M5 adds a separate [dynamic observation and translation stage](dynamic_elf_observation.md).
The M4 stage and its conservative admission requirements remain unchanged.

SourceArtifact -> elf::inspect(source, module_context) -> ElfInspection.artifact() -> admit -> plan.
Inspection describes observations; admission decides which descriptions Astero accepts. Acceptance
here is an immutable work description, not proof of a valid PS5 image or ability to load/execute it.

## Evidence and chosen scope

Standard ELF evidence, read 2026-09-08: [gABI file header](https://gabi.xinuos.com/elf/02-eheader.html),
[program headers](https://gabi.xinuos.com/elf/07-pheader.html),
[machine assignments](https://gabi.xinuos.com/elf/a-emachine.html), and
[Linux UAPI extended numbering definition](https://github.com/torvalds/linux/blob/master/include/uapi/linux/elf.h).
These establish layouts/constants, not Sony platform semantics. No PS5Rust source or real ELF payload
was used. Repository policy choices below are deliberately narrower than general ELF support.

M4 supports class 64, little-endian, identification/file version 1, a 64-byte file header and
56-byte ordinary program entries. OS ABI/version and machine flags are retained as observations.
There is no platform field asserting PS5 support: family Elf identifies format, Architecture identifies
ISA, and ArtifactRole identifies executable/module/unknown. Raw OS ABI is not converted to a platform.

## Owners and observations

- elf/identification/decode.rs checks the 16-byte identification before any header field read.
- elf/header/decode.rs decodes all ELF64 header fields: type, machine, version, entry, program/section
  offsets, flags, header size, program/section entry sizes/counts and section-name table index.
- elf/program_headers/decode.rs lazily decodes kind, flags, file/virtual/physical addresses, file size,
  memory size and alignment. Physical addresses have no runtime interpretation.
- elf/inspect/adapter.rs translates PT_LOAD into RegionObservation. Permissions map PF_R/W/X
  independently. Alignment 0 becomes 1 (no alignment); other values remain for admission.
- elf/error/failure.rs owns byte-inspection failures; decoding.rs provides checked source reads and
  bounded explicit standard-library little-endian scalar conversion. No unsafe casts or dependencies.

ElfInspection has private fields and read-only header/artifact access. Its program_headers() iterator
is created only from the report's validated header/source pair. Header and program structures stay in
ELF; they never enter TargetMetadata, ValidatedTarget or LoadPlan. They are detached observations,
not independently constructible proof tokens. Program indices remain recoverable in the report;
admission region indices count PT_LOAD observations only, in program-table order.

ModuleMetadata is explicit caller context: a loader-graph ID and display name, not an ELF SONAME,
export, source identity or inferred filename. Missing/blank context still fails normal admission.
ET_EXEC maps to Executable; ET_DYN maps to Module (no PIE-versus-library inference). Machine 62 maps
to X86_64, 183 to Aarch64; others remain Unknown with the raw value retained in the report.
Entry zero maps to absent. No ET_DYN load bias/base is calculated: plans retain virtual-address intent.

## Safe structure and structured errors

Header/table validation precedes iteration. Nonempty tables must start at or after byte 64.
Zero-count tables require offset zero and entry size 0 or 56. Other header/table sizes are unsupported.
Extended count 0xffff is explicitly unsupported because it requires section-zero inspection.
Section metadata is observed without following or validating section references, including extended
section numbering; no section validity is certified.

ElfError distinguishes InvalidMagic; UnsupportedClass/ByteOrder/Version/HeaderSize/ProgramHeaderSize;
UnsupportedExtendedProgramCount; InvalidTableOffset; TableOverflow; and defensive MalformedField.
Source errors retain Structure (identification/header/table/entry index), requested offset/length and
actual source length via SourceError and Error::source. Overflow is checked before table bounds.
A partially present entry is a truncated whole-table error: immutable bytes cannot become shorter
between full-table validation and iteration. MalformedField is a defensive scalar-read failure,
not an alternative target-admission error. No malformed-input panic or unchecked external slice index.

## Admission and deferred semantics

The only admission algorithm change permits Synthetic or Elf families. Unsupported machine/role,
missing context/regions, bad source or virtual ranges, sizes, overlap, alignment and entry relationships
remain the existing Rejection taxonomy. Target and plan construction and source ownership are unchanged.
No bytes are duplicated into parser storage or copy operations. Plans keep the original source identity
and checked copy tokens; BSS tails remain explicit zero-fill work.

PT_NULL is unused and does not obstruct admission. Every other non-load type is retained in the report
and raises UninspectedProgramSemantics, blocking admission until its requirements are understood.
This includes PT_DYNAMIC, PT_INTERP, PT_TLS, notes, PHDR and unknown extensions; M4 interprets none of
their payloads. Nonzero OS ABI/ABI version/header flags or unknown load-flag bits raise PlatformSemantics.
Empty dependency/import/export/relocation lists mean not derived by this stage, not verified absence.
These requirements prevent uninspected headers from silently becoming an apparently complete plan.

The existing address-alignment rule is intentionally conservative: address must be a multiple of
alignment. ELF allows congruent nonzero file/virtual residues; those may be rejected by Astero today.
Conversely this copy-intent admission does not certify ELF file/virtual congruence or page-mapping
semantics. A future application contract must address both before mapping; M4 does not change generic
region representation merely to implement ELF mapping rules. Empty load memory still fails admission.

## Resource and test bounds

Ordinary count is u16 and 0xffff is reserved, so iteration is bounded to 65,534 entries and a table
of 3,669,904 bytes. Multiplication cannot overflow u64 for supported fields; addition to the untrusted
offset can, and is checked. There is no arbitrary smaller count ceiling and no allocation from the
count alone. The adapter grows only its PT_LOAD observations after whole-table bounds validation;
raw entries are not eagerly collected. The report retains one shared immutable source owner.
Existing admission overlap checks are quadratic for many load regions; this remains a downstream
hostile-input scaling pressure, not a parser safety guarantee. Source/allocation ceilings and future
parser work limits need explicit policy as richer tables arrive; standard allocator OOM is unchanged.

[Generated fixtures](../../crates/astero-loader/tests/elf_fixtures/mod.rs) produce exact owned bytes.
[Tests](../../crates/astero-loader/tests/elf_inspection.rs) exercise the pipeline, all public failure
families reachable from bytes, conservative requirements, immutable identity/copy/BSS behavior and
boundary oracles: 129 source lengths, 72 table offset/count combinations, 49 segment source extents,
and the maximum ordinary all-PT_NULL table. Existing M2/M3 tests remain. See [validation](validation.md).

## Deferred and next boundary

No section, dynamic, symbol or string table parsing; relocations; import/export derivation; NIDs;
Sony metadata; SELF; filesystem/mmap input; memory application; relocation execution or guest execution.
No session/frontend/service changes or PS5Rust migration. No platform, networking, media, cache or
configuration decision gate is resolved. Recommended M5: bounded dynamic-observation/completeness
contracts and pointer-to-source rules using generated bytes, before dependent linking tables or runtime.
