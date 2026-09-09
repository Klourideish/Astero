use astero_loader::elf::dynamic::symbol_table::synthetic::{Variant, image_with_symbols};
use std::{fs::OpenOptions, io::Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected output path and fixture mode")?;
    let mode = args.next().ok_or("expected fixture mode")?;
    if args.next().is_some() {
        return Err("extra argument".into());
    }
    let variant = match mode.to_str() {
        Some("sysv") => Variant::SysV,
        Some("gnu") => Variant::Gnu,
        Some("both") => Variant::Both,
        Some("raw") => Variant::Raw,
        Some("empty") => Variant::Empty,
        Some("unnamed") => Variant::Unnamed,
        Some("unknown") => Variant::Unknown,
        Some("bad-name") => Variant::BadName,
        Some("lower") => Variant::LowerBound,
        Some("none") => Variant::None,
        _ => return Err("unknown fixture mode".into()),
    };
    let b = image_with_symbols(variant);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(&b)?;
    println!("Wrote {} synthetic bytes; no guest input.", b.len());
    Ok(())
}
