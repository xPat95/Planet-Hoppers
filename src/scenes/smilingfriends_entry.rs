//! Smiling Friends entry: play the supplied clip, then the minigame (`smilingfriends_game`).
use crate::graphics::resources::save_frame;
use super::smilingfriends_game::{draw_intro, skip_button};
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

pub fn run(args: impl Iterator<Item = String>) -> Result<MinigameExit, Box<dyn std::error::Error>> {
    run_with_results(args, &mut |_| Ok(()))
}

pub fn run_with_results(args: impl Iterator<Item = String>, observer: &mut crate::minigames::common::host::ResultObserver<'_>) -> Result<MinigameExit, Box<dyn std::error::Error>> {
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
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let font = rl.load_font_ex(&thread, &root.join("assets/fonts/Bungee-Regular.ttf").to_string_lossy(), 64, None)?;
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
            draw_intro(&mut d, &texture, &font, 1.0, Vector2::zero());
            save_frame(&dir.join("intro-first.png"), w, h)?;
        }
        for frame in 1..frames.spans.len() {
            update_frame(&mut texture, &frames, frame)?;
        }
        {
            let mut d = rl.begin_drawing(&thread);
            draw_intro(&mut d, &texture, &font, 2.0, Vector2::zero());
            save_frame(&dir.join("intro-last.png"), w, h)?;
        }
        // El juego completo, jugado solo con las mejores respuestas
        let result = super::smilingfriends_game::play_with_results(&mut rl, &thread, root, true, None, observer)?;
        if result != Some(MinigameResult::Victory) {
            return Err(format!("La prueba del minijuego terminó en {result:?}").into());
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
    let mut construction = false;                       // true: terminó el video, empieza el juego
    while !rl.window_should_close() {
        if let Some(track) = music.as_mut() {
            track.update_stream();
        }
        if !construction {
            let skip_clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                && skip_button(rl.get_screen_width() as f32, rl.get_screen_height() as f32)
                    .check_collision_point_rec(rl.get_mouse_position());
            if skip_clicked
                || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
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
        } else {
            return Ok(match super::smilingfriends_game::play_with_results(&mut rl, &thread, root, false, device.as_ref(), observer)? {
                Some(result) => MinigameExit::Returned(result),
                None => MinigameExit::WindowClosed,
            });
        }
        let mouse = rl.get_mouse_position();
        let time = started.elapsed().as_secs_f32();
        let mut d = rl.begin_drawing(&thread);
        draw_intro(&mut d, &texture, &font, time, mouse);
    }
    Ok(MinigameExit::WindowClosed)
}
