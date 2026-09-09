use astero_core::input::inspection::{InspectionOutcome, InspectionReport};
use std::fmt::Write;
pub fn render(report: &InspectionReport) -> String {
    let mut text = format!(
        "Inspection explicitly requested: ELF64 file/program headers only\nInspection budget: max-program-headers={}\nSource: {:?} | bytes: {} | provenance: {:?}\n",
        report.limits().max_program_headers,
        report.source().identity(),
        report.source().len(),
        report.source().provenance()
    );
    match report.outcome() {
        InspectionOutcome::Failed(error) => {
            writeln!(text, "Inspection: Failed\n{error}").expect("String write");
        }
        InspectionOutcome::Complete(e) => {
            let h = e.header();
            writeln!(text,"Inspection: Complete (header scope only)\nELF identification: {:?}\nObject type: {} | machine: {} | entry: {:#x}\nProgram headers observed: {}",h.identification,h.object_type,h.machine,h.entry,e.program_headers().len()).expect("String write");
            for (i, p) in e.program_headers().iter().enumerate() {
                writeln!(text,"Header {i}: type={} flags={:#x} offset={:#x} vaddr={:#x} file-bytes={} memory-bytes={} alignment={}",p.kind,p.flags,p.file_offset,p.virtual_address,p.file_size,p.memory_size,p.alignment).expect("String write");
            }
        }
    }
    text.push_str("No guest loaded. No execution or linkage performed. No sections or dynamic metadata interpreted.\n");
    text
}
