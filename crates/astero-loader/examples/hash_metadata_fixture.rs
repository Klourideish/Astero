use astero_loader::elf::dynamic::hash::synthetic::{Variant, image_with_hashes};
use std::{fs::OpenOptions, io::Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("expected output path and sysv|gnu|both|conflict|none|lower|truncated")?;
    let mode = args.next().ok_or("expected mode")?;
    if args.next().is_some() {
        return Err("extra argument".into());
    }
    let variant = match mode.to_str() {
        Some("sysv") => Variant::SysV,
        Some("gnu") => Variant::Gnu,
        Some("both") => Variant::Both,
        Some("conflict") => Variant::Conflict,
        Some("none") => Variant::None,
        Some("lower") => Variant::LowerBound,
        Some("truncated") => Variant::Truncated,
        _ => return Err("unsupported fixture mode".into()),
    };
    let bytes = image_with_hashes(variant);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(&bytes)?;
    println!("Wrote {} synthetic bytes; no guest content.", bytes.len());
    Ok(())
}
