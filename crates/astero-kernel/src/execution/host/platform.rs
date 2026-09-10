//! M30 sole native-execution unsafe leaf. See native_entry_closure.md.
//! A process-exclusive, thread-affine adapter owns its VEH and TLS slot. Public calls
//! run only statically linked Astero probes, never caller pointers or artifact bytes.
use astero_abi::layouts::entry::CallFrame;
use astero_memory::mapping::GuestAddress;
use astero_memory::mapping::windows_native::{
    self, GuestRange, NativeLimits, NativeRegion, Protection,
};
use std::{
    arch::global_asm,
    ffi::c_void,
    marker::PhantomData,
    mem::offset_of,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
static OWNED: AtomicBool = AtomicBool::new(false);
static SLOT_OFFSET: AtomicUsize = AtomicUsize::new(0);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    Busy,
    HostFeature,
    Os(&'static str),
    Memory(windows_native::NativeError),
    Validation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntheticProbe {
    Return,
    Import,
    ImportOrdinal(u32),
    IllegalInstruction,
    AccessViolation,
    FsRead,
    GuardRead,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeExit {
    pub reason: u64,
    pub value: u64,
    pub rip: u64,
    pub rsp: u64,
    pub exception: u32,
    pub import_ordinal: u32,
    pub fault_address: u64,
    pub host_fs_restored: bool,
    pub host_gs_preserved: bool,
    pub registers: [u64; 16],
}
#[repr(C, align(16))]
struct Frame<'a> {
    host_rsp: u64,
    guest_rsp: u64,
    target: u64,
    fs: u64,
    old_fs: u64,
    active: u64,
    reason: u64,
    result: u64,
    fault_rip: u64,
    fault_rsp: u64,
    code: u64,
    address: u64,
    args: [u64; 6],
    xmm: [[u8; 16]; 8],
    out_xmm: [u8; 16],
    saved: [u64; 6],
    ranges: &'a [(u64, u64)],
    ordinal: u64,
    stack_start: u64,
    stack_end: u64,
    fault_registers: [u64; 16],
    callback: &'a mut dyn FnMut(u32, &mut CallFrame) -> bool,
}
#[repr(C)]
struct ExceptionRecord {
    code: u32,
    flags: u32,
    nested: *mut c_void,
    address: *mut c_void,
    count: u32,
    info: [usize; 15],
}
#[repr(C)]
struct ContextPrefix {
    home: [u64; 6],
    flags: u32,
    mxcsr: u32,
    segments: [u16; 6],
    eflags: u32,
    debug: [u64; 6],
    regs: [u64; 16],
    rip: u64,
}
#[repr(C)]
struct ExceptionPointers {
    record: *mut ExceptionRecord,
    context: *mut ContextPrefix,
}
const _: () =
    assert!(offset_of!(ContextPrefix, regs) == 120 && offset_of!(ContextPrefix, rip) == 248);
#[link(name = "kernel32")]
unsafe extern "system" {
    fn TlsAlloc() -> u32;
    fn TlsFree(slot: u32) -> i32;
    fn TlsSetValue(slot: u32, value: *mut c_void) -> i32;
    fn AddVectoredExceptionHandler(
        first: u32,
        handler: unsafe extern "system" fn(*mut ExceptionPointers) -> i32,
    ) -> *mut c_void;
    fn RemoveVectoredExceptionHandler(handle: *mut c_void) -> u32;
    fn IsProcessorFeaturePresent(feature: u32) -> i32;
}
unsafe extern "system" {
    fn checked_enter(frame: *mut c_void) -> u64;
    fn landing();
    fn import_landing();
    fn veh(info: *mut ExceptionPointers) -> i32;
    fn probe_return();
    fn probe_import();
    fn probe_illegal();
    fn probe_av();
    fn probe_fs();
    fn probe_guard();
    fn probes_end();
    fn read_fs() -> u64;
    fn read_gs() -> u64;
}
/// Owns process-global Windows adapter plumbing, not a global guest/runtime.
/// A second adapter refuses explicitly. Drop removes the handler before freeing TLS.
pub struct Bridge {
    slot: u32,
    handler: *mut c_void,
    validated: bool,
    ranges: Vec<(u64, u64)>,
    _thread: PhantomData<Rc<()>>,
}
impl Bridge {
    pub fn new() -> Result<Self, BridgeError> {
        if OWNED
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(BridgeError::Busy);
        }
        // SAFETY: OS feature query has no pointer arguments; checks CPU and OS FSGSBASE enablement.
        if unsafe { IsProcessorFeaturePresent(22) } == 0 {
            OWNED.store(false, Ordering::SeqCst);
            return Err(BridgeError::HostFeature);
        }
        // SAFETY: explicit exclusive lifecycle, no active handler yet.
        let slot = unsafe { TlsAlloc() };
        if slot >= 64 {
            if slot != u32::MAX {
                unsafe {
                    TlsFree(slot);
                }
            }
            OWNED.store(false, Ordering::SeqCst);
            return Err(BridgeError::Os("inline TLS slot"));
        }
        SLOT_OFFSET.store(0x1480 + slot as usize * 8, Ordering::SeqCst);
        let handler = unsafe { AddVectoredExceptionHandler(1, veh) };
        if handler.is_null() {
            unsafe {
                TlsFree(slot);
            }
            OWNED.store(false, Ordering::SeqCst);
            return Err(BridgeError::Os("VEH install"));
        }
        Ok(Self {
            slot,
            handler,
            validated: false,
            ranges: Vec::new(),
            _thread: PhantomData,
        })
    }
    /// Checked executable-range metadata retained by the runtime lease. No execution API.
    pub fn prepare_ranges(&mut self, ranges: &[(u64, u64)]) -> Result<(), BridgeError> {
        if ranges.len() > 65536
            || ranges
                .iter()
                .any(|&(a, n)| n == 0 || a.checked_add(n).is_none())
        {
            return Err(BridgeError::Validation);
        }
        self.ranges
            .try_reserve(ranges.len())
            .map_err(|_| BridgeError::Validation)?;
        self.ranges.clear();
        self.ranges.extend_from_slice(ranges);
        Ok(())
    }
    pub fn prepared_ranges(&self) -> usize {
        self.ranges.len()
    }
    pub fn validated(&self) -> bool {
        self.validated
    }
    pub fn return_landing(&self) -> u64 {
        landing as *const () as u64
    }
    pub fn import_landing(&self) -> u64 {
        import_landing as *const () as u64
    }
    /// Runs only one of five Astero assembly routines, on owned RW/NX stack and TLS.
    /// `true` resumes the synthetic import; `false` produces a controlled unresolved exit.
    pub fn synthetic(
        &mut self,
        probe: SyntheticProbe,
        callback: &mut dyn FnMut(u32, &mut CallFrame) -> bool,
    ) -> Result<NativeExit, BridgeError> {
        let base = 0x0000_0007_3000_0000;
        let mut stack = vec![0u8; 128 * 1024];
        let rsp = base + stack.len() as u64 - 136;
        let n = stack.len();
        stack[n - 136..n - 128].copy_from_slice(&self.return_landing().to_le_bytes());
        let tls_base = base + 0x40000;
        let mut tls = vec![0u8; 4096];
        tls[..8].copy_from_slice(&tls_base.to_le_bytes());
        let limits = NativeLimits {
            max_reserved_bytes: 1024 * 1024,
            max_committed_bytes: 1024 * 1024,
        };
        let rw = Protection {
            read: true,
            write: true,
            execute: false,
        };
        let (image, _) = windows_native::realize(
            &[
                NativeRegion {
                    range: GuestRange {
                        start: GuestAddress(base),
                        size: stack.len() as u64,
                    },
                    bytes: &stack,
                    protection: rw,
                },
                NativeRegion {
                    range: GuestRange {
                        start: GuestAddress(tls_base),
                        size: 4096,
                    },
                    bytes: &tls,
                    protection: rw,
                },
                NativeRegion {
                    range: GuestRange {
                        start: GuestAddress(tls_base + 4096),
                        size: 4096,
                    },
                    bytes: &tls,
                    protection: Protection::default(),
                },
            ],
            limits,
        );
        let _image = image.map_err(BridgeError::Memory)?;
        let target = match probe {
            SyntheticProbe::Return => probe_return,
            SyntheticProbe::Import | SyntheticProbe::ImportOrdinal(_) => probe_import,
            SyntheticProbe::IllegalInstruction => probe_illegal,
            SyntheticProbe::AccessViolation => probe_av,
            SyntheticProbe::FsRead => probe_fs,
            SyntheticProbe::GuardRead => probe_guard,
        } as *const () as u64;
        // SAFETY: feature-gated read-only registers, host GS is never written by this adapter.
        let (old_fs, old_gs) = unsafe { (read_fs(), read_gs()) };
        let probe_ranges = [(
            probe_return as *const () as u64,
            probes_end as *const () as u64 - probe_return as *const () as u64,
        )];
        let mut frame = Frame {
            host_rsp: 0,
            guest_rsp: rsp,
            target,
            fs: tls_base,
            old_fs,
            active: 0,
            reason: 0,
            result: 0,
            fault_rip: 0,
            fault_rsp: 0,
            code: 0,
            address: 0,
            args: [
                if probe == SyntheticProbe::GuardRead {
                    tls_base + 4096
                } else {
                    7
                },
                9,
                0,
                0,
                0,
                0,
            ],
            xmm: [[0; 16]; 8],
            out_xmm: [0; 16],
            saved: [0; 6],
            ranges: &probe_ranges,
            ordinal: match probe {
                SyntheticProbe::ImportOrdinal(i) => u64::from(i),
                _ => 0,
            },
            stack_start: base,
            stack_end: base + n as u64,
            fault_registers: [0; 16],
            callback,
        };
        // SAFETY: frame and mappings outlive synchronous assembly; no reference to frame is used
        // in Rust during transfer. Only the built-in probe range can be claimed by VEH.
        if unsafe { TlsSetValue(self.slot, (&mut frame as *mut Frame<'_>).cast()) } == 0 {
            return Err(BridgeError::Os("TLS install"));
        }
        let preservation = unsafe { checked_enter((&mut frame as *mut Frame<'_>).cast()) };
        unsafe {
            TlsSetValue(self.slot, std::ptr::null_mut());
        }
        if preservation != (1 << 18) - 1 {
            return Err(BridgeError::Validation);
        }
        let result = NativeExit {
            reason: frame.reason,
            value: frame.result,
            rip: frame.fault_rip,
            rsp: frame.fault_rsp,
            exception: frame.code as u32,
            import_ordinal: frame.ordinal as u32,
            fault_address: frame.address,
            host_fs_restored: unsafe { read_fs() } == old_fs,
            host_gs_preserved: unsafe { read_gs() } == old_gs,
            registers: frame.fault_registers,
        };
        if !result.host_fs_restored || !result.host_gs_preserved {
            return Err(BridgeError::Validation);
        }
        Ok(result)
    }
    pub fn validate(&mut self) -> Result<(), BridgeError> {
        let r = self.synthetic(SyntheticProbe::Return, &mut |_, _| false)?;
        let f = self.synthetic(SyntheticProbe::IllegalInstruction, &mut |_, _| false)?;
        let a = self.synthetic(SyntheticProbe::AccessViolation, &mut |_, _| false)?;
        let h = self.synthetic(SyntheticProbe::Import, &mut |_, c| {
            c.rax = c.arguments[0] + c.arguments[1];
            true
        })?;
        let u = self.synthetic(SyntheticProbe::Import, &mut |_, _| false)?;
        let t = self.synthetic(SyntheticProbe::FsRead, &mut |_, _| false)?;
        if r.value != 16
            || r.reason != 0
            || f.exception != 0xc000001d
            || a.exception != 0xc0000005
            || h.value != 16
            || u.reason != 2
            || t.value != 0x730040000
        {
            return Err(BridgeError::Validation);
        }
        let g = self.synthetic(SyntheticProbe::GuardRead, &mut |_, _| false)?;
        if g.exception != 0xc0000005 || g.fault_address != 0x730041000 {
            return Err(BridgeError::Validation);
        }
        self.validated = true;
        Ok(())
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        // SAFETY: &mut self and !Send prevent concurrent execution; no active lease remains.
        // Failure quarantines process slot/ownership so a stale callback is never reused.
        unsafe {
            if RemoveVectoredExceptionHandler(self.handler) != 0 {
                TlsSetValue(self.slot, std::ptr::null_mut());
                TlsFree(self.slot);
                OWNED.store(false, Ordering::SeqCst);
            }
        }
    }
}
unsafe extern "system" fn dispatch(frame: *mut Frame<'_>) {
    // SAFETY: called on saved host stack, host FS restored, from the active frame only.
    let f = unsafe { &mut *frame };
    let mut c = CallFrame {
        arguments: f.args,
        xmm: f.xmm,
        ..CallFrame::default()
    };
    if f.guest_rsp < f.stack_start
        || f.guest_rsp
            .checked_add(56)
            .is_none_or(|end| end > f.stack_end)
    {
        f.reason = 2;
        return;
    }
    // SAFETY: suspended native call, same owned RW stack, preflight above covers all six words.
    for (i, v) in c.stack_arguments.iter_mut().enumerate() {
        *v = unsafe { std::ptr::read_unaligned((f.guest_rsp + 8 + i as u64 * 8) as *const u64) };
    }
    let ok = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (f.callback)(f.ordinal as u32, &mut c)
    }))
    .unwrap_or(false);
    f.result = c.rax;
    f.out_xmm = c.xmm0;
    if !ok {
        f.reason = 2;
    }
}
unsafe extern "system" fn handle(info: *mut ExceptionPointers, frame: *mut Frame<'_>) -> i32 {
    // SAFETY: OS supplies the documented x64 prefix; assembly supplies its live frame.
    let (p, f) = unsafe { (&mut *info, &mut *frame) };
    let (r, c) = unsafe { (&*p.record, &mut *p.context) };
    if f.active != 1
        || !f.ranges.iter().any(|&(a, n)| c.rip >= a && c.rip - a < n)
        || r.flags & 1 != 0
        || !matches!(r.code, 0xc0000005 | 0xc000001d)
    {
        return 0;
    }
    f.fault_registers = c.regs;
    f.reason = 3;
    f.code = r.code as u64;
    f.fault_rip = c.rip;
    f.fault_rsp = c.regs[4];
    f.address = if r.code == 0xc0000005 && r.count >= 2 {
        r.info[1] as u64
    } else {
        0
    };
    c.rip = landing as *const () as u64;
    c.regs[4] = f.host_rsp;
    -1
}
global_asm!(r#"// Private M30 x64 bridge. host FXSAVE64 includes XMM0-15 and x87/control state.
// Windows TEB TLS inline slot offset is installed before VEH and retired after VEH.
.macro frame reg
 mov \reg, [rip+{slot}]
 mov \reg, gs:[\reg]
.endm
.global enter, landing, import_landing, veh, read_fs, read_gs
.global probe_return, probe_import, probe_illegal, probe_av, probe_fs, probe_guard, probes_end
.text
read_fs:
 rdfsbase rax
 ret
read_gs:
 rdgsbase rax
 ret
enter:
 push rbx
 push rbp
 push rsi
 push rdi
 push r12
 push r13
 push r14
 push r15
 pushfq
 pop rax
 sub rsp, 536
 fxsave64 [rsp]
 mov [rsp+512], rax
 mov [rsp+528], rcx
 mov [rcx+{host}], rsp
 mov rax,[rcx+{fs}]
 wrfsbase rax
 mov qword ptr [rcx+{active}],1
 mov r11,rcx
 mov rsp,[r11+{rsp}]
 mov rax,[r11+{target}]
 push rax
 mov rdi,[r11+{args}]
 mov rsi,[r11+{args}+8]
 mov rdx,[r11+{args}+16]
 mov rcx,[r11+{args}+24]
 mov r8,[r11+{args}+32]
 mov r9,[r11+{args}+40]
 xor eax,eax
 xor ebx,ebx
 xor ebp,ebp
 xor r10d,r10d
 xor r11d,r11d
 xor r12d,r12d
 xor r13d,r13d
 xor r14d,r14d
 xor r15d,r15d
 pxor xmm0,xmm0
 pxor xmm1,xmm1
 pxor xmm2,xmm2
 pxor xmm3,xmm3
 pxor xmm4,xmm4
 pxor xmm5,xmm5
 pxor xmm6,xmm6
 pxor xmm7,xmm7
 pxor xmm8,xmm8
 pxor xmm9,xmm9
 pxor xmm10,xmm10
 pxor xmm11,xmm11
 pxor xmm12,xmm12
 pxor xmm13,xmm13
 pxor xmm14,xmm14
 pxor xmm15,xmm15
 fninit
 ldmxcsr [rip+guest_mxcsr]
 push 0x202
 popfq
 ret
landing:
 frame r11
 mov [r11+{result}],rax
 mov qword ptr [r11+{active}],0
 mov rax,[r11+{oldfs}]
 wrfsbase rax
 mov rsp,[r11+{host}]
 fxrstor64 [rsp]
 mov rax,[rsp+512]
 mov [rsp+520],rax
 add rsp,536
 pop r15
 pop r14
 pop r13
 pop r12
 pop rdi
 pop rsi
 pop rbp
 pop rbx
 // Only ABI-preserved flag DF is relevant to compiled caller; always clear.
 cld
 ret
import_landing:
 // Stub pushed a validated ordinal. First implementation has one synthetic provider.
 pop rax
 frame r11
 mov [r11+{ordinal}],rax
 mov qword ptr [r11+{active}],0
 mov [r11+{rsp}],rsp
 mov [r11+{args}],rdi
 mov [r11+{args}+8],rsi
 mov [r11+{args}+16],rdx
 mov [r11+{args}+24],rcx
 mov [r11+{args}+32],r8
 mov [r11+{args}+40],r9
 movdqu [r11+{xmm}],xmm0
 movdqu [r11+{xmm}+16],xmm1
 movdqu [r11+{xmm}+32],xmm2
 movdqu [r11+{xmm}+48],xmm3
 movdqu [r11+{xmm}+64],xmm4
 movdqu [r11+{xmm}+80],xmm5
 movdqu [r11+{xmm}+96],xmm6
 movdqu [r11+{xmm}+112],xmm7
 mov [r11+{saved}],rbx
 mov [r11+{saved}+8],rbp
 mov [r11+{saved}+16],r12
 mov [r11+{saved}+24],r13
 mov [r11+{saved}+32],r14
 mov [r11+{saved}+40],r15
 mov rax,[r11+{oldfs}]
 wrfsbase rax
 mov rsp,[r11+{host}]
 sub rsp,32
 mov rcx,r11
 cld
 call {dispatch}
 frame r11
 cmp qword ptr [r11+{reason}],0
 jne stop_import
 mov rax,[r11+{fs}]
 wrfsbase rax
 mov qword ptr [r11+{active}],1
 mov rsp,[r11+{rsp}]
 mov rbx,[r11+{saved}]
 mov rbp,[r11+{saved}+8]
 mov r12,[r11+{saved}+16]
 mov r13,[r11+{saved}+24]
 mov r14,[r11+{saved}+32]
 mov r15,[r11+{saved}+40]
 movdqu xmm0,[r11+{outxmm}]
 mov rax,[r11+{result}]
 ret
stop_import:
 mov rax,[r11+{result}]
 jmp landing
veh:
 frame r10
 test r10,r10
 jz foreign_fault
 cmp qword ptr [r10+{active}],1
 jne foreign_fault
 // Preserve Windows handler callback stack across the call on the saved host stack.
 mov r11,rsp
 mov rsp,[r10+{host}]
 sub rsp,48
 mov [rsp+32],r11
 mov rdx,r10
 mov rax,[r10+{oldfs}]
 wrfsbase rax
 cld
 call {handle}
 mov rsp,[rsp+32]
 ret
foreign_fault:
 xor eax,eax
 ret
.balign 16
probe_return:
 mov rax,rdi
 add rax,rsi
 // SysV may clobber Windows-preserved vector lanes.
 pcmpeqb xmm6,xmm6
 pcmpeqb xmm15,xmm15
 ret
probe_import:
 sub rsp,8
 call synthetic_stub
 add rsp,8
 ret
synthetic_stub:
 frame r11
 push qword ptr [r11+{ordinal}]
 jmp import_landing
probe_illegal:
 ud2
 ret
probe_av:
 xor eax,eax
 mov [rax],rax
 ret
probe_fs:
 mov rax,fs:[0]
 ret
probe_guard:
 mov rax,[rdi]
 ret
probes_end:
 ret
.section .rdata,"dr"
.balign 4
guest_mxcsr: .long 0x1f80

.text
.global checked_enter
checked_enter:
 push rbx
 push rbp
 push rsi
 push rdi
 push r12
 push r13
 push r14
 push r15
 sub rsp,536
 fxsave64 [rsp]
 mov [rsp+512],rcx
 mov rbx,1192960
 mov rbp,1192961
 mov rsi,1192962
 mov rdi,1192963
 mov r12,1192964
 mov r13,1192965
 mov r14,1192966
 mov r15,1192967
 pcmpeqb xmm6,xmm6
 pcmpeqb xmm7,xmm7
 pcmpeqb xmm8,xmm8
 pcmpeqb xmm9,xmm9
 pcmpeqb xmm10,xmm10
 pcmpeqb xmm11,xmm11
 pcmpeqb xmm12,xmm12
 pcmpeqb xmm13,xmm13
 pcmpeqb xmm14,xmm14
 pcmpeqb xmm15,xmm15
 sub rsp,32
 call enter
 add rsp,32
 xor r10d,r10d
 cmp rbx,1192960
 sete al
 movzx eax,al
 shl rax,0
 or r10,rax
 cmp rbp,1192961
 sete al
 movzx eax,al
 shl rax,1
 or r10,rax
 cmp rsi,1192962
 sete al
 movzx eax,al
 shl rax,2
 or r10,rax
 cmp rdi,1192963
 sete al
 movzx eax,al
 shl rax,3
 or r10,rax
 cmp r12,1192964
 sete al
 movzx eax,al
 shl rax,4
 or r10,rax
 cmp r13,1192965
 sete al
 movzx eax,al
 shl rax,5
 or r10,rax
 cmp r14,1192966
 sete al
 movzx eax,al
 shl rax,6
 or r10,rax
 cmp r15,1192967
 sete al
 movzx eax,al
 shl rax,7
 or r10,rax
 pmovmskb eax,xmm6
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,8
 or r10,rax
 pmovmskb eax,xmm7
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,9
 or r10,rax
 pmovmskb eax,xmm8
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,10
 or r10,rax
 pmovmskb eax,xmm9
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,11
 or r10,rax
 pmovmskb eax,xmm10
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,12
 or r10,rax
 pmovmskb eax,xmm11
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,13
 or r10,rax
 pmovmskb eax,xmm12
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,14
 or r10,rax
 pmovmskb eax,xmm13
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,15
 or r10,rax
 pmovmskb eax,xmm14
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,16
 or r10,rax
 pmovmskb eax,xmm15
 cmp eax,65535
 sete al
 movzx eax,al
 shl rax,17
 or r10,rax
 mov rax,r10
 fxrstor64 [rsp]
 add rsp,536
 pop r15
 pop r14
 pop r13
 pop r12
 pop rdi
 pop rsi
 pop rbp
 pop rbx
 ret
"#,
 slot=sym SLOT_OFFSET, dispatch=sym dispatch, handle=sym handle,
 host=const offset_of!(Frame<'static>,host_rsp), rsp=const offset_of!(Frame<'static>,guest_rsp), target=const offset_of!(Frame<'static>,target),
 fs=const offset_of!(Frame<'static>,fs), oldfs=const offset_of!(Frame<'static>,old_fs), active=const offset_of!(Frame<'static>,active),
 reason=const offset_of!(Frame<'static>,reason), result=const offset_of!(Frame<'static>,result), args=const offset_of!(Frame<'static>,args),
 xmm=const offset_of!(Frame<'static>,xmm), outxmm=const offset_of!(Frame<'static>,out_xmm), ordinal=const offset_of!(Frame<'static>,ordinal), saved=const offset_of!(Frame<'static>,saved));
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_fault_classifier_declines_host_and_inactive_frames() {
        let mut callback = |_: u32, _: &mut CallFrame| false;
        let mut f = Frame {
            host_rsp: 0,
            guest_rsp: 0,
            target: 0,
            fs: 0,
            old_fs: 0,
            active: 1,
            reason: 0,
            result: 0,
            fault_rip: 0,
            fault_rsp: 0,
            code: 0,
            address: 0,
            args: [0; 6],
            xmm: [[0; 16]; 8],
            out_xmm: [0; 16],
            saved: [0; 6],
            ranges: &[(0x1000, 0x1000)],
            ordinal: 0,
            stack_start: 0,
            stack_end: 0,
            fault_registers: [0; 16],
            callback: &mut callback,
        };
        // SAFETY: these C layout records contain only integers/pointers; zero is valid data.
        let mut context: ContextPrefix = unsafe { std::mem::zeroed() };
        let mut record: ExceptionRecord = unsafe { std::mem::zeroed() };
        record.code = 0xc0000005;
        context.rip = 0x3000;
        let mut p = ExceptionPointers {
            record: &mut record,
            context: &mut context,
        };
        assert_eq!(unsafe { handle(&mut p, &mut f) }, 0);
        assert_eq!(context.rip, 0x3000);
        f.active = 0;
        context.rip = 0x1000;
        p.context = &mut context;
        assert_eq!(unsafe { handle(&mut p, &mut f) }, 0);
        assert_eq!(f.reason, 0);
        f.active = 1;
        record.flags = 1;
        p.record = &mut record;
        assert_eq!(unsafe { handle(&mut p, &mut f) }, 0);
    }
}
