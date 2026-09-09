use astero_core::session::Session;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--linkage"] || args == ["--linkage", "--details"] {
        let snapshot = astero_debug::snapshots::linkage::inspect_linkage(None);
        print!(
            "{}",
            astero_cli::linkage::render(&snapshot, args.len() == 2)
        );
        return Ok(());
    }
    if !args.is_empty() {
        return Err(
            "Usage: astero-cli [--linkage [--details]] (no target input adapter yet)".into(),
        );
    }
    let mut session = Session::new().map_err(|e| format!("Create session: {e:?}"))?;
    session
        .initialize()
        .map_err(|e| format!("Initialize session: {e:?}"))?;
    let report = astero_debug::inspection::inspect_session(&session.observer())
        .map_err(|e| format!("Inspect session: {e:?}"))?;
    print!("{}", astero_cli::render(&report));
    Ok(())
}
