use astero_hle::{dispatch::prepared::*, providers::modules::*};
use std::sync::Arc;
fn key(id: u16) -> ProviderKey {
    ProviderKey {
        nid: id as u64,
        library: b"test".to_vec(),
        module: b"test".to_vec(),
    }
}
fn declare(m: &Modules, id: u16, deps: Vec<u16>) {
    m.declare(
        Declaration {
            id,
            name: format!("module{id}"),
            backing: Backing::Hle,
            providers: vec![key(id)],
            dependencies: deps,
        },
        &[key(id)],
    )
    .unwrap();
}
#[test]
fn load_refcount_query_unload() {
    let m = Modules::new(8);
    declare(&m, 1, vec![]);
    assert!(!m.is_loaded(1));
    m.load(1).unwrap();
    m.load(1).unwrap();
    assert_eq!(m.snapshot().records[0].references, 2);
    m.unload(1).unwrap();
    assert!(m.is_loaded(1));
    m.unload(1).unwrap();
    assert!(!m.is_loaded(1));
    assert!(m.unload(1).is_err());
}
#[test]
fn unknown_never_loaded() {
    let m = Modules::new(2);
    assert_eq!(m.load(267), Err(Error::Unknown));
    assert_eq!(m.snapshot().failed, 1);
    assert!(!m.is_loaded(267));
}
#[test]
fn missing_provider_refused() {
    let m = Modules::new(2);
    assert_eq!(
        m.declare(
            Declaration {
                id: 1,
                name: "x".into(),
                backing: Backing::Hle,
                providers: vec![key(1)],
                dependencies: vec![]
            },
            &[]
        ),
        Err(Error::MissingProvider)
    );
}
#[test]
fn conflicting_identity_refused() {
    let m = Modules::new(2);
    declare(&m, 1, vec![]);
    assert_eq!(
        m.declare(m.snapshot().records[0].declaration.clone(), &[key(1)]),
        Err(Error::Conflict)
    );
}
#[test]
fn dependencies_and_busy_unload() {
    let m = Modules::new(3);
    declare(&m, 1, vec![]);
    declare(&m, 2, vec![1]);
    m.load(2).unwrap();
    assert!(m.is_loaded(1));
    assert_eq!(m.unload(1), Err(Error::Busy));
    m.unload(2).unwrap();
    assert!(!m.is_loaded(1));
}
#[test]
fn missing_dependency_rolls_back() {
    let m = Modules::new(3);
    declare(&m, 1, vec![2]);
    assert!(m.load(1).is_err());
    assert_eq!(m.snapshot().records[0].references, 0);
}
#[test]
fn dependency_cycle_refused() {
    let m = Modules::new(3);
    declare(&m, 1, vec![2]);
    declare(&m, 2, vec![1]);
    assert_eq!(m.load(1), Err(Error::Dependency));
}
#[test]
fn artifact_and_partial_do_not_fake_loading() {
    for backing in [
        Backing::Artifact {
            source: "configured-source".into(),
        },
        Backing::Partial,
    ] {
        let m = Modules::new(2);
        m.declare(
            Declaration {
                id: 1,
                name: "x".into(),
                backing,
                providers: vec![key(1)],
                dependencies: vec![],
            },
            &[],
        )
        .unwrap();
        assert_eq!(m.load(1), Err(Error::Unsupported));
        assert!(!m.is_loaded(1));
    }
}
#[test]
fn execution_lease_guards_unload_and_shutdown() {
    let m = Arc::new(Modules::new(2));
    declare(&m, 1, vec![]);
    m.load(1).unwrap();
    let l = m.lease(1).unwrap();
    assert_eq!(m.unload(1), Err(Error::Busy));
    assert_eq!(m.shutdown(), Err(Error::Busy));
    drop(l);
    m.unload(1).unwrap();
    m.shutdown().unwrap();
    assert!(m.snapshot().records.is_empty());
    assert_eq!(m.load(1), Err(Error::Stopped));
}
#[test]
fn bounded_declarations() {
    let m = Modules::new(1);
    declare(&m, 1, vec![]);
    assert_eq!(
        m.declare(
            Declaration {
                id: 2,
                name: "x".into(),
                backing: Backing::Hle,
                providers: vec![key(2)],
                dependencies: vec![]
            },
            &[key(2)]
        ),
        Err(Error::Capacity)
    );
}
#[test]
fn published_handler_obeys_load_lifetime() {
    let m = Arc::new(Modules::new(1));
    declare(&m, 1, vec![]);
    let r = m
        .publish(
            1,
            Registration {
                key: key(1),
                kind: ProviderKind::HleImplementation,
                handler: Some(Box::new(|f, _| {
                    f.rax = 7;
                    CallResult::Returned
                })),
            },
        )
        .unwrap();
    let h = r.handler.unwrap();
    let mut f = astero_abi::layouts::entry::CallFrame::default();
    let mut mem = astero_hle::calls::memory::Unavailable;
    assert_eq!(h(&mut f, &mut mem), CallResult::Unsupported);
    m.load(1).unwrap();
    assert_eq!(h(&mut f, &mut mem), CallResult::Returned);
    assert_eq!(f.rax, 7);
    m.unload(1).unwrap();
    assert_eq!(h(&mut f, &mut mem), CallResult::Unsupported);
}
#[test]
fn concurrent_repeated_loads_share_one_record() {
    let m = Arc::new(Modules::new(1));
    declare(&m, 1, vec![]);
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let m = m.clone();
            std::thread::spawn(move || m.load(1).unwrap())
        })
        .collect();
    for t in threads {
        t.join().unwrap();
    }
    assert_eq!(m.snapshot().records.len(), 1);
    assert_eq!(m.snapshot().records[0].references, 8);
}
