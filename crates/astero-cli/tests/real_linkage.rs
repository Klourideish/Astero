use astero_cli::{
    acquisition::{Selection, State},
    linkage_evidence,
};

use astero_loader::elf::dynamic::candidates::workload::synthetic::linkage_image;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m23-input-fixtures");
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

fn linkage_args(path: &std::path::Path, max: &str) -> Vec<OsString> {
    let mut a = symbol_args(path, "3", "4", "6", "24");
    a.extend(["--max-relocations".into(), max.into()]);
    a
}
#[test]
fn live_capability_output_preserves_counts_and_explicit_refusal() {
    let f = Fixture::new();
    let path = f.file(&linkage_image());
    for (max, code, label) in [
        ("4", 0, "external_candidates: 1"),
        ("3", 1, "EntryBudget"),
        ("0", 1, "EntryBudget"),
    ] {
        let a = linkage_args(&path, max);
        let req = linkage_evidence::parse(a.clone()).unwrap();
        assert_eq!(req.max_relocations, max.parse::<u64>().unwrap());
        let mut selection = Selection::new(req.symbols.acquisition);
        let limits = astero_core::input::linkage_evidence::LinkageLimits {
            hash: req.symbols.hash,
            symbols: req.symbols.symbols,
            max_relocations: req.max_relocations,
        };
        assert!(linkage_evidence::observe_acquired(&selection, limits).is_err());
        selection.acquire();
        let r = linkage_evidence::observe_acquired(&selection, limits).unwrap();
        assert!(linkage_evidence::render(&r).contains(label));
        assert!(matches!(selection.state(), State::Acquired(_)));
        let out = exe().arg("linkage-evidence").args(a).output().unwrap();
        assert_eq!(out.status.code(), Some(code));
        let text = String::from_utf8(if code == 0 { out.stdout } else { out.stderr }).unwrap();
        assert!(text.contains(label), "{text}");
        assert!(text.contains("All providers unresolved"));
    }
    assert!(linkage_evidence::parse(symbol_args(&path, "3", "4", "6", "24")).is_err());
}
#[test]
fn previous_modes_do_not_associate_malformed_relocation_symbols() {
    let f = Fixture::new();
    let mut bytes = linkage_image();
    bytes[0x564..0x568].copy_from_slice(&99u32.to_le_bytes());
    let path = f.file(&bytes);
    let mut classification = symbol_args(&path, "3", "4", "6", "24");
    classification.extend(["--max-classifications".into(), "3".into()]);
    for (mode, a) in [
        ("symbols", symbol_args(&path, "3", "4", "6", "24")),
        ("classify-symbols", classification),
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success());
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("ExternalReferenceCandidate")
        );
    }
    let out = exe()
        .arg("linkage-evidence")
        .args(linkage_args(&path, "4"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("SymbolIndex")
    );
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_is_preserved_through_linkage_evidence_request() {
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
    fs::write(&path, linkage_image()).unwrap();
    let req = linkage_evidence::parse(linkage_args(&path, "4")).unwrap();
    assert_eq!(req.symbols.acquisition.path, path);
    let out = exe()
        .arg("linkage-evidence")
        .args(linkage_args(&path, "4"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("ExternalReferenceCandidate")
    );
}
