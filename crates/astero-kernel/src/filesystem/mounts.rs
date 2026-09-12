//! Explicit mount policy. Canonical checks contain links; this is not a hostile-input sandbox.
use super::service::Error;
use std::path::{Path, PathBuf};
#[derive(Clone, Debug)]
pub struct Mounts {
    pub code: PathBuf,
    pub data: PathBuf,
    pub writable: Option<PathBuf>,
}
impl Mounts {
    pub fn detect(executable: &Path, explicit: Option<PathBuf>) -> Result<Self, Error> {
        let code = explicit
            .clone()
            .or_else(|| executable.parent().map(Path::to_path_buf))
            .ok_or(Error::Invalid)?
            .canonicalize()
            .map_err(Error::from)?;
        let shape = |p: &Path| p.join("sce_sys").is_dir() || p.join("sce_module").is_dir();
        let data = if explicit.is_none() && shape(&code) {
            code.parent()
                .filter(|p| shape(p))
                .unwrap_or(&code)
                .to_path_buf()
        } else {
            code.clone()
        };
        Ok(Self {
            code,
            data,
            writable: None,
        })
    }
    pub fn resolve(&self, guest: &str, write: bool) -> Result<PathBuf, Error> {
        if guest.len() > 4096 || guest.contains(['\\', ':', '\0']) {
            return Err(Error::Invalid);
        }
        let (root, tail, overlay) = if let Some(t) = guest.strip_prefix("/app0/") {
            (&self.data, t, true)
        } else if guest == "/app0" {
            (&self.data, "", true)
        } else if let Some(t) = guest.strip_prefix("/data/") {
            (self.writable.as_ref().ok_or(Error::Permission)?, t, false)
        } else if !guest.starts_with('/') {
            (&self.data, guest, true)
        } else {
            return Err(Error::NotFound);
        };
        if write && overlay {
            return Err(Error::Permission);
        }
        let mut relative = PathBuf::new();
        for part in tail.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    if !relative.pop() {
                        return Err(Error::Permission);
                    }
                }
                _ => {
                    if part.ends_with([' ', '.']) {
                        return Err(Error::Invalid);
                    }
                    let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
                    if ["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
                        || (stem.len() == 4
                            && (stem.starts_with("COM") || stem.starts_with("LPT"))
                            && stem.as_bytes()[3].is_ascii_digit())
                    {
                        return Err(Error::Invalid);
                    }
                    relative.push(part);
                }
            }
        }
        let root = if overlay && self.code.join(&relative).exists() {
            &self.code
        } else {
            root
        };
        let root = root.canonicalize().map_err(Error::from)?;
        let candidate = root.join(relative);
        let check = if candidate.exists() {
            candidate.clone()
        } else {
            candidate.parent().ok_or(Error::Invalid)?.to_path_buf()
        };
        if !check
            .canonicalize()
            .map_err(Error::from)?
            .starts_with(&root)
        {
            return Err(Error::Permission);
        }
        Ok(candidate)
    }
}
