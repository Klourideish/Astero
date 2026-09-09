//! Write a small generated M4 fixture for manual inspection; no real binary input.
use astero_loader::elf::inspect::synthetic as elf_fixtures;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("Usage: inspection_fixture <output-path>")?;
    if args.next().is_some() {
        return Err("Usage: inspection_fixture <output-path>".into());
    }
    use std::io::Write;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    output.write_all(&elf_fixtures::executable())?;
    println!(
        "Wrote 272 generated fixture bytes to {:?}; no guest executable used.",
        path
    );
    Ok(())
}
