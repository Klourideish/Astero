use astero_cli::{
    acquisition::{Selection, State},
    dynamic, inspection,
};
use astero_core::input::{
    dynamic::{DynamicLimits, DynamicOutcome},
    inspection::{InspectionLimits, InspectionOutcome},
};
use astero_loader::elf::dynamic::synthetic::*;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m17-input-fixtures");
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
        "2048".into(),
        "--max-read-calls".into(),
        "4".into(),
        "--max-program-headers".into(),
        headers.into(),
    ]
}
fn exe() -> Command {
    Command::new(env!("CARGO_BIN_EXE_astero-cli"))
}

fn dynamic_args(path: &std::path::Path, entries: &str) -> Vec<OsString> {
    let mut a = args(path, "2");
    a.extend(["--max-dynamic-entries".into(), entries.into()]);
    a
}
#[test]
fn independent_requests_preserve_acquisition_and_header_only_behaviour() {
    let f = Fixture::new();
    let path = f.file(&image(&[(1, 0)]));
    let req = dynamic::parse(dynamic_args(&path, "1")).unwrap();
    assert_eq!(req.acquisition.limits.max_bytes, 2048);
    assert_eq!(req.acquisition.limits.max_read_calls, 4);
    let mut s = Selection::new(req.acquisition);
    assert!(dynamic::observe_acquired(&s, req.limits).is_err());
    s.acquire();
    let State::Acquired(source) = s.state() else {
        panic!("acquired")
    };
    let id = source.identity();
    let h = inspection::inspect_acquired(
        &s,
        InspectionLimits {
            max_program_headers: 2,
        },
    )
    .unwrap();
    assert!(matches!(h.outcome(), InspectionOutcome::Complete(_)));
    let d = dynamic::observe_acquired(&s, req.limits).unwrap();
    assert!(matches!(d.outcome(), DynamicOutcome::Failed(_)));
    assert_eq!(d.source().identity(), id);
    assert!(matches!(s.state(),State::Acquired(a) if a.identity()==id));
    for (mode, a) in [
        ("acquire", args(&path, "2").into_iter().take(6).collect()),
        ("inspect", args(&path, "2")),
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success());
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("Dynamic observation:")
        );
    }
}
#[test]
fn real_cli_budget_status_raw_values_and_nonsemantic_boundary() {
    let f = Fixture::new();
    let path = f.file(&image(&[(5, u64::MAX), (-42, 123), (0, 7)]));
    for budget in ["0", "2", "3"] {
        let out = exe()
            .arg("dynamic")
            .args(dynamic_args(&path, budget))
            .output()
            .unwrap();
        assert_eq!(out.status.success(), budget == "3");
        let text = String::from_utf8(if out.status.success() {
            out.stdout
        } else {
            out.stderr
        })
        .unwrap();
        for label in [
            "Status: Acquired",
            "Dynamic observation explicitly requested",
            "No guest loaded. No linkage derived. No execution occurred.",
            "max-program-headers=2",
        ] {
            assert!(text.contains(label), "{text}");
        }
        if budget == "3" {
            for label in [
                "Complete",
                "Observed entries: 3",
                "Unknown(-42)",
                "0xffffffffffffffff",
                "Termination: DT_NULL",
            ] {
                assert!(text.contains(label), "{text}");
            }
        } else {
            assert!(text.contains("EntryLimit"));
            assert!(!text.contains("Entry 0:"));
        }
    }
    let req = dynamic::parse(dynamic_args(&path, "3")).unwrap();
    let mut s = Selection::new(req.acquisition);
    s.acquire();
    let a = dynamic::observe_acquired(&s, req.limits).unwrap();
    let b = dynamic::observe_acquired(&s, req.limits).unwrap();
    assert_eq!(dynamic::render(&a), dynamic::render(&b));
    assert_eq!(a.limits().max_dynamic_entries, 3);
}
#[test]
fn arguments_absence_malformed_and_acquisition_failures_remain_distinct() {
    let f = Fixture::new();
    let path = f.file(&image(&[(0, 0)]));
    assert!(dynamic::parse(args(&path, "2")).is_err());
    for value in ["-1", "overflow", "18446744073709551616"] {
        assert!(dynamic::parse(dynamic_args(&path, value)).is_err());
    }
    let mut duplicate = dynamic_args(&path, "1");
    duplicate.extend(["--max-dynamic-entries".into(), "2".into()]);
    assert!(dynamic::parse(duplicate).is_err());
    let mut b = image(&[(0, 0)]);
    put32(&mut b, 120, 0);
    fs::write(&path, b).unwrap();
    let out = exe()
        .arg("dynamic")
        .args(dynamic_args(&path, "0"))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("Unavailable (no PT_DYNAMIC)")
    );
    fs::write(&path, [1, 2, 3]).unwrap();
    let out = exe()
        .arg("dynamic")
        .args(dynamic_args(&path, "1"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8(out.stderr).unwrap().contains("Headers"));
    let out = exe()
        .arg("dynamic")
        .args(dynamic_args(&f.0.join("missing"), "1"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("Status: Failed"));
    assert!(!text.contains("Dynamic observation:"));
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_survives_dynamic_frontend_selection_and_process() {
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
    fs::write(&path, image(&[(0, 0)])).unwrap();
    let req = dynamic::parse(dynamic_args(&path, "1")).unwrap();
    assert_eq!(req.acquisition.path, path);
    assert_eq!(
        req.limits,
        DynamicLimits {
            max_program_headers: 2,
            max_dynamic_entries: 1
        }
    );
    let out = exe()
        .arg("dynamic")
        .args(dynamic_args(&path, "1"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
}
