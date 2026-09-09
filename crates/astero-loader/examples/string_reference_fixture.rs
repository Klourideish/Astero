//! Focused synthetic string references using the established dynamic fixture layout.
use astero_loader::elf::dynamic::synthetic;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or(
        "Usage: string_reference_fixture <output-path> <valid|raw|multiple|none|unterminated>",
    )?;
    let mode = args.next().ok_or("fixture mode required")?;
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let (payload, refs): (Vec<u8>, Vec<u64>) = match mode.to_str() {
        Some("valid") => (b"\0libdemo.so\0\xff\0unreferenced".to_vec(), vec![1]),
        Some("raw") => (b"\0libdemo.so\0\xff\0unreferenced".to_vec(), vec![12]),
        Some("multiple") => (
            b"\0libdemo.so\0\xff\0unreferenced".to_vec(),
            vec![1, 12, 0, 1],
        ),
        Some("none") => (b"unreferenced".to_vec(), vec![]),
        Some("unterminated") => (b"abc".to_vec(), vec![0]),
        _ => return Err("unsupported fixture mode".into()),
    };
    let mut entries = vec![(5, 0x1400), (10, payload.len() as u64), (14, u64::MAX)];
    entries.extend(refs.into_iter().map(|offset| (1, offset)));
    entries.push((0, 0));
    let mut bytes = synthetic::image(&entries);
    bytes[0x400..0x400 + payload.len()].copy_from_slice(&payload);
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
