use anyhow::{Context, Result, ensure};
use deck_domain::artwork::{Clip, Frame, set_pixel};
use image::{AnimationDecoder, DynamicImage, GenericImageView, ImageDecoder, Limits};
use std::{fs::File, io::BufReader, path::Path};
fn bitmap(im: DynamicImage) -> Vec<u8> {
    let resized = im.thumbnail(128, 50).to_rgba8();
    let mut b = vec![0; 1024];
    let ox = (128 - resized.width()) / 2;
    let oy = 14 + (50 - resized.height()) / 2;
    for (x, y, p) in resized.enumerate_pixels() {
        let lum = (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32)
            * p[3] as f32
            / 255.0;
        set_pixel(&mut b, (x + ox) as usize, (y + oy) as usize, lum >= 128.0);
    }
    b
}
pub fn load_clip(path: &Path) -> Result<Clip> {
    ensure!(
        path.metadata()?.len() <= 8 * 1024 * 1024,
        "Χρησιμοποίησε εικόνα/GIF έως 8 MB."
    );
    let mut limits = Limits::default();
    limits.max_image_width = Some(2048);
    limits.max_image_height = Some(2048);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = image::ImageReader::open(path)?.with_guessed_format()?;
    reader.limits(limits.clone());
    let clip = if reader.format() == Some(image::ImageFormat::Gif) {
        let mut d = image::codecs::gif::GifDecoder::new(BufReader::new(File::open(path)?))?;
        limits.max_image_width = Some(512);
        limits.max_image_height = Some(512);
        limits.max_alloc = Some(16 * 1024 * 1024);
        d.set_limits(limits)?;
        let frames = d
            .into_frames()
            .take(25)
            .collect::<image::ImageResult<Vec<_>>>()
            .context("Δεν αποκωδικοποιήθηκε το GIF.")?;
        ensure!(
            (1..=24).contains(&frames.len()),
            "Χρειάζεται μικρό GIF με 1–24 καρέ. Δεν έγινε σιωπηλή περικοπή."
        );
        Clip {
            frames: frames
                .into_iter()
                .map(|f| {
                    let (n, d) = f.delay().numer_denom_ms();
                    let ms = n.checked_div(d).unwrap_or(125);
                    Frame {
                        bitmap: bitmap(DynamicImage::ImageRgba8(f.into_buffer())),
                        duration_ms: (((ms + 62) / 125) * 125).clamp(125, 2000) as u16,
                    }
                })
                .collect(),
        }
    } else {
        let im = reader.decode()?;
        let (w, h) = im.dimensions();
        ensure!(w <= 2048 && h <= 2048, "Η εικόνα ξεπερνά τα 2048×2048.");
        Clip {
            frames: vec![Frame {
                bitmap: bitmap(im),
                duration_ms: 125,
            }],
        }
    };
    clip.validate()?;
    Ok(clip)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_fits_below_mode_header() {
        let b = bitmap(DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            20,
            20,
            image::Rgba([255, 255, 255, 255]),
        )));
        assert!(b[..14 * 16].iter().all(|b| *b == 0));
        assert!(b.iter().any(|b| *b != 0));
    }
}
