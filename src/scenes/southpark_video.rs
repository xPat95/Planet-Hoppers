//! The supplied South Park clip, prepared offline as SFV1 JPEGs + Ogg audio.
//! Only the displayed JPEG is decoded; the audio controller owns the one clock.
use super::button_style;
use raylib::prelude::*;
use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

pub const VIDEO: &str = "assets/video/south park/intro.sfv";
pub const AUDIO: &str = "assets/video/south park/intro.ogg";
type Error = Box<dyn std::error::Error>;

pub struct VideoFrames {
    bytes: Vec<u8>,
    file: Option<RefCell<File>>,
    spans: Vec<(usize, usize)>,
    pub width: i32,
    pub height: i32,
    pub fps: u32,
}

impl VideoFrames {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let mut file = std::fs::File::open(path).map_err(|e| {
            format!(
                "{}: {e}. Prepare el MP4 con python tools/video/prepare_southpark_intro.py",
                path.display()
            )
        })?;
        // Bound memory before allocating, including for a corrupt/untrusted pack.
        if file.metadata()?.len() > 256 * 1024 * 1024 {
            return Err("South Park video pack exceeds 256 MiB".into());
        }
        let length = file.metadata()?.len();
        let mut header = [0u8; 20];
        file.read_exact(&mut header)?;
        // Reuse metadata validation without retaining the encoded frames.
        let field = |at| u32::from_le_bytes(header[at..at + 4].try_into().unwrap());
        let (width, height, fps, count) = (field(4), field(8), field(12), field(16));
        if &header[..4] != b"SFV1"
            || !(1..=1920).contains(&width)
            || !(1..=1080).contains(&height)
            || !(1..=60).contains(&fps)
            || !(1..=10_000).contains(&count)
        {
            return Err("Invalid South Park video metadata".into());
        }
        let mut spans = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut size = [0u8; 4];
            file.read_exact(&mut size)?;
            let size = u32::from_le_bytes(size) as u64;
            let start = file.stream_position()?;
            let end = start.checked_add(size).ok_or("Invalid frame length")?;
            if size == 0 || end > length {
                return Err("Truncated or empty video frame".into());
            }
            spans.push((start as usize, end as usize));
            file.seek(SeekFrom::Start(end))?;
        }
        if file.stream_position()? != length {
            return Err("Unexpected video data".into());
        }
        Ok(Self {
            bytes: vec![],
            file: Some(RefCell::new(file)),
            spans,
            width: width as i32,
            height: height as i32,
            fps,
        })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        if bytes.len() < 20 || &bytes[..4] != b"SFV1" {
            return Err("Invalid South Park video header".into());
        }
        let field = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let (width, height, fps, count) = (field(4), field(8), field(12), field(16));
        if !(1..=1920).contains(&width)
            || !(1..=1080).contains(&height)
            || !(1..=60).contains(&fps)
            || !(1..=10_000).contains(&count)
        {
            return Err("Invalid South Park video metadata".into());
        }
        let mut spans = Vec::with_capacity(count as usize);
        let mut offset = 20usize;
        for _ in 0..count {
            let end = offset
                .checked_add(4)
                .filter(|v| *v <= bytes.len())
                .ok_or("Truncated video frame length")?;
            let length = u32::from_le_bytes(bytes[offset..end].try_into().unwrap()) as usize;
            offset = end;
            let end = offset
                .checked_add(length)
                .filter(|v| *v <= bytes.len())
                .ok_or("Truncated video frame")?;
            if length == 0 {
                return Err("Empty video frame".into());
            }
            spans.push((offset, end));
            offset = end;
        }
        if offset != bytes.len() {
            return Err("Unexpected data after video frames".into());
        }
        Ok(Self {
            bytes,
            file: None,
            spans,
            width: width as i32,
            height: height as i32,
            fps,
        })
    }

    pub fn duration(&self) -> f32 {
        self.spans.len() as f32 / self.fps as f32
    }
    pub fn frame_at(&self, seconds: f32) -> usize {
        ((seconds.max(0.0) * self.fps as f32) as usize).min(self.spans.len() - 1)
    }
    fn image(&self, frame: usize) -> Result<Image, Error> {
        let (start, end) = self.spans[frame];
        let mut encoded = Vec::new();
        let data = if let Some(file) = &self.file {
            let mut file = file.borrow_mut();
            file.seek(SeekFrom::Start(start as u64))?;
            encoded.resize(end - start, 0);
            file.read_exact(&mut encoded)?;
            &encoded[..]
        } else {
            &self.bytes[start..end]
        };
        let image = Image::load_image_from_mem(".jpg", data)?;
        if image.width != self.width || image.height != self.height {
            return Err("South Park video frame dimensions changed".into());
        }
        Ok(image)
    }
}

pub struct VideoTexture {
    frames: VideoFrames,
    texture: Texture2D,
    current: usize,
}

impl VideoTexture {
    /// World-space cylindrical billboard, centered on Ike rather than the screen.
    pub fn draw_world(
        &self,
        view: &mut RaylibMode3D<RaylibDrawHandle>,
        camera: Camera3D,
        feet: Vector3,
    ) {
        let aspect = self.frames.width as f32 / self.frames.height as f32;
        let height = 1.05f32.min(1.65 / aspect);
        let width = height * aspect;
        let position = feet + Vector3::up() * 0.48;
        let level_camera = Camera3D::perspective(
            Vector3::new(camera.position.x, position.y, camera.position.z),
            position,
            Vector3::up(),
            camera.fovy,
        );
        view.draw_billboard_pro(
            level_camera,
            *self.texture.as_ref(),
            Rectangle::new(
                0.0,
                0.0,
                self.frames.width as f32,
                self.frames.height as f32,
            ),
            position,
            Vector3::up(),
            Vector2::new(width, height),
            Vector2::new(width * 0.5, height * 0.5),
            0.0,
            Color::WHITE,
        );
    }
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, path: &Path) -> Result<Self, Error> {
        let frames = VideoFrames::load(path)?;
        let image = frames.image(0)?;
        let texture = rl.load_texture_from_image(thread, &image)?;
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(Self {
            frames,
            texture,
            current: 0,
        })
    }
    pub fn duration(&self) -> f32 {
        self.frames.duration()
    }
    pub fn update(&mut self, seconds: f32) -> Result<(), Error> {
        let frame = self.frames.frame_at(seconds);
        if frame != self.current {
            let image = self.frames.image(frame)?;
            if image.format != self.texture.format {
                return Err("Video pixel format changed".into());
            }
            let pixels = unsafe {
                std::slice::from_raw_parts(image.data.cast::<u8>(), image.get_pixel_data_size())
            };
            self.texture.update_texture(pixels)?;
            self.current = frame;
        }
        Ok(())
    }
}

pub struct VideoIntro {
    clip: VideoTexture,
    font: Font,
}
impl VideoIntro {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Result<Self, Error> {
        let clip = VideoTexture::load(rl, thread, &root.join(VIDEO))?;
        let font = rl.load_font_ex(
            thread,
            &root
                .join("assets/fonts/Bungee-Regular.ttf")
                .to_string_lossy(),
            64,
            None,
        )?;
        Ok(Self { clip, font })
    }
    pub fn duration(&self) -> f32 {
        self.clip.duration()
    }
    pub fn update(&mut self, seconds: f32) -> Result<(), Error> {
        self.clip.update(seconds)
    }
    pub fn draw(&self, d: &mut RaylibDrawHandle, w: i32, h: i32, mouse: Vector2) {
        let (wf, hf) = (w as f32, h as f32);
        d.clear_background(Color::new(159, 201, 214, 255));
        // One cut-paper cloud and a quiet snow bank; the frame is the focal point.
        let cloud = Color::new(226, 238, 229, 255);
        d.draw_circle_v(Vector2::new(wf * 0.88, hf * 0.105), hf * 0.038, cloud);
        d.draw_circle_v(Vector2::new(wf * 0.92, hf * 0.105), hf * 0.052, cloud);
        d.draw_rectangle(
            (wf * 0.85) as i32,
            (hf * 0.104) as i32,
            (wf * 0.11) as i32,
            (hf * 0.035) as i32,
            cloud,
        );
        polygon(
            d,
            &[
                Vector2::new(0.0, hf * 0.89),
                Vector2::new(wf * 0.3, hf * 0.87),
                Vector2::new(wf * 0.7, hf * 0.90),
                Vector2::new(wf, hf * 0.86),
                Vector2::new(wf, hf),
                Vector2::new(0.0, hf),
            ],
            Color::new(224, 231, 213, 255),
        );
        let title_size = (hf * 0.04).clamp(18.0, 32.0);
        d.draw_text_ex(
            &self.font,
            "SOUTH PARK",
            Vector2::new(wf * 0.065, hf * 0.035),
            title_size,
            1.0,
            Color::new(70, 61, 45, 255),
        );
        let rect = video_rect(w, h, self.clip.frames.width, self.clip.frames.height);
        let border = (hf * 0.021).clamp(8.0, 19.0);
        let paper = Rectangle::new(
            rect.x - border,
            rect.y - border,
            rect.width + border * 2.0,
            rect.height + border * 2.0,
        );
        cutout(
            d,
            Rectangle::new(paper.x + 5.0, paper.y + 7.0, paper.width, paper.height),
            Color::new(93, 106, 100, 100),
        );
        cutout(d, paper, Color::new(128, 91, 52, 255));
        cutout(
            d,
            Rectangle::new(
                paper.x + 3.0,
                paper.y + 3.0,
                paper.width - 6.0,
                paper.height - 6.0,
            ),
            Color::new(202, 163, 108, 255),
        );
        d.draw_texture_pro(
            &self.clip.texture,
            Rectangle::new(
                0.0,
                0.0,
                self.clip.frames.width as f32,
                self.clip.frames.height as f32,
            ),
            rect,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        // Two short pieces of masking tape outside the picture.
        for x in [paper.x + border, paper.x + paper.width - border * 4.0] {
            polygon(
                d,
                &[
                    Vector2::new(x, paper.y - 5.0),
                    Vector2::new(x + border * 3.2, paper.y - 1.0),
                    Vector2::new(x + border * 3.0, paper.y + border * 0.6),
                    Vector2::new(x - 2.0, paper.y + border * 0.35),
                ],
                Color::new(228, 209, 162, 245),
            );
        }
        let skip = skip_button(w, h);
        button_style::draw(
            d,
            skip,
            "SKIP >>",
            (skip.height * 0.43).min(23.0) as i32,
            mouse,
            true,
        );
        d.draw_text(
            "ENTER / ESPACIO: saltar intro",
            (wf * 0.065) as i32,
            (hf * 0.925) as i32,
            (hf * 0.022).clamp(12.0, 18.0) as i32,
            Color::new(70, 76, 65, 255),
        );
    }
}

pub fn video_rect(w: i32, h: i32, source_w: i32, source_h: i32) -> Rectangle {
    let scale = (w as f32 * 0.86 / source_w as f32).min(h as f32 * 0.72 / source_h as f32);
    let (width, height) = (source_w as f32 * scale, source_h as f32 * scale);
    Rectangle::new(
        (w as f32 - width) * 0.5,
        h as f32 * 0.48 - height * 0.5,
        width,
        height,
    )
}

pub fn skip_button(w: i32, h: i32) -> Rectangle {
    let height = (h as f32 * 0.065).clamp(34.0, 54.0);
    let width = height * 3.4;
    Rectangle::new(w as f32 * 0.935 - width, h as f32 * 0.90, width, height)
}

fn polygon(d: &mut RaylibDrawHandle, points: &[Vector2], color: Color) {
    let center = points.iter().fold(Vector2::zero(), |a, b| a + *b) / points.len() as f32;
    for i in 0..points.len() {
        d.draw_triangle(center, points[(i + 1) % points.len()], points[i], color);
    }
}

fn cutout(d: &mut RaylibDrawHandle, r: Rectangle, color: Color) {
    let (x, y, w, h) = (r.x, r.y, r.width, r.height);
    let k = (h * 0.07).min(5.0);
    polygon(
        d,
        &[
            Vector2::new(x + k, y + 1.0),
            Vector2::new(x + w * 0.46, y - 2.0),
            Vector2::new(x + w - k, y + 2.0),
            Vector2::new(x + w, y + h * 0.47),
            Vector2::new(x + w - 2.0, y + h - 1.0),
            Vector2::new(x + w * 0.53, y + h + 2.0),
            Vector2::new(x, y + h - k),
            Vector2::new(x + 2.0, y + h * 0.52),
        ],
        color,
    );
}
