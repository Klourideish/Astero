use astero_abi::layouts::entry::CallFrame;
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_kernel::process::output::Output;
use astero_libs::libc::formatting::{Error, arguments::Arguments, engine, exports::*};
use std::sync::{Arc, Mutex};
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
fn render(fmt: &[u8], values: &[u64]) -> Vec<u8> {
    let mut f = CallFrame::default();
    for (i, v) in values.iter().enumerate() {
        if i < 6 {
            f.arguments[i] = *v
        } else {
            f.stack_arguments[i - 6] = *v
        }
    }
    engine::format(&Memory::new(), fmt, &mut Arguments::direct(&f, 0))
        .unwrap()
        .bytes
}
fn setup() -> (Memory, Arc<Formatting>, PreparedRegistry) {
    let o = Arc::new(Formatting::new(
        Arc::new(Mutex::new(Output::new(65536))),
        0x100,
        0x120,
    ));
    let r = PreparedRegistry::new(registrations(o.clone()), 8).unwrap();
    (Memory::new(), o, r)
}
fn invoke(r: &PreparedRegistry, m: &mut Memory, i: usize, args: [u64; 6]) -> (CallResult, u64) {
    let mut f = CallFrame {
        arguments: args,
        ..Default::default()
    };
    let out = r.invoke(&key(EXPORTS[i].1), &mut f, m).unwrap();
    (out, f.rax)
}
fn list(m: &mut Memory, gp: u32, fp: u32) {
    m.put(64, &gp.to_le_bytes());
    m.put(68, &fp.to_le_bytes());
    m.put(72, &1024u64.to_le_bytes());
    m.put(80, &512u64.to_le_bytes());
}
#[test]
fn integers_length_and_signed_min() {
    assert_eq!(
        render(
            b"%d %u %lld %hhd %hu",
            &[0xffffffff, 0xffffffff, i64::MIN as u64, 255, 65537]
        ),
        b"-1 4294967295 -9223372036854775808 -1 1"
    );
}
#[test]
fn hex_octal_precision_and_unsigned_sign() {
    assert_eq!(
        render(b"%#08x %#o %.0u %+u", &[42, 8, 0, 5]),
        b"0x00002a 010  5"
    );
}
#[test]
fn width_precision_and_star_i32_promotion() {
    assert_eq!(
        render(
            b"%*.*d|%.*d",
            &[(-7i32) as u32 as u64, 4, 12, (-1i32) as u32 as u64, 12]
        ),
        b"0012   |12"
    );
}
#[test]
fn literal_percent_and_raw_character() {
    assert_eq!(render(b"\xff%%:%c", &[0x80]), [255, 37, 58, 128]);
}
#[test]
fn pointer_padding() {
    assert_eq!(render(b"%020p", &[0xabc]), b"0x000000000000000abc");
}
#[test]
fn direct_register_to_stack_transition() {
    assert_eq!(
        render(b"%d%d%d%d%d%d%d%d", &[1, 2, 3, 4, 5, 6, 7, 8]),
        b"12345678"
    );
}
#[test]
fn direct_guest_stack_beyond_snapshot() {
    let mut m = Memory::new();
    for i in 0..10 {
        m.put(1024 + i * 8, &(i as u64).to_le_bytes());
    }
    let f = CallFrame {
        stack_argument_address: Some(1024),
        ..Default::default()
    };
    let mut a = Arguments::direct(&f, 6);
    for i in 0..10 {
        assert_eq!(a.integer(&m).unwrap(), i);
    }
    assert_eq!(a.value.overflow_arg_area, 1104);
}
#[test]
fn va_list_advances_local_cursor_and_preserves_caller() {
    let mut m = Memory::new();
    list(&mut m, 40, 176);
    m.put(552, &11u64.to_le_bytes());
    m.put(1024, &22u64.to_le_bytes());
    let before = m.b[64..88].to_vec();
    let mut a = Arguments::list(&m, 64).unwrap();
    assert_eq!(a.integer(&m).unwrap(), 11);
    assert_eq!(a.integer(&m).unwrap(), 22);
    assert_eq!(a.value.gp_offset, 48);
    assert_eq!(a.value.overflow_arg_area, 1032);
    assert_eq!(&m.b[64..88], before);
}
#[test]
fn malformed_va_list_refused() {
    let mut m = Memory::new();
    for (gp, fp) in [(1, 48), (56, 48), (0, 0), (0, 49), (0, 192)] {
        list(&mut m, gp, fp);
        assert!(Arguments::list(&m, 64).is_err());
    }
    list(&mut m, 48, 176);
    m.put(72, &1025u64.to_le_bytes());
    assert!(Arguments::list(&m, 64).is_err());
}
#[test]
fn mixed_gp_fp_and_shared_overflow() {
    let mut m = Memory::new();
    list(&mut m, 40, 160);
    m.put(552, &12u64.to_le_bytes());
    m.put(672, &1.5f64.to_bits().to_le_bytes());
    m.put(1024, &2.5f64.to_bits().to_le_bytes());
    m.put(1032, &13u64.to_le_bytes());
    let mut a = Arguments::list(&m, 64).unwrap();
    assert_eq!(a.integer(&m).unwrap(), 12);
    assert_eq!(a.float(&m).unwrap(), 1.5);
    assert_eq!(a.float(&m).unwrap(), 2.5);
    assert_eq!(a.integer(&m).unwrap(), 13);
}
#[test]
fn floats_direct_decimal_rounding_and_sign() {
    let mut f = CallFrame::default();
    for (i, v) in [1.25f64, -0.0, 123.0].iter().enumerate() {
        f.xmm[i][..8].copy_from_slice(&v.to_bits().to_le_bytes());
    }
    assert_eq!(
        engine::format(
            &Memory::new(),
            b"%.1f %+f %.2e",
            &mut Arguments::direct(&f, 0)
        )
        .unwrap()
        .bytes,
        b"1.2 -0.000000 1.23e+02"
    );
}
#[test]
fn general_float_and_special_values() {
    let mut f = CallFrame::default();
    for (i, v) in [1000f64, f64::INFINITY, f64::NAN].iter().enumerate() {
        f.xmm[i][..8].copy_from_slice(&v.to_bits().to_le_bytes());
    }
    assert_eq!(
        engine::format(&Memory::new(), b"%.4g %F %g", &mut Arguments::direct(&f, 0))
            .unwrap()
            .bytes,
        b"1000 INF nan"
    );
}
#[test]
fn strings_precision_and_non_utf8() {
    let mut m = Memory::new();
    m.put(400, b"\xffabc\0");
    let f = CallFrame {
        arguments: [400, 0, 0, 0, 0, 0],
        ..Default::default()
    };
    assert_eq!(
        engine::format(&m, b"%6.3s", &mut Arguments::direct(&f, 0))
            .unwrap()
            .bytes,
        [32, 32, 32, 255, 97, 98]
    );
}
#[test]
fn invalid_string_and_precision_zero() {
    let m = Memory::new();
    let f = CallFrame {
        arguments: [u64::MAX, 0, 0, 0, 0, 0],
        ..Default::default()
    };
    assert!(engine::format(&m, b"%s", &mut Arguments::direct(&f, 0)).is_err());
    assert_eq!(
        engine::format(&m, b"%.0s", &mut Arguments::direct(&f, 0))
            .unwrap()
            .bytes,
        b""
    );
}
#[test]
fn snprintf_truncates_but_returns_required() {
    let (mut m, o, r) = setup();
    m.put(256, b"value=%d\0");
    assert_eq!(
        invoke(&r, &mut m, 1, [128, 5, 256, 123, 0, 0]),
        (CallResult::Returned, 9)
    );
    assert_eq!(&m.b[128..133], b"valu\0");
    assert!(o.snapshot()[0].truncated);
}
#[test]
fn snprintf_zero_and_one() {
    let (mut m, _, r) = setup();
    m.put(256, b"abc\0");
    assert_eq!(invoke(&r, &mut m, 1, [0, 0, 256, 0, 0, 0]).1, 3);
    assert_eq!(invoke(&r, &mut m, 1, [128, 1, 256, 0, 0, 0]).1, 3);
    assert_eq!(m.b[128], 0);
    assert!(matches!(
        invoke(&r, &mut m, 1, [0, 4, 256, 0, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
}
#[test]
fn vsnprintf_uses_explicit_list_not_direct_registers() {
    let (mut m, _, r) = setup();
    list(&mut m, 40, 176);
    m.put(552, &99u64.to_le_bytes());
    m.put(256, b"%d\0");
    assert_eq!(
        invoke(&r, &mut m, 0, [128, 8, 256, 64, 123, 0]),
        (CallResult::Returned, 2)
    );
    assert_eq!(&m.b[128..131], b"99\0");
}
#[test]
fn sprintf_and_vsprintf_share_core() {
    let (mut m, _, r) = setup();
    m.put(256, b"%x\0");
    assert_eq!(invoke(&r, &mut m, 3, [128, 256, 42, 0, 0, 0]).1, 2);
    assert_eq!(&m.b[128..131], b"2a\0");
    list(&mut m, 40, 176);
    m.put(552, &43u64.to_le_bytes());
    assert_eq!(invoke(&r, &mut m, 2, [128, 256, 64, 0, 0, 0]).1, 2);
    assert_eq!(&m.b[128..131], b"2b\0");
}
#[test]
fn output_preflight_avoids_partial_invalid_tail() {
    let (mut m, _, r) = setup();
    m.put(256, b"abcdef\0");
    let dest = m.b.len() as u64 - 3;
    let before = m.b.clone();
    assert!(matches!(
        invoke(&r, &mut m, 3, [dest, 256, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
    assert_eq!(m.b, before);
    m.ro = true;
    assert!(matches!(
        invoke(&r, &mut m, 1, [128, 8, 256, 0, 0, 0]).0,
        CallResult::AccessFailure(_)
    ));
}
#[test]
fn unsupported_percent_n_positional_wide_and_hexfloat() {
    for fmt in [
        b"%n".as_slice(),
        b"%2$d",
        b"%ls",
        b"%Lf",
        b"%a",
        b"%q",
        b"%",
    ] {
        let f = CallFrame::default();
        assert!(matches!(
            engine::format(&Memory::new(), fmt, &mut Arguments::direct(&f, 0)),
            Err(Error::Format { .. })
        ));
    }
}
#[test]
fn field_and_conversion_budgets_refuse() {
    let f = CallFrame::default();
    assert!(engine::format(&Memory::new(), b"%1048577d", &mut Arguments::direct(&f, 0)).is_err());
    let mut m = Memory::new();
    list(&mut m, 48, 176);
    let fmt = b"%.0s".repeat(16384);
    assert!(
        engine::format(&m, &fmt, &mut Arguments::list(&m, 64).unwrap())
            .unwrap()
            .bytes
            .is_empty()
    );
    let fmt = b"%.0s".repeat(16385);
    assert!(engine::format(&m, &fmt, &mut Arguments::list(&m, 64).unwrap()).is_err());
}
#[test]
fn console_output_and_owned_streams() {
    let (mut m, o, r) = setup();
    m.put(256, b"hello %d\0");
    assert_eq!(invoke(&r, &mut m, 4, [256, 1, 0, 0, 0, 0]).1, 7);
    assert_eq!(invoke(&r, &mut m, 6, [0x120, 256, 2, 0, 0, 0]).1, 7);
    assert_eq!(o.output.lock().unwrap().bytes(), b"hello 1hello 2");
    assert!(matches!(
        invoke(&r, &mut m, 6, [2, 256, 3, 0, 0, 0]).0,
        CallResult::FormatFailure { .. }
    ));
}
#[test]
fn vprintf_and_vfprintf_and_output_capacity() {
    let (mut m, o, r) = setup();
    list(&mut m, 40, 176);
    m.put(552, &7u64.to_le_bytes());
    m.put(256, b"%d\0");
    assert_eq!(invoke(&r, &mut m, 5, [256, 64, 0, 0, 0, 0]).1, 1);
    assert_eq!(invoke(&r, &mut m, 7, [0x100, 256, 64, 0, 0, 0]).1, 1);
    o.output.lock().unwrap().append(&vec![b'x'; 65534]).unwrap();
    assert!(matches!(
        invoke(&r, &mut m, 4, [256, 7, 0, 0, 0, 0]).0,
        CallResult::AccessFailure(AccessError::Limit)
    ));
    assert_eq!(o.output.lock().unwrap().bytes().len(), 65536);
}
#[test]
fn concurrent_output_is_call_atomic() {
    let (_, o, _) = setup();
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let o = o.clone();
            std::thread::spawn(move || {
                let r = PreparedRegistry::new(registrations(o), 8).unwrap();
                let mut m = Memory::new();
                m.put(256, b"record\n\0");
                for _ in 0..20 {
                    assert_eq!(invoke(&r, &mut m, 4, [256, 0, 0, 0, 0, 0]).1, 7);
                }
            })
        })
        .collect();
    for w in workers {
        w.join().unwrap();
    }
    assert_eq!(o.output.lock().unwrap().bytes(), b"record\n".repeat(80));
    assert_eq!(o.snapshot().len(), 80);
}
