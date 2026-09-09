//! Small explicit M18 layouts, built with the existing M5 byte fixture generator.
use astero_loader::elf::dynamic::synthetic;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("Usage: descriptor_fixture <output-path> <valid|none|conflict>")?;
    let mode = args.next().ok_or("fixture mode required")?;
    if args.next().is_some() {
        return Err("too many fixture arguments".into());
    }
    let entries = match mode.to_str() {
        Some("valid") => vec![(5, 0x1400), (10, 16), (6, 0x1420), (11, 24), (0, 0)],
        Some("none") => vec![(4, u64::MAX), (7, u64::MAX), (-42, 9), (0, 0)],
        Some("conflict") => vec![(5, 0x1400), (5, 0x1410), (10, 16), (0, 0)],
        _ => return Err("expected valid, none, or conflict".into()),
    };
    let mut bytes = synthetic::image(&entries);
    // Deliberately unsuitable payload bytes: no string terminator and invalid symbol fields.
    bytes[0x400..0x440].fill(0xff);
    use std::io::Write;
    let mut out = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    out.write_all(&bytes)?;
    println!(
        "Wrote {} synthetic {:?} fixture bytes to {:?}; no guest input.",
        bytes.len(),
        mode,
        path
    );
    Ok(())
}
