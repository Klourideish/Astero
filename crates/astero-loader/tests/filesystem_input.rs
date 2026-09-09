use astero_loader::artifact::{
    SourceArtifact,
    filesystem::{AcquisitionLimits, Failure, Operation, acquire},
};
use std::{fs, path::PathBuf};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/m14-input-fixtures");
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
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn limits(max_bytes: u64) -> AcquisitionLimits {
    AcquisitionLimits {
        max_bytes,
        max_read_calls: 8,
    }
}
fn bytes(s: &SourceArtifact) -> &[u8] {
    s.read(&s.checked_range(0, s.len()).unwrap()).unwrap()
}

#[test]
fn empty_and_arbitrary_files_stop_at_immutable_source_with_new_object_identity() {
    let f = Fixture::new();
    for data in [
        vec![],
        vec![0, 255, 42],
        b"not an ELF or a loadable target".to_vec(),
    ] {
        let path = f.0.join("input");
        fs::write(&path, &data).unwrap();
        let first = acquire(&path, limits(data.len() as u64)).unwrap();
        let second = acquire(&path, limits(data.len() as u64)).unwrap();
        assert_eq!(bytes(&first), data);
        assert_eq!(bytes(&first), bytes(&second));
        assert_ne!(first.identity(), second.identity());
        assert_eq!(first.clone().identity(), first.identity());
        assert_eq!(
            first.provenance(),
            Some(format!("filesystem input: {path:?}").as_str())
        );
        fs::write(&path, b"changed").unwrap();
        assert_eq!(bytes(&first), data);
    }
}
#[test]
fn over_limit_missing_invalid_and_directory_inputs_are_structured() {
    let f = Fixture::new();
    let path = f.0.join("bytes");
    fs::write(&path, [1, 2, 3]).unwrap();
    let e = acquire(&path, limits(2)).unwrap_err();
    assert_eq!(e.path, path);
    assert!(matches!(
        e.failure,
        Failure::SizeLimit {
            observed: 3,
            maximum: 2
        }
    ));
    let missing = f.0.join("missing");
    let e = acquire(&missing, limits(10)).unwrap_err();
    assert_eq!(e.path, missing);
    assert_eq!(e.operation, Operation::InspectPath);
    assert!(matches!(e.failure,Failure::Io(ref io) if io.kind()==std::io::ErrorKind::NotFound));
    assert!(matches!(
        acquire(&f.0, limits(10)).unwrap_err().failure,
        Failure::NotRegularFile
    ));
    assert!(matches!(
        acquire(f.0.join("bad\0path"), limits(10))
            .unwrap_err()
            .failure,
        Failure::Io(_)
    ));
}

#[cfg(windows)]
#[test]
fn exclusively_open_file_reports_access_failure_without_permission_changes() {
    use std::os::windows::fs::OpenOptionsExt;
    let f = Fixture::new();
    let path = f.0.join("locked");
    fs::write(&path, [1]).unwrap();
    let _lock = fs::OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    let e = acquire(&path, limits(8)).unwrap_err();
    assert_eq!(e.path, path);
    assert!(matches!(e.failure,Failure::Io(ref io) if io.raw_os_error().is_some()));
}

#[cfg(any(windows, unix))]
#[test]
fn native_non_utf8_filename_does_not_require_text_conversion() {
    use std::ffi::OsString;
    #[cfg(windows)]
    let name = {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[b'n' as u16, 0xd800])
    };
    #[cfg(unix)]
    let name = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![b'n', 255])
    };
    assert!(name.to_str().is_none());
    let f = Fixture::new();
    let path = f.0.join(name);
    fs::write(&path, [9, 8]).unwrap();
    let s = acquire(&path, limits(2)).unwrap();
    assert_eq!(bytes(&s), [9, 8]);
    assert_eq!(
        s.provenance(),
        Some(format!("filesystem input: {path:?}").as_str())
    );
}

#[cfg(unix)]
#[test]
fn final_symlink_is_not_acquired() {
    let f = Fixture::new();
    let target = f.0.join("target");
    fs::write(&target, [1]).unwrap();
    let link = f.0.join("link");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(matches!(
        acquire(&link, limits(8)).unwrap_err().failure,
        Failure::NotRegularFile
    ));
}
