//! Process-owned bounded diagnostic output, not a filesystem or host stdio handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputError {
    Capacity,
    Allocation,
}
pub struct Output {
    bytes: Vec<u8>,
    maximum: usize,
}
impl Output {
    pub fn new(maximum: usize) -> Self {
        Self {
            bytes: Vec::new(),
            maximum,
        }
    }
    pub fn append(&mut self, b: &[u8]) -> Result<(), OutputError> {
        if self
            .bytes
            .len()
            .checked_add(b.len())
            .is_none_or(|n| n > self.maximum)
        {
            return Err(OutputError::Capacity);
        }
        self.bytes
            .try_reserve(b.len())
            .map_err(|_| OutputError::Allocation)?;
        self.bytes.extend_from_slice(b);
        Ok(())
    }
    pub fn append_line(&mut self, b: &[u8]) -> Result<(), OutputError> {
        let n = b.len().checked_add(1).ok_or(OutputError::Capacity)?;
        if self
            .bytes
            .len()
            .checked_add(n)
            .is_none_or(|v| v > self.maximum)
        {
            return Err(OutputError::Capacity);
        }
        self.bytes
            .try_reserve(n)
            .map_err(|_| OutputError::Allocation)?;
        self.bytes.extend_from_slice(b);
        self.bytes.push(b'\n');
        Ok(())
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
