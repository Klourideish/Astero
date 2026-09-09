use astero_core::session::Session;

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_some_and(|a| a == "acquire") {
        let request = astero_cli::acquisition::parse(args.into_iter().skip(1))
            .map_err(|e| format!("{e}\n{}", astero_cli::acquisition::USAGE))?;
        let mut selection = astero_cli::acquisition::Selection::new(request);
        selection.acquire();
        let text = astero_cli::acquisition::render(&selection);
        if matches!(selection.state(), astero_cli::acquisition::State::Failed(_)) {
            return Err(text);
        }
        print!("{text}");
        return Ok(());
    }
    if args.first().is_some_and(|a| a == "--linkage") {
        let synthetic = args.iter().skip(1).any(|a| a == "--synthetic");
        let details = args.iter().skip(1).any(|a| a == "--details");
        if args.len() != 1 + usize::from(synthetic) + usize::from(details)
            || args
                .iter()
                .skip(1)
                .any(|a| a != "--synthetic" && a != "--details")
        {
            return Err("Usage: astero-cli --linkage [--synthetic] [--details]".into());
        }
        let mut session = if synthetic {
            astero_cli::linkage::synthetic::session(8)?
        } else {
            Session::new().map_err(|e| format!("Create session: {e:?}"))?
        };
        session
            .initialize()
            .map_err(|e| format!("Initialize session: {e:?}"))?;
        let snapshot = astero_debug::snapshots::linkage::inspect_linkage(&session.observer())
            .map_err(|e| format!("Observe linkage: {e:?}"))?;
        if synthetic {
            println!("Synthetic in-memory ELF evidence demo; no loaded PS5 executable.");
        }
        print!("{}", astero_cli::linkage::render(&snapshot, details));
        return Ok(());
    }
    if !args.is_empty() {
        return Err(format!(
            "Usage: astero-cli [--linkage [--synthetic] [--details]]\n{}",
            astero_cli::acquisition::USAGE
        ));
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
