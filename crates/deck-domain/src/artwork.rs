use crate::{ValidationError, require};
use serde::{Deserialize, Serialize};
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 64;
pub const HEADER: usize = 14;
pub const FRAME_BYTES: usize = 1024;
pub const MAX_FRAMES: usize = 24;
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub bitmap: Vec<u8>,
    pub duration_ms: u16,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            bitmap: vec![0; FRAME_BYTES],
            duration_ms: 125,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub frames: Vec<Frame>,
}
impl Default for Clip {
    fn default() -> Self {
        Self {
            frames: vec![Frame::default()],
        }
    }
}
impl Clip {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require(
            (1..=MAX_FRAMES).contains(&self.frames.len()),
            "Χρειάζονται 1–24 καρέ.",
        )?;
        require(
            self.frames.iter().all(|f| {
                f.bitmap.len() == FRAME_BYTES
                    && (125..=2000).contains(&f.duration_ms)
                    && f.duration_ms % 125 == 0
            }),
            "Καρέ: 1024 bytes, διάρκεια 125–2000 ms σε βήματα 125 ms.",
        )
    }
    pub fn pack(&self) -> Result<Vec<u8>, ValidationError> {
        self.validate()?;
        let mut b = vec![0; 12 + self.frames.len() * 1026];
        b[..6].copy_from_slice(&[67, 68, 65, 78, 1, self.frames.len() as u8]);
        for (i, f) in self.frames.iter().enumerate() {
            let start = 12 + i * 1026;
            b[start..start + 2].copy_from_slice(&f.duration_ms.to_le_bytes());
            b[start + 2..start + 1026].copy_from_slice(&f.bitmap);
        }
        let crc = crc32(&b[12..]);
        b[8..12].copy_from_slice(&crc.to_le_bytes());
        Ok(b)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artwork {
    pub version: u32,
    pub keys: Vec<Clip>,
}
impl Default for Artwork {
    fn default() -> Self {
        Self {
            version: 1,
            keys: vec![Clip::default(); 9],
        }
    }
}
impl Artwork {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require(
            self.version == 1 && self.keys.len() == 9,
            "Το artwork χρειάζεται έκδοση 1 και 9 πλήκτρα.",
        )?;
        for c in &self.keys {
            c.validate()?;
        }
        Ok(())
    }
}
/// Firmware bitmap layout: row-major, 16 bytes per row, MSB first. Not SSD1306 native page layout.
pub fn pixel(frame: &[u8], x: usize, y: usize) -> bool {
    x < WIDTH
        && y < HEIGHT
        && frame
            .get(y * 16 + x / 8)
            .is_some_and(|v| v & (0x80 >> (x % 8)) != 0)
}
pub fn set_pixel(frame: &mut [u8], x: usize, y: usize, on: bool) {
    if x < WIDTH
        && (HEADER..HEIGHT).contains(&y)
        && let Some(v) = frame.get_mut(y * 16 + x / 8)
    {
        let m = 0x80 >> (x % 8);
        if on { *v |= m } else { *v &= !m }
    }
}
pub fn crc32(bytes: &[u8]) -> u32 {
    let mut c = u32::MAX;
    for b in bytes {
        c ^= *b as u32;
        for _ in 0..8 {
            c = (c >> 1) ^ if c & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    !c
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn row_layout_and_header() {
        let mut f = vec![0; 1024];
        set_pixel(&mut f, 10, 14, true);
        assert!(pixel(&f, 10, 14));
        assert_eq!(f[14 * 16 + 1], 32);
        set_pixel(&mut f, 10, 0, true);
        assert!(!pixel(&f, 10, 0));
        set_pixel(&mut f, 10, 14, false);
        assert!(!pixel(&f, 10, 14));
    }
    #[test]
    fn bounded_frames() {
        let mut a = Artwork::default();
        a.validate().unwrap();
        a.keys[0].frames[0].bitmap.pop();
        assert!(a.validate().is_err());
    }
    #[test]
    fn known_crc_and_blob() {
        assert_eq!(crc32(b"123456789"), 0xcbf43926);
        let c = Clip::default();
        let b = c.pack().unwrap();
        assert_eq!(b.len(), 1038);
        assert_eq!(&b[..6], b"CDAN\x01\x01");
        assert_eq!(&b[12..14], &125u16.to_le_bytes());
        assert_eq!(
            u32::from_le_bytes(b[8..12].try_into().unwrap()),
            crc32(&b[12..])
        );
    }
}
