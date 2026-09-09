use astero_core::input::string_references::{
    StringEncoding, StringReferenceObservationReport, StringReferenceOutcome, StringReferenceRecord,
};
use std::fmt::Write;
/// Presentation only: UTF-8 is quoted/escaped, raw bytes are hex, empty is explicit.
pub fn format_bytes(record: &StringReferenceRecord) -> String {
    match record.encoding() {
        StringEncoding::Empty => "<empty>".into(),
        StringEncoding::Utf8 => format!(
            "{:?}",
            std::str::from_utf8(record.as_bytes()).expect("loader UTF-8 evidence")
        ),
        StringEncoding::RawBytes => {
            let mut text = String::from("<non-UTF8:");
            for byte in record.as_bytes() {
                write!(text, " {byte:02X}").expect("String write");
            }
            text.push('>');
            text
        }
    }
}
pub fn render(report: &StringReferenceObservationReport) -> String {
    let l = report.limits();
    let mut text = format!(
        "String-reference observation explicitly requested: DT_NEEDED bytes only\nBudgets: max-program-headers={} max-dynamic-entries={} max-descriptors={} max-string-references={} max-scan-bytes-per-reference={} max-total-scan-bytes={}\nSource: {:?} | bytes: {} | provenance: {:?}\n",
        l.descriptors.dynamic.max_program_headers,
        l.descriptors.dynamic.max_dynamic_entries,
        l.descriptors.max_descriptors,
        l.max_string_references,
        l.strings.max_scan_bytes_per_reference,
        l.strings.max_total_scan_bytes,
        report.source().identity(),
        report.source().len(),
        report.source().provenance()
    );
    match report.outcome() {
        StringReferenceOutcome::Unavailable => {
            text.push_str("String references: Unavailable (no supported references)\n")
        }
        StringReferenceOutcome::Failed(e) => {
            writeln!(
                text,
                "String references: Failed\n{e}\nNo partial successful string list retained."
            )
            .expect("String write");
        }
        StringReferenceOutcome::Complete(records) => {
            writeln!(
                text,
                "String references: Complete (referenced bytes only)\nReferences observed: {}",
                records.len()
            )
            .expect("String write");
            for r in records {
                let e = r.entry();
                writeln!(text,"Dynamic entry {}: tag={:?} offset={} encoding={:?} value={} | source={:?} | scanned bytes={}",e.index,e.tag,e.value,r.encoding(),format_bytes(r),r.source_range(),r.scanned_bytes()).expect("String write");
            }
        }
    }
    text.push_str("No whole-table enumeration. No dependency resolution or linkage. No guest loaded or executed.\n");
    text
}
