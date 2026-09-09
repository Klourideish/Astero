//! Write a small generated M5 dynamic fixture for manual inspection; no real binary input.
use astero_loader::elf::dynamic::synthetic as elf_fixtures;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("Usage: dynamic_fixture <output-path>")?;
    if args.next().is_some() {
        return Err("Usage: dynamic_fixture <output-path>".into());
    }
    use std::io::Write;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    output.write_all(&elf_fixtures::image(&[
        (5, u64::MAX),
        (-42, 0xfedcba9876543210),
        (0, 7),
    ]))?;
    println!(
        "Wrote 1536 generated fixture bytes to {:?}; no guest executable used.",
        path
    );
    Ok(())
}
