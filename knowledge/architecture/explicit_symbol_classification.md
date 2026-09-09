# Explicit structural symbol classification (M22)

`SymbolObservationReport` + `ClassificationLimits` -> `structural::classify` ->
immutable `SymbolClassificationReport` -> STOP.

Loader owns the rules under `elf/dynamic/candidates/structural`; core input/classification
only delegates. CLI classification selects/presents. Dependencies are unchanged. The M10
classifier has import/export eligibility rules and its enumeration associates relocations;
those semantics are not this operation and are not invoked. M7 section views are reused.

## Exact rules

In M21 order, with no sorting, deduplication or skips:

| Role | Required evidence |
|---|---|
| NullSymbol | index zero (M21 already validated the null entry) |
| UndefinedCandidate | nonzero index, Section::Undefined |
| DefinitionCandidate | nonzero index, Section::Index (ordinary section) |
| SpecialCandidate | nonzero index, Absolute, Common, Extended or Reserved |

Null takes precedence. ABS/COMMON are not promoted to ordinary definitions; extended/reserved
indices are not resolved. All section values have a conservative home, so no extra Unknown
role is needed. The original section view and numeric st_shndx remain available.
Binding, type, visibility and name do not gate these roles. Local/global/weak, unknown numeric
attributes, hidden/protected visibility, unnamed, empty, UTF-8 and raw bytes remain unchanged.
No name spelling, UTF-8 requirement or visibility-based export eligibility is inferred.

**Undefined does not mean import. Defined/global does not mean export.**
Classification is evidence, not resolution.

## Input, identity and budget

The API accepts an Arc of the immutable M21 report, not a source for hidden enumeration.
Only Complete input can produce roles. Unavailable stays Unavailable; Failed produces a
PrerequisiteFailed status with the original structured failure accessible in the retained input.
No public constructor can fabricate successful report fields. Source identity, provenance,
trusted exact proof, raw fields and bounded names remain in that same shared input.
`entries()` pairs each role with its original symbol record without copying names or proof.

`max_classifications` is required, in symbol records. Every input record consumes one unit,
including null and special records. Zero is valid but cannot classify a nonempty input.
Exact count succeeds; count greater than the limit fails before allocation/iteration with
Budget { count, maximum }. Allocation uses try_reserve_exact and has a structured refusal.
No partial success or successful prefix is exposed. An empty Complete input would consume zero;
current upstream trusted-count policy does not produce one. Unknown attributes do not fail.
Work and role storage are O(observed count); source bytes and M21 observations are never mutated.

## Explicit frontend composition

`classify-symbols` explicitly acquires, requests existing M20/M21 prerequisites with all their
limits, then passes that M21 report to classification. It prints the prerequisite separately
from classification and preserves structured outcomes internally. Native paths remain native.
The CLI shares the M21 byte-faithful name formatter. Prior commands, synthetic linkage and GUI
remain unchanged; none automatically call classification. See [usage](../../USAGE.md).

No relocation or PLT/GOT traversal, dependency lookup, linkage, NID/ABI interpretation, runtime
state, guest admission/loading/execution or GUI browsing is added. No external catalogue was
needed. Future richer use evidence would require a separate request and contract; this milestone
does not establish import/export semantics. A possible M23 is explicit bounded relocation report
exposure using existing M9 machinery and trusted references, without application or resolution.
