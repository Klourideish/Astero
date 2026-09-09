use astero_cli::staging;
use astero_loader::elf::dynamic::identity::synthetic::identity_image;
use std::{ffi::OsString, fs, path::PathBuf, process::Command};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m27-input-fixtures");
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
fn explicit_stage_command_and_budget_refusal_leave_no_mappings() {
    use astero_loader::elf::dynamic::synthetic::{put32, put64};
    let f = Fixture::new();
    let mut bytes = identity_image();
    put64(&mut bytes, 24, 0x1100);
    put32(&mut bytes, 68, 7);
    put64(&mut bytes, 104, 0xa00);
    for i in 0..4 {
        put64(&mut bytes, 0x540 + i * 24, 0x1800 + i as u64 * 8);
    }
    put64(&mut bytes, 0x580, 4);
    let path = f.file(&bytes);
    let mut a = plan_args(&path);
    a.extend(["--max-mapped-bytes".into(), "2560".into()]);
    let output = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("stage-image")
        .args(&a)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    for s in [
        "STAGING RESULT",
        "StagedWithPendingWork",
        "NO GUEST CODE EXECUTED",
        "Teardown: 0",
        "Ready for execution: false",
        "MetadataOnly",
    ] {
        assert!(text.contains(s), "{text}");
    }
    *a.last_mut().unwrap() = "2559".into();
    let o = Command::new(env!("CARGO_BIN_EXE_astero-cli"))
        .arg("stage-image")
        .args(a)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8(o.stderr).unwrap().contains("Budget"));
    assert_eq!(fs::read(path).unwrap(), bytes);
}
#[test]
fn staging_limit_is_explicit_and_native_path_survives_selection() {
    let f = Fixture::new();
    let path = f.file(&identity_image());
    assert!(staging::parse(plan_args(&path)).is_err());
    let mut a = plan_args(&path);
    a.extend(["--max-mapped-bytes".into(), "0".into()]);
    let r = staging::parse(a).unwrap();
    assert_eq!(r.plan.identity.linkage.symbols.acquisition.path, path);
    assert_eq!(r.limits.max_mapped_bytes, 0);
}
