use astero_hle::{calls::metrics::Metrics, dispatch::prepared::*};
fn key() -> ProviderKey {
    ProviderKey {
        nid: 7,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    }
}
#[test]
fn providers_preserve_exact_identity_and_outcomes() {
    let m = Metrics::new(vec![Some(key())]).unwrap();
    for r in [
        Ok(CallResult::Returned),
        Err(RegistryError::Missing),
        Ok(CallResult::Unsupported),
    ] {
        m.begin(0, 1);
        m.complete(0, r);
    }
    let s = m.snapshot();
    let e = &s.entries[0];
    assert_eq!((e.calls, e.returned, e.unknown, e.refused), (3, 1, 1, 1));
    assert_eq!(e.key, Some(key()));
}
#[test]
fn concurrent_calls_are_not_lost() {
    let m = std::sync::Arc::new(Metrics::new(vec![Some(key())]).unwrap());
    std::thread::scope(|s| {
        for i in 0..4 {
            let m = m.clone();
            s.spawn(move || {
                for _ in 0..1000 {
                    m.begin(0, i);
                    m.complete(0, Ok(CallResult::Returned));
                }
            });
        }
    });
    assert_eq!(m.snapshot().entries[0].returned, 4000);
}
#[test]
fn invalid_and_capacity_are_explicit() {
    let m = Metrics::new(vec![]).unwrap();
    m.begin(10, 1);
    assert_eq!(m.snapshot().invalid_ordinals, 1);
    assert!(Metrics::new(vec![None; 65537]).is_err());
}

#[test]
fn last_provider_pair_and_completion_counts_remain_consistent() {
    let m = std::sync::Arc::new(Metrics::new(vec![Some(key()), Some(key())]).unwrap());
    std::thread::scope(|s| {
        for i in 0..2 {
            let m = m.clone();
            s.spawn(move || {
                for _ in 0..10000 {
                    m.begin(i, i as u64 + 1);
                    m.complete(i, Ok(CallResult::Returned));
                }
            });
        }
        for _ in 0..1000 {
            let x = m.snapshot();
            for e in x.entries {
                assert!(e.returned + e.unknown + e.refused <= e.calls);
            }
            if let Some(i) = x.last_ordinal {
                assert_eq!(x.last_thread, Some(i as u64 + 1));
            }
        }
    });
}
