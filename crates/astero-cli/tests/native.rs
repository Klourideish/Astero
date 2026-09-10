use astero_cli::native;
use astero_loader::elf::dynamic::identity::synthetic::identity_image;
#[cfg(all(windows, target_arch = "x86_64"))]
use std::process::Command;
use std::{ffi::OsString, fs, path::PathBuf};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m28-input-fixtures");
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
fn native_limits_required_and_path_preserved() {
    let f = Fixture::new();
    let path = f.file(&identity_image());
    let mut a = plan_args(&path);
    a.extend(["--max-mapped-bytes".into(), "65536".into()]);
    assert!(native::parse(a.clone()).is_err());
    a.extend(["--max-native-bytes".into(), "131072".into()]);
    let r = native::parse(a).unwrap();
    assert_eq!(r.max_native_bytes, 131072);
    assert_eq!(
        r.staging.plan.identity.linkage.symbols.acquisition.path,
        path
    );
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn native_cli_realizes_without_execution_and_refuses_budget() {
    use astero_loader::elf::dynamic::synthetic::{put32, put64};
    let f = Fixture::new();
    let mut b = identity_image();
    put32(&mut b, 68, 6);
    put64(&mut b, 104, 0xa00);
    for i in 0..4 {
        put64(&mut b, 0x540 + i * 24, 0x1800 + i as u64 * 8);
    }
    put64(&mut b, 0x580, 4);
    let path = f.file(&b);
    let mut a = plan_args(&path);
    let i = a.iter().position(|s| s == "--image-bias").unwrap();
    a[i + 1] = "38654705664".into();
    a.extend([
        "--max-mapped-bytes".into(),
        "65536".into(),
        "--max-native-bytes".into(),
        "65536".into(),
    ]);
    let o = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("native-map")
        .args(&a)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let text = String::from_utf8(o.stdout).unwrap();
    for s in [
        "NATIVE VM RESULT",
        "exact identity",
        "OS protections verified: true",
        "NO GUEST CODE EXECUTED",
        "Teardown: 0 native reservations",
    ] {
        assert!(text.contains(s), "{text}");
    }
    *a.last_mut().unwrap() = "1".into();
    let o = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("native-map")
        .args(a)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8(o.stderr).unwrap().contains("Budget"));
    assert_eq!(fs::read(path).unwrap(), b);
}

#[test]
fn entry_limits_are_explicit_and_native_path_is_preserved() {
    let f = Fixture::new();
    let path = f.file(&identity_image());
    let mut a = plan_args(&path);
    a.extend(
        [
            "--max-mapped-bytes",
            "65536",
            "--max-native-bytes",
            "131072",
        ]
        .map(OsString::from),
    );
    assert!(astero_cli::entry::parse(a.clone()).is_err());
    a.extend(
        [
            "--stack-base",
            "8589934592",
            "--stack-bytes",
            "8192",
            "--tls-base",
            "8858370048",
            "--max-runtime-bytes",
            "16384",
        ]
        .map(OsString::from),
    );
    let r = astero_cli::entry::parse(a.clone()).unwrap();
    assert_eq!(r.limits.stack_bytes, 8192);
    assert_eq!(
        r.native
            .staging
            .plan
            .identity
            .linkage
            .symbols
            .acquisition
            .path,
        path
    );
    a.extend(["--stack-bytes", "8192"].map(OsString::from));
    assert!(astero_cli::entry::parse(a).is_err());
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn entry_cli_refuses_runtime_budget_without_execution() {
    let f = Fixture::new();
    let mut b = identity_image();
    astero_loader::elf::dynamic::synthetic::put32(&mut b, 68, 6);
    use astero_loader::elf::dynamic::synthetic::put64;
    put64(&mut b, 104, 0xa00);
    for i in 0..4 {
        put64(&mut b, 0x540 + i * 24, 0x1800 + i as u64 * 8);
    }
    put64(&mut b, 0x580, 4);
    let path = f.file(&b);
    let mut a = plan_args(&path);
    a.extend(
        [
            "--max-mapped-bytes",
            "65536",
            "--max-native-bytes",
            "131072",
            "--stack-base",
            "8589934592",
            "--stack-bytes",
            "8192",
            "--tls-base",
            "8858370048",
            "--max-runtime-bytes",
            "1",
        ]
        .map(OsString::from),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("entry-readiness")
        .args(a)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let err = String::from_utf8(output.stderr).unwrap();
    assert!(err.contains("Budget"), "{err}");
}
#[test]
fn closure_opt_in_is_separate_and_duplicates_refuse() {
    let f = Fixture::new();
    let path = f.file(&identity_image());
    let mut a = plan_args(&path);
    a.extend(
        [
            "--max-mapped-bytes",
            "65536",
            "--max-native-bytes",
            "131072",
            "--stack-base",
            "8589934592",
            "--stack-bytes",
            "8192",
            "--tls-base",
            "8858370048",
            "--max-runtime-bytes",
            "20480",
        ]
        .map(OsString::from),
    );
    assert!(!astero_cli::entry::parse(a.clone()).unwrap().close_entry);
    a.push("--close-entry".into());
    assert!(astero_cli::entry::parse(a.clone()).unwrap().close_entry);
    a.push("--close-entry".into());
    assert!(astero_cli::entry::parse(a).is_err());
}
#[cfg(all(windows, target_arch = "x86_64"))]
#[test]
fn closure_cli_issues_synthetic_ready_capability_without_executing_fixture() {
    let f = Fixture::new();
    let mut b = identity_image();
    use astero_loader::elf::dynamic::synthetic::{put32, put64};
    put32(&mut b, 68, 5);
    put64(&mut b, 24, 0x1100);
    put64(&mut b, 0x668, 0);
    put64(&mut b, 0x698, 0);
    let path = f.file(&b);
    let mut a = plan_args(&path);
    a.extend(
        [
            "--max-mapped-bytes",
            "65536",
            "--max-native-bytes",
            "131072",
            "--stack-base",
            "8589934592",
            "--stack-bytes",
            "8192",
            "--tls-base",
            "8858370048",
            "--max-runtime-bytes",
            "20480",
            "--close-entry",
        ]
        .map(OsString::from),
    );
    let o = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("entry-readiness")
        .args(&a)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let t = String::from_utf8(o.stdout).unwrap();
    assert!(t.contains("EntryReadyGuest capability issued"), "{t}");
    assert!(t.contains("NO REAL GUEST ARTIFACT CODE EXECUTED"));
    assert_eq!(fs::read(path).unwrap(), b);
}

#[test]
fn first_entry_requires_explicit_execution_limits_before_input() {
    let o = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("first-entry")
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(!String::from_utf8_lossy(&o.stdout).contains("REAL GUEST CODE WILL EXECUTE"));
}
#[test]
fn first_entry_refuses_oversized_and_duplicate_deadlines() {
    for a in [
        vec!["--wall-ms", "501", "--containment-ms", "1000"],
        vec![
            "--wall-ms",
            "1",
            "--wall-ms",
            "2",
            "--containment-ms",
            "1000",
        ],
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
            .arg("first-entry")
            .args(a)
            .output()
            .unwrap();
        assert!(!o.status.success());
        assert!(!String::from_utf8_lossy(&o.stdout).contains("REAL GUEST CODE WILL EXECUTE"));
    }
}
