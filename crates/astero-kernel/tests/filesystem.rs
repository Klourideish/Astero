use astero_kernel::{
    filesystem::{mounts::Mounts, service::*},
    process::output::Output,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
static NEXT: AtomicU64 = AtomicU64::new(1);
struct Fixture {
    root: PathBuf,
    fs: Arc<Filesystem>,
    output: Arc<Mutex<Output>>,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "astero-fs-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("file"), b"abcdef").unwrap();
        let output = Arc::new(Mutex::new(Output::new(1024)));
        let fs = Arc::new(Filesystem::new(32, output.clone()));
        fs.configure(Mounts {
            code: root.clone(),
            data: root.clone(),
            writable: Some(root.clone()),
        })
        .unwrap();
        Self { root, fs, output }
    }
    fn open(&self) -> u32 {
        self.fs
            .open("/app0/file", Mode::parse("r").unwrap())
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.fs.shutdown();
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn open_read_close() {
    let t = Fixture::new();
    let fd = t.open();
    let mut b = [0; 6];
    assert_eq!(t.fs.read(fd, &mut b, None), Ok(6));
    assert_eq!(&b, b"abcdef");
    t.fs.close(fd).unwrap();
    assert_eq!(t.fs.read(fd, &mut b, None), Err(Error::BadDescriptor));
}
#[test]
fn write_reopen() {
    let t = Fixture::new();
    let fd = t.fs.open("/data/new", Mode::parse("w").unwrap()).unwrap();
    t.fs.write(fd, b"hello", None).unwrap();
    t.fs.close(fd).unwrap();
    assert_eq!(std::fs::read(t.root.join("new")).unwrap(), b"hello");
}
#[test]
fn pread_preserves_cursor() {
    let t = Fixture::new();
    let fd = t.open();
    let mut b = [0; 2];
    t.fs.read(fd, &mut b, Some(3)).unwrap();
    assert_eq!(&b, b"de");
    assert_eq!(t.fs.seek(fd, 0, 1), Ok(0));
}
#[test]
fn pwrite_preserves_cursor() {
    let t = Fixture::new();
    let fd = t.fs.open("/data/file", Mode::parse("r+").unwrap()).unwrap();
    t.fs.write(fd, b"XY", Some(2)).unwrap();
    assert_eq!(t.fs.seek(fd, 0, 1), Ok(0));
    assert_eq!(std::fs::read(t.root.join("file")).unwrap(), b"abXYef");
}
#[test]
fn eof_and_seek() {
    let t = Fixture::new();
    let fd = t.open();
    let mut b = [0; 8];
    t.fs.read(fd, &mut b, None).unwrap();
    assert_eq!(t.fs.indicators(fd), Ok((true, false)));
    t.fs.seek(fd, 0, 0).unwrap();
    assert_eq!(t.fs.indicators(fd), Ok((false, false)));
}
#[test]
fn modes() {
    for m in ["r", "rb", "r+b", "rb+", "w", "w+", "a", "ab+"] {
        assert!(Mode::parse(m).is_ok());
    }
    for m in ["", "rw", "r++", "rbb", "x", "rZ"] {
        assert_eq!(Mode::parse(m), Err(Error::Invalid));
    }
}
#[test]
fn read_only_app0() {
    let t = Fixture::new();
    assert_eq!(
        t.fs.open("/app0/file", Mode::parse("w").unwrap()),
        Err(Error::Permission)
    );
    assert_eq!(std::fs::read(t.root.join("file")).unwrap(), b"abcdef");
}
#[test]
fn traversal_and_devices() {
    let t = Fixture::new();
    for p in [
        "/app0/../../evil",
        "C:/evil",
        "/app0/CON",
        "/app0/NUL.txt",
        "/app0/a:stream",
        "/app0/a\\b",
        "/system/file",
    ] {
        assert!(t.fs.open(p, Mode::parse("r").unwrap()).is_err(), "{p}");
    }
}
#[test]
fn stale_descriptor_not_reused() {
    let t = Fixture::new();
    let fd = t.open();
    t.fs.close(fd).unwrap();
    assert!(t.open() > fd);
    assert_eq!(t.fs.close(fd), Err(Error::BadDescriptor));
}
#[test]
fn streams_share_descriptor() {
    let t = Fixture::new();
    let fd = t.open();
    t.fs.bind_stream(0x1000, fd).unwrap();
    assert_eq!(t.fs.stream(0x1000), Ok(fd));
    assert_eq!(t.fs.bind_stream(0x1000, fd), Err(Error::Invalid));
    t.fs.close(fd).unwrap();
    assert_eq!(t.fs.stream(0x1000), Err(Error::BadDescriptor));
}
#[test]
fn standard_output_capture() {
    let t = Fixture::new();
    t.fs.write(1, b"out", None).unwrap();
    t.fs.write(2, b"err", None).unwrap();
    assert_eq!(t.output.lock().unwrap().bytes(), b"outerr");
    assert_eq!(t.fs.read(0, &mut [0; 4], None), Ok(0));
}
#[test]
fn capacity() {
    let t = Fixture::new();
    for _ in 0..29 {
        t.open();
    }
    assert_eq!(
        t.fs.open("file", Mode::parse("r").unwrap()),
        Err(Error::Capacity)
    );
}
#[test]
fn shutdown_closes() {
    let t = Fixture::new();
    t.open();
    t.fs.shutdown();
    assert_eq!(t.fs.snapshot().open, 0);
    assert_eq!(
        t.fs.open("file", Mode::parse("r").unwrap()),
        Err(Error::Stopped)
    );
}
#[test]
fn concurrent_positioned_reads() {
    let t = Fixture::new();
    let fd = t.open();
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let fs = t.fs.clone();
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let mut b = [0; 2];
                    assert_eq!(fs.read(fd, &mut b, Some(2)), Ok(2));
                    assert_eq!(&b, b"cd");
                }
            })
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
    assert_eq!(t.fs.seek(fd, 0, 1), Ok(0));
}
#[test]
fn truncate() {
    let t = Fixture::new();
    let fd = t.fs.open("/data/file", Mode::parse("r+").unwrap()).unwrap();
    t.fs.truncate(fd, 2).unwrap();
    assert_eq!(t.fs.fmetadata(fd).unwrap().len(), 2);
}
#[test]
fn overlay_and_explicit_root() {
    let t = Fixture::new();
    let code = t.root.join("code");
    std::fs::create_dir_all(code.join("sce_module")).unwrap();
    std::fs::create_dir_all(t.root.join("sce_sys")).unwrap();
    std::fs::write(code.join("file"), b"code").unwrap();
    let m = Mounts::detect(&code.join("eboot.bin"), None).unwrap();
    assert_eq!(
        std::fs::read(m.resolve("/app0/file", false).unwrap()).unwrap(),
        b"code"
    );
    std::fs::write(t.root.join("asset"), b"data").unwrap();
    assert_eq!(
        std::fs::read(m.resolve("/app0/asset", false).unwrap()).unwrap(),
        b"data"
    );
    let m = Mounts::detect(&code.join("eboot.bin"), Some(t.root.clone())).unwrap();
    assert_eq!(
        std::fs::read(m.resolve("/app0/file", false).unwrap()).unwrap(),
        b"abcdef"
    );
}
#[test]
fn append_and_positioned_refusal() {
    let t = Fixture::new();
    let fd = t.fs.open("/data/file", Mode::parse("a").unwrap()).unwrap();
    t.fs.write(fd, b"g", None).unwrap();
    assert_eq!(t.fs.write(fd, b"X", Some(0)), Err(Error::Invalid));
    assert_eq!(std::fs::read(t.root.join("file")).unwrap(), b"abcdefg");
}
#[test]
fn wrong_mode() {
    let t = Fixture::new();
    assert_eq!(
        t.fs.write(t.open(), b"bad", None),
        Err(Error::BadDescriptor)
    );
}
#[test]
fn metadata_counts() {
    let t = Fixture::new();
    assert_eq!(t.fs.metadata("/app0/file").unwrap().len(), 6);
    assert_eq!(t.fs.snapshot().stats, 1);
    assert_eq!(t.fs.snapshot().last_path, "/app0/file");
}
