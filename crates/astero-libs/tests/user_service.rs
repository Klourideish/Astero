use astero_abi::layouts::entry::CallFrame;
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_kernel::process::users::Users;
use astero_libs::user_service::exports::*;
use std::sync::{Arc, Mutex};
struct Memory {
    bytes: [u8; 256],
    ro: bool,
    fail: bool,
}
impl Default for Memory {
    fn default() -> Self {
        Self {
            bytes: [0x55; 256],
            ro: false,
            fail: false,
        }
    }
}
impl GuestMemory for Memory {
    fn validate(&self, a: u64, n: u64, w: bool) -> Result<(), AccessError> {
        if a.checked_add(n).is_none_or(|e| e > 256) || w && self.ro {
            Err(AccessError::Range)
        } else {
            Ok(())
        }
    }
    fn read(&self, a: u64, n: u64) -> Result<Vec<u8>, AccessError> {
        self.validate(a, n, false)?;
        Ok(self.bytes[a as usize..(a + n) as usize].to_vec())
    }
    fn write(&mut self, a: u64, b: &[u8]) -> Result<(), AccessError> {
        self.validate(a, b.len() as u64, true)?;
        if self.fail {
            return Err(AccessError::HostCopy {
                operation: "test",
                address: a,
                size: b.len() as u64,
                code: 299,
            });
        }
        self.bytes[a as usize..a as usize + b.len()].copy_from_slice(b);
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
fn setup() -> (Arc<Mutex<Users>>, PreparedRegistry, Memory) {
    let u = Arc::new(Mutex::new(Users::new(USER_ID)));
    let r = PreparedRegistry::new(registrations(u.clone()), 15).unwrap();
    (u, r, Memory::default())
}
fn call(r: &PreparedRegistry, m: &mut Memory, i: usize, a: [u64; 6]) -> (CallResult, u64) {
    let mut f = CallFrame {
        arguments: a,
        ..Default::default()
    };
    let result = r.invoke(&key(EXPORTS[i].1), &mut f, m).unwrap();
    (result, f.rax)
}
fn init(r: &PreparedRegistry, m: &mut Memory) {
    assert_eq!(call(r, m, 0, [0; 6]), (CallResult::Returned, 0));
}
#[test]
fn init_repeat_terminate_reinitialize() {
    let (u, r, mut m) = setup();
    init(&r, &mut m);
    init(&r, &mut m);
    assert_eq!(u.lock().unwrap().snapshot().pending_events, 1);
    assert_eq!(call(&r, &mut m, 14, [0; 6]).1, 0);
    assert!(!u.lock().unwrap().snapshot().initialized);
    init(&r, &mut m);
    assert_eq!(u.lock().unwrap().snapshot().generation, 2);
}
#[test]
fn before_init_and_bad_user_are_not_success() {
    let (_, r, mut m) = setup();
    assert_eq!(call(&r, &mut m, 1, [16, 0, 0, 0, 0, 0]).1, 0x80960002);
    init(&r, &mut m);
    assert_eq!(call(&r, &mut m, 11, [123, 16, 7, 0, 0, 0]).1, 0x80960005);
    assert_eq!(m.bytes, [0x55; 256]);
}
#[test]
fn initial_and_list_layout() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    call(&r, &mut m, 1, [16, 0, 0, 0, 0, 0]);
    assert_eq!(&m.bytes[16..20], &USER_ID.to_le_bytes());
    call(&r, &mut m, 10, [32, 0, 0, 0, 0, 0]);
    assert_eq!(
        &m.bytes[32..48],
        &[
            0, 0, 0, 16, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255
        ]
    );
}
#[test]
fn name_exact_capacity_and_small_refusal() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    assert_eq!(
        call(&r, &mut m, 11, [USER_ID as u64, 16, 6, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::Limit)
    );
    assert_eq!(m.bytes, [0x55; 256]);
    assert_eq!(call(&r, &mut m, 11, [USER_ID as u64, 16, 7, 0, 0, 0]).1, 0);
    assert_eq!(&m.bytes[16..23], b"Player\0");
    assert_eq!(m.bytes[23], 0x55);
}
#[test]
fn output_invalid_tail_is_unchanged() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    assert_eq!(
        call(&r, &mut m, 10, [250, 0, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::Range)
    );
    assert_eq!(m.bytes, [0x55; 256]);
}
#[test]
fn output_readonly_and_overflow_refuse() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    m.ro = true;
    assert_eq!(
        call(&r, &mut m, 1, [16, 0, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::Range)
    );
    m.ro = false;
    assert_eq!(
        call(&r, &mut m, 1, [u64::MAX, 0, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::Range)
    );
}
#[test]
fn events_are_bounded_ordered_and_do_not_relogin() {
    let (u, r, mut m) = setup();
    init(&r, &mut m);
    call(&r, &mut m, 13, [USER_ID as u64, 0, 0, 0, 0, 0]);
    call(&r, &mut m, 13, [USER_ID as u64, 0, 0, 0, 0, 0]);
    assert_eq!(u.lock().unwrap().snapshot().pending_events, 2);
    for kind in [0u8, 1] {
        assert_eq!(call(&r, &mut m, 12, [16, 0, 0, 0, 0, 0]).1, 0);
        assert_eq!(m.bytes[16], kind);
        assert!(!u.lock().unwrap().snapshot().logged_in);
    }
    assert_eq!(call(&r, &mut m, 12, [16, 0, 0, 0, 0, 0]).1, 0x80960007);
}
#[test]
fn failed_event_copy_is_retryable() {
    let (u, r, mut m) = setup();
    init(&r, &mut m);
    m.fail = true;
    assert!(matches!(
        call(&r, &mut m, 12, [16, 0, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::HostCopy { .. })
    ));
    assert_eq!(u.lock().unwrap().snapshot().pending_events, 1);
    m.fail = false;
    assert_eq!(call(&r, &mut m, 12, [16, 0, 0, 0, 0, 0]).1, 0);
    assert_eq!(u.lock().unwrap().snapshot().pending_events, 0);
}
#[test]
fn logout_changes_enumeration_not_initial_identity() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    call(&r, &mut m, 13, [USER_ID as u64, 0, 0, 0, 0, 0]);
    call(&r, &mut m, 10, [16, 0, 0, 0, 0, 0]);
    assert_eq!(&m.bytes[16..32], &[255; 16]);
    call(&r, &mut m, 1, [32, 0, 0, 0, 0, 0]);
    assert_eq!(&m.bytes[32..36], &USER_ID.to_le_bytes());
}
#[test]
fn default_setting_outputs() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    for (i, value) in [(2, 18i32), (4, 0), (5, 0), (6, 1), (7, 1), (8, 0), (9, 0)] {
        assert_eq!(call(&r, &mut m, i, [USER_ID as u64, 16, 0, 0, 0, 0]).1, 0);
        assert_eq!(&m.bytes[16..20], &value.to_le_bytes());
    }
}
#[test]
fn presets_never_overwrite_small_size() {
    let (_, r, mut m) = setup();
    init(&r, &mut m);
    for size in [0u64, 4, 8, 39] {
        m.bytes[16..24].copy_from_slice(&size.to_le_bytes());
        let before = m.bytes;
        assert_eq!(
            call(&r, &mut m, 3, [USER_ID as u64, 16, 0, 0, 0, 0]).0,
            CallResult::AccessFailure(AccessError::Limit)
        );
        assert_eq!(before, m.bytes);
    }
    m.bytes[16..24].copy_from_slice(&40u64.to_le_bytes());
    assert_eq!(call(&r, &mut m, 3, [USER_ID as u64, 16, 0, 0, 0, 0]).1, 0);
    assert_eq!(&m.bytes[24..56], &[0; 32]);
    assert_eq!(m.bytes[56], 0x55);
}
#[test]
fn initialize_options_are_checked_before_state() {
    let (u, r, mut m) = setup();
    assert_eq!(
        call(&r, &mut m, 0, [255, 0, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::Range)
    );
    assert!(!u.lock().unwrap().snapshot().initialized);
    m.bytes[16..20].copy_from_slice(&700i32.to_le_bytes());
    assert_eq!(call(&r, &mut m, 0, [16, 0, 0, 0, 0, 0]).1, 0);
}
#[test]
fn exact_identity_unknown_and_capacity() {
    let (u, r, mut m) = setup();
    let mut wrong = key(EXPORTS[0].1);
    wrong.module = b"other".to_vec();
    assert!(matches!(
        r.invoke(&wrong, &mut CallFrame::default(), &mut m),
        Err(RegistryError::Missing)
    ));
    assert!(r.find(&key(1)).is_err());
    assert!(matches!(
        PreparedRegistry::new(registrations(u), 14),
        Err(RegistryError::Capacity)
    ));
}
#[test]
fn concurrent_queries_share_only_process_state() {
    let (u, _, _) = setup();
    u.lock().unwrap().initialize();
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let u = u.clone();
            std::thread::spawn(move || {
                let r = PreparedRegistry::new(registrations(u), 15).unwrap();
                let mut m = Memory::default();
                for _ in 0..100 {
                    assert_eq!(call(&r, &mut m, 1, [16, 0, 0, 0, 0, 0]).1, 0);
                    assert_eq!(&m.bytes[16..20], &USER_ID.to_le_bytes());
                }
            })
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
}
#[test]
fn independent_runtime_and_drop_ownership() {
    let (u, r, _) = setup();
    let weak = Arc::downgrade(&u);
    u.lock().unwrap().initialize();
    let (other, _, _) = setup();
    assert!(!other.lock().unwrap().snapshot().initialized);
    drop(u);
    assert!(weak.upgrade().is_some());
    drop(r);
    assert!(weak.upgrade().is_none());
}
