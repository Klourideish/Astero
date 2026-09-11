//! Pure parser/integer/decimal rendering adapted from PS5Rust kernel/printf.rs.
use super::{Error, MAX_FIELD_WIDTH};
#[derive(Clone, Copy, Default)]
pub(super) struct Flags {
    pub(super) left: bool,
    pub(super) plus: bool,
    pub(super) space: bool,
    pub(super) alternate: bool,
    pub(super) zero: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Length {
    None,
    Hh,
    H,
    L,
    Ll,
    J,
    Z,
    T,
    BigL,
}

#[derive(Clone, Copy)]
pub(super) enum Width {
    None,
    Literal(usize),
    Argument,
}

#[derive(Clone, Copy)]
pub(super) struct Spec {
    pub(super) flags: Flags,
    pub(super) width: Width,
    pub(super) precision: Option<usize>,
    pub(super) length: Length,
    pub(super) conversion: u8,
}

fn unsupported(_message: impl Into<String>) -> Error {
    Error::contract("unsupported format or field bound")
}

fn parse_width(bytes: &[u8]) -> Result<usize, Error> {
    if bytes.is_empty() {
        return Ok(0);
    }
    let value = std::str::from_utf8(bytes)
        .unwrap_or("")
        .parse::<usize>()
        .map_err(|_| unsupported("printf field width overflows host size"))?;
    if value > MAX_FIELD_WIDTH {
        return Err(unsupported(format!(
            "printf field width exceeds {}",
            MAX_FIELD_WIDTH
        )));
    }
    Ok(value)
}

pub(super) fn parse_spec(bytes: &[u8], mut i: usize) -> Result<(Spec, usize), Error> {
    let mut flags = Flags::default();
    while let Some(byte) = bytes.get(i) {
        match byte {
            b'-' => flags.left = true,
            b'+' => flags.plus = true,
            b' ' => flags.space = true,
            b'#' => flags.alternate = true,
            b'0' => flags.zero = true,
            _ => break,
        }
        i += 1;
    }
    let width = if bytes.get(i) == Some(&b'*') {
        i += 1;
        let position_start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if bytes.get(i) == Some(&b'$') || position_start != i {
            return Err(unsupported("positional printf width"));
        }
        Width::Argument
    } else {
        let start = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if bytes.get(i) == Some(&b'$') {
            return Err(unsupported("positional printf argument"));
        }
        if start == i {
            Width::None
        } else {
            Width::Literal(parse_width(&bytes[start..i])?)
        }
    };
    let precision = if bytes.get(i) == Some(&b'.') {
        i += 1;
        if bytes.get(i) == Some(&b'*') {
            i += 1;
            let position_start = i;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            if bytes.get(i) == Some(&b'$') || position_start != i {
                return Err(unsupported("positional printf precision"));
            }
            Some(usize::MAX)
        } else {
            let start = i;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
            Some(if start == i {
                0
            } else {
                parse_width(&bytes[start..i])?
            })
        }
    } else {
        None
    };
    let length = if bytes.get(i..i + 2) == Some(b"hh") {
        i += 2;
        Length::Hh
    } else if bytes.get(i..i + 2) == Some(b"ll") {
        i += 2;
        Length::Ll
    } else if bytes.get(i) == Some(&b'h') {
        i += 1;
        Length::H
    } else if bytes.get(i) == Some(&b'l') {
        i += 1;
        Length::L
    } else if bytes.get(i) == Some(&b'j') {
        i += 1;
        Length::J
    } else if bytes.get(i) == Some(&b'z') {
        i += 1;
        Length::Z
    } else if bytes.get(i) == Some(&b't') {
        i += 1;
        Length::T
    } else if bytes.get(i) == Some(&b'L') {
        i += 1;
        Length::BigL
    } else {
        Length::None
    };
    let conversion = *bytes
        .get(i)
        .ok_or_else(|| unsupported("unterminated printf directive"))?;
    Ok((
        Spec {
            flags,
            width,
            precision,
            length,
            conversion,
        },
        i + 1,
    ))
}

fn signed_value(raw: u64, length: Length) -> i64 {
    match length {
        Length::Hh => raw as i8 as i64,
        Length::H => raw as i16 as i64,
        Length::L | Length::Ll | Length::J | Length::Z | Length::T => raw as i64,
        _ => raw as i32 as i64,
    }
}
fn unsigned_value(raw: u64, length: Length) -> u64 {
    match length {
        Length::Hh => raw as u8 as u64,
        Length::H => raw as u16 as u64,
        Length::L | Length::Ll | Length::J | Length::Z | Length::T => raw,
        _ => raw as u32 as u64,
    }
}
pub(super) fn add_sign(flags: Flags, negative: bool, body: String) -> String {
    if negative {
        format!("-{body}")
    } else if flags.plus {
        format!("+{body}")
    } else if flags.space {
        format!(" {body}")
    } else {
        body
    }
}
pub(super) fn pad(mut value: String, width: usize, flags: Flags, numeric: bool) -> String {
    if value.len() >= width {
        return value;
    }
    let count = width - value.len();
    if flags.left {
        value.push_str(&" ".repeat(count));
        return value;
    }
    if flags.zero && numeric {
        let prefix = if value.starts_with(['+', '-', ' ']) {
            1
        } else if value.starts_with("0x") || value.starts_with("0X") {
            2
        } else {
            0
        };
        format!(
            "{}{}{}",
            &value[..prefix],
            "0".repeat(count),
            &value[prefix..]
        )
    } else {
        format!("{}{}", " ".repeat(count), value)
    }
}
pub(super) fn render_integer(raw: u64, mut spec: Spec, width: usize) -> String {
    let signed = matches!(spec.conversion, b'd' | b'i');
    if !signed {
        spec.flags.plus = false;
        spec.flags.space = false;
    }
    let signed_number = signed_value(raw, spec.length);
    let negative = signed && signed_number < 0;
    let number = if signed {
        signed_number.unsigned_abs()
    } else {
        unsigned_value(raw, spec.length)
    };
    let base = match spec.conversion {
        b'o' => 8,
        b'x' | b'X' => 16,
        _ => 10,
    };
    let mut digits = match base {
        8 => format!("{number:o}"),
        16 if spec.conversion == b'X' => format!("{number:X}"),
        16 => format!("{number:x}"),
        _ => number.to_string(),
    };
    if number == 0 && spec.precision == Some(0) {
        digits.clear();
    }
    if let Some(precision) = spec.precision
        && digits.len() < precision
    {
        digits = format!("{}{}", "0".repeat(precision - digits.len()), digits);
    }
    if spec.flags.alternate {
        if base == 8 && !digits.starts_with('0') {
            digits.insert(0, '0');
        }
        if base == 16 && number != 0 {
            digits = format!(
                "{}{}",
                if spec.conversion == b'X' { "0X" } else { "0x" },
                digits
            );
        }
    }
    pad(
        add_sign(spec.flags, negative, digits),
        width,
        Flags {
            zero: spec.flags.zero && spec.precision.is_none(),
            ..spec.flags
        },
        true,
    )
}

fn trim_zeros(mut value: String, alternate: bool) -> String {
    if alternate {
        return value;
    }
    if let Some(position) = value.find(['e', 'E']) {
        let exponent = value[position..].to_string();
        let mut mantissa = value[..position].to_string();
        while mantissa.ends_with('0') {
            mantissa.pop();
        }
        if mantissa.ends_with('.') {
            mantissa.pop();
        }
        mantissa.push_str(&exponent);
        value = mantissa;
    } else if value.contains('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
    }
    value
}

pub(super) fn normalize_exponent(value: String, upper: bool) -> String {
    let Some(position) = value.rfind(['e', 'E']) else {
        return value;
    };
    let exponent = value[position + 1..].parse::<i32>().unwrap_or(0);
    let marker = if upper { 'E' } else { 'e' };
    let sign = if exponent >= 0 { '+' } else { '-' };
    format!(
        "{}{}{}{:02}",
        &value[..position],
        marker,
        sign,
        exponent.unsigned_abs()
    )
}

pub(super) fn render_general(value: f64, precision: usize, alternate: bool, upper: bool) -> String {
    if value.is_nan() {
        return if upper { "NAN".into() } else { "nan".into() };
    }
    if value.is_infinite() {
        return if upper { "INF".into() } else { "inf".into() };
    }
    let scientific = normalize_exponent(
        format!(
            "{:.prec$e}",
            value.abs(),
            prec = precision.saturating_sub(1)
        ),
        false,
    );
    let exponent = scientific
        .rfind(['e', 'E'])
        .and_then(|position| scientific[position + 1..].parse::<i32>().ok())
        .unwrap_or(0);
    let (mantissa, _) = scientific
        .split_once(['e', 'E'])
        .unwrap_or((&scientific, ""));
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let mut result = if exponent >= -4 && exponent < precision as i32 {
        let point = 1 + exponent;
        if point <= 0 {
            format!("0.{}{}", "0".repeat((-point) as usize), digits)
        } else if point as usize >= digits.len() {
            format!("{}{}", digits, "0".repeat(point as usize - digits.len()))
        } else {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        }
    } else {
        scientific
    };
    result = trim_zeros(result, alternate);
    if alternate && !result.contains('.') {
        if let Some(position) = result.find(['e', 'E']) {
            result.insert(position, '.');
        } else {
            result.push('.');
        }
    }
    if upper {
        result = result.replace('e', "E");
    }
    result
}
