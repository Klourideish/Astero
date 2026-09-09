use astero_core::input::descriptors::{
    DescriptorObservationReport, DescriptorOutcome, DescriptorValue,
};
use std::fmt::Write;
pub fn render(report: &DescriptorObservationReport) -> String {
    let limits = report.limits();
    let mut text = format!(
        "Descriptor observation explicitly requested: STRTAB/STRSZ and SYMTAB/SYMENT only\nBudgets: max-program-headers={} max-dynamic-entries={} max-descriptors={}\nSource: {:?} | bytes: {} | provenance: {:?}\n",
        limits.dynamic.max_program_headers,
        limits.dynamic.max_dynamic_entries,
        limits.max_descriptors,
        report.source().identity(),
        report.source().len(),
        report.source().provenance()
    );
    if let Some(raw) = report.raw() {
        writeln!(
            text,
            "Raw table: program header {} | source range {:?} | entries {} (DT_NULL retained)",
            raw.program_index(),
            raw.source_range(),
            raw.entries().len()
        )
        .expect("String write");
    }
    match report.outcome() {
        DescriptorOutcome::Unavailable(reason) => {
            writeln!(text, "Descriptor observation: Unavailable ({reason:?})")
                .expect("String write");
        }
        DescriptorOutcome::Failed(error) => {
            writeln!(
                text,
                "Descriptor observation: Failed\n{error}\nNo complete descriptor list retained."
            )
            .expect("String write");
        }
        DescriptorOutcome::Complete(records) => {
            writeln!(text,"Descriptor observation: Complete (selected metadata only)\nDescriptors observed: {}",records.len()).expect("String write");
            for record in records {
                writeln!(text, "Family: {:?}", record.family()).expect("String write");
                for field in record.fields() {
                    writeln!(
                        text,
                        "  Dynamic entry {}: tag={:?} raw-value={:#018x}",
                        field.index, field.tag, field.value
                    )
                    .expect("String write");
                }
                match record.value() {
                    DescriptorValue::Strings(d) => {
                        writeln!(
                            text,
                            "  Address: {:#x} | declared bytes: {} | source range: {:?}",
                            d.address.0, d.size, d.source
                        )
                        .expect("String write");
                    }
                    DescriptorValue::Symbols(d) => {
                        writeln!(text,"  Address: {:#x} | entry bytes: {} | first entry only: {:?} | symbol count unproven",d.address.0,d.entry_size,d.first_entry).expect("String write");
                    }
                }
            }
        }
    }
    text.push_str("No payload traversal occurred. No guest loaded. No linkage derived. No execution occurred.\n");
    text
}
