//! Astero-owned infinite-loop proof only. No artifact is accepted.
fn main() {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        use astero_kernel::execution::host::{Bridge, SyntheticProbe};
        let worker = std::thread::Builder::new()
            .name("synthetic-guest".into())
            .spawn(|| {
                let mut bridge = Bridge::new().unwrap();
                bridge.validate().unwrap();
                let result = bridge
                    .supervised_synthetic(SyntheticProbe::InfiniteLoop, 50, &mut |_, _| false)
                    .unwrap();
                assert_eq!(result.reason, 4);
                assert!(result.host_fs_restored && result.host_gs_preserved);
                let s = result.supervision.as_ref().unwrap();
                assert!(s.redirected && Bridge::synthetic_loop_contains(s.rip));
                assert_eq!(s.suspends, s.resumes);
                result
            })
            .unwrap();
        let result = worker.join().unwrap();
        println!("SYNTHETIC LOOP STOPPED: {result:?}; guest thread joined");
        let _next = Bridge::new().unwrap();
        println!("Adapter released; no real guest artifact code executed");
    }
}
