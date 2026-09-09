use astero_cli::{
    acquisition::{Selection, State},
    linkage_evidence, ps5_identity,
};
use astero_loader::elf::dynamic::identity::synthetic::identity_image;
use std::{ffi::OsString, fs, path::PathBuf, process::Command, sync::Arc};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m24-input-fixtures");
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

fn args(path: &std::path::Path, max: &str) -> Vec<OsString> {
    let mut a: Vec<OsString> = vec!["--path".into(), path.as_os_str().into()];
    for (k, v) in [
        ("--max-bytes", "4096"),
        ("--max-read-calls", "4"),
        ("--max-program-headers", "2"),
        ("--max-dynamic-entries", "32"),
        ("--max-hash-words", "64"),
        ("--max-descriptors", "2"),
        ("--max-symbols", "3"),
        ("--max-name-lookups", "16"),
        ("--max-name-scan-bytes", "64"),
        ("--max-total-name-scan-bytes", "256"),
        ("--max-relocations", "4"),
        ("--max-identity-records", max),
    ] {
        a.extend([k.into(), v.into()]);
    }
    a
}
#[test]
fn explicit_cli_complete_and_refusal_preserve_stop_boundary() {
    let f = Fixture::new();
    let path = f.file(&identity_image());
    for (max, code, label) in [
        ("6", 0, "Encoded numeric NIDs: 2"),
        ("5", 1, "Budget { count: 6, maximum: 5 }"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
            .arg("ps5-identity")
            .args(args(&path, max))
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(code));
        let text = format!(
            "{}{}",
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
        assert!(text.contains(label), "{text}");
        assert!(text.contains("No providers resolved"));
        assert!(text.contains("no HLE binding"));
        assert!(text.contains("guest loading or execution"));
    }
    assert_eq!(fs::read(path).unwrap(), identity_image());
}
#[test]
fn native_selection_delegates_through_core_without_creating_session_state() {
    let f = Fixture::new();
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
    let path = f.0.join(name);
    fs::write(&path, identity_image()).unwrap();
    let r = ps5_identity::parse(args(&path, "6")).unwrap();
    assert_eq!(r.max_identity_records, 6);
    let lower = r.linkage;
    let mut selected = Selection::new(lower.symbols.acquisition);
    let limits = astero_core::input::linkage_evidence::LinkageLimits {
        hash: lower.symbols.hash,
        symbols: lower.symbols.symbols,
        max_relocations: lower.max_relocations,
    };
    assert!(linkage_evidence::observe_acquired(&selected, limits).is_err());
    selected.acquire();
    assert!(matches!(selected.state(), State::Acquired(_)));
    let linkage = Arc::new(linkage_evidence::observe_acquired(&selected, limits).unwrap());
    let report = astero_core::input::ps5_identity::observe(
        linkage.clone(),
        astero_core::input::ps5_identity::IdentityLimits {
            max_identity_records: 6,
        },
    );
    assert!(Arc::ptr_eq(report.linkage(), &linkage));
    let text = ps5_identity::render(&report);
    assert!(text.contains("experimental hypothesis"));
    assert!(text.contains("0x4694092552938853"));
}
