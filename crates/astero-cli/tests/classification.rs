use astero_cli::{
    acquisition::{Selection, State},
    classification,
};

use astero_loader::elf::dynamic::symbol_table::synthetic::{Variant, image_with_symbols};
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m22-input-fixtures");
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

fn symbol_args(
    path: &std::path::Path,
    count: &str,
    lookups: &str,
    scan: &str,
    total: &str,
) -> Vec<OsString> {
    let mut a = hash_args(path, "64");
    a.extend([
        "--max-descriptors".into(),
        "2".into(),
        "--max-symbols".into(),
        count.into(),
        "--max-name-lookups".into(),
        lookups.into(),
        "--max-name-scan-bytes".into(),
        scan.into(),
        "--max-total-name-scan-bytes".into(),
        total.into(),
    ]);
    a
}

fn classification_args(path: &std::path::Path, max: &str) -> Vec<OsString> {
    let mut a = symbol_args(path, "3", "2", "6", "12");
    a.extend(["--max-classifications".into(), max.into()]);
    a
}
#[test]
fn explicit_roles_and_refusals_use_the_same_acquired_source() {
    let f = Fixture::new();
    for (variant, max, expected, code) in [
        (Variant::SysV, "3", "DefinitionCandidate", 0),
        (Variant::Unknown, "3", "SpecialCandidate", 0),
        (Variant::Raw, "3", "<non-UTF8: FF>", 0),
        (Variant::Empty, "3", "<empty>", 0),
        (Variant::Unnamed, "3", "<unnamed>", 0),
        (Variant::SysV, "2", "Budget", 1),
        (Variant::SysV, "0", "Budget", 1),
        (Variant::BadName, "3", "PrerequisiteFailed", 1),
        (Variant::LowerBound, "3", "Classification: Unavailable", 0),
    ] {
        let mut b = image_with_symbols(variant);
        if matches!(variant, Variant::SysV) {
            b[0x434] = 0x12;
        }
        let path = f.file(&b);
        let a = classification_args(&path, max);
        let req = classification::parse(a.clone()).unwrap();
        assert_eq!(req.limits.max_classifications, max.parse::<u64>().unwrap());
        let mut selection = Selection::new(req.symbols.acquisition);
        assert!(
            classification::classify_acquired(
                &selection,
                req.symbols.hash,
                req.symbols.symbols,
                req.limits
            )
            .is_err()
        );
        selection.acquire();
        let report = classification::classify_acquired(
            &selection,
            req.symbols.hash,
            req.symbols.symbols,
            req.limits,
        )
        .unwrap();
        assert!(classification::render(&report).contains(expected));
        assert!(matches!(selection.state(), State::Acquired(_)));
        let out = exe().arg("classify-symbols").args(a).output().unwrap();
        assert_eq!(out.status.code(), Some(code));
        let text = String::from_utf8(if code == 0 { out.stdout } else { out.stderr }).unwrap();
        assert!(text.contains(expected), "{text}");
        assert!(text.contains("undefined is not an import; defined/global is not an export"));
    }
}
#[test]
fn previous_commands_do_not_classify_and_budget_is_mandatory() {
    let f = Fixture::new();
    let path = f.file(&image_with_symbols(Variant::SysV));
    let mut descriptors = dynamic_args(&path, "16");
    descriptors.extend(["--max-descriptors".into(), "2".into()]);
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
        ("dynamic", dynamic_args(&path, "16")),
        ("descriptors", descriptors),
        ("string-references", strings),
        ("hash-metadata", hash_args(&path, "64")),
        ("symbols", symbol_args(&path, "3", "2", "6", "12")),
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success());
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("Classification:")
        );
    }
    assert!(classification::parse(symbol_args(&path, "3", "2", "6", "12")).is_err());
    let mut a = classification_args(&path, "3");
    a.extend(["--max-classifications".into(), "3".into()]);
    assert!(classification::parse(a).is_err());
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_is_preserved_through_classification_request() {
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
    fs::write(&path, image_with_symbols(Variant::Raw)).unwrap();
    let req = classification::parse(classification_args(&path, "3")).unwrap();
    assert_eq!(req.symbols.acquisition.path, path);
    let out = exe()
        .arg("classify-symbols")
        .args(classification_args(&path, "3"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("<non-UTF8: FF>")
    );
}
