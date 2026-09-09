use astero_cli::acquisition::{ArgumentError, Request, Selection, State, parse, render};
use astero_core::input::acquisition::{AcquisitionLimits, Failure};
use std::{ffi::OsString, fs, path::PathBuf, process::Command};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m15-input-fixtures");
        fs::create_dir_all(&root).unwrap();
        for i in 0..1000 {
            let p = root.join(format!("{}-{i}", std::process::id()));
            match fs::create_dir(&p) {
                Ok(()) => return Self(p),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        }
        panic!("fixture namespace exhausted");
    }
    fn file(&self) -> PathBuf {
        let path = self.0.join("unparsed.elf");
        fs::write(&path, [0xff, 0, 42]).unwrap();
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn args(path: &std::path::Path, bytes: &str, calls: &str) -> Vec<OsString> {
    vec![
        "--path".into(),
        path.as_os_str().into(),
        "--max-bytes".into(),
        bytes.into(),
        "--max-read-calls".into(),
        calls.into(),
    ]
}
fn exe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_astero-cli"))
}

#[test]
fn required_limits_have_no_defaults_and_syntax_errors_remain_structured() {
    assert!(matches!(parse([]), Err(ArgumentError::Missing("--path"))));
    let good = args(std::path::Path::new("arbitrary"), "3", "2");
    for length in [2, 4] {
        assert!(matches!(
            parse(good[..length].to_vec()),
            Err(ArgumentError::Missing(_))
        ));
    }
    assert!(matches!(
        parse([OsString::from("--path")]),
        Err(ArgumentError::MissingValue("--path"))
    ));
    for bad in ["", "-1", "+1", "1.5", "18446744073709551616"] {
        assert!(matches!(
            parse(args(std::path::Path::new("x"), bad, "2")),
            Err(ArgumentError::InvalidLimit {
                option: "--max-bytes",
                ..
            })
        ));
    }
    let mut duplicate = good.clone();
    duplicate.extend(["--max-bytes".into(), "4".into()]);
    assert!(matches!(
        parse(duplicate),
        Err(ArgumentError::Duplicate("--max-bytes"))
    ));
    let mut mixed = good;
    mixed.push("--linkage".into());
    assert!(matches!(parse(mixed), Err(ArgumentError::UnknownOption(_))));
    assert_eq!(
        parse(args(std::path::Path::new("x"), "18446744073709551615", "0"))
            .unwrap()
            .limits
            .max_bytes,
        u64::MAX
    );
}

#[test]
fn selection_retains_native_request_and_replaces_only_its_acquisition_result() {
    let f = Fixture::new();
    let path = f.file();
    let request = parse(args(&path, "3", "2")).unwrap();
    let mut selection = Selection::new(request.clone());
    assert!(matches!(selection.state(), State::Ready));
    assert_eq!(selection.request(), &request);
    assert!(render(&selection).contains("Status: Ready"));
    selection.acquire();
    let State::Acquired(first) = selection.state() else {
        panic!("expected source")
    };
    let first = first.clone();
    assert_eq!(first.len(), 3);
    selection.acquire();
    let State::Acquired(second) = selection.state() else {
        panic!("expected source")
    };
    assert_ne!(first.identity(), second.identity());
    assert_eq!(first.provenance(), second.provenance());
    assert_eq!(
        first.read(&first.checked_range(0, 3).unwrap()).unwrap(),
        second.read(&second.checked_range(0, 3).unwrap()).unwrap()
    );
    fs::write(&path, [1, 2, 3, 4]).unwrap();
    selection.acquire();
    assert!(
        matches!(selection.state(),State::Failed(e) if matches!(e.failure,Failure::SizeLimit{..}))
    );
    assert_eq!(
        first.read(&first.checked_range(0, 3).unwrap()).unwrap(),
        [0xff, 0, 42]
    );
}

#[test]
fn limits_propagate_without_frontend_revalidation_or_silent_defaults() {
    let f = Fixture::new();
    let path = f.file();
    for (bytes, calls, expected) in [
        (2, 2, "size"),
        (3, 0, "budget"),
        (3, 1, "budget"),
        (3, 2, "acquired"),
    ] {
        let mut s = Selection::new(Request {
            path: path.clone(),
            limits: AcquisitionLimits {
                max_bytes: bytes,
                max_read_calls: calls,
            },
        });
        s.acquire();
        match (expected, s.state()) {
            ("size", State::Failed(e)) => assert!(matches!(
                e.failure,
                Failure::SizeLimit {
                    observed: 3,
                    maximum: 2
                }
            )),
            ("budget", State::Failed(e)) => {
                assert!(matches!(e.failure,Failure::ReadBudget{maximum,..} if maximum==calls))
            }
            ("acquired", State::Acquired(source)) => assert_eq!(source.len(), 3),
            _ => panic!("wrong result: {:?}", s.state()),
        }
        let text = render(&s);
        assert!(text.contains(&format!("max-bytes={bytes} max-read-calls={calls}")));
    }
}

#[test]
fn nonexistent_input_preserves_io_error_path_and_failure_presentation() {
    use std::error::Error;
    let f = Fixture::new();
    let path = f.0.join("missing");
    let mut s = Selection::new(parse(args(&path, "3", "2")).unwrap());
    s.acquire();
    let State::Failed(e) = s.state() else {
        panic!("expected I/O error")
    };
    assert_eq!(e.path, path);
    assert_eq!(
        e.source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::NotFound
    );
    let text = render(&s);
    assert!(text.contains("Status: Failed"));
    assert!(text.contains("InspectPath"));
    assert!(text.contains("No guest loaded"));
}

#[test]
fn live_command_stops_at_arbitrary_bytes_and_reports_limits_and_exit_status() {
    let f = Fixture::new();
    let path = f.file();
    let out = exe()
        .arg("acquire")
        .args(args(&path, "3", "2"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    for expected in [
        "Astero acquisition only",
        "max-bytes=3 max-read-calls=2",
        "Status: Acquired",
        "Observed bytes: 3",
        "No guest loaded. No parsing or linkage performed.",
    ] {
        assert!(text.contains(expected), "{text}");
    }
    assert!(!text.contains("Lifecycle:"));
    assert!(!text.contains("Import candidates"));
    let out = exe()
        .arg("acquire")
        .args(args(&path, "2", "2"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("Status: Failed"));
    assert!(text.contains("SizeLimit"));
    let out = exe()
        .arg("acquire")
        .arg("--path")
        .arg(&path)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("--max-bytes")
    );
}

#[cfg(any(windows, unix))]
#[test]
fn native_non_utf8_selection_reaches_acquisition_including_process_arguments() {
    #[cfg(windows)]
    let name = {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[b'n' as u16, 0xd800])
    };
    #[cfg(unix)]
    let name = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![b'n', 255])
    };
    let f = Fixture::new();
    let path = f.0.join(name);
    assert!(path.to_str().is_none());
    fs::write(&path, [7]).unwrap();
    let request = parse(args(&path, "1", "2")).unwrap();
    assert_eq!(request.path, path);
    let mut s = Selection::new(request);
    s.acquire();
    assert!(matches!(s.state(),State::Acquired(a) if a.len()==1));
    let out = exe()
        .arg("acquire")
        .args(args(&path, "1", "2"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("Observed bytes: 1")
    );
}
