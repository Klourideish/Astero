//! Exact libc/libc scalar exports; errno is unchanged as in the inherited math handlers.
use super::values;
use astero_abi::layouts::entry::CallFrame;
use astero_hle::{
    calls::memory::{AccessError, GuestMemory},
    dispatch::prepared::*,
};
#[derive(Clone, Copy, Debug)]
pub enum Precision {
    F32,
    F64,
}
#[derive(Clone, Copy, Debug)]
pub enum Op {
    Pow,
    Powi,
    Frexp,
    Scale,
    SinCos,
    Modf,
    Nearby,
    Min,
    Max,
    Hypot,
    Logb,
    Next,
    Exp,
    Exp2,
    Log,
    Log2,
    Log10,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Round,
    Fmod,
    IsNan,
    IsFinite,
    IsInf,
}
#[derive(Clone, Copy, Debug)]
pub struct Export {
    pub name: &'static str,
    pub nid: u64,
    pub op: Op,
    pub precision: Precision,
}
pub const EXPORTS: &[Export] = &[
    Export {
        name: "powf",
        nid: 0xD43D07D8A363B211,
        op: Op::Pow,
        precision: Precision::F32,
    },
    Export {
        name: "__powisf2",
        nid: 0x122324810B0E7D4D,
        op: Op::Powi,
        precision: Precision::F32,
    },
    Export {
        name: "pow",
        nid: 0xF4B0A3A56C90E597,
        op: Op::Pow,
        precision: Precision::F64,
    },
    Export {
        name: "frexp",
        nid: 0x900FD3762382B1A6,
        op: Op::Frexp,
        precision: Precision::F64,
    },
    Export {
        name: "frexpf",
        nid: 0x69A0CC186917171A,
        op: Op::Frexp,
        precision: Precision::F32,
    },
    Export {
        name: "ldexp",
        nid: 0x26BC0520CCCA36BD,
        op: Op::Scale,
        precision: Precision::F64,
    },
    Export {
        name: "ldexpf",
        nid: 0x927D32898784C600,
        op: Op::Scale,
        precision: Precision::F32,
    },
    Export {
        name: "scalbn",
        nid: 0x28628179572A2637,
        op: Op::Scale,
        precision: Precision::F64,
    },
    Export {
        name: "sincos",
        nid: 0x8CC07B105CAEDF46,
        op: Op::SinCos,
        precision: Precision::F64,
    },
    Export {
        name: "sincosf",
        nid: 0xA73B55E00175F222,
        op: Op::SinCos,
        precision: Precision::F32,
    },
    Export {
        name: "modf",
        nid: 0xD163070DBE43B7DE,
        op: Op::Modf,
        precision: Precision::F64,
    },
    Export {
        name: "modff",
        nid: 0xDFE50F33FF44EB16,
        op: Op::Modf,
        precision: Precision::F32,
    },
    Export {
        name: "nearbyintf",
        nid: 0x73EE2BFD3FED1087,
        op: Op::Nearby,
        precision: Precision::F32,
    },
    Export {
        name: "fminf",
        nid: 0xB9545C336C8574FE,
        op: Op::Min,
        precision: Precision::F32,
    },
    Export {
        name: "fmaxf",
        nid: 0x2F2C760F350BECB7,
        op: Op::Max,
        precision: Precision::F32,
    },
    Export {
        name: "hypotf",
        nid: 0x8B3DAC8401852317,
        op: Op::Hypot,
        precision: Precision::F32,
    },
    Export {
        name: "logb",
        nid: 0xA302AE7A0654E1EC,
        op: Op::Logb,
        precision: Precision::F64,
    },
    Export {
        name: "nextafter",
        nid: 0x87E27AD114657E79,
        op: Op::Next,
        precision: Precision::F64,
    },
    Export {
        name: "nextafterf",
        nid: 0xDE6DABA3E0E2F829,
        op: Op::Next,
        precision: Precision::F32,
    },
    Export {
        name: "exp",
        nid: 0x35569D7E7CD08474,
        op: Op::Exp,
        precision: Precision::F64,
    },
    Export {
        name: "expf",
        nid: 0xF33B2ED385CDB19E,
        op: Op::Exp,
        precision: Precision::F32,
    },
    Export {
        name: "exp2",
        nid: 0x76769E1976E33FA1,
        op: Op::Exp2,
        precision: Precision::F64,
    },
    Export {
        name: "exp2f",
        nid: 0xC2E010B7F8FEA78A,
        op: Op::Exp2,
        precision: Precision::F32,
    },
    Export {
        name: "log",
        nid: 0xAED57BFE3582E988,
        op: Op::Log,
        precision: Precision::F64,
    },
    Export {
        name: "logf",
        nid: 0x4505CB6DD4F695CE,
        op: Op::Log,
        precision: Precision::F32,
    },
    Export {
        name: "log2",
        nid: 0x6390E1B832869674,
        op: Op::Log2,
        precision: Precision::F64,
    },
    Export {
        name: "log2f",
        nid: 0x86C8BD76BCC74769,
        op: Op::Log2,
        precision: Precision::F32,
    },
    Export {
        name: "log10",
        nid: 0x5AE31B3C128DD535,
        op: Op::Log10,
        precision: Precision::F64,
    },
    Export {
        name: "log10f",
        nid: 0x961A5DE9693A71CB,
        op: Op::Log10,
        precision: Precision::F32,
    },
    Export {
        name: "sin",
        nid: 0x1FCC9AD87D348DB2,
        op: Op::Sin,
        precision: Precision::F64,
    },
    Export {
        name: "sinf",
        nid: 0x438AD12F7E0211E1,
        op: Op::Sin,
        precision: Precision::F32,
    },
    Export {
        name: "cos",
        nid: 0xD96137053615C0A3,
        op: Op::Cos,
        precision: Precision::F64,
    },
    Export {
        name: "cosf",
        nid: 0xFCFE8534CCE4D8A7,
        op: Op::Cos,
        precision: Precision::F32,
    },
    Export {
        name: "tan",
        nid: 0x4FBBB236A3FBBD00,
        op: Op::Tan,
        precision: Precision::F64,
    },
    Export {
        name: "tanf",
        nid: 0x644E9134BF9E2DB9,
        op: Op::Tan,
        precision: Precision::F32,
    },
    Export {
        name: "asin",
        nid: 0xECBCB9DB368BE384,
        op: Op::Asin,
        precision: Precision::F64,
    },
    Export {
        name: "asinf",
        nid: 0x1995A317F6081459,
        op: Op::Asin,
        precision: Precision::F32,
    },
    Export {
        name: "acos",
        nid: 0x24172062E5BC94F5,
        op: Op::Acos,
        precision: Precision::F64,
    },
    Export {
        name: "acosf",
        nid: 0x408FF1D122FC8E1C,
        op: Op::Acos,
        precision: Precision::F32,
    },
    Export {
        name: "atan",
        nid: 0x39799AB8B750F246,
        op: Op::Atan,
        precision: Precision::F64,
    },
    Export {
        name: "atanf",
        nid: 0xC1E0EE83C403FE51,
        op: Op::Atan,
        precision: Precision::F32,
    },
    Export {
        name: "atan2",
        nid: 0x1D46D998E9D3FC38,
        op: Op::Atan2,
        precision: Precision::F64,
    },
    Export {
        name: "atan2f",
        nid: 0x107FF1EF5DC0F7D7,
        op: Op::Atan2,
        precision: Precision::F32,
    },
    Export {
        name: "round",
        nid: 0x9E56A88CBF610ED0,
        op: Op::Round,
        precision: Precision::F64,
    },
    Export {
        name: "roundf",
        nid: 0x0C31C6D5AEBEDEAD,
        op: Op::Round,
        precision: Precision::F32,
    },
    Export {
        name: "fmod",
        nid: 0xA4AC2C96C3149929,
        op: Op::Fmod,
        precision: Precision::F64,
    },
    Export {
        name: "fmodf",
        nid: 0xF3C56FFC0CC7563F,
        op: Op::Fmod,
        precision: Precision::F32,
    },
    Export {
        name: "__isnan",
        nid: 0x19FC40A7D5F28AAB,
        op: Op::IsNan,
        precision: Precision::F64,
    },
    Export {
        name: "__isnanf",
        nid: 0x940F786604FEBCC3,
        op: Op::IsNan,
        precision: Precision::F32,
    },
    Export {
        name: "__isfinite",
        nid: 0x7612B5E822B08508,
        op: Op::IsFinite,
        precision: Precision::F64,
    },
    Export {
        name: "__isfinitef",
        nid: 0x43CA6F2629945A2B,
        op: Op::IsFinite,
        precision: Precision::F32,
    },
    Export {
        name: "__isinf",
        nid: 0x574DA816FFBF2730,
        op: Op::IsInf,
        precision: Precision::F64,
    },
    Export {
        name: "__isinff",
        nid: 0xAC333201FD4986E8,
        op: Op::IsInf,
        precision: Precision::F32,
    },
];
pub fn key(nid: u64) -> ProviderKey {
    ProviderKey {
        nid,
        library: b"libc".to_vec(),
        module: b"libc".to_vec(),
    }
}
fn preflight(m: &dyn GuestMemory, p: u64, n: u64) -> Result<(), AccessError> {
    if p == 0 {
        return Err(AccessError::Range);
    }
    m.charge(n)?;
    m.validate(p, n, true)
}
macro_rules! scalar {
    ($fun:ident,$t:ty,$u:ty,$width:literal,$next:ident) => {
        fn $fun(e: &Export, f: &mut CallFrame, m: &mut dyn GuestMemory) -> Result<(), AccessError> {
            let x = <$t>::from_bits(<$u>::from_le_bytes(
                f.xmm[0][..$width].try_into().expect("lane"),
            ));
            let y = <$t>::from_bits(<$u>::from_le_bytes(
                f.xmm[1][..$width].try_into().expect("lane"),
            ));
            let a = f.arguments;
            let result: $t = match e.op {
                Op::Pow => x.powf(y),
                Op::Powi => x.powi(a[0] as i32),
                Op::Exp => x.exp(),
                Op::Exp2 => x.exp2(),
                Op::Log => x.ln(),
                Op::Log2 => x.log2(),
                Op::Log10 => x.log10(),
                Op::Sin => x.sin(),
                Op::Cos => x.cos(),
                Op::Tan => x.tan(),
                Op::Asin => x.asin(),
                Op::Acos => x.acos(),
                Op::Atan => x.atan(),
                Op::Atan2 => x.atan2(y),
                Op::Round => x.round(),
                Op::Nearby => x.round_ties_even(),
                Op::Fmod => x % y,
                Op::Hypot => x.hypot(y),
                Op::Min => {
                    if x == 0.0 && y == 0.0 {
                        if x.is_sign_negative() || y.is_sign_negative() {
                            -0.0
                        } else {
                            0.0
                        }
                    } else {
                        x.min(y)
                    }
                }
                Op::Max => {
                    if x == 0.0 && y == 0.0 {
                        if x.is_sign_negative() && y.is_sign_negative() {
                            -0.0
                        } else {
                            0.0
                        }
                    } else {
                        x.max(y)
                    }
                }
                Op::Scale => values::scale(x as f64, a[0] as i32) as $t,
                Op::Logb => values::logb(x as f64) as $t,
                Op::Next => values::$next(x, y),
                Op::Frexp => {
                    let (fraction, exponent) = values::frexp(x as f64);
                    preflight(m, a[0], 4)?;
                    m.write(a[0], &exponent.to_le_bytes())?;
                    fraction as $t
                }
                Op::Modf => {
                    let integer = x.trunc();
                    preflight(m, a[0], $width)?;
                    m.write(a[0], &integer.to_le_bytes())?;
                    if x.is_infinite() || x == integer {
                        (0.0 as $t).copysign(x)
                    } else {
                        x - integer
                    }
                }
                Op::SinCos => {
                    let (s, c) = x.sin_cos();
                    preflight(m, a[0], $width)?;
                    preflight(m, a[1], $width)?;
                    m.write(a[0], &s.to_le_bytes())?;
                    m.write(a[1], &c.to_le_bytes())?;
                    f.rax = 0;
                    return Ok(());
                }
                Op::IsNan | Op::IsFinite | Op::IsInf => {
                    f.rax = match e.op {
                        Op::IsNan => x.is_nan(),
                        Op::IsFinite => x.is_finite(),
                        _ => x.is_infinite(),
                    } as u64;
                    return Ok(());
                }
            };
            f.xmm0 = [0; 16];
            f.xmm0[..$width].copy_from_slice(&result.to_le_bytes());
            f.rax = 0;
            Ok(())
        }
    };
}
scalar!(run32, f32, u32, 4, next32);
scalar!(run64, f64, u64, 8, next64);
pub fn registrations() -> Vec<Registration> {
    EXPORTS
        .iter()
        .map(|e| Registration {
            key: key(e.nid),
            kind: ProviderKind::HleImplementation,
            handler: Some(Box::new(move |f, m| {
                let result = match e.precision {
                    Precision::F32 => run32(e, f, m),
                    Precision::F64 => run64(e, f, m),
                };
                match result {
                    Ok(()) => CallResult::Returned,
                    Err(e) => CallResult::AccessFailure(e),
                }
            })),
        })
        .collect()
}
