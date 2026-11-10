//! Videos de Smiling Friends empaquetados por `tools/video/prepare_smilingfriends_intro.py`: cuadros JPEG uno tras
//! otro (formato `SFV1`), que se decodifican de a uno sobre una sola textura.
use raylib::prelude::*;
use std::path::Path;

pub struct Frames {
    bytes: Vec<u8>,
    spans: Vec<(usize, usize)>,
    width: i32,
    height: i32,
    fps: u32,
}

impl Frames {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        if bytes.len() < 20 || &bytes[..4] != b"SFV1" {
            return Err(format!("Invalid Smiling Friends video: {}", path.display()).into());
        }
        let field = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let (width, height, fps, count) = (field(4), field(8), field(12), field(16));
        if !(1..=4096).contains(&width)
            || !(1..=4096).contains(&height)
            || !(1..=120).contains(&fps)
            || !(1..=10_000).contains(&count)
        {
            return Err("Invalid Smiling Friends video metadata".into());
        }
        let mut spans = Vec::with_capacity(count as usize);
        let mut offset = 20usize;
        for _ in 0..count {
            let length_end = offset
                .checked_add(4)
                .filter(|v| *v <= bytes.len())
                .ok_or("Truncated Smiling Friends video")?;
            let len = u32::from_le_bytes(bytes[offset..length_end].try_into().unwrap()) as usize;
            offset = length_end;
            let end = offset
                .checked_add(len)
                .filter(|v| *v <= bytes.len())
                .ok_or("Truncated Smiling Friends frame")?;
            spans.push((offset, end));
            offset = end;
        }
        if offset != bytes.len() {
            return Err("Unexpected data after Smiling Friends video".into());
        }
        Ok(Self {
            bytes,
            spans,
            width: width as i32,
            height: height as i32,
            fps,
        })
    }

    pub fn image(&self, frame: usize) -> Result<Image, Box<dyn std::error::Error>> {
        let (start, end) = self.spans[frame];
        let image = Image::load_image_from_mem(".jpg", &self.bytes[start..end])?;
        if image.width != self.width || image.height != self.height {
            return Err(format!("Smiling Friends frame {frame} has an unexpected size").into());
        }
        Ok(image)
    }

    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn duration(&self) -> f32 {
        self.spans.len() as f32 / self.fps as f32
    }
}

pub fn update_frame(
    texture: &mut Texture2D,
    frames: &Frames,
    frame: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let image = frames.image(frame)?;
    if image.format != texture.format {
        return Err("Smiling Friends frames have inconsistent pixel formats".into());
    }
    let pixels =
        unsafe { std::slice::from_raw_parts(image.data.cast::<u8>(), image.get_pixel_data_size()) };
    texture.update_texture(pixels)?;
    Ok(())
}
