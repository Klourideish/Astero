//! Guest 16-byte timespec: two signed little-endian 64-bit fields; no host layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timespec {
    pub seconds: i64,
    pub nanos: i64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidTime;
impl Timespec {
    pub fn decode(b: &[u8]) -> Result<Self, InvalidTime> {
        if b.len() != 16 {
            return Err(InvalidTime);
        }
        Ok(Self {
            seconds: i64::from_le_bytes(b[..8].try_into().map_err(|_| InvalidTime)?),
            nanos: i64::from_le_bytes(b[8..].try_into().map_err(|_| InvalidTime)?),
        })
    }
    pub fn total_nanos(self) -> Result<u64, InvalidTime> {
        if self.seconds < 0 || !(0..1_000_000_000).contains(&self.nanos) {
            return Err(InvalidTime);
        }
        (self.seconds as u64)
            .checked_mul(1_000_000_000)
            .and_then(|n| n.checked_add(self.nanos as u64))
            .ok_or(InvalidTime)
    }
    pub fn from_nanos(n: u64) -> Self {
        Self {
            seconds: (n / 1_000_000_000) as i64,
            nanos: (n % 1_000_000_000) as i64,
        }
    }
    pub fn encode(self) -> [u8; 16] {
        let mut b = [0; 16];
        b[..8].copy_from_slice(&self.seconds.to_le_bytes());
        b[8..].copy_from_slice(&self.nanos.to_le_bytes());
        b
    }
}
