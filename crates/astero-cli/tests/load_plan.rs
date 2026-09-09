use astero_cli::{
    acquisition::{Selection, State},
    load_plan,
};
use astero_loader::elf::dynamic::identity::synthetic::identity_image;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m26-input-fixtures");
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

fn plan_args(path: &std::path::Path) -> Vec<OsString> {
    let mut a = args(path, "16");
    a.extend(
        [
            "--image-bias",
            "65536",
            "--max-providers",
            "1",
            "--max-plan-records",
            "128",
        ]
        .map(OsString::from),
    );
    a
}
#[test]
fn explicit_command_reports_blockers_and_never_loads() {
    let f = Fixture::new();
    let bytes = identity_image();
    let path = f.file(&bytes);
    let output = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("load-plan")
        .args(plan_args(&path))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let s = String::from_utf8(output.stderr).unwrap();
    for expected in [
        "PLAN ONLY",
        "Readiness: Blocked",
        "NO GUEST MEMORY MUTATED",
        "NO RELOCATIONS APPLIED",
        "NO GUEST EXECUTION",
    ] {
        assert!(s.contains(expected), "{s}");
    }
    assert_eq!(fs::read(path).unwrap(), bytes);
}
#[test]
fn provider_capacity_is_checked_before_any_file_acquisition() {
    let mut a = plan_args(std::path::Path::new("nonexistent"));
    a.extend(["--provider", "a", "--provider", "b"].map(OsString::from));
    let r = load_plan::parse(a).unwrap();
    assert!(format!("{:?}", load_plan::execute(&r)).contains("ProviderBudget"));
}
#[test]
fn acquisition_and_native_path_remain_independent() {
    let f = Fixture::new();
    let path = f.file(b"not ELF");
    let r = load_plan::parse(plan_args(&path)).unwrap();
    assert_eq!(r.identity.linkage.symbols.acquisition.path, path);
    let mut s = Selection::new(r.identity.linkage.symbols.acquisition.clone());
    s.acquire();
    assert!(matches!(s.state(), State::Acquired(_)));
    let p = load_plan::execute(&r).unwrap();
    assert_eq!(
        p.readiness(),
        astero_core::input::load_plan::Readiness::Blocked
    );
}
#[cfg(unix)]
#[test]
fn non_utf8_path_is_preserved() {
    use std::os::unix::ffi::OsStringExt;
    let p = PathBuf::from(OsString::from_vec(vec![0xff]));
    let r = load_plan::parse(plan_args(&p)).unwrap();
    assert_eq!(r.identity.linkage.symbols.acquisition.path, p);
}
