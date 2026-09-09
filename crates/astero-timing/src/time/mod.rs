//! Nanosecond ticks relative to one engine's clock origin, never calendar/guest time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeError {
    Overflow,
    BeforeOrigin,
}
impl std::fmt::Display for TimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "time arithmetic: {self:?}")
    }
}
impl std::error::Error for TimeError {}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span(u64);
impl Span {
    pub const ZERO: Self = Self(0);
    pub const fn from_nanos(n: u64) -> Self {
        Self(n)
    }
    pub fn from_millis(n: u64) -> Result<Self, TimeError> {
        n.checked_mul(1_000_000)
            .map(Self)
            .ok_or(TimeError::Overflow)
    }
    pub fn from_std(d: std::time::Duration) -> Result<Self, TimeError> {
        u64::try_from(d.as_nanos())
            .map(Self)
            .map_err(|_| TimeError::Overflow)
    }
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
    pub fn as_std(self) -> std::time::Duration {
        std::time::Duration::from_nanos(self.0)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(u64);
impl Tick {
    pub const ZERO: Self = Self(0);
    pub const fn from_nanos(n: u64) -> Self {
        Self(n)
    }
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
    pub fn checked_add(self, span: Span) -> Result<Self, TimeError> {
        self.0
            .checked_add(span.0)
            .map(Self)
            .ok_or(TimeError::Overflow)
    }
    pub fn elapsed_since(self, earlier: Self) -> Result<Span, TimeError> {
        self.0
            .checked_sub(earlier.0)
            .map(Span)
            .ok_or(TimeError::BeforeOrigin)
    }
}
/// Explicitly interpreted in the receiving engine's epoch. Do not mix different clock origins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(Tick);
impl Deadline {
    pub const fn at(t: Tick) -> Self {
        Self(t)
    }
    pub const fn tick(self) -> Tick {
        self.0
    }
    pub fn after(now: Tick, delay: Span) -> Result<Self, TimeError> {
        now.checked_add(delay).map(Self)
    }
    pub fn is_due(self, now: Tick) -> bool {
        self.0 <= now
    }
    pub fn remaining(self, now: Tick) -> Span {
        if self.is_due(now) {
            Span::ZERO
        } else {
            Span(self.0.0 - now.0)
        }
    }
}
