use astero_abi::layouts::entry::CallFrame;
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
use astero_kernel::threading::thread::{attributes::*, lifecycle::*};
use astero_timing::{
    scheduler::{Config, TimingEngine},
    time::{Deadline, Span},
};
use std::sync::{Arc, Mutex};
struct Memory([u8; 128]);
impl GuestMemory for Memory {
    fn read(&self, a: u64, n: u64) -> std::result::Result<Vec<u8>, AccessError> {
        self.0
            .get(a as usize..a.checked_add(n).ok_or(AccessError::Range)? as usize)
            .map(|s| s.to_vec())
            .ok_or(AccessError::Range)
    }
    fn write(&mut self, a: u64, b: &[u8]) -> std::result::Result<(), AccessError> {
        self.0
            .get_mut(a as usize..a as usize + b.len())
            .ok_or(AccessError::Range)?
            .copy_from_slice(b);
        Ok(())
    }
    fn allocate(&mut self, _: u64) -> std::result::Result<u64, AccessError> {
        Err(AccessError::Allocation)
    }
    fn free(&mut self, _: u64) -> std::result::Result<(), AccessError> {
        Err(AccessError::InvalidAllocation)
    }
    fn allocation_size(&self, _: u64) -> std::result::Result<u64, AccessError> {
        Err(AccessError::InvalidAllocation)
    }
}

struct RefuseSpawn;
impl astero_libs::pthread::thread::exports::Spawner for RefuseSpawn {
    fn create(
        &self,
        _: Attributes,
        _: u64,
        _: u64,
        _: Vec<u8>,
        _: u64,
        _: &mut dyn GuestMemory,
    ) -> Result {
        Err(Error::Unsupported)
    }
}
fn setup() -> (
    TimingEngine,
    PreparedRegistry,
    Memory,
    Arc<Mutex<Option<u64>>>,
) {
    let e = TimingEngine::real(Config {
        max_pending: 8,
        max_snapshot_entries: 8,
    })
    .unwrap();
    let table = Arc::new(ThreadTable::new(e.scheduler(), 4));
    table.arm(
        Deadline::after(
            e.scheduler().now().unwrap(),
            Span::from_millis(500).unwrap(),
        )
        .unwrap(),
    );
    let exit = Arc::new(Mutex::new(None));
    let r = PreparedRegistry::new(
        astero_libs::pthread::thread::exports::registrations(
            Arc::new(AttributeTable::new(8)),
            table,
            Arc::new(RefuseSpawn),
            Thread(1),
            exit.clone(),
        ),
        256,
    )
    .unwrap();
    (e, r, Memory([0; 128]), exit)
}
fn call(
    r: &PreparedRegistry,
    m: &mut Memory,
    operation: &str,
    sce: bool,
    args: [u64; 6],
) -> (u64, CallResult) {
    let x = astero_libs::pthread::thread::exports::EXPORTS
        .iter()
        .find(|x| x.operation == operation && x.sce == sce)
        .unwrap();
    let mut frame = CallFrame {
        arguments: args,
        ..Default::default()
    };
    let k = ProviderKey {
        nid: x.nid,
        library: b"libkernel".to_vec(),
        module: b"libkernel".to_vec(),
    };
    let result = r.invoke(&k, &mut frame, m).unwrap();
    (frame.rax, result)
}
#[test]
fn lifecycle_attributes_sce_posix_aliases_share_exact_object() {
    let (_e, r, mut m, _) = setup();
    assert_eq!(call(&r, &mut m, "ATTR_INIT", true, [8, 0, 0, 0, 0, 0]).0, 0);
    assert_eq!(
        call(
            &r,
            &mut m,
            "ATTR_SETSTACKSIZE",
            false,
            [8, 0x4000, 0, 0, 0, 0]
        )
        .0,
        0
    );
    assert_eq!(
        call(&r, &mut m, "ATTR_GETSTACKSIZE", true, [8, 16, 0, 0, 0, 0]).0,
        0
    );
    assert_eq!(m.read(16, 8).unwrap(), 0x4000u64.to_le_bytes());
    assert_eq!(
        call(&r, &mut m, "ATTR_DESTROY", false, [8, 0, 0, 0, 0, 0]).0,
        0
    );
    assert_eq!(
        call(&r, &mut m, "ATTR_GETSTACKSIZE", true, [8, 16, 0, 0, 0, 0]).0,
        0x80020016
    );
    assert_eq!(
        call(&r, &mut m, "ATTR_GETSTACKSIZE", false, [8, 16, 0, 0, 0, 0]).0,
        22
    );
}
#[test]
fn lifecycle_copied_attr_token_and_unsupported_policy_refuse() {
    let (_e, r, mut m, _) = setup();
    call(&r, &mut m, "ATTR_INIT", true, [8, 0, 0, 0, 0, 0]);
    m.write(32, &123u64.to_le_bytes()).unwrap();
    assert_eq!(
        call(&r, &mut m, "ATTR_GETSTACK", true, [8, 32, 127, 0, 0, 0]).0,
        0x80020016
    );
    assert_eq!(m.read(32, 8).unwrap(), 123u64.to_le_bytes());
    let v = m.read(8, 8).unwrap();
    m.write(16, &v).unwrap();
    assert_eq!(
        call(&r, &mut m, "ATTR_DESTROY", true, [16, 0, 0, 0, 0, 0]).0,
        0x80020016
    );
    assert_eq!(
        call(
            &r,
            &mut m,
            "ATTR_SETSTACK",
            false,
            [8, 0x8000, 0x4000, 0, 0, 0]
        )
        .0,
        45
    );
    assert_eq!(
        call(
            &r,
            &mut m,
            "ATTR_SETSCHEDPOLICY",
            false,
            [8, 99, 0, 0, 0, 0]
        )
        .0,
        45
    );
}
#[test]
fn self_equal_and_fixed_32_byte_name_contract() {
    let (_e, r, mut m, _) = setup();
    assert_eq!(call(&r, &mut m, "SELF", true, [0; 6]).0, 1);
    assert_eq!(call(&r, &mut m, "EQUAL", false, [1, 1, 0, 0, 0, 0]).0, 1);
    m.write(8, b"worker\0").unwrap();
    assert_eq!(call(&r, &mut m, "RENAME", true, [1, 8, 0, 0, 0, 0]).0, 0);
    assert_eq!(call(&r, &mut m, "GETNAME", true, [1, 32, 0, 0, 0, 0]).0, 0);
    assert_eq!(&m.0[32..39], b"worker\0");
    assert_eq!(
        call(&r, &mut m, "GETNAME", true, [1, 127, 0, 0, 0, 0]).0,
        14
    );
}
#[test]
fn pthread_exit_is_a_controlled_value_not_host_unwind() {
    let (_e, r, mut m, exit) = setup();
    let (v, result) = call(&r, &mut m, "EXIT", true, [0x1234, 0, 0, 0, 0, 0]);
    assert_eq!(v, 0x1234);
    assert_eq!(result, CallResult::StopRequested);
    assert_eq!(*exit.lock().unwrap(), Some(0x1234));
}
#[test]
fn lifecycle_dispatch_never_uses_nid_only_fallback() {
    let (_e, r, mut m, _) = setup();
    let mut f = CallFrame::default();
    assert_eq!(
        r.invoke(
            &ProviderKey {
                nid: 0x9ec628351cb0c0d8,
                library: b"other".to_vec(),
                module: b"libkernel".to_vec()
            },
            &mut f,
            &mut m
        ),
        Err(RegistryError::Missing)
    );
    assert_eq!(
        call(&r, &mut m, "ATTR_INIT", true, [127, 0, 0, 0, 0, 0]).0,
        0x80020016
    );
    assert_eq!(call(&r, &mut m, "ATTR_INIT", true, [8, 0, 0, 0, 0, 0]).0, 0);
}
