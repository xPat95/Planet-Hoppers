//! Indexed on-disk video: only the current JPEG is decoded. No runtime FFmpeg.
use raylib::prelude::*;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};
type Error = Box<dyn std::error::Error>;
pub struct Index {
    pub width: i32,
    pub height: i32,
    pub fps: u32,
    pub spans: Vec<(u64, usize)>,
}
impl Index {
    pub fn read(reader: &mut (impl Read + Seek)) -> Result<Self, Error> {
        let length = reader.seek(SeekFrom::End(0))?;
        reader.rewind()?;
        if length > 512 * 1024 * 1024 {
            return Err("El video excede el tamaño permitido".into());
        }
        let mut header = [0u8; 20];
        reader.read_exact(&mut header)?;
        let field = |i| u32::from_le_bytes(header[i..i + 4].try_into().unwrap());
        let (w, h, fps, count) = (field(4), field(8), field(12), field(16));
        if &header[..4] != b"SFV1"
            || !(1..=1920).contains(&w)
            || !(1..=1080).contains(&h)
            || !(1..=60).contains(&fps)
            || !(1..=18000).contains(&count)
        {
            return Err("Cabecera de video inválida".into());
        }
        let mut spans = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let mut size = [0u8; 4];
            reader.read_exact(&mut size)?;
            let size = u32::from_le_bytes(size) as u64;
            let start = reader.stream_position()?;
            let end = start.checked_add(size).ok_or("Tamaño inválido")?;
            if size == 0 || size > 16 * 1024 * 1024 || end > length {
                return Err("Fotograma incompleto".into());
            }
            spans.push((start, size as usize));
            reader.seek(SeekFrom::Start(end))?;
        }
        if reader.stream_position()? != length {
            return Err("Datos inesperados al final del video".into());
        }
        Ok(Self {
            width: w as i32,
            height: h as i32,
            fps,
            spans,
        })
    }
    pub fn duration(&self) -> f32 {
        self.spans.len() as f32 / self.fps as f32
    }
    pub fn frame_at(&self, time: f32) -> usize {
        ((time.max(0.0) * self.fps as f32) as usize).min(self.spans.len() - 1)
    }
}
pub struct Video<'a> {
    file: File,
    index: Index,
    texture: Texture2D,
    current: usize,
    pub elapsed: f32,
    music: Option<Music<'a>>,
    pub paused: bool,
    started: bool,
}
impl<'a> Video<'a> {
    pub fn load(
        rl: &mut RaylibHandle,
        t: &RaylibThread,
        audio: Option<&'a RaylibAudio>,
        root: &Path,
        stem: &str,
    ) -> Result<Self, Error> {
        let mut file = File::open(root.join(format!("video/{stem}.sfv")))?;
        let index = Index::read(&mut file)?;
        let image = Self::read_image(&mut file, &index, 0)?;
        let texture = rl.load_texture_from_image(t, &image)?;
        texture.set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let music = if let Some(a) = audio {
            let mut m = a.new_music(&root.join(format!("video/{stem}.ogg")).to_string_lossy())?;
            m.looping = false;
            if (m.get_time_length() - index.duration()).abs() > 0.25 {
                return Err("La duración del audio no coincide con el video".into());
            }
            Some(m)
        } else {
            None
        };
        Ok(Self {
            file,
            index,
            texture,
            current: 0,
            elapsed: 0.0,
            music,
            paused: false,
            started: false,
        })
    }
    fn read_image(file: &mut File, index: &Index, frame: usize) -> Result<Image, Error> {
        let (offset, size) = index.spans[frame];
        file.seek(SeekFrom::Start(offset))?;
        let mut bytes = vec![0u8; size];
        file.read_exact(&mut bytes)?;
        let image = Image::load_image_from_mem(".jpg", &bytes)?;
        if image.width != index.width || image.height != index.height {
            return Err("Dimensiones de video inconsistentes".into());
        }
        Ok(image)
    }
    pub fn seek_frame(&mut self, seconds: f32) -> Result<(), Error> {
        let frame = self.index.frame_at(seconds);
        if frame != self.current {
            let image = Self::read_image(&mut self.file, &self.index, frame)?;
            if image.format != self.texture.format {
                return Err("Formato de video inconsistente".into());
            }
            let bytes = unsafe {
                std::slice::from_raw_parts(image.data.cast::<u8>(), image.get_pixel_data_size())
            };
            self.texture.update_texture(bytes)?;
            self.current = frame;
        }
        Ok(())
    }
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        if let Some(m) = &self.music {
            if self.paused {
                m.pause_stream();
            } else if self.started {
                m.resume_stream();
            }
        }
    }
    pub fn update(&mut self, dt: f32) -> Result<bool, Error> {
        if self.paused {
            return Ok(false);
        }
        if !self.started {
            if let Some(m) = &self.music {
                m.play_stream();
            }
            self.started = true;
        }
        self.elapsed += dt;
        let mut position = self.elapsed;
        if let Some(m) = &self.music {
            m.update_stream();
            let audio_time = m.get_time_played();
            if m.is_stream_playing() && audio_time > 0.0 {
                position = audio_time;
            } else if self.elapsed > self.index.duration() - 0.15 {
                return Ok(true);
            }
        }
        if position >= self.index.duration() || self.elapsed > self.index.duration() + 2.0 {
            return Ok(true);
        }
        self.seek_frame(position)?;
        Ok(false)
    }
    pub fn draw(&self, d: &mut impl RaylibDraw, w: f32, h: f32) {
        d.clear_background(Color::BLACK);
        let scale = (w / self.index.width as f32).min(h / self.index.height as f32);
        let (vw, vh) = (
            self.index.width as f32 * scale,
            self.index.height as f32 * scale,
        );
        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(0.0, 0.0, self.index.width as f32, self.index.height as f32),
            Rectangle::new((w - vw) * 0.5, (h - vh) * 0.5, vw, vh),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        if self.paused {
            d.draw_rectangle(0, 0, w as i32, h as i32, Color::new(0, 0, 0, 160));
            d.draw_text(
                "PAUSA / ESC, P O ENTER PARA CONTINUAR",
                (w * 0.5 - 230.0) as i32,
                (h * 0.5) as i32,
                20,
                Color::WHITE,
            );
        } else if self.elapsed < 4.0 {
            d.draw_rectangle(
                (w - 316.0) as i32,
                (h - 38.0) as i32,
                312,
                30,
                Color::new(0, 0, 0, 170),
            );
            d.draw_text(
                "ENTER  OMITIR     P  PAUSA",
                (w - 300.0) as i32,
                (h - 30.0) as i32,
                16,
                Color::new(204, 212, 215, 210),
            );
        }
    }
}
impl Drop for Video<'_> {
    fn drop(&mut self) {
        if let Some(m) = &self.music {
            m.stop_stream();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    #[ignore = "Requires graphics and audio devices; run with --ignored --test-threads=1"]
    fn real_videos_finish_with_synchronized_audio_and_pause() {
        let (mut rl, thread) = raylib::init()
            .size(960, 540)
            .title("Media verification")
            .build();
        rl.set_window_state(WindowState::default().set_window_hidden(true));
        rl.set_trace_log(TraceLogLevel::LOG_WARNING);
        rl.set_target_fps(60);
        let audio = RaylibAudio::init_audio_device().unwrap();
        let root = &Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/minigames/regularshow");
        for stem in ["transicion", "celebracion"] {
            let mut video = Video::load(&mut rl, &thread, Some(&audio), root, stem).unwrap();
            video.music.as_ref().unwrap().set_volume(0.0);
            assert!(!video.update(1.0 / 60.0).unwrap());
            assert!(video.music.as_ref().unwrap().is_stream_playing());
            video.toggle_pause();
            let elapsed = video.elapsed;
            assert!(!video.update(1.0).unwrap());
            assert_eq!(video.elapsed, elapsed);
            assert!(!video.music.as_ref().unwrap().is_stream_playing());
            video.toggle_pause();
            let start = std::time::Instant::now();
            let mut previous = start;
            loop {
                let now = std::time::Instant::now();
                let finished = video.update((now - previous).as_secs_f32()).unwrap();
                previous = now;
                if finished {
                    break;
                }
                assert!(start.elapsed().as_secs_f32() < video.index.duration() + 4.0);
                let mut d = rl.begin_drawing(&thread);
                video.draw(&mut d, 960.0, 540.0);
            }
            assert!(video.elapsed > video.index.duration() - 0.5);
            println!("VIDEO_PLAYBACK_OK {stem} elapsed={:.2}", video.elapsed);
        }
    }
    fn pack() -> Vec<u8> {
        let mut b = b"SFV1".to_vec();
        for n in [1280u32, 720, 30, 2, 3] {
            b.extend(n.to_le_bytes());
        }
        b.extend([1, 2, 3]);
        b.extend(2u32.to_le_bytes());
        b.extend([4, 5]);
        b
    }
    #[test]
    fn indexed_frames_and_duration() {
        let idx = Index::read(&mut Cursor::new(pack())).unwrap();
        assert_eq!(idx.spans, vec![(24, 3), (31, 2)]);
        assert_eq!(idx.frame_at(0.0), 0);
        assert_eq!(idx.frame_at(999.0), 1);
        assert!((idx.duration() - 2.0 / 30.0).abs() < 0.0001);
    }
    #[test]
    fn reject_truncation_zero_frame_and_trailing_data() {
        let b = pack();
        for size in [0, 4, 18, 25, 32] {
            assert!(Index::read(&mut Cursor::new(&b[..size])).is_err());
        }
        let mut extra = b.clone();
        extra.push(0);
        assert!(Index::read(&mut Cursor::new(extra)).is_err());
        let mut zero = b;
        zero[20..24].copy_from_slice(&0u32.to_le_bytes());
        assert!(Index::read(&mut Cursor::new(zero)).is_err());
    }
    #[test]
    fn supplied_media_headers() {
        let root = &Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/minigames/regularshow");
        for stem in ["transicion", "celebracion"] {
            let mut f = File::open(root.join(format!("video/{stem}.sfv"))).unwrap();
            let idx = Index::read(&mut f).unwrap();
            assert!(idx.spans.len() > 30);
            assert_eq!(idx.fps, 30);
        }
    }
}
