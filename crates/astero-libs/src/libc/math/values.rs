//! Bounded IEEE scalar adaptations; not a complete fenv implementation.
pub(super) fn frexp(x: f64) -> (f64, i32) {
    if x == 0.0 || !x.is_finite() {
        return (x, 0);
    }
    let bits = x.to_bits();
    let e = ((bits >> 52) & 0x7ff) as i32;
    if e == 0 {
        let (f, n) = frexp(x * 18446744073709551616.0);
        return (f, n - 64);
    }
    (
        f64::from_bits((bits & 0x800fffffffffffff) | (1022u64 << 52)),
        e - 1022,
    )
}
/// Unlike x*2.powi(n), intermediate powers cannot overflow a representable result.
/// At most eight exact binary scaling steps plus one final factor.
pub(super) fn scale(mut x: f64, n: i32) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    if n > 4096 {
        return f64::INFINITY.copysign(x);
    }
    if n < -4096 {
        return 0.0f64.copysign(x);
    }
    let mut n = n;
    while n > 512 {
        x *= f64::from_bits(1535u64 << 52);
        n -= 512;
    }
    while n < -512 {
        x *= f64::from_bits(511u64 << 52);
        n += 512;
    }
    x * f64::from_bits(((n + 1023) as u64) << 52)
}
pub(super) fn logb(x: f64) -> f64 {
    if x.is_nan() {
        x
    } else if x == 0.0 {
        f64::NEG_INFINITY
    } else if x.is_infinite() {
        f64::INFINITY
    } else {
        (frexp(x).1 - 1) as f64
    }
}
pub(super) fn next64(x: f64, y: f64) -> f64 {
    if x.is_nan() || y.is_nan() {
        x + y
    } else if x == y {
        y
    } else if x < y {
        x.next_up()
    } else {
        x.next_down()
    }
}
pub(super) fn next32(x: f32, y: f32) -> f32 {
    if x.is_nan() || y.is_nan() {
        x + y
    } else if x == y {
        y
    } else if x < y {
        x.next_up()
    } else {
        x.next_down()
    }
}
