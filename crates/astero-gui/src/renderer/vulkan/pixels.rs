//! Bounded CPU host-test raster adapter, independent of ImGui and guest GPU mechanisms.
use astero_video::presentation::{Backing, Error, Format, Frame};
pub(super) struct Run {
    pub color: [f32; 4],
    x: u32,
    y: u32,
    width: u32,
    source: [u32; 2],
}
impl Run {
    pub fn rect(&self, w: u32, h: u32) -> [u32; 4] {
        let x = u64::from(self.x) * w as u64 / self.source[0] as u64;
        let y = u64::from(self.y) * h as u64 / self.source[1] as u64;
        let right = u64::from(self.x + self.width) * w as u64 / self.source[0] as u64;
        let bottom = u64::from(self.y + 1) * h as u64 / self.source[1] as u64;
        [x as u32, y as u32, (right - x) as u32, (bottom - y) as u32]
    }
}
pub(super) fn runs(f: &Frame) -> Result<Vec<Run>, Error> {
    f.validate()?;
    let Backing::Cpu(bytes) = &f.backing else {
        return Err(Error::Unsupported);
    };
    let mut out = Vec::new();
    for y in 0..f.height {
        let mut x = 0;
        while x < f.width {
            if out.len() == 16384 {
                return Err(Error::Unsupported);
            }
            let at = (y * f.stride + x * 4) as usize;
            let b = &bytes[at..at + 4];
            let mut width = 1;
            while x + width < f.width
                && bytes[at + width as usize * 4..at + width as usize * 4 + 4] == *b
            {
                width += 1;
            }
            let (r, bv) = if f.format == Format::Rgba8 {
                (b[0], b[2])
            } else {
                (b[2], b[0])
            };
            out.push(Run {
                color: [
                    r as f32 / 255.,
                    b[1] as f32 / 255.,
                    bv as f32 / 255.,
                    b[3] as f32 / 255.,
                ],
                x,
                y,
                width,
                source: [f.width, f.height],
            });
            x += width;
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn synthetic_runs_cover_scaled_output() {
        let r = runs(&astero_video::presentation::synthetic(1)).unwrap();
        assert_eq!(r.iter().map(|r| r.width as u64).sum::<u64>(), 640 * 360);
        assert_eq!(
            r.iter()
                .map(|r| {
                    let a = r.rect(800, 450);
                    a[2] as u64 * a[3] as u64
                })
                .sum::<u64>(),
            800 * 450
        );
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
    #[test]
    fn bgra_and_row_padding_are_decoded_explicitly() {
        let f = Frame {
            id: 1,
            width: 1,
            height: 2,
            stride: 8,
            format: Format::Bgra8,
            timestamp_ns: None,
            backing: Backing::Cpu(
                vec![0, 0, 255, 255, 9, 9, 9, 9, 255, 0, 0, 255, 8, 8, 8, 8].into(),
            ),
        };
        let r = runs(&f).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].color, [1., 0., 0., 1.]);
        assert_eq!(r[1].color, [0., 0., 1., 1.]);
        assert_eq!(r[0].rect(0, 0), [0, 0, 0, 0]);
    }
    #[test]
    fn complex_cpu_frames_refuse_instead_of_truncating() {
        let mut b = vec![0; 256 * 256 * 4];
        for (i, p) in b.chunks_exact_mut(4).enumerate() {
            p[0] = (i % 2) as u8;
        }
        let f = Frame {
            id: 1,
            width: 256,
            height: 256,
            stride: 1024,
            format: Format::Rgba8,
            timestamp_ns: None,
            backing: Backing::Cpu(b.into()),
        };
        assert!(matches!(runs(&f), Err(Error::Unsupported)));
    }
}
