//! Explicit guest epochs and modeled granularity. CPU clocks are not elapsed clocks.
use astero_abi::layouts::time::Timespec;
use astero_timing::{
    scheduler::Scheduler,
    time::{Deadline, Span, Tick},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Unsupported,
    Interrupted,
    Capacity,
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Copy, Debug)]
pub enum Realtime {
    HostUnix,
    Fixed(u64),
}
impl Realtime {
    pub fn now(self) -> Result<u64> {
        match self {
            Self::Fixed(n) => Ok(n),
            Self::HostUnix => u64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| Error::Invalid)?
                    .as_nanos(),
            )
            .map_err(|_| Error::Invalid),
        }
    }
}
pub fn units(value: u64, nanos_per_unit: u64) -> Result<Span> {
    value
        .checked_mul(nanos_per_unit)
        .map(Span::from_nanos)
        .ok_or(Error::Invalid)
}
pub fn timespec(t: Timespec) -> Result<Span> {
    t.total_nanos()
        .map(Span::from_nanos)
        .map_err(|_| Error::Invalid)
}
/// Supported FreeBSD-derived ID aliases from prototype; quantization is guest policy.
pub fn quantum(id: i32) -> Result<u64> {
    match id {
        0 | 4 | 5 | 7 | 9 | 11 => Ok(1_000),
        8 | 10 | 12 => Ok(1_000_000),
        13 => Ok(1_000_000_000),
        1 | 2 | 14 | 15 => Err(Error::Unsupported),
        _ => Err(Error::Invalid),
    }
}
pub fn query(s: &Scheduler, wall: Realtime, id: i32) -> Result<u64> {
    let q = quantum(id)?;
    let n = match id {
        0 | 9 | 10 | 13 => wall.now()?,
        _ => s.now().map_err(|_| Error::Interrupted)?.as_nanos(),
    };
    Ok(n / q * q)
}
/// Wall absolute waits snapshot the current wall-to-monotonic relation; later wall jumps do not retime.
pub fn absolute(s: &Scheduler, wall: Realtime, t: Timespec, id: i32) -> Result<Deadline> {
    let target = timespec(t)?.as_nanos();
    let now = s.now().map_err(|_| Error::Interrupted)?;
    let delta = match id {
        0 => target.saturating_sub(wall.now()?),
        4 => target.saturating_sub(now.as_nanos()),
        _ => return Err(Error::Invalid),
    };
    Deadline::after(now, Span::from_nanos(delta)).map_err(|_| Error::Invalid)
}
pub const TSC_FREQUENCY: u64 = 3_500_000_000;
pub fn virtual_tsc(t: Tick) -> Result<u64> {
    u64::try_from(t.as_nanos() as u128 * TSC_FREQUENCY as u128 / 1_000_000_000)
        .map_err(|_| Error::Invalid)
}

pub fn virtual_tsc_nanos(n: u64) -> Result<u64> {
    virtual_tsc(Tick::from_nanos(n))
}
