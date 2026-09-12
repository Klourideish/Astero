//! Bounded immutable host-call registry. Not native import installation or guest dispatch.
use astero_abi::layouts::entry::CallFrame;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderKey {
    pub nid: u64,
    pub library: Vec<u8>,
    pub module: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderKind {
    SyntheticHostTest,
    HleImplementation,
    ArtifactDeclaration,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallResult {
    Returned,
    StopRequested,
    Unsupported,
    FormatFailure { offset: usize, reason: &'static str },
    AccessFailure(crate::calls::memory::AccessError),
}
pub type HostHandler =
    dyn Fn(&mut CallFrame, &mut dyn crate::calls::memory::GuestMemory) -> CallResult;
pub struct Registration {
    pub key: ProviderKey,
    pub kind: ProviderKind,
    pub handler: Option<Box<HostHandler>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryError {
    Capacity,
    IdentityTooLong,
    InvalidHandler,
    Missing,
    Ambiguous,
    NoHostHandler,
    HandlerPanicked,
}
pub struct PreparedRegistry {
    entries: Vec<Registration>,
}
impl PreparedRegistry {
    pub fn new(entries: Vec<Registration>, maximum: usize) -> Result<Self, RegistryError> {
        if entries.len() > maximum {
            return Err(RegistryError::Capacity);
        }
        for e in &entries {
            if e.key.library.len() > 256 || e.key.module.len() > 256 {
                return Err(RegistryError::IdentityTooLong);
            }
            if (e.kind == ProviderKind::ArtifactDeclaration) != e.handler.is_none() {
                return Err(RegistryError::InvalidHandler);
            }
        }
        Ok(Self { entries })
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Exact identity count, independent of aliases sharing a handler or duplicate registrations.
    pub fn identity_count(&self) -> usize {
        self.entries
            .iter()
            .map(|e| (&e.key.module, &e.key.library, e.key.nid))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn find(&self, key: &ProviderKey) -> Result<&Registration, RegistryError> {
        let mut found = self.entries.iter().filter(|e| e.key == *key);
        let entry = found.next().ok_or(RegistryError::Missing)?;
        if found.next().is_some() {
            return Err(RegistryError::Ambiguous);
        }
        Ok(entry)
    }
    pub fn invoke_host_model(
        &self,
        key: &ProviderKey,
        frame: &mut CallFrame,
    ) -> Result<CallResult, RegistryError> {
        self.invoke(key, frame, &mut crate::calls::memory::Unavailable)
    }
    pub fn invoke(
        &self,
        key: &ProviderKey,
        frame: &mut CallFrame,
        memory: &mut dyn crate::calls::memory::GuestMemory,
    ) -> Result<CallResult, RegistryError> {
        let f = self
            .find(key)?
            .handler
            .as_ref()
            .ok_or(RegistryError::NoHostHandler)?;
        let mut local = frame.clone();
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut local, memory)))
                .map_err(|_| RegistryError::HandlerPanicked)?;
        // Arguments are observations; providers publish return lanes only.
        frame.rax = local.rax;
        frame.xmm0 = local.xmm0;
        Ok(result)
    }
}
