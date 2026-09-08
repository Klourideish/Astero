use astero_core::session::Session;

fn main() -> Result<(), String> {
    if std::env::args().len() != 1 {
        return Err("Usage: astero-cli (creates and inspects an unloaded host session)".into());
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
