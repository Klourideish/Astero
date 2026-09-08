use astero_core::session::Session;
use astero_debug::inspection::inspect_session;

#[test]
fn presentation_tracks_common_state_and_all_capabilities() {
    let mut session = Session::new().unwrap();
    let observer = session.observer();
    session.initialize().unwrap();
    let report = inspect_session(&observer).unwrap();
    let text = astero_cli::render(&report);
    assert!(text.contains(&report.session.id.to_string()));
    assert!(text.contains("Lifecycle: Ready"));
    assert!(text.contains("Loaded target: No guest loaded"));
    assert!(text.contains("Host lifecycle changes: 1"));
    for (capability, support) in report.capabilities {
        assert!(text.contains(&format!("Debugger {capability:?}: {support:?}")));
    }
    session.stop().unwrap();
    assert!(
        astero_cli::render(&inspect_session(&observer).unwrap()).contains("Lifecycle: Stopped")
    );
}

#[test]
fn executable_inspects_and_rejects_unknown_arguments() {
    let exe = env!("CARGO_BIN_EXE_astero-cli");
    let output = std::process::Command::new(exe).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Lifecycle: Ready"));
    assert!(text.contains("Debugger ActualPc: Unsupported"));
    assert!(
        !std::process::Command::new(exe)
            .arg("--run-game")
            .output()
            .unwrap()
            .status
            .success()
    );
}
