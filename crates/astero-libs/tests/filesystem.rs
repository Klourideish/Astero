use astero_abi::layouts::entry::CallFrame;
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_kernel::{
    filesystem::{mounts::Mounts, service::Filesystem},
    process::output::Output,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Memory {
    b: Vec<u8>,
    next: u64,
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, _: bool) -> Result<(), AccessError> {
        if a > 0 && a.checked_add(n).is_some_and(|e| e <= self.b.len() as u64) {
            Ok(())
        } else {
            Err(AccessError::Range)
        }
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.validate(a, n, false)?;
        Ok(self.b[a as usize..(a + n) as usize].to_vec())
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        self.validate(a, b.len() as u64, true)?;
        self.b[a as usize..a as usize + b.len()].copy_from_slice(b);
        Ok(())
    }
    fn allocate(&mut self, n: u64) -> Result<u64, AccessError> {
        let p = self.next;
        self.validate(p, n, true)?;
        self.next += n;
        Ok(p)
    }
    fn free(&mut self, _: u64) -> Result<(), AccessError> {
        Ok(())
    }
    fn allocation_size(&self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::InvalidAllocation)
    }
}
struct Fixture {
    root: std::path::PathBuf,
    m: Memory,
    r: Vec<Registration>,
    fs: Arc<Filesystem>,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "astero-adapter-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("file"), b"abcdef").unwrap();
        let fs = Arc::new(Filesystem::new(32, Arc::new(Mutex::new(Output::new(1024)))));
        fs.configure(Mounts {
            code: root.clone(),
            data: root.clone(),
            writable: Some(root.clone()),
        })
        .unwrap();
        let r = astero_libs::filesystem::registrations(fs.clone(), 8);
        let mut m = Memory {
            b: vec![0; 4096],
            next: 2048,
        };
        m.write(32, b"/app0/file\0").unwrap();
        m.write(64, b"r\0").unwrap();
        Self { root, m, r, fs }
    }
    fn call(&mut self, name: &str, a: [u64; 6]) -> (CallResult, u64) {
        let i = astero_libs::filesystem::EXPORTS
            .iter()
            .position(|x| x.0 == name)
            .unwrap();
        let mut f = CallFrame {
            arguments: a,
            ..Default::default()
        };
        let result = self.r[i].handler.as_ref().unwrap()(&mut f, &mut self.m);
        (result, f.rax)
    }
    fn open(&mut self) -> u64 {
        self.call("fopen", [32, 64, 0, 0, 0, 0]).1
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.fs.shutdown();
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}
#[test]
fn stdio_lifecycle() {
    let mut t = Fixture::new();
    let p = t.open();
    assert_ne!(p, 0);
    assert_eq!(t.call("fread", [128, 2, 3, p, 0, 0]).1, 3);
    assert_eq!(&t.m.b[128..134], b"abcdef");
    assert_eq!(t.call("ftell", [p, 0, 0, 0, 0, 0]).1, 6);
    assert_eq!(t.call("fclose", [p, 0, 0, 0, 0, 0]).1, 0);
    assert_eq!(t.fs.snapshot().streams, 0);
}
#[test]
fn invalid_tail_preflight() {
    let mut t = Fixture::new();
    let p = t.open();
    assert!(matches!(
        t.call("fread", [4090, 1, 20, p, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(t.call("ftell", [p, 0, 0, 0, 0, 0]).1, 0);
}
#[test]
fn stat_exact_record() {
    let mut t = Fixture::new();
    t.m.b[128..249].fill(0xaa);
    assert_eq!(t.call("stat", [32, 128, 0, 0, 0, 0]).1, 0);
    assert_eq!(u64::from_le_bytes(t.m.b[200..208].try_into().unwrap()), 6);
    assert_eq!(t.m.b[248], 0xaa);
}
#[test]
fn errno_and_sce_distinct() {
    let mut t = Fixture::new();
    assert_eq!(t.call("close", [999, 0, 0, 0, 0, 0]).1, u64::MAX);
    assert_eq!(t.m.b[8], 9);
    assert_eq!(
        t.call("sceKernelClose", [999, 0, 0, 0, 0, 0]).1,
        0x80020009u32 as i32 as i64 as u64
    );
}
#[test]
fn eof_rewind() {
    let mut t = Fixture::new();
    let p = t.open();
    t.call("fread", [128, 1, 8, p, 0, 0]);
    assert_eq!(t.call("feof", [p, 0, 0, 0, 0, 0]).1, 1);
    t.call("rewind", [p, 0, 0, 0, 0, 0]);
    assert_eq!(t.call("feof", [p, 0, 0, 0, 0, 0]).1, 0);
}
#[test]
fn invalid_mode_no_open() {
    let mut t = Fixture::new();
    t.m.write(64, b"r++\0").unwrap();
    assert_eq!(t.open(), 0);
    assert_eq!(t.fs.snapshot().open, 0);
}
#[test]
fn stream_error_distinct_from_eof() {
    let mut t = Fixture::new();
    let p = t.open();
    t.call("fwrite", [32, 1, 1, p, 0, 0]);
    assert_eq!(t.call("ferror", [p, 0, 0, 0, 0, 0]).1, 1);
    assert_eq!(t.call("feof", [p, 0, 0, 0, 0, 0]).1, 0);
}
