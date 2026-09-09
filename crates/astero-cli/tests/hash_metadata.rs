use astero_cli::{
    acquisition::{Selection, State},
    hash_metadata,
};
use astero_core::input::hash_metadata::HashMetadataOutcome;
use astero_loader::elf::dynamic::{
    hash::synthetic::{Variant, image_with_hashes},
    synthetic::*,
};
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m20-input-fixtures");
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

fn hash_args(path: &std::path::Path, words: &str) -> Vec<OsString> {
    let mut a = dynamic_args(path, "16");
    a.extend(["--max-hash-words".into(), words.into()]);
    a
}
#[test]
fn earlier_commands_do_not_follow_a_bad_hash_pointer() {
    let f = Fixture::new();
    let path = f.file(&image(&[(4, u64::MAX), (0, 0)]));
    let mut descriptors = dynamic_args(&path, "2");
    descriptors.extend(["--max-descriptors".into(), "0".into()]);
    let mut strings = descriptors.clone();
    strings.extend([
        "--max-string-references".into(),
        "0".into(),
        "--max-scan-bytes-per-reference".into(),
        "0".into(),
        "--max-total-scan-bytes".into(),
        "0".into(),
    ]);
    for (mode, a) in [
        ("acquire", args(&path, "2").into_iter().take(6).collect()),
        ("inspect", args(&path, "2")),
        ("dynamic", dynamic_args(&path, "2")),
        ("descriptors", descriptors),
        ("string-references", strings),
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success(), "{:?}", out.stderr);
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("Hash metadata:")
        );
    }
    let req = hash_metadata::parse(hash_args(&path, "9")).unwrap();
    let mut s = Selection::new(req.acquisition);
    assert!(hash_metadata::observe_acquired(&s, req.limits).is_err());
    s.acquire();
    assert!(matches!(s.state(), State::Acquired(_)));
    assert!(matches!(
        hash_metadata::observe_acquired(&s, req.limits)
            .unwrap()
            .outcome(),
        HashMetadataOutcome::Failed(_)
    ));
    assert!(matches!(s.state(), State::Acquired(_)));
}
#[test]
fn live_count_refusal_absence_and_raw_proof_match_shared_model() {
    let f = Fixture::new();
    for (variant, words, expected, code) in [
        (Variant::SysV, "9", "Trusted symbol count: 3", 0),
        (Variant::Both, "18", "Trusted symbol count: 3", 0),
        (Variant::SysV, "8", "WorkLimit", 1),
        (Variant::None, "0", "Unavailable", 0),
        (Variant::LowerBound, "7", "lower-bound-only", 0),
        (Variant::Conflict, "100", "ConflictingEvidence", 1),
    ] {
        let path = f.file(&image_with_hashes(variant));
        let a = hash_args(&path, words);
        let req = hash_metadata::parse(a.clone()).unwrap();
        assert_eq!(req.limits.hash.max_words, words.parse::<u64>().unwrap());
        let mut selection = Selection::new(req.acquisition);
        selection.acquire();
        let report = hash_metadata::observe_acquired(&selection, req.limits).unwrap();
        assert!(hash_metadata::render(&report).contains(expected));
        let out = exe().arg("hash-metadata").args(a).output().unwrap();
        assert_eq!(out.status.code(), Some(code));
        let text = String::from_utf8(if code == 0 { out.stdout } else { out.stderr }).unwrap();
        assert!(text.contains(expected), "{text}");
        assert!(text.contains("Acquisition stage:"));
        assert!(text.contains(
            "No symbols enumerated. No names resolved. No linkage. No guest loaded or executed."
        ));
    }
}
#[test]
fn hash_limit_is_required_and_acquisition_errors_stop_before_observation() {
    let f = Fixture::new();
    let path = f.file(&image_with_hashes(Variant::SysV));
    assert!(hash_metadata::parse(dynamic_args(&path, "16")).is_err());
    let mut a = hash_args(&path, "9");
    a.extend(["--max-hash-words".into(), "10".into()]);
    assert!(hash_metadata::parse(a).is_err());
    let mut a = hash_args(&path, "9");
    a[3] = "1".into();
    for a in [a, hash_args(&f.0.join("missing"), "9")] {
        let out = exe().arg("hash-metadata").args(a).output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(
            !String::from_utf8(out.stderr)
                .unwrap()
                .contains("Hash observation explicitly")
        );
    }
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_is_preserved_through_hash_request() {
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
    fs::write(&path, image_with_hashes(Variant::SysV)).unwrap();
    let req = hash_metadata::parse(hash_args(&path, "9")).unwrap();
    assert_eq!(req.acquisition.path, path);
    let out = exe()
        .arg("hash-metadata")
        .args(hash_args(&path, "9"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("Trusted symbol count: 3")
    );
}
