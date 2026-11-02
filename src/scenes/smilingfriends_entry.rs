//! Smiling Friends entry: play the supplied clip before the temporary game screen.
use crate::graphics::resources::save_frame;
use crate::minigames::common::host::{MinigameExit, MinigameResult};
use raylib::prelude::*;
use std::path::Path;
use std::time::Instant;

const VIDEO: &str = "assets/video/smilingfriends/intro.sfv";
const AUDIO: &str = "assets/video/smilingfriends/intro.ogg";

struct Frames {
    bytes: Vec<u8>,
    spans: Vec<(usize, usize)>,
    width: i32,
    height: i32,
    fps: u32,
}

impl Frames {
    fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
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

    fn image(&self, frame: usize) -> Result<Image, Box<dyn std::error::Error>> {
        let (start, end) = self.spans[frame];
        let image = Image::load_image_from_mem(".jpg", &self.bytes[start..end])?;
        if image.width != self.width || image.height != self.height {
            return Err(format!("Smiling Friends frame {frame} has an unexpected size").into());
        }
        Ok(image)
    }

    fn duration(&self) -> f32 {
        self.spans.len() as f32 / self.fps as f32
    }
}

fn update_frame(
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

fn draw_video(d: &mut RaylibDrawHandle<'_>, texture: &Texture2D, width: i32, height: i32) {
    d.clear_background(Color::BLACK);
    let scale = (width as f32 / texture.width as f32).min(height as f32 / texture.height as f32);
    let dest = Rectangle::new(
        (width as f32 - texture.width as f32 * scale) / 2.0,
        (height as f32 - texture.height as f32 * scale) / 2.0,
        texture.width as f32 * scale,
        texture.height as f32 * scale,
    );
    d.draw_texture_pro(
        texture,
        Rectangle::new(0.0, 0.0, texture.width as f32, texture.height as f32),
        dest,
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}

fn draw_construction(d: &mut RaylibDrawHandle<'_>, width: i32, height: i32) {
    d.clear_background(Color::new(18, 29, 42, 255));
    let title = "SMILING FRIENDS";
    let status = "EN CONSTRUCCION";
    let hint = "Enter o Esc: volver al sistema solar";
    d.draw_text(
        title,
        (width - d.measure_text(title, 52)) / 2,
        height / 2 - 96,
        52,
        Color::new(255, 223, 68, 255),
    );
    d.draw_text(
        status,
        (width - d.measure_text(status, 30)) / 2,
        height / 2 - 23,
        30,
        Color::new(245, 245, 245, 255),
    );
    d.draw_text(
        hint,
        (width - d.measure_text(hint, 20)) / 2,
        height / 2 + 47,
        20,
        Color::new(173, 201, 212, 255),
    );
}

pub fn run(args: impl Iterator<Item = String>) -> Result<MinigameExit, Box<dyn std::error::Error>> {
    let mut smoke = false;
    let mut mute = false;
    for arg in args {
        match arg.as_str() {
            "--smoke" | "--smoke-entry" => smoke = true,
            "--mute" => mute = true,
            _ => return Err(format!("Unknown Smiling Friends option: {arg}").into()),
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let frames = Frames::load(&root.join(VIDEO))?;
    let first = frames.image(0)?;
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .title("Planet-Hoppers | Smiling Friends")
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(640, 360);
    rl.set_target_fps(60);
    if smoke {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    let mut texture = rl.load_texture_from_image(&thread, &first)?;
    let device = if mute {
        None
    } else {
        RaylibAudio::init_audio_device().ok()
    };
    let mut music = device
        .as_ref()
        .and_then(|d| d.new_music(&root.join(AUDIO).to_string_lossy()).ok());
    let audio_length = music.as_ref().map(|track| track.get_time_length());
    if let Some(track) = music.as_mut().filter(|_| !smoke) {
        track.looping = false;
        track.play_stream();
    }
    if smoke {
        let dir = root.join("target/smilingfriends");
        std::fs::create_dir_all(&dir)?;
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        {
            let mut d = rl.begin_drawing(&thread);
            draw_video(&mut d, &texture, w, h);
            save_frame(&dir.join("intro-first.png"), w, h)?;
        }
        for frame in 1..frames.spans.len() {
            update_frame(&mut texture, &frames, frame)?;
        }
        {
            let mut d = rl.begin_drawing(&thread);
            draw_video(&mut d, &texture, w, h);
            save_frame(&dir.join("intro-last.png"), w, h)?;
        }
        {
            let mut d = rl.begin_drawing(&thread);
            draw_construction(&mut d, w, h);
            save_frame(&dir.join("construction.png"), w, h)?;
        }
        println!(
            "SMILINGFRIENDS_ENTRY_SMOKE_PASS frames={} duration={:.2}s audio={audio_length:?}",
            frames.spans.len(),
            frames.duration()
        );
        return Ok(MinigameExit::Returned(MinigameResult::Abandoned));
    }
    let started = Instant::now();
    let mut current = 0usize;
    let mut construction = false;
    while !rl.window_should_close() {
        if let Some(track) = music.as_mut() {
            track.update_stream();
        }
        if !construction {
            if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
                || rl.is_key_pressed(KeyboardKey::KEY_SPACE)
                || rl.is_key_pressed(KeyboardKey::KEY_ENTER)
            {
                construction = true;
                if let Some(track) = music.as_mut() {
                    track.stop_stream();
                }
            } else {
                let frame = (started.elapsed().as_secs_f32() * frames.fps as f32) as usize;
                if frame >= frames.spans.len() {
                    construction = true;
                    if let Some(track) = music.as_mut() {
                        track.stop_stream();
                    }
                } else if frame != current {
                    update_frame(&mut texture, &frames, frame)?;
                    current = frame;
                }
            }
        } else if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
            || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
        {
            return Ok(MinigameExit::Returned(MinigameResult::Abandoned));
        }
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        let mut d = rl.begin_drawing(&thread);
        if construction {
            draw_construction(&mut d, w, h);
        } else {
            draw_video(&mut d, &texture, w, h);
        }
    }
    Ok(MinigameExit::WindowClosed)
}
