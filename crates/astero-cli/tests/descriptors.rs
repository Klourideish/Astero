use astero_cli::{
    acquisition::{Selection, State},
    descriptors, dynamic,
};
use astero_core::input::descriptors::{DescriptorFailure, DescriptorOutcome};
use astero_loader::elf::dynamic::synthetic::*;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m18-input-fixtures");
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

fn descriptor_args(path: &std::path::Path, max: &str) -> Vec<OsString> {
    let mut a = dynamic_args(path, "5");
    a.extend(["--max-descriptors".into(), max.into()]);
    a
}
#[test]
fn prior_commands_do_not_request_descriptors_even_when_pairing_would_fail() {
    let f = Fixture::new();
    let path = f.file(&image(&[(5, u64::MAX), (0, 0)]));
    for (mode, a) in [
        ("acquire", args(&path, "2").into_iter().take(6).collect()),
        ("inspect", args(&path, "2")),
        ("dynamic", dynamic_args(&path, "2")),
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success(), "{:?}", out.stderr);
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("Descriptor observation:")
        );
    }
    let req = descriptors::parse(descriptor_args(&path, "1")).unwrap();
    let mut s = Selection::new(req.acquisition);
    assert!(descriptors::observe_acquired(&s, req.limits).is_err());
    s.acquire();
    let State::Acquired(source) = s.state() else {
        panic!()
    };
    let id = source.identity();
    let r = descriptors::observe_acquired(&s, req.limits).unwrap();
    assert!(matches!(
        r.outcome(),
        DescriptorOutcome::Failed(DescriptorFailure::Interpretation(_))
    ));
    assert_eq!(r.source().identity(), id);
    assert!(matches!(s.state(),State::Acquired(v)if v.identity()==id));
    assert!(dynamic::observe_acquired(&s, req.limits.dynamic).is_ok());
}
#[test]
fn limits_and_complete_failed_unavailable_status_survive_real_cli() {
    let f = Fixture::new();
    let path = f.file(&image(&[
        (5, 0x1400),
        (10, 16),
        (6, 0x1420),
        (11, 24),
        (0, 0),
    ]));
    for maximum in ["0", "1", "2"] {
        let req = descriptors::parse(descriptor_args(&path, maximum)).unwrap();
        assert_eq!(req.acquisition.limits.max_bytes, 2048);
        assert_eq!(req.acquisition.limits.max_read_calls, 4);
        assert_eq!(req.limits.dynamic.max_program_headers, 2);
        assert_eq!(req.limits.dynamic.max_dynamic_entries, 5);
        let out = exe()
            .arg("descriptors")
            .args(descriptor_args(&path, maximum))
            .output()
            .unwrap();
        assert_eq!(out.status.success(), maximum == "2");
        let text = String::from_utf8(if out.status.success() {
            out.stdout
        } else {
            out.stderr
        })
        .unwrap();
        for label in [
            "Status: Acquired",
            "Descriptor observation explicitly requested",
            "No payload traversal occurred. No guest loaded. No linkage derived. No execution occurred.",
        ] {
            assert!(text.contains(label), "{text}");
        }
        if maximum == "2" {
            for label in [
                "Descriptors observed: 2",
                "Family: Strings",
                "Family: Symbols",
                "first entry only",
                "symbol count unproven",
            ] {
                assert!(text.contains(label), "{text}");
            }
        } else {
            assert!(text.contains("Budget"));
            assert!(!text.contains("Family:"));
        }
    }
    fs::write(&path, image(&[(4, u64::MAX), (-42, 0), (0, 0)])).unwrap();
    let out = exe()
        .arg("descriptors")
        .args(descriptor_args(&path, "0"))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("NoSupportedDescriptors")
    );
}
#[test]
fn syntax_acquisition_and_conflict_failures_are_distinct() {
    let f = Fixture::new();
    let path = f.file(&image(&[(5, 0x1400), (5, 0x1410), (10, 16), (0, 0)]));
    assert!(descriptors::parse(dynamic_args(&path, "5")).is_err());
    for value in ["-1", "18446744073709551616", "bad"] {
        assert!(descriptors::parse(descriptor_args(&path, value)).is_err());
    }
    let mut duplicate = descriptor_args(&path, "2");
    duplicate.extend(["--max-descriptors".into(), "1".into()]);
    assert!(descriptors::parse(duplicate).is_err());
    let out = exe()
        .arg("descriptors")
        .args(descriptor_args(&path, "2"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("DuplicateTag")
    );
    let out = exe()
        .arg("descriptors")
        .args(descriptor_args(&f.0.join("missing"), "2"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("Status: Failed"));
    assert!(!text.contains("Descriptor observation:"));
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_and_repeated_evidence_remain_immutable() {
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
    fs::write(&path, image(&[(5, 0x1400), (10, 16), (0, 0)])).unwrap();
    let req = descriptors::parse(descriptor_args(&path, "1")).unwrap();
    assert_eq!(req.acquisition.path, path);
    let mut s = Selection::new(req.acquisition);
    s.acquire();
    let a = descriptors::observe_acquired(&s, req.limits).unwrap();
    let b = descriptors::observe_acquired(&s, req.limits).unwrap();
    assert_eq!(descriptors::render(&a), descriptors::render(&b));
    assert_eq!(a.source().provenance(), b.source().provenance());
    let out = exe()
        .arg("descriptors")
        .args(descriptor_args(&path, "1"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
}
