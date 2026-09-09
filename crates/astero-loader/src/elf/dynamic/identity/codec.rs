//! Encoded NID bytes, not plain-name SHA-1 hashing or a symbol database.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+-";
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodingError {
    Length,
    Alphabet { index: usize },
    NonCanonicalPadding,
    Context,
}
fn digit(byte: u8, index: usize) -> Result<u128, EncodingError> {
    ALPHABET
        .iter()
        .position(|b| *b == byte)
        .map(|v| v as u128)
        .ok_or(EncodingError::Alphabet { index })
}
/// Require canonical padding; legacy's acceptance of aliases is deliberately not inherited.
pub fn decode_nid(bytes: &[u8]) -> Result<u64, EncodingError> {
    if bytes.len() != 11 {
        return Err(EncodingError::Length);
    }
    let mut bits = 0u128;
    for (i, b) in bytes.iter().enumerate() {
        bits = (bits << 6) | digit(*b, i)?;
    }
    if bits & 3 != 0 {
        return Err(EncodingError::NonCanonicalPadding);
    }
    Ok((bits >> 2) as u64)
}
pub fn encode_nid(nid: u64) -> [u8; 11] {
    let bits = (nid as u128) << 2;
    std::array::from_fn(|i| ALPHABET[((bits >> ((10 - i) * 6)) & 63) as usize])
}
/// Experimental numeric suffix IDs: shortest base-64 integer, at most u16.
pub fn decode_context(bytes: &[u8]) -> Result<u16, EncodingError> {
    if bytes.is_empty() || bytes.len() > 3 || (bytes.len() > 1 && bytes[0] == b'A') {
        return Err(EncodingError::Context);
    }
    let mut value = 0u128;
    for (i, b) in bytes.iter().enumerate() {
        value = (value << 6) | digit(*b, i)?;
    }
    u16::try_from(value).map_err(|_| EncodingError::Context)
}
