use astero_core::session::Session;

fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args == ["--presentation-smoke"] {
        let mut session = Session::new().map_err(|e| format!("Session: {e:?}"))?;
        let endpoint = astero_video::presentation::Endpoint::new("window");
        session.attach_presentation(endpoint.clone());
        #[cfg(all(windows, target_arch = "x86_64"))]
        let observation = {
            let o = astero_core::input::entry::observability::Observer::new(
                "synthetic host presentation".into(),
                None,
            )?;
            o.observe_presentation(Some(&endpoint));
            o
        };
        let result = astero_gui::presentation_smoke(endpoint);
        #[cfg(all(windows, target_arch = "x86_64"))]
        eprintln!(
            "Host presentation: {:?}",
            observation.snapshot().host_presentation
        );
        session.detach_presentation();
        return result;
    }
    let mut session = if args == ["--synthetic-linkage"] {
        astero_core::session::inputs::synthetic::session(8)
            .map_err(|e| format!("Synthetic session: {e:?}"))?
    } else if args.is_empty() {
        Session::new().map_err(|e| format!("Create session: {e:?}"))?
    } else {
        return Err("Usage: astero-gui [--synthetic-linkage | --presentation-smoke]".into());
    };
    session
        .initialize()
        .map_err(|e| format!("Initialize session: {e:?}"))?;
    let result = astero_gui::run(session.observer());
    drop(session); // Core owner outlives windowing and every weak observer.
    result
}
