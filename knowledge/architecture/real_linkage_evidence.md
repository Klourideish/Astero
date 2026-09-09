# M23: real ELF linkage evidence

`SourceArtifact` -> explicitly requested `workload::observe` -> immutable `LinkageReport` -> STOP.
Loader owns evidence; core input/linkage_evidence delegates; CLI linkage-evidence selects and prints.
No dependency changes. Acquisition and M16-M22 remain independently callable and unchanged.
This capability composes established contracts instead of a new parser or runtime loader.

## Focused evidence and hypotheses

Catalogue roots are local routes in AGENTS.md; references below are catalogue-relative, not machine
paths. Only these focused routes were read, after each catalogue's README/00_START_HERE router:

- PS5Rust: 03_LOADER_LINKER/INDEX.md, relocation_subset.md, provider_authority.md;
  source/ps5-core__src__loader__elf/{INDEX.md,002.md} and relocation source
  source/ps5-core__src__loader__relocations/{INDEX.md,001.md}. These preserve ordinary dynamic
  constants, explicit provider/unresolved distinctions and a limited legacy relocation subset.
  ImportedSymbol's legacy NID/module/library fields are not copied as established M23 identities.
  Runtime stand-in/scratch/unresolved fallback classes are not migrated or treated as resolution.
- PS5Rust firmware route: 15_FIRMWARE_KNOWLEDGE/INDEX.md and
  document_guides/403_MODULES_SYMBOLS.md.md. Documentary firmware inventory uses defined/undefined
  rows; this does not prove runtime exportability/provider ownership. No firmware payload read or
  ABI/NID contract implemented. Its unwind ABI is outside this capability.
- Decrypted ELF: 00_START_HERE/key_findings.md, 01_IMAGES/INDEX.md, and
  01_IMAGES/PS5Util.elf/{README.md,mapping.md,dynamic.md,imports/INDEX.md,imports/0001.md,
  import_relocations/INDEX.md,import_relocations/0001.md}. Program/file deltas, ordinary tags,
  two dependency names and mixed ordinary/JMPREL references guided the experiment. Catalogue labels
  like IMPORT and decoded NIDs are not adopted as Astero resolution facts.

Initial hypothesis: PS5Util might require SCE DYNLIBDATA-relative translation. **Refined**: the
selected builds have ordinary PT_LOAD-backed dynamic addresses, already handled by Astero. No
SCE-relative adapter or address fallback is necessary or introduced. Unknown SCE tags stay raw.
Hypothesis: existing hash/symbol/RELA machinery can yield useful real evidence. **Supported** on two
small corpus builds, without widening format parsing. Hypothesis: referenced undefined eligible
symbols support an external-reference candidate. **Supported structurally**, not as actual calls,
resolved imports or functioning APIs. Provider/export/NID interpretation remains uncertain.
External emulator implementations were not needed. No catalogue code was bulk-copied.

## Composition and exact rules

M20 proves exact same-source membership; M21 observes fields/names; M22 preserves structural roles.
The report retains the Arc-backed M22/M21/proof chain and SourceArtifact. Names/fields are not copied.
M5 adaptation of bounded M17 raw discovery is shared via from_raw. M9 descriptor validation, RELA
24-byte decoding, tail aliases and canonical iteration are reused. Its raw iterator was extracted
so the existing validated M9 iterator and M23 share order/decoding. M9 validated enumeration still
requires its original trusted symbol evidence; raw enumeration explicitly makes no reference claim.

Every M23 raw r_info index must resolve into the complete M21 array and same source, even zero.
An invalid index fails with the raw record/count. No name rescans per relocation are needed.
Canonical order is ordinary prefix then PLT; supported aliases appear once with both labels. Within
each table original order is retained. Raw offset/info/type/addend, source token and descriptor/index
labels remain auditable. Source proof is not relocation destination validity or a runtime address.

For nonzero UndefinedCandidate with at least one actual relocation reference:
- Global or Weak binding; Default visibility; no reserved st_other bits;
- NoType, Object, Function or Tls type; nonempty byte name;
- then ExternalReferenceCandidate; otherwise AmbiguousReference.

No spelling/UTF-8/NID/namespace heuristics. Unknown numeric attributes remain preserved. Unreferenced
undefined symbols remain StructuralOnly. Null is explicit, never external. M22 ordinary definitions
remain DefinitionCandidate (including local/hidden/unknown-attribute definitions); none becomes an
export. ABS/COMMON/reserved/extended remain special. Duplicate names stay separate source/index
identities. Counts include references to zero separately; symbol_associated means nonzero indices.
Ordinary/PLT per-symbol counts may overlap for aliases; unique reference counts never double-count.
All classification labels are derived structural evidence, not resolved meanings.

DT_NEEDED names use M6 bounded lookup after the symbols, retain original entry index, offset and
source token, and preserve duplicate declaration order. libkernel.prx/libc.prx names are declarations,
not proof of which symbols they provide. Packed SCE module/library tags and NID-looking names remain
raw evidence via the retained dynamic table. No packed-ID layout or provider mapping is guessed.

## Budgets, failure and scope

LinkageLimits reuses all M20 and M21 limits and adds only max_relocations (canonical raw entries).
This bounds relocation storage, one index association per entry and total counts. Symbol budget also
bounds role/use records; no extra candidate/association knob. All limits are explicit; zero valid.
Exact relocation count succeeds; excess fails before traversal. Allocation uses fallible Vec reserves.
M20/M21 discovery plus M23 descriptor discovery means three bounded discovery passes, not an unbounded
pipeline. No guest-state mutation occurs between them: they share immutable bytes.

The name lookup and total scan budgets cover symbol names **plus DT_NEEDED names**, in that order.
Repeated names charge repeated lookups/scans, including NUL. Per-name scan limit applies to both.
Names remain source-backed. The M21 report itself retains its narrower symbol-only budget semantics.
O(symbols + relocations + names) association/retention work follows bounded existing hash/discovery.
No eager allocation from an unvalidated advertised relocation count.

Complete means this supported observation scope succeeded, not resolved linkage or admitted ELF.
Unavailable means no trusted exact symbol observation. Failed retains structured prerequisite,
M5/M9/source/name, index, allocation or budget failure; no successful linkage prefix survives.
Complete M21 evidence can still be inspected when later work fails. Ordinary REL/RELR descriptors
and M9 deferred PLT REL fail explicitly. Unknown relocation type numbers remain observable; **all
relocation application semantics are unsupported**. Unknown dynamic tags remain uninterpreted,
so Complete does not certify every OS-specific structure. Empty relocation inventories are valid.

## Real experiment (2026-09-09)

Inputs selected only through ignored LOCAL_TEST_CORPUS.json; no paths or payloads embedded in tests.

| Role | Bytes | SHA-256 |
|---|---:|---|
| linkage_sample (PS5Util.prx) | 67,668 | 2f824c2a233c540d7220a4b1bc3b9c76e1a314267e47fe31740db5b93f582a3f |
| utility_build_comparison (PS5Util.prx) | 67,628 | daa15b69a637b29a34f93d5e9dd28c112e15f82f18ca12a9858b2eab55acef0b |

Both: 14 symbols (null + 8 undefined + 5 ordinary definitions), 11 relocations (4 ordinary/7 PLT),
9 nonzero associated records, 2 null records, 8 external-reference candidates, 5 definitions,
zero ambiguous-reference candidates. Primary types: 8 x 2, 6 x 1, 1 x 1, 7 x 7; all are in the
catalogue's legacy numeric subset, but none has application semantics here. All providers unresolved.
Both declare libkernel.prx and libc.prx. Hashes before/after match. No substitution was needed.

Catalogue PS5Util.elf is a different 68,080-byte artifact (SHA-256
2356785672798daee9c90cec6f5ffd023e21ac86f41ceb70172a174b11cb76cf): 10 undefined, 7 named definitions,
13 relocations and the same two dependency declarations. Similar mapping/reference shape is
corroboration, not byte/build identity. Extra catalogue symbols/relocations are not a regression.
No whole-corpus analysis, guest execution, relocation application, provider resolution, dependency
loading, HLE/NID binding, admission or memory mapping occurred.

## Remaining pressure

SCE-relative tables, hashless SCE symbol-count evidence, broader relocation encodings, provider
identity and runtime export rules remain unsupported. A useful M24 would be a bounded, evidence-led
PS5 module/library descriptor and encoded-name correlation capability, validated on these artifacts,
without final provider binding or NID-to-HLE resolution. No M24 implementation is included.

See [USAGE](../../USAGE.md) for exact validated commands and [validation](validation.md) for results.
