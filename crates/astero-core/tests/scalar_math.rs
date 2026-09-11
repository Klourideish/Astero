use astero_abi::layouts::entry::CallFrame;
use astero_hle::{calls::memory::*, dispatch::prepared::*};
use astero_libs::libc::math::exports::*;
struct Memory {
    b: Vec<u8>,
    ro: bool,
}
impl Memory {
    fn new() -> Self {
        Self {
            b: vec![0; 256],
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

fn invoke(name: &str, x: f64, y: f64, arg: u64, m: &mut Memory) -> (CallResult, CallFrame, f64) {
    let e = EXPORTS.iter().find(|e| e.name == name).unwrap();
    let mut f = CallFrame {
        arguments: [arg, 160, 0, 0, 0, 0],
        rax: 99,
        xmm0: [0xaa; 16],
        ..Default::default()
    };
    match e.precision {
        Precision::F32 => {
            f.xmm[0][..4].copy_from_slice(&(x as f32).to_le_bytes());
            f.xmm[1][..4].copy_from_slice(&(y as f32).to_le_bytes());
            f.xmm[0][4..].fill(0x7f);
        }
        Precision::F64 => {
            f.xmm[0][..8].copy_from_slice(&x.to_le_bytes());
            f.xmm[1][..8].copy_from_slice(&y.to_le_bytes());
        }
    }
    let r = PreparedRegistry::new(registrations(), EXPORTS.len()).unwrap();
    let result = r.invoke(&key(e.nid), &mut f, m).unwrap();
    let v = match e.precision {
        Precision::F32 => f32::from_le_bytes(f.xmm0[..4].try_into().unwrap()) as f64,
        Precision::F64 => f64::from_le_bytes(f.xmm0[..8].try_into().unwrap()),
    };
    (result, f, v)
}
fn value(n: &str, x: f64, y: f64, arg: u64) -> f64 {
    let (r, _, v) = invoke(n, x, y, arg, &mut Memory::new());
    assert_eq!(r, CallResult::Returned);
    v
}
#[test]
fn exact_identity_and_capacity() {
    let r = PreparedRegistry::new(registrations(), 53).unwrap();
    assert_eq!(EXPORTS.len(), 53);
    for e in EXPORTS {
        assert!(r.find(&key(e.nid)).is_ok());
        let mut k = key(e.nid);
        k.library = b"wrong".to_vec();
        assert!(r.find(&k).is_err());
    }
    assert!(PreparedRegistry::new(registrations(), 52).is_err());
}
#[test]
fn normal_unary_family() {
    for (n, x, want) in [
        ("sin", 0., 0.),
        ("cos", 0., 1.),
        ("tan", 0., 0.),
        ("asin", 0., 0.),
        ("acos", 1., 0.),
        ("atan", 0., 0.),
        ("exp", 0., 1.),
        ("exp2", 3., 8.),
        ("log", 1., 0.),
        ("log2", 8., 3.),
        ("log10", 100., 2.),
        ("round", 1.5, 2.),
    ] {
        for suffix in ["", "f"] {
            assert_eq!(value(&format!("{n}{suffix}"), x, 0., 0), want);
        }
    }
}
#[test]
fn binary_argument_order_and_width() {
    assert_eq!(value("powf", 2., 3., 0), 8.);
    assert_eq!(value("pow", 2., -3., 0), 0.125);
    assert_eq!(value("atan2", 0., 1., 0), 0.);
    assert_eq!(value("hypotf", 3., 4., 0), 5.);
    assert_eq!(value("fmod", 5.5, 2., 0), 1.5);
}
#[test]
fn float_is_not_double() {
    let x = 1.0 + 2f64.powi(-30);
    assert_eq!(value("powf", x, 1., 0), 1.);
    assert_eq!(value("pow", x, 1., 0), x);
}
#[test]
fn negative_zero_and_infinity() {
    for n in ["sin", "sinf", "atan", "atanf", "round", "roundf"] {
        assert!(value(n, -0., 0., 0).is_sign_negative());
    }
    assert_eq!(value("log", 0., 0., 0), f64::NEG_INFINITY);
    assert_eq!(value("exp", f64::NEG_INFINITY, 0., 0), 0.);
}
#[test]
fn domains_ranges_and_errno_unchanged() {
    let mut m = Memory::new();
    m.b.fill(0x73);
    for (n, x, y) in [
        ("acosf", 2., 0.),
        ("log", -1., 0.),
        ("pow", -1., 0.5),
        ("fmod", 1., 0.),
    ] {
        let (r, _, v) = invoke(n, x, y, 0, &mut m);
        assert_eq!(r, CallResult::Returned);
        assert!(v.is_nan());
    }
    assert!(value("exp", 10000., 0., 0).is_infinite());
    assert!(m.b.iter().all(|b| *b == 0x73));
}
#[test]
fn min_max_nan_and_signed_zero() {
    assert_eq!(value("fminf", f64::NAN, 3., 0), 3.);
    assert_eq!(value("fmaxf", 3., f64::NAN, 0), 3.);
    assert!(value("fminf", 0., -0., 0).is_sign_negative());
    assert!(value("fmaxf", -0., 0., 0).is_sign_positive());
}
#[test]
fn nearby_even_vs_round_away() {
    assert_eq!(value("nearbyintf", 2.5, 0., 0), 2.);
    assert_eq!(value("roundf", 2.5, 0., 0), 3.);
    assert_eq!(value("nearbyintf", -1.5, 0., 0), -2.);
}
#[test]
fn integer_exponent_uses_gp_lane() {
    assert_eq!(value("__powisf2", 2., 999., (-3i64) as u64), 0.125);
    assert_eq!(value("ldexp", 1.5, 999., 3), 12.);
    assert_eq!(value("scalbn", 1.5, 0., 3), 12.);
}
#[test]
fn scale_extremes_avoid_intermediate_overflow() {
    assert_eq!(value("ldexp", f64::from_bits(1), 0., 1074), 1.);
    assert_eq!(value("ldexp", 1., 0., (-1074i64) as u64), f64::from_bits(1));
    assert_eq!(value("ldexpf", f32::from_bits(1) as f64, 0., 149), 1.);
    assert!(value("ldexp", 0., 0., i32::MAX as u64) == 0.);
    assert!(value("ldexp", -1., 0., i32::MAX as u64).is_infinite());
    assert!(value("ldexp", -1., 0., (i32::MIN as i64) as u64).is_sign_negative());
}
#[test]
fn frexp_exact_width_and_subnormals() {
    for (n, x, want) in [
        ("frexp", f64::from_bits(1), -1073),
        ("frexpf", f32::from_bits(1) as f64, -148),
    ] {
        let mut m = Memory::new();
        m.b.fill(0x44);
        let (r, _, v) = invoke(n, x, 0., 252, &mut m);
        assert_eq!(r, CallResult::Returned);
        assert_eq!(v, 0.5);
        assert_eq!(i32::from_le_bytes(m.b[252..].try_into().unwrap()), want);
        assert_eq!(m.b[251], 0x44);
    }
}
#[test]
fn modf_normal_and_infinity() {
    for (n, width) in [("modf", 8), ("modff", 4)] {
        let mut m = Memory::new();
        let (r, _, v) = invoke(n, -3.5, 0., 128, &mut m);
        assert_eq!(r, CallResult::Returned);
        assert_eq!(v, -0.5);
        let integer = if width == 8 {
            f64::from_le_bytes(m.b[128..136].try_into().unwrap())
        } else {
            f32::from_le_bytes(m.b[128..132].try_into().unwrap()) as f64
        };
        assert_eq!(integer, -3.);
        let (_, _, v) = invoke(n, f64::NEG_INFINITY, 0., 128, &mut m);
        assert_eq!(v, 0.);
        assert!(v.is_sign_negative());
    }
}
#[test]
fn modf_negative_integral_zero() {
    for n in ["modf", "modff"] {
        assert!(value(n, -4., 0., 128).is_sign_negative());
        assert!(value(n, -0., 0., 128).is_sign_negative());
        assert!(value(n, f64::NAN, 0., 128).is_nan());
    }
}
#[test]
fn invalid_outputs_do_not_publish_return() {
    for (n, p) in [("frexp", 253), ("modf", 249), ("modff", 253), ("frexpf", 0)] {
        let mut m = Memory::new();
        m.b.fill(0x55);
        let (r, f, _) = invoke(n, 3.5, 0., p, &mut m);
        assert_eq!(r, CallResult::AccessFailure(AccessError::Range));
        assert_eq!(f.xmm0, [0xaa; 16]);
        assert_eq!(f.rax, 99);
        assert!(m.b.iter().all(|b| *b == 0x55));
    }
}
#[test]
fn read_only_output_refuses() {
    let mut m = Memory::new();
    m.ro = true;
    assert_eq!(
        invoke("frexp", 1., 0., 128, &mut m).0,
        CallResult::AccessFailure(AccessError::Range)
    );
}
#[test]
fn sincos_preflights_both_outputs() {
    let mut m = Memory::new();
    m.b.resize(156, 0x77);
    m.b.fill(0x77);
    assert_eq!(
        invoke("sincos", 0., 0., 128, &mut m).0,
        CallResult::AccessFailure(AccessError::Range)
    );
    assert!(m.b.iter().all(|b| *b == 0x77));
}
#[test]
fn sincos_two_checked_outputs() {
    let mut m = Memory::new();
    assert_eq!(
        invoke("sincos", 0., 0., 128, &mut m).0,
        CallResult::Returned
    );
    assert_eq!(f64::from_le_bytes(m.b[128..136].try_into().unwrap()), 0.);
    assert_eq!(f64::from_le_bytes(m.b[160..168].try_into().unwrap()), 1.);
}
#[test]
fn classification_returns_integer() {
    for (n, x, want) in [
        ("__isnan", f64::NAN, 1),
        ("__isfinite", f64::INFINITY, 0),
        ("__isinf", f64::NEG_INFINITY, 1),
    ] {
        for suffix in ["", "f"] {
            let (_, f, _) = invoke(&format!("{n}{suffix}"), x, 0., 0, &mut Memory::new());
            assert_eq!(f.rax, want);
            assert_eq!(f.xmm0, [0xaa; 16]);
        }
    }
}
#[test]
fn nextafter_exact_ieee_steps() {
    assert_eq!(value("nextafter", 0., 1., 0).to_bits(), 1);
    assert_eq!(
        value("nextafterf", 1., 2., 0) as f32,
        f32::from_bits(1f32.to_bits() + 1)
    );
    assert!(value("nextafter", 0., -0., 0).is_sign_negative());
}
#[test]
fn logb_special_and_subnormal() {
    assert_eq!(value("logb", f64::from_bits(1), 0., 0), -1074.);
    assert_eq!(value("logb", 8., 0., 0), 3.);
    assert_eq!(value("logb", 0., 0., 0), f64::NEG_INFINITY);
}
