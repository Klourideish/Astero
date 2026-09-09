use astero_cli::{
    acquisition::{Selection, State},
    symbols,
};
use astero_core::input::symbols::SymbolOutcome;
use astero_loader::elf::dynamic::symbol_table::synthetic::{Variant, image_with_symbols};
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m21-input-fixtures");
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
#[test]
fn prior_frontend_operations_do_not_observe_an_invalid_null_symbol() {
    let f = Fixture::new();
    let mut b = image_with_symbols(Variant::SysV);
    b[0x400] = 1;
    let path = f.file(&b);
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
    ] {
        let out = exe().arg(mode).args(a).output().unwrap();
        assert!(out.status.success(), "{:?}", out.stderr);
        assert!(
            !String::from_utf8(out.stdout)
                .unwrap()
                .contains("Symbols: Complete")
        );
    }
    let req = symbols::parse(symbol_args(&path, "3", "2", "6", "12")).unwrap();
    let mut s = Selection::new(req.acquisition);
    assert!(symbols::observe_acquired(&s, req.hash, req.symbols).is_err());
    s.acquire();
    let r = symbols::observe_acquired(&s, req.hash, req.symbols).unwrap();
    assert!(matches!(r.outcome(), SymbolOutcome::Failed(_)));
    assert!(matches!(s.state(), State::Acquired(_)));
}
#[test]
fn live_symbol_report_preserves_raw_names_and_explicit_refusals() {
    let f = Fixture::new();
    for (variant, count, lookups, scan, total, expected, code) in [
        (Variant::SysV, "3", "2", "6", "12", "Symbols: Complete", 0),
        (Variant::Raw, "3", "2", "6", "8", "<non-UTF8: FF>", 0),
        (Variant::Empty, "3", "2", "6", "7", "<empty>", 0),
        (Variant::Unnamed, "3", "1", "6", "6", "<unnamed>", 0),
        (Variant::SysV, "2", "2", "6", "12", "EntryBudget", 1),
        (Variant::SysV, "3", "1", "6", "12", "NameLookupBudget", 1),
        (Variant::SysV, "3", "2", "5", "12", "ScanLimit", 1),
        (
            Variant::LowerBound,
            "3",
            "2",
            "6",
            "12",
            "Symbols: Unavailable",
            0,
        ),
        (Variant::Unknown, "3", "2", "6", "12", "Unknown(15)", 0),
    ] {
        let path = f.file(&image_with_symbols(variant));
        let a = symbol_args(&path, count, lookups, scan, total);
        let req = symbols::parse(a.clone()).unwrap();
        assert_eq!(req.symbols.max_symbols, count.parse::<u64>().unwrap());
        assert_eq!(
            req.symbols.max_name_lookups,
            lookups.parse::<u64>().unwrap()
        );
        let mut selection = Selection::new(req.acquisition);
        selection.acquire();
        let r = symbols::observe_acquired(&selection, req.hash, req.symbols).unwrap();
        assert!(symbols::render(&r).contains(expected));
        let out = exe().arg("symbols").args(a).output().unwrap();
        assert_eq!(out.status.code(), Some(code));
        let text = String::from_utf8(if code == 0 { out.stdout } else { out.stderr }).unwrap();
        assert!(text.contains(expected), "{text}");
        assert!(text.contains("No import/export classification. No linkage or dependency resolution. No NID resolution. No guest loaded or executed."));
    }
}
#[test]
fn mandatory_limits_and_acquisition_failure_remain_distinct() {
    let f = Fixture::new();
    let path = f.file(&image_with_symbols(Variant::SysV));
    let a = symbol_args(&path, "3", "2", "6", "12");
    for at in (12..a.len()).step_by(2) {
        let mut short = a.clone();
        short.drain(at..at + 2);
        assert!(symbols::parse(short).is_err());
    }
    let mut duplicate = a.clone();
    duplicate.extend(["--max-symbols".into(), "3".into()]);
    assert!(symbols::parse(duplicate).is_err());
    let mut small = a;
    small[3] = "1".into();
    for args in [
        small,
        symbol_args(&f.0.join("missing"), "3", "2", "6", "12"),
    ] {
        let out = exe().arg("symbols").args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(
            !String::from_utf8(out.stderr)
                .unwrap()
                .contains("Symbol observation explicitly")
        );
    }
}
#[cfg(any(windows, unix))]
#[test]
fn native_path_is_preserved_through_symbol_request() {
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
    let req = symbols::parse(symbol_args(&path, "3", "2", "6", "8")).unwrap();
    assert_eq!(req.acquisition.path, path);
    let out = exe()
        .arg("symbols")
        .args(symbol_args(&path, "3", "2", "6", "8"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out.stderr);
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("<non-UTF8: FF>")
    );
}
