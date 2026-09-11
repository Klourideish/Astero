use astero_abi::layouts::entry::CallFrame;
use astero_audio::output::service::{AudioService, Config, Kind};
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_libs::audio::exports::*;
use astero_timing::scheduler::{Config as TC, TimingEngine};
use std::sync::Arc;
struct Memory {
    b: Vec<u8>,
    ro: bool,
}
impl Memory {
    fn new() -> Self {
        Self {
            b: vec![0; 2 * 1024 * 1024],
            ro: false,
        }
    }
    fn put(&mut self, a: usize, b: &[u8]) {
        self.b[a..a + b.len()].copy_from_slice(b)
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, w: bool) -> Result<(), AccessError> {
        if a.checked_add(n).is_none_or(|e| e > self.b.len() as u64) || w && self.ro {
            Err(AccessError::Range)
        } else {
            Ok(())
        }
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.validate(a, n, false)?;
        Ok(self.b[a as usize..(a + n) as usize].to_vec())
    }
    fn read_window(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.read(a, n.min(4096 - a % 4096))
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        self.validate(a, b.len() as u64, true)?;
        self.put(a as usize, b);
        Ok(())
    }
    fn allocate(&mut self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::Allocation)
    }
    fn free(&mut self, _: u64) -> Result<(), AccessError> {
        Err(AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, _: u64) -> Result<u64, AccessError> {
        Err(AccessError::InvalidAllocation)
    }
}

fn setup() -> (TimingEngine, Arc<AudioService>, PreparedRegistry, Memory) {
    let e = TimingEngine::real(TC {
        max_pending: 64,
        max_snapshot_entries: 64,
    })
    .unwrap();
    let s = Arc::new(AudioService::new(e.scheduler()));
    let r = PreparedRegistry::new(registrations(s.clone()), 25).unwrap();
    (e, s, r, Memory::new())
}
fn call(
    r: &PreparedRegistry,
    m: &mut Memory,
    two: bool,
    name: &str,
    a: [u64; 6],
) -> (CallResult, u64) {
    let n = if two { AUDIO2 } else { LEGACY }
        .iter()
        .find(|(s, _)| *s == name)
        .unwrap()
        .1;
    let mut f = CallFrame {
        arguments: a,
        ..Default::default()
    };
    (r.invoke(&key(n, two), &mut f, m).unwrap(), f.rax)
}
fn open(r: &PreparedRegistry, m: &mut Memory) -> u64 {
    let (c, p) = call(r, m, false, "Open", [0, 0, 0, 480, 48000, 1]);
    assert_eq!(c, CallResult::Returned);
    p
}
#[test]
fn exact_provider_family_keys() {
    let (_e, _s, r, _m) = setup();
    assert!(r.find(&key(AUDIO2[0].1, false)).is_err());
    assert!(r.find(&key(AUDIO2[0].1, true)).is_ok());
    assert_eq!(LEGACY.len() + AUDIO2.len(), 25);
}
#[test]
fn checked_legacy_output_and_null_submission() {
    let (_e, s, r, mut m) = setup();
    let p = open(&r, &mut m);
    assert_eq!(
        call(&r, &mut m, false, "Output", [p, 256, 0, 0, 0, 0]).1,
        480
    );
    assert_eq!(s.snapshot().bytes, 1920);
    assert_eq!(call(&r, &mut m, false, "Output", [p, 0, 0, 0, 0, 0]).1, 0);
    assert_eq!(s.snapshot().submitted, 1);
}
#[test]
fn invalid_buffer_prevents_admission() {
    let (_e, s, r, mut m) = setup();
    let p = open(&r, &mut m);
    assert!(matches!(
        call(&r, &mut m, false, "Output", [p, 2097150, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().submitted, 0);
}
#[test]
fn batch_guest_tail_preflight() {
    let (_e, s, r, mut m) = setup();
    let a = open(&r, &mut m);
    let b = open(&r, &mut m);
    m.put(64, &(a as u32).to_le_bytes());
    m.put(72, &256u64.to_le_bytes());
    m.put(80, &(b as u32).to_le_bytes());
    m.put(88, &2097150u64.to_le_bytes());
    assert!(matches!(
        call(&r, &mut m, false, "Outputs", [64, 2, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(s.snapshot().submitted, 0);
}
#[test]
fn state_and_volume_bytes() {
    let (_e, _s, r, mut m) = setup();
    let p = open(&r, &mut m);
    m.put(256, &12i32.to_le_bytes());
    assert_eq!(
        call(&r, &mut m, false, "SetVolume", [p, 1, 256, 0, 0, 0]).1,
        0
    );
    assert_eq!(
        call(&r, &mut m, false, "GetPortState", [p, 512, 0, 0, 0, 0]).1,
        0
    );
    assert_eq!(&m.b[512..515], &[1, 0, 2]);
    assert_eq!(&m.b[516..518], &12i16.to_le_bytes());
}
#[test]
fn reset_query_create_owned_context() {
    let (_e, s, r, mut m) = setup();
    assert_eq!(call(&r, &mut m, true, "Initialize", [0; 6]).1, 0);
    call(&r, &mut m, true, "ContextResetParam", [64, 0, 0, 0, 0, 0]);
    assert_eq!(&m.b[80..84], &512u32.to_le_bytes());
    call(
        &r,
        &mut m,
        true,
        "ContextQueryMemory",
        [64, 160, 0, 0, 0, 0],
    );
    assert_eq!(&m.b[160..168], &0x100000u64.to_le_bytes());
    assert_eq!(
        call(
            &r,
            &mut m,
            true,
            "ContextCreate",
            [64, 4096, 0x100000, 192, 0, 0]
        )
        .1,
        0
    );
    let id = u64::from_le_bytes(m.b[192..200].try_into().unwrap());
    assert_eq!(s.record(id, Kind::Context).unwrap().config.depth, 4);
}
#[test]
fn failed_publication_does_not_create() {
    let (_e, s, r, mut m) = setup();
    m.ro = true;
    assert!(matches!(
        call(&r, &mut m, true, "UserCreate", [0, 64, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
    assert!(s.snapshot().objects.is_empty());
}
#[test]
fn nonempty_unknown_attributes_are_not_success() {
    let (_e, s, r, mut m) = setup();
    let id = s
        .create(Kind::Context, None, Config::context(480, 1).unwrap())
        .unwrap();
    assert_eq!(
        call(
            &r,
            &mut m,
            true,
            "ContextSetAttributes",
            [id, 256, 1, 0, 0, 0]
        )
        .0,
        CallResult::Unsupported
    );
    assert_eq!(
        call(
            &r,
            &mut m,
            true,
            "ContextSetAttributes",
            [id, 0, 0, 0, 0, 0]
        )
        .1,
        0
    );
}
#[test]
fn queue_output_pair_preflight() {
    let (_e, s, r, mut m) = setup();
    let id = s
        .create(Kind::Context, None, Config::context(480, 1).unwrap())
        .unwrap();
    m.put(64, &99u32.to_le_bytes());
    assert!(matches!(
        call(
            &r,
            &mut m,
            true,
            "ContextGetQueueLevel",
            [id, 64, 2097151, 0, 0, 0]
        )
        .0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(&m.b[64..68], &99u32.to_le_bytes());
}
#[test]
fn audio2_port_parent_and_system_state() {
    let (_e, s, r, mut m) = setup();
    let id = s
        .create(Kind::Context, None, Config::context(480, 1).unwrap())
        .unwrap();
    m.put(264, &48000u32.to_le_bytes());
    assert_eq!(
        call(&r, &mut m, true, "PortCreate", [id, 256, 64, 0, 0, 0]).1,
        0
    );
    let p = u64::from_le_bytes(m.b[64..72].try_into().unwrap());
    assert_eq!(
        call(&r, &mut m, true, "PortGetState", [p, 512, 0, 0, 0, 0]).1,
        0
    );
    assert_eq!(&m.b[512..515], &[1, 0, 2]);
    m.b[512..576].fill(255);
    call(&r, &mut m, true, "GetSystemState", [512, 0, 0, 0, 0, 0]);
    assert!(m.b[512..576].iter().all(|x| *x == 0));
}
