//! Runs only Astero-owned synthetic assembly, never corpus input.
fn main() {
    #[cfg(all(windows, target_arch = "x86_64"))]
    {
        use astero_kernel::execution::host::{Bridge, SyntheticProbe};
        let mut bridge = Bridge::new().expect("bridge installation");
        for probe in [
            SyntheticProbe::Return,
            SyntheticProbe::Import,
            SyntheticProbe::IllegalInstruction,
            SyntheticProbe::AccessViolation,
            SyntheticProbe::FsRead,
        ] {
            println!("START {probe:?}");
            let r = bridge
                .synthetic(probe, &mut |_, c| {
                    c.rax = c.arguments[0] + c.arguments[1];
                    true
                })
                .expect("synthetic bridge");
            println!("RESULT {r:?}");
        }
        bridge.validate().expect("full validation");
        println!("SYNTHETIC BRIDGE VALIDATED; NO REAL GUEST ARTIFACT CODE EXECUTED");
    }
}
