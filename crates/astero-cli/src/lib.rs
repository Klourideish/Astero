//! Human-readable presentation of the shared debugger report; no emulator state.
use astero_debug::inspection::Inspection;
use std::fmt::Write;

pub fn render(report: &Inspection) -> String {
    let s = &report.session;
    let mut text = format!("Astero | {}\nLifecycle: {:?}\n", s.id, s.lifecycle);
    writeln!(
        text,
        "Loaded target: {}",
        s.loaded_target
            .as_ref()
            .map_or("No guest loaded", |t| t.display_name.as_str())
    )
    .expect("String write");
    writeln!(
        text,
        "Host lifecycle changes: {}",
        s.statistics.lifecycle_changes
    )
    .expect("String write");
    for subsystem in &s.subsystems {
        writeln!(
            text,
            "Subsystem {}: {:?}",
            subsystem.name, subsystem.availability
        )
        .expect("String write");
    }
    for diagnostic in &s.diagnostics {
        writeln!(text, "Diagnostic: {diagnostic:?}").expect("String write");
    }
    for (capability, support) in report.capabilities {
        writeln!(text, "Debugger {capability:?}: {support:?}").expect("String write");
    }
    text
}

pub mod linkage;

pub mod acquisition;

pub mod inspection;

pub mod dynamic;

pub mod descriptors;

pub mod string_references;

pub mod hash_metadata;

pub mod symbols;

pub mod classification;

pub mod linkage_evidence;
