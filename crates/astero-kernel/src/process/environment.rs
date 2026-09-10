//! Process-owned bounded environment identities; guest storage remains memory-owned.
pub struct Environment {
    entries: Vec<(Vec<u8>, u64)>,
}
impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
impl Environment {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    pub fn get(&self, name: &[u8]) -> Option<u64> {
        self.entries
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, a)| *a)
    }
    pub fn can_insert(&self, name: &[u8]) -> bool {
        self.get(name).is_some() || self.entries.len() < 128
    }
    pub fn put(&mut self, name: Vec<u8>, address: u64) -> Result<Option<u64>, &'static str> {
        if name.is_empty() || name.contains(&b'=') || name.len() > 8192 || address == 0 {
            return Err("invalid environment entry");
        }
        if let Some((_, a)) = self.entries.iter_mut().find(|(n, _)| n == &name) {
            return Ok(Some(std::mem::replace(a, address)));
        }
        if !self.can_insert(&name) {
            return Err("environment capacity");
        }
        self.entries
            .try_reserve(1)
            .map_err(|_| "environment allocation")?;
        self.entries.push((name, address));
        Ok(None)
    }
}
