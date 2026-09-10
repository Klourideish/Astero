//! M29 legacy-correlated process-entry value contract; not Windows host ABI.
//! M31 consumes this exact context policy; see first_native_entry.md for its narrow runtime evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InitialContext {
    pub rip: u64,
    pub rsp: u64,
    pub rflags: u64,
    /// RAX,RBX,RCX,RDX,RSI,RDI,RBP,R8,R9,R10,R11,R12,R13,R14,R15.
    pub gpr: [u64; 15],
    pub xmm: [[u8; 16]; 16],
    pub mxcsr: u32,
    pub x87_control: u16,
    pub fs_base: u64,
    /// Windows TEB uses GS; never replace it with a guest pointer.
    pub preserve_host_gs: bool,
}
impl InitialContext {
    pub fn planned(rip: u64, rsp: u64, params: u64, fs_base: u64) -> Self {
        let mut gpr = [0; 15];
        gpr[5] = params;
        Self {
            rip,
            rsp,
            rflags: 0x202,
            gpr,
            xmm: [[0; 16]; 16],
            mxcsr: 0x1f80,
            x87_control: 0x37f,
            fs_base,
            preserve_host_gs: true,
        }
    }
}
/// argc=1, padding=0, argv[0], argv terminator, empty env terminator.
/// Byte encoding is explicit; no unsafe host-struct serialization.
pub fn process_arguments(argv0: u64) -> [u8; 32] {
    let mut out = [0; 32];
    out[..4].copy_from_slice(&1u32.to_le_bytes());
    out[8..16].copy_from_slice(&argv0.to_le_bytes());
    out
}
/// Captured scalar and vector lanes for provider contract tests and the native import boundary.
/// Native assembly/Windows shadow space and preserved register saves are separate.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CallFrame {
    pub arguments: [u64; 6],
    pub stack_arguments: [u64; 6],
    pub xmm: [[u8; 16]; 8],
    pub rax: u64,
    pub xmm0: [u8; 16],
}
