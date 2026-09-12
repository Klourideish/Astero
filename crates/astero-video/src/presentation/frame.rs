use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Rgba8,
    Bgra8,
}
/// A future backend-specific resource lease. IDs are scoped to its owner, never guest addresses.
pub trait HostResource: Send + Sync {
    fn identity(&self) -> u64;
}
#[derive(Clone)]
pub enum Backing {
    Cpu(Arc<[u8]>),
    Host(Arc<dyn HostResource>),
}
#[derive(Clone)]
pub struct Frame {
    pub id: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: Format,
    pub timestamp_ns: Option<u64>,
    pub backing: Backing,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Dimensions,
    Stride,
    Backing,
    Unsupported,
    Order,
    Closed,
    Unavailable,
    Host(String),
}
impl Frame {
    pub fn validate(&self) -> Result<(), Error> {
        if self.width == 0 || self.height == 0 || self.width > 4096 || self.height > 2160 {
            return Err(Error::Dimensions);
        }
        if self.stride < self.width * 4 || !self.stride.is_multiple_of(4) {
            return Err(Error::Stride);
        }
        let bytes = u64::from(self.stride) * u64::from(self.height);
        if bytes > 64 * 1024 * 1024 {
            return Err(Error::Backing);
        }
        if let Backing::Cpu(b) = &self.backing
            && b.len() as u64 != bytes
        {
            return Err(Error::Backing);
        }
        Ok(())
    }
}
/// Deterministic host test pattern, explicitly unrelated to PS5 VideoOut formats.
pub fn synthetic(id: u64) -> Frame {
    let (w, h) = (640, 360);
    let mut bytes = vec![0; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let phase = (x / 80 + y / 60 + id as usize % 2).is_multiple_of(2);
            let c = if phase {
                [240, 40, 60, 255]
            } else {
                [20, 180, 230, 255]
            };
            bytes[(y * w + x) * 4..(y * w + x + 1) * 4].copy_from_slice(&c);
        }
    }
    Frame {
        id,
        width: w as u32,
        height: h as u32,
        stride: w as u32 * 4,
        format: Format::Rgba8,
        timestamp_ns: None,
        backing: Backing::Cpu(bytes.into()),
    }
}
