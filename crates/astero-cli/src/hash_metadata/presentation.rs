use astero_core::input::hash_metadata::{HashMetadataOutcome, HashMetadataReport};
use std::fmt::Write;
pub fn render(report: &HashMetadataReport) -> String {
    let l = report.limits();
    let mut text = format!(
        "Hash observation explicitly requested\nBudgets: max-program-headers={} max-dynamic-entries={} max-hash-words={}\nSource: {:?} | bytes: {} | provenance: {:?}\n",
        l.dynamic.max_program_headers,
        l.dynamic.max_dynamic_entries,
        l.hash.max_words,
        report.source().identity(),
        report.source().len(),
        report.source().provenance()
    );
    for field in report.descriptor_fields() {
        writeln!(
            text,
            "Descriptor entry {}: {:?} raw=0x{:X}",
            field.index, field.tag, field.value
        )
        .expect("String write");
    }
    match report.outcome() {
        HashMetadataOutcome::Unavailable => text.push_str(
            "Hash metadata: Unavailable (no supported hash descriptor)\nCount: Unavailable\n",
        ),
        HashMetadataOutcome::Failed(e) => {
            writeln!(
                text,
                "Hash metadata: Failed\n{e}\nNo trusted count returned."
            )
            .expect("String write");
        }
        HashMetadataOutcome::Complete(e) => {
            text.push_str("Hash metadata: Complete\n");
            if let Some(h) = e.sysv() {
                writeln!(
                    text,
                    "SYSV: nbucket={} nchain={} source={:?}",
                    h.bucket_count(),
                    h.chain_count(),
                    h.source_range()
                )
                .expect("String write");
            }
            if let Some(h) = e.gnu() {
                writeln!(text, "GNU: nbuckets={} symoffset={} bloom_size={} bloom_shift={} claim={:?} source={:?}",h.bucket_count(),h.first_symbol(),h.bloom_size(),h.bloom_shift(),h.count_claim(),h.source_range()).expect("String write");
            }
            if let Some(extent) = e.extent() {
                writeln!(text, "Trusted symbol count: {} (entries; exclusive upper index)\nSymbol backing extent: {:?}\nProof: {:?}",extent.symbol_count(),extent.source_range(),extent.evidence()).expect("String write");
            } else {
                text.push_str("Count: Unavailable (GNU lower-bound-only evidence)\n");
            }
        }
    }
    text.push_str(
        "No symbols enumerated. No names resolved. No linkage. No guest loaded or executed.\n",
    );
    text
}
