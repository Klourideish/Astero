use astero_loader::elf::dynamic::symbol_table::synthetic::{Variant, image_with_symbols};
use std::{fs::OpenOptions, io::Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected output path and fixture mode")?;
    let mode = args.next().ok_or("expected fixture mode")?;
    if args.next().is_some() {
        return Err("extra argument".into());
    }
    let mut b = match mode.to_str() {
        Some("ordinary") => image_with_symbols(Variant::SysV),
        Some("special") => image_with_symbols(Variant::Unknown),
        _ => return Err("expected ordinary or special".into()),
    };
    if mode == "ordinary" {
        b[0x434] = 0x12;
    } // Existing symbol 2: GLOBAL/FUNC, ordinary section.
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(&b)?;
    println!("Wrote {} synthetic bytes; no guest input.", b.len());
    Ok(())
}
