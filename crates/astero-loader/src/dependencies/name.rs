/// Owned byte identity, not a host path, normalized text, or resolved module.
/// ```compile_fail
/// use astero_loader::dependencies::DependencyName;
/// let name = DependencyName::new(b"lib".to_vec()).unwrap();
/// name.as_bytes()[0] = 0;
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyName(Box<[u8]>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencyNameError {
    Empty,
    InteriorNul { index: usize },
}
impl DependencyName {
    pub fn new(bytes: Vec<u8>) -> Result<Self, DependencyNameError> {
        if bytes.is_empty() {
            return Err(DependencyNameError::Empty);
        }
        if let Some(index) = bytes.iter().position(|b| *b == 0) {
            return Err(DependencyNameError::InteriorNul { index });
        }
        Ok(Self(bytes.into_boxed_slice()))
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    /// Optional explicit text validation. Failed conversion never changes byte identity.
    pub fn as_utf8(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.0)
    }
}
impl std::fmt::Display for DependencyNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dependency name: {self:?}")
    }
}
impl std::error::Error for DependencyNameError {}
