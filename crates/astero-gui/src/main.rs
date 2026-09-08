use astero_core::session::Session;

fn main() -> Result<(), String> {
    let mut session = Session::new().map_err(|e| format!("Create session: {e:?}"))?;
    session
        .initialize()
        .map_err(|e| format!("Initialize session: {e:?}"))?;
    let result = astero_gui::run(session.observer());
    drop(session); // Core owner outlives windowing and every weak observer.
    result
}
