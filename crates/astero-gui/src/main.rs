use astero_core::session::Session;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut session = if args == ["--synthetic-linkage"] {
        astero_core::session::inputs::synthetic::session(8)
            .map_err(|e| format!("Synthetic session: {e:?}"))?
    } else if args.is_empty() {
        Session::new().map_err(|e| format!("Create session: {e:?}"))?
    } else {
        return Err("Usage: astero-gui [--synthetic-linkage]".into());
    };
    session
        .initialize()
        .map_err(|e| format!("Initialize session: {e:?}"))?;
    let result = astero_gui::run(session.observer());
    drop(session); // Core owner outlives windowing and every weak observer.
    result
}
