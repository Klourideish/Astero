use astero_cli::{
    acquisition::{self, Selection, State},
    inspection::{self, RequestError},
};
use astero_core::input::inspection::{InspectionFailure, InspectionLimits, InspectionOutcome};
use astero_loader::elf::inspect::synthetic as elf_fixtures;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m16-input-fixtures");
        fs::create_dir_all(&root).unwrap();
        for i in 0..1000 {
            let p = root.join(format!("{}-{i}", std::process::id()));
            match fs::create_dir(&p) {
                Ok(()) => return Self(p),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => panic!("{e}"),
            }
        }
        panic!("fixture namespace exhausted")
    }
    fn file(&self, bytes: &[u8]) -> PathBuf {
        let p = self.0.join("input");
        fs::write(&p, bytes).unwrap();
        p
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn args(path: &std::path::Path, headers: &str) -> Vec<OsString> {
    vec![
        "--path".into(),
        path.as_os_str().into(),
        "--max-bytes".into(),
        "1024".into(),
        "--max-read-calls".into(),
        "4".into(),
        "--max-program-headers".into(),
        headers.into(),
    ]
}
fn exe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_astero-cli"))
}
#[test]
fn acquisition_does_not_inspect_until_explicit_request_and_never_changes_selection() {
    let f = Fixture::new();
    let path = f.file(&[1, 2, 3]);
    let request = inspection::parse(args(&path, "1")).unwrap();
    assert_eq!(request.acquisition.limits.max_bytes, 1024);
    assert_eq!(request.acquisition.limits.max_read_calls, 4);
    let mut s = Selection::new(request.acquisition);
    assert!(matches!(
        inspection::inspect_acquired(&s, request.limits),
        Err(RequestError::NotAcquired)
    ));
    s.acquire();
    let State::Acquired(source) = s.state() else {
        panic!("invalid ELF still acquires")
    };
    let id = source.identity();
    let report = inspection::inspect_acquired(&s, request.limits).unwrap();
    assert!(matches!(
        report.outcome(),
        InspectionOutcome::Failed(InspectionFailure::Header(_))
    ));
    assert_eq!(report.source().identity(), id);
    assert!(matches!(s.state(),State::Acquired(a) if a.identity()==id));
    let out = exe()
        .arg("acquire")
        .args(args(&path, "1").into_iter().take(6))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        !String::from_utf8(out.stdout)
            .unwrap()
            .contains("Inspection:")
    );
}
#[test]
fn explicit_limits_exact_boundary_and_exhaustion_survive_frontend_layers() {
    let f = Fixture::new();
    let path = f.file(&elf_fixtures::executable());
    for (n, expected) in [("0", "Failed"), ("1", "Complete")] {
        let request = inspection::parse(args(&path, n)).unwrap();
        let mut s = Selection::new(request.acquisition);
        s.acquire();
        let report = inspection::inspect_acquired(&s, request.limits).unwrap();
        assert_eq!(
            report.limits().max_program_headers,
            n.parse::<u64>().unwrap()
        );
        assert!(inspection::render(&report).contains(&format!("Inspection: {expected}")));
        let out = exe().arg("inspect").args(args(&path, n)).output().unwrap();
        assert_eq!(out.status.success(), n == "1");
        let text = String::from_utf8(if out.status.success() {
            out.stdout
        } else {
            out.stderr
        })
        .unwrap();
        for label in [
            "Status: Acquired",
            "max-bytes=1024 max-read-calls=4",
            "Inspection explicitly requested",
            "No guest loaded. No execution or linkage performed.",
        ] {
            assert!(text.contains(label), "{text}");
        }
        if n == "0" {
            assert!(text.contains("HeaderBudget"));
        }
    }
}
#[test]
fn syntax_acquisition_failures_and_inspection_failures_are_distinct() {
    let f = Fixture::new();
    let path = f.file(&[0; 64]);
    assert!(inspection::parse(args(&path, "1").into_iter().take(6)).is_err());
    let mut duplicate = args(&path, "1");
    duplicate.extend(["--max-program-headers".into(), "2".into()]);
    assert!(inspection::parse(duplicate).is_err());
    assert!(inspection::parse(args(&path, "-1")).is_err());
    let out = exe()
        .arg("inspect")
        .args(args(&path, "1"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("InvalidMagic"));
    assert!(text.contains("Status: Acquired"));
    let out = exe()
        .arg("inspect")
        .args(args(&f.0.join("missing"), "1"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("Status: Failed"));
    assert!(!text.contains("Inspection:"));
    let mut limited = args(&path, "1");
    limited[3] = "1".into();
    let out = exe().arg("inspect").args(limited).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("SizeLimit"));
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_and_header_only_zero_budget_work_through_real_executable() {
    #[cfg(windows)]
    let name = {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[110, 0xd800])
    };
    #[cfg(unix)]
    let name = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![110, 255])
    };
    let f = Fixture::new();
    let path = f.0.join(name);
    fs::write(&path, elf_fixtures::header(0)).unwrap();
    let out = exe()
        .arg("inspect")
        .args(args(&path, "0"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Program headers observed: 0"));
    assert!(text.contains("No guest loaded"));
    let req = inspection::parse(args(&path, "0")).unwrap();
    assert_eq!(req.acquisition.path, path);
    let mut s = Selection::new(req.acquisition);
    s.acquire();
    let a = inspection::inspect_acquired(
        &s,
        InspectionLimits {
            max_program_headers: 0,
        },
    )
    .unwrap();
    let b = inspection::inspect_acquired(&s, a.limits()).unwrap();
    assert_eq!(inspection::render(&a), inspection::render(&b));
    assert_eq!(a.source().provenance(), b.source().provenance());
    assert!(acquisition::render(&s).contains("No parsing or linkage performed"));
}
