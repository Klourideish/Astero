use astero_cli::{
    acquisition::{Selection, State},
    string_references,
};
use astero_core::input::string_references::StringReferenceOutcome;
use astero_loader::elf::dynamic::synthetic::*;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m19-input-fixtures");
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

fn string_args(path: &std::path::Path, references: &str, per: &str, total: &str) -> Vec<OsString> {
    let mut a = descriptor_args(path, "1");
    a[9] = "8".into();
    a.extend([
        "--max-string-references".into(),
        references.into(),
        "--max-scan-bytes-per-reference".into(),
        per.into(),
        "--max-total-scan-bytes".into(),
        total.into(),
    ]);
    a
}
fn bytes(payload: &[u8], refs: &[u64]) -> Vec<u8> {
    let mut e = vec![(5, 0x1400), (10, payload.len() as u64)];
    e.extend(refs.iter().map(|&v| (1, v)));
    e.push((0, 0));
    let mut b = image(&e);
    b[0x400..0x400 + payload.len()].copy_from_slice(payload);
    b
}
#[test]
fn earlier_frontend_commands_never_lookup_strings() {
    let f = Fixture::new();
    let path = f.file(&bytes(b"bad", &[0]));
    for (mode, a) in [
        ("acquire", args(&path, "2").into_iter().take(6).collect()),
        ("inspect", args(&path, "2")),
        ("dynamic", dynamic_args(&path, "4")),
        ("descriptors", descriptor_args(&path, "1")),
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success(), "{:?}", out.stderr);
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("String references:")
        );
    }
    let req = string_references::parse(string_args(&path, "1", "3", "3")).unwrap();
    let mut selection = Selection::new(req.acquisition);
    assert!(string_references::observe_acquired(&selection, req.limits).is_err());
    selection.acquire();
    let State::Acquired(source) = selection.state() else {
        panic!()
    };
    let id = source.identity();
    let r = string_references::observe_acquired(&selection, req.limits).unwrap();
    assert!(matches!(r.outcome(), StringReferenceOutcome::Failed(_)));
    assert_eq!(r.source().identity(), id);
    assert!(matches!(selection.state(),State::Acquired(s)if s.identity()==id));
}
#[test]
fn real_cli_preserves_utf8_empty_raw_duplicates_and_explicit_limits() {
    let f = Fixture::new();
    let path = f.file(&bytes(b"\0a\0\xff\0ignored", &[1, 3, 0, 1]));
    let a = string_args(&path, "4", "2", "7");
    let req = string_references::parse(a.clone()).unwrap();
    assert_eq!(req.acquisition.limits.max_bytes, 2048);
    assert_eq!(req.acquisition.limits.max_read_calls, 4);
    assert_eq!(req.limits.max_string_references, 4);
    assert_eq!(req.limits.strings.max_scan_bytes_per_reference, 2);
    assert_eq!(req.limits.strings.max_total_scan_bytes, 7);
    let out = exe().arg("string-references").args(a).output().unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    let text = String::from_utf8(out.stdout).unwrap();
    for label in [
        "References observed: 4",
        "value=\"a\"",
        "<non-UTF8: FF>",
        "<empty>",
        "No dependency resolution or linkage",
        "No guest loaded or executed",
    ] {
        assert!(text.contains(label), "{text}");
    }
    assert!(!text.contains("ignored"));
    let mut s = Selection::new(req.acquisition);
    s.acquire();
    let one = string_references::observe_acquired(&s, req.limits).unwrap();
    let two = string_references::observe_acquired(&s, req.limits).unwrap();
    assert_eq!(
        string_references::render(&one),
        string_references::render(&two)
    );
}
#[test]
fn required_limits_refusals_absence_and_acquisition_errors_are_distinct() {
    let f = Fixture::new();
    let path = f.file(&bytes(b"a\0", &[0, 0]));
    assert!(string_references::parse(descriptor_args(&path, "1")).is_err());
    for value in ["-1", "18446744073709551616", "bad"] {
        assert!(string_references::parse(string_args(&path, value, "2", "4")).is_err());
    }
    let mut duplicate = string_args(&path, "2", "2", "4");
    duplicate.extend(["--max-total-scan-bytes".into(), "4".into()]);
    assert!(string_references::parse(duplicate).is_err());
    for (refs, per, total, expected) in [
        ("1", "2", "4", "ReferenceBudget"),
        ("2", "1", "4", "ScanLimit"),
        ("2", "2", "3", "ScanLimit"),
    ] {
        let out = exe()
            .arg("string-references")
            .args(string_args(&path, refs, per, total))
            .output()
            .unwrap();
        assert!(!out.status.success());
        let text = String::from_utf8(out.stderr).unwrap();
        assert!(text.contains(expected));
        assert!(!text.contains("value=\"a\""));
    }
    fs::write(&path, bytes(b"no-terminator", &[])).unwrap();
    let out = exe()
        .arg("string-references")
        .args(string_args(&path, "0", "0", "0"))
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("Unavailable")
    );
    let out = exe()
        .arg("string-references")
        .args(string_args(&f.0.join("missing"), "1", "1", "1"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    let text = String::from_utf8(out.stderr).unwrap();
    assert!(text.contains("Status: Failed"));
    assert!(!text.contains("String references:"));
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_is_preserved_through_reference_request() {
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
    fs::write(&path, bytes(b"\0", &[0])).unwrap();
    let req = string_references::parse(string_args(&path, "1", "1", "1")).unwrap();
    assert_eq!(req.acquisition.path, path);
    let out = exe()
        .arg("string-references")
        .args(string_args(&path, "1", "1", "1"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    assert!(String::from_utf8(out.stdout).unwrap().contains("<empty>"));
}
