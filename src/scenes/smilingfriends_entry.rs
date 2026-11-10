//! Smiling Friends entry: play the supplied clip, then the minigame (`smilingfriends_game`).
use crate::graphics::resources::save_frame;
use super::smilingfriends_game::{draw_intro, skip_button};
use crate::minigames::common::host::{MinigameExit, MinigameResult};
use raylib::prelude::*;
use super::sfv::{update_frame, Frames};
use std::path::Path;
use std::time::Instant;

const VIDEO: &str = "assets/video/smilingfriends/intro.sfv";
const AUDIO: &str = "assets/video/smilingfriends/intro.ogg";

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
        for frame in 1..frames.len() {
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
            frames.len(),
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
                let frame = (started.elapsed().as_secs_f32() * frames.fps() as f32) as usize;
                if frame >= frames.len() {
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
