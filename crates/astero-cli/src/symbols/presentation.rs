use astero_core::input::{
    hash_metadata::HashMetadataOutcome,
    symbols::{SymbolObservationReport, SymbolOutcome},
};
use std::fmt::Write;
fn name(bytes: Option<&[u8]>) -> String {
    match bytes {
        None => "<unnamed>".into(),
        Some([]) => "<empty>".into(),
        Some(b) => match std::str::from_utf8(b) {
            Ok(s) => format!("{s:?}"),
            Err(_) => {
                let mut s = String::from("<non-UTF8:");
                for byte in b {
                    write!(s, " {byte:02X}").expect("String write");
                }
                s.push('>');
                s
            }
        },
    }
}
pub fn render(report: &SymbolObservationReport) -> String {
    let l = report.limits();
    let h = report.proof().limits();
    let mut text = format!(
        "Symbol observation explicitly requested\nSource: {:?} | bytes: {} | provenance: {:?}\nBudgets: max-program-headers={} max-dynamic-entries={} max-hash-words={} max-descriptors={} max-symbols={} max-name-lookups={} max-name-scan-bytes={} max-total-name-scan-bytes={}\n",
        report.source().identity(),
        report.source().len(),
        report.source().provenance(),
        h.dynamic.max_program_headers,
        h.dynamic.max_dynamic_entries,
        h.hash.max_words,
        l.max_descriptors,
        l.max_symbols,
        l.max_name_lookups,
        l.max_name_scan_bytes,
        l.max_total_name_scan_bytes
    );
    match report.proof().outcome() {
        HashMetadataOutcome::Complete(h) => {
            if let Some(e) = h.extent() {
                writeln!(
                    text,
                    "Expected symbol count: {}\nTrusted extent: {:?}\nProof: {:?}",
                    e.symbol_count(),
                    e.source_range(),
                    e.evidence()
                )
                .expect("String write");
            } else {
                text.push_str("Exact count: Unavailable\n");
            }
        }
        HashMetadataOutcome::Unavailable => text.push_str("Exact count: Unavailable\n"),
        HashMetadataOutcome::Failed(e) => {
            writeln!(text, "Count proof failed: {e}").expect("String write");
        }
    }
    match report.outcome() {
        SymbolOutcome::Unavailable => {
            text.push_str("Symbols: Unavailable (no exact count; no enumeration)\n")
        }
        SymbolOutcome::Failed(e) => {
            writeln!(
                text,
                "Symbols: Failed\n{e}\nNo partial successful symbol list retained."
            )
            .expect("String write");
        }
        SymbolOutcome::Complete(records) => {
            writeln!(
                text,
                "Symbols: Complete\nObserved symbols: {}",
                records.len()
            )
            .expect("String write");
            for record in records {
                let f = record.fields();
                writeln!(text,"[{}] st_name={} st_info=0x{:02X} st_other=0x{:02X} st_shndx=0x{:04X} st_value=0x{:X} st_size={}\n  binding={:?} type={:?} visibility={:?} section={:?} name={} source={:?} name_source={:?}",f.index,f.name_offset,f.info,f.other,f.shndx,f.value,f.size,f.binding,f.symbol_type,f.visibility,f.section,name(record.name_bytes()),f.source,record.name_range()).expect("String write");
            }
        }
    }
    text.push_str("No import/export classification. No linkage or dependency resolution. No NID resolution. No guest loaded or executed.\n");
    text
}
