use astero_loader::elf::dynamic::identity::synthetic::identity_image;
use std::{fs::OpenOptions, io::Write};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected output path")?;
    if args.next().is_some() {
        return Err("extra argument".into());
    }
    let b = identity_image();
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(&b)?;
    println!("Wrote {} synthetic bytes; no guest input.", b.len());
    Ok(())
}
