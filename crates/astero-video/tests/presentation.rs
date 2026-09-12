use astero_video::presentation::*;
#[test]
fn valid_pattern() {
    let f = synthetic(1);
    assert_eq!(f.validate(), Ok(()));
    assert_eq!((f.width, f.height, f.stride), (640, 360, 2560));
}
#[test]
fn invalid_dimensions_stride_backing() {
    let mut f = synthetic(1);
    f.width = 0;
    assert_eq!(f.validate(), Err(Error::Dimensions));
    f.width = 640;
    f.stride = 1;
    assert_eq!(f.validate(), Err(Error::Stride));
    f.stride = 2564;
    assert_eq!(f.validate(), Err(Error::Backing));
}
#[test]
fn headless_replacement_and_checksum() {
    let h = Headless::default();
    h.present(synthetic(1)).unwrap();
    h.drain().unwrap();
    let first = h.snapshot();
    h.present(synthetic(2)).unwrap();
    h.drain().unwrap();
    let second = h.snapshot();
    assert_eq!(second.presented, 2);
    assert_eq!(second.bytes, 640 * 360 * 4);
    assert_ne!(first.checksum, second.checksum);
    assert_eq!(second.state, State::Active);
}
#[test]
fn mailbox_latest_wins() {
    let h = Headless::default();
    for id in 1..10 {
        h.present(synthetic(id)).unwrap();
    }
    h.drain().unwrap();
    let s = h.snapshot();
    assert_eq!(s.dropped, 8);
    assert_eq!(s.presented, 1);
    assert_eq!(s.last_presented_id, Some(9));
}
#[test]
fn rejects_nonmonotonic_ids() {
    let h = Headless::default();
    h.present(synthetic(2)).unwrap();
    assert_eq!(h.present(synthetic(1)), Err(Error::Order));
    assert_eq!(h.present(synthetic(2)), Err(Error::Order));
    assert_eq!(h.snapshot().refused, 2);
}
#[test]
fn teardown_releases_pending() {
    let endpoint;
    {
        let h = Headless::default();
        endpoint = h.endpoint();
        h.present(synthetic(1)).unwrap();
    }
    assert_eq!(endpoint.snapshot().state, State::Closed);
    assert!(endpoint.take().is_none());
    assert_eq!(endpoint.present(synthetic(2)), Err(Error::Closed));
}
#[test]
fn producer_thread_mailbox() {
    let h = Headless::default();
    let e = h.endpoint();
    std::thread::spawn(move || e.present(synthetic(1)).unwrap())
        .join()
        .unwrap();
    assert!(h.drain().unwrap());
    assert!(!h.drain().unwrap());
}
#[test]
fn host_resource_retains_owner_but_headless_refuses() {
    struct R;
    impl HostResource for R {
        fn identity(&self) -> u64 {
            42
        }
    }
    let r = std::sync::Arc::new(R);
    let mut f = synthetic(1);
    f.backing = Backing::Host(r.clone());
    let h = Headless::default();
    h.present(f).unwrap();
    assert_eq!(std::sync::Arc::strong_count(&r), 2);
    assert_eq!(h.drain(), Err(Error::Unsupported));
    assert_eq!(std::sync::Arc::strong_count(&r), 1);
    assert_eq!(h.snapshot().state, State::Failed);
}
#[test]
fn bgra_is_explicit_host_format() {
    let mut f = synthetic(1);
    f.format = Format::Bgra8;
    let h = Headless::default();
    h.present(f).unwrap();
    h.drain().unwrap();
    assert_eq!(h.snapshot().format, Some(Format::Bgra8));
}

#[test]
fn one_inflight_receipt_and_terminal_completion() {
    let e = Endpoint::new("test");
    e.present(synthetic(1)).unwrap();
    let f = e.take().unwrap();
    e.present(synthetic(2)).unwrap();
    assert!(e.take().is_none());
    e.completed(&f);
    e.completed(&f);
    assert_eq!(e.snapshot().presented, 1);
    let g = e.take().unwrap();
    e.close();
    e.completed(&g);
    assert_eq!(e.snapshot().presented, 1);
    assert_eq!(e.snapshot().dropped, 1);
    assert_eq!(e.snapshot().state, State::Closed);
}

#[test]
fn discard_receipt_unblocks_pending_without_duplicate_drop() {
    let e = Endpoint::new("test");
    e.present(synthetic(1)).unwrap();
    let f = e.take().unwrap();
    e.present(synthetic(2)).unwrap();
    e.discard(&f);
    e.discard(&f);
    assert_eq!(e.snapshot().dropped, 1);
    let g = e.take().unwrap();
    e.completed(&g);
    assert_eq!(e.snapshot().last_presented_id, Some(2));
}
