//! Hosted minigame window, keyboard input, sound and automated smoke run.
use super::{
    media::{MediaFlow, Soundtrack},
    render as art,
    simulation::*,
    support::{save_frame, ExitConfirmation, ExitReason, GameResult},
    video::Video,
};
use raylib::prelude::*;
use std::path::Path;

type Error = Box<dyn std::error::Error>;

pub fn run_visit(
    project_root: &Path,
    args: impl Iterator<Item = String>,
    observer: &mut crate::minigames::common::host::ResultObserver<'_>,
) -> Result<ExitReason, Error> {
    let mut reward_shortcut = crate::minigames::common::shortcuts::RewardShortcut::default();
    let mut reporter = crate::minigames::common::host::AttemptReporter::default();
    let mut smoke = false;
    let mut return_goal = None;
    let mut check_menus = false;
    let mut mute = false;
    let mut coop = false;
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke-return" => {
                let goal = args
                    .next()
                    .ok_or("--smoke-return necesita abandoned o defeat")?;
                if !matches!(goal.as_str(), "abandoned" | "defeat") {
                    return Err("Objetivo de regreso desconocido".into());
                }
                return_goal = Some(goal);
                smoke = true;
                mute = true;
            }
            "--smoke" => {
                smoke = true;
                mute = true;
            }
            "--check-menus" => {
                check_menus = true;
                mute = true;
                smoke = true;
            }
            "--mute" => mute = true,
            "--coop" => coop = true,
            "--help" | "-h" => {
                println!(
                    "cargo run --bin regularshow -- [--coop] [--mute] [--smoke] [--check-menus]"
                );
                println!("P1: WASD mover, J puño, K patada, F mantener bloqueo, E agarrar mueble, L esquivar, Espacio saltar.");
                println!("P2 opcional: flechas mover, U puño, I patada, Y mantener bloqueo, P agarrar mueble, O esquivar, Shift derecho saltar.");
                println!("Enter empezar/continuar; Esc pausa; Retroceso volver al sistema solar; F12 captura.");
                println!("--smoke: simulación automática y capturas en target/regularshow/.");
                return Ok(ExitReason::ReturnedWithoutResult);
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let root = project_root.join("assets/minigames/regularshow");
    let captures = project_root.join("target/regularshow");
    std::fs::create_dir_all(&captures)?;
    let (mut rl, thread) = crate::graphics::window::build(
        raylib::init()
            .size(1280, 720)
            .resizable()
            .title("Regular Show: El Martillo"),
        smoke,
    );
    rl.set_exit_key(None);
    // JPEG decoding happens continuously during cinematics.
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);
    rl.set_window_min_size(960, 540);
    rl.set_target_fps(if smoke { 0 } else { 60 });
    let mut canvas = rl.load_render_texture(&thread, WIDTH as u32, HEIGHT as u32)?;
    canvas
        .texture()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_POINT);
    let audio = if mute {
        None
    } else {
        RaylibAudio::init_audio_device().ok()
    };
    let sounds = audio.as_ref().and_then(|a| {
        Sounds::load(a, &captures)
            .map_err(|e| eprintln!("Audio opcional: {e}"))
            .ok()
    });
    let mut music = audio
        .as_ref()
        .map(|a| Soundtrack::load(a, &root))
        .transpose()?;
    let art = art::Art::load(&mut rl, &thread)?;
    let mut g = Game::new();
    g.coop = coop;
    if check_menus {
        for (name, phase) in [
            ("title", Phase::Title),
            ("pause", Phase::Paused),
            ("defeat", Phase::Defeat),
            ("victory", Phase::ArcadeClear),
        ] {
            g.phase = phase;
            for _ in 0..3 {
                {
                    let mut d = rl.begin_texture_mode(&thread, &mut canvas);
                    art::draw(&mut d, &g, &art);
                }
                let w = rl.get_screen_width();
                let h = rl.get_screen_height();
                let (dest, _) = viewport(w, h);
                let mut d = crate::graphics::window::begin_drawing(&mut rl, &thread);
                d.clear_background(Color::BLACK);
                d.draw_texture_pro(
                    canvas.texture(),
                    Rectangle::new(0.0, 0.0, WIDTH as f32, -(HEIGHT as f32)),
                    dest,
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                );
                save_frame(&captures.join(format!("menu-{name}.png")), w, h)?;
            }
        }
        println!("REGULARSHOW_MENUS_OK");
        return Ok(ExitReason::ReturnedWithoutResult);
    }
    let mut media_flow = MediaFlow::new(&g);
    let mut video: Option<Video<'_>> = None;
    let mut video_was_paused = false;
    let mut video_capture = None;
    let mut confirmation = ExitConfirmation::default();
    let mut frame = 0u32;
    let mut previous_impacts = 0;
    let mut previous_slams = 0;
    let mut previous_phase = g.phase;
    let mut smoke_stage = 0;
    let mut smoke_ticks = 0;
    let mut room_captures = [false; 3];
    loop {
        if crate::graphics::window::should_close(&rl)
            || (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                && rl.is_key_pressed(KeyboardKey::KEY_Q))
        {
            return Ok(ExitReason::WindowClosed);
        }
        if let Some(exit) = reward_shortcut.poll(&rl, observer)? {
            return Ok(exit);
        }
        let dt = if smoke {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.1)
        };
        if let Some(goal) = &return_goal {
            for key in [
                KeyboardKey::KEY_ENTER,
                KeyboardKey::KEY_ESCAPE,
                KeyboardKey::KEY_BACKSPACE,
            ] {
                unsafe {
                    raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                        frame: 0,
                        type_: 1,
                        params: [key as i32, 0, 0, 0],
                    });
                }
            }
            let key = if goal == "abandoned" {
                match frame {
                    1 | 22 => Some(KeyboardKey::KEY_ENTER),
                    10 | 14 | 16 => Some(KeyboardKey::KEY_ESCAPE),
                    12 | 20 => Some(KeyboardKey::KEY_BACKSPACE),
                    _ => None,
                }
            } else if frame == 1 {
                Some(KeyboardKey::KEY_ENTER)
            } else if g.phase == Phase::Defeat {
                Some(KeyboardKey::KEY_BACKSPACE)
            } else {
                None
            };
            if let Some(key) = key {
                unsafe {
                    raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                        frame: 0,
                        type_: 2,
                        params: [key as i32, 0, 0, 0],
                    });
                }
            }
            if frame > 36_000 {
                return Err("REGULARSHOW_RETURN_FAIL: tiempo agotado".into());
            }
        }
        let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
        let escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        let retry = rl.is_key_pressed(KeyboardKey::KEY_R);
        let mut capture = None;
        if let Some(movie) = video.as_mut() {
            let mut finished = false;
            if confirmation.pending {
                if enter {
                    return Ok(ExitReason::Returned(
                        confirmation.confirm().expect("pending"),
                    ));
                }
                if escape {
                    confirmation.cancel();
                    if !video_was_paused {
                        movie.toggle_pause();
                    }
                }
            } else if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
                video_was_paused = movie.paused;
                if !movie.paused {
                    movie.toggle_pause();
                }
                confirmation.request();
            } else if escape || rl.is_key_pressed(KeyboardKey::KEY_P) || (enter && movie.paused) {
                movie.toggle_pause();
            } else if enter {
                finished = true;
            } else {
                finished = movie.update(dt)?;
            }
            if smoke && movie.elapsed >= 1.0 {
                capture = video_capture.take();
            }
            if finished {
                video = None;
                if smoke && g.phase == Phase::ArcadeClear {
                    capture = Some("09-room-clear.png");
                }
            }
        } else if smoke && return_goal.is_none() {
            // Real AI/combat driven by the same Input as the keyboard, never a forced victory.
            if frame == 0 {
                capture = Some("01-title.png");
            }
            if frame == 1 {
                g.start();
            }
            if frame > 1 {
                match g.phase {
                    Phase::Playing => {
                        let inputs = autoplay(&g);
                        g.update(dt, inputs);
                        smoke_ticks += 1;
                        if g.phase == Phase::StageClear {
                            capture = Some(if g.stage == Stage::Tutorial {
                                "03-tutorial-clear.png"
                            } else {
                                "05-dojo-clear.png"
                            });
                        }
                        if g.phase == Phase::ArcadeClear {
                            capture = Some("09-room-clear.png");
                        }
                        if g.stage == Stage::LivingRoom {
                            if !room_captures[0] && (0.32..0.35).contains(&g.intro_left) {
                                capture = Some("06-tv.png");
                                room_captures[0] = true;
                            } else if !room_captures[1]
                                && matches!(
                                    g.enemies[0].action,
                                    Action::Strike {
                                        attack: Attack::Punch,
                                        fired: true,
                                        ..
                                    }
                                )
                            {
                                capture = Some("07-punch.png");
                                room_captures[1] = true;
                            } else if !room_captures[2] && g.delivery.is_some() {
                                capture = Some("08-delivery.png");
                                room_captures[2] = true;
                            }
                        }
                        if smoke_ticks == 45 && smoke_stage == 0 {
                            capture = Some("02-tutorial.png");
                        }
                        if g.stage == Stage::Hammer
                            && smoke_stage == 1
                            && g.stage_time > 1.05
                            && g.props
                                .iter()
                                .any(|p| matches!(p.state, PropState::Held(_)))
                        {
                            capture = Some("04-hammer.png");
                            smoke_stage = 2;
                        }
                    }
                    Phase::StageClear => {
                        g.advance();
                        smoke_stage = 1;
                    }
                    Phase::ArcadeClear => {
                        capture = Some("09-room-clear.png");
                    }
                    Phase::Defeat => {
                        return Err(
                            format!("REGULARSHOW_SMOKE_FAIL: derrota en {:?}", g.stage).into()
                        )
                    }
                    _ => {}
                }
            }
            if frame > 36_000 {
                return Err("REGULARSHOW_SMOKE_FAIL: tiempo agotado".into());
            }
        } else if confirmation.pending {
            if enter {
                return Ok(ExitReason::Returned(
                    confirmation.confirm().expect("pending"),
                ));
            }
            if escape {
                confirmation.cancel();
            }
        } else {
            if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
                match g.phase {
                    Phase::ArcadeClear => return Ok(ExitReason::Returned(GameResult::Victory)),
                    Phase::Defeat => return Ok(ExitReason::Returned(GameResult::Defeat)),
                    Phase::Title => return Ok(ExitReason::Returned(GameResult::Abandoned)),
                    _ => {
                        confirmation.request();
                    }
                }
            } else {
                match g.phase {
                    Phase::Title => {
                        g.elapsed += dt;
                        if rl.is_key_pressed(KeyboardKey::KEY_F2) {
                            g.coop = !g.coop;
                        }
                        let mouse = rl.get_mouse_position();
                        let (dest, scale) = viewport(rl.get_screen_width(), rl.get_screen_height());
                        let mx = (mouse.x - dest.x) / scale;
                        let my = (mouse.y - dest.y) / scale;
                        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                            && (80.0..628.0).contains(&mx)
                            && (389.0..442.0).contains(&my);
                        if enter || clicked {
                            g.start();
                        }
                    }
                    Phase::Playing => {
                        if escape {
                            g.toggle_pause();
                        } else {
                            g.update(dt, keyboard(&rl));
                        }
                    }
                    Phase::Paused => {
                        if enter || escape {
                            g.toggle_pause();
                        } else if retry {
                            g.retry();
                        }
                    }
                    Phase::StageClear => {
                        if enter {
                            g.advance();
                        }
                    }
                    Phase::Defeat => {
                        if enter || retry {
                            g.retry();
                        }
                    }
                    Phase::ArcadeClear => {
                        if enter {
                            return Ok(ExitReason::Returned(GameResult::Victory));
                        }
                        if retry {
                            g.start();
                        }
                    }
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_F12) {
                capture = Some("capture.png");
            }
        }
        reporter.observe(&g, observer)?;
        if let Some(clip) = media_flow.update(&g) {
            video = Some(Video::load(
                &mut rl,
                &thread,
                audio.as_ref(),
                &root,
                clip.stem(),
            )?);
            video_capture = Some(match clip {
                super::media::Clip::Transition => "06-transition-video.png",
                super::media::Clip::Celebration => "10-celebration-video.png",
            });
            if smoke {
                capture = None;
            }
            if let Some(s) = &sounds {
                s.hit.stop();
                s.slam.stop();
                s.clear.stop();
            }
        }
        if let Some(m) = &mut music {
            m.update(&g, video.is_some(), confirmation.pending);
        }
        if let Some(s) = sounds.as_ref().filter(|_| video.is_none()) {
            if g.impacts != previous_impacts {
                s.hit.play();
            }
            if g.slams != previous_slams {
                s.slam.play();
            }
            if g.phase != previous_phase
                && matches!(g.phase, Phase::StageClear | Phase::ArcadeClear)
            {
                s.clear.play();
            }
        }
        previous_impacts = g.impacts;
        previous_slams = g.slams;
        previous_phase = g.phase;
        {
            let mut d = rl.begin_texture_mode(&thread, &mut canvas);
            if let Some(movie) = &video {
                movie.draw(&mut d, WIDTH as f32, HEIGHT as f32);
            } else {
                art::draw(&mut d, &g, &art);
            }
            if confirmation.pending {
                art::confirm_return(&mut d);
            }
        }
        let width = rl.get_screen_width();
        let height = rl.get_screen_height();
        let (mut dest, scale) = viewport(width, height);
        if !confirmation.pending && video.is_none() && g.phase == Phase::Playing {
            dest.x += (g.elapsed * 97.0).sin() * g.shake * scale;
            dest.y += (g.elapsed * 71.0).cos() * g.shake * 0.4 * scale;
        }
        {
            let mut d = crate::graphics::window::begin_drawing(&mut rl, &thread);
            d.clear_background(Color::new(13, 11, 20, 255));
            d.draw_texture_pro(
                canvas.texture(),
                Rectangle::new(0.0, 0.0, WIDTH as f32, -(HEIGHT as f32)),
                dest,
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
            if let Some(name) = capture {
                save_frame(&captures.join(name), width, height)?;
            }
        }
        if smoke && video.is_none() && g.phase == Phase::ArcadeClear {
            println!(
                "REGULARSHOW_SMOKE_PASS score={} hp={} slams={} frames={frame}",
                g.score, g.players[0].hp, g.slams
            );
            return Ok(ExitReason::Returned(GameResult::Victory));
        }
        frame += 1;
    }
}

fn viewport(w: i32, h: i32) -> (Rectangle, f32) {
    let scale = (w as f32 / WIDTH as f32).min(h as f32 / HEIGHT as f32);
    (
        Rectangle::new(
            (w as f32 - WIDTH as f32 * scale) * 0.5,
            (h as f32 - HEIGHT as f32 * scale) * 0.5,
            WIDTH as f32 * scale,
            HEIGHT as f32 * scale,
        ),
        scale,
    )
}
fn keyboard(rl: &RaylibHandle) -> [Input; 2] {
    let axis = |a, b| rl.is_key_down(a) as i32 as f32 - rl.is_key_down(b) as i32 as f32;
    [
        Input {
            x: axis(KeyboardKey::KEY_D, KeyboardKey::KEY_A),
            y: axis(KeyboardKey::KEY_S, KeyboardKey::KEY_W),
            guard: rl.is_key_down(KeyboardKey::KEY_F),
            grab: rl.is_key_pressed(KeyboardKey::KEY_E),
            punch: rl.is_key_pressed(KeyboardKey::KEY_J),
            kick: rl.is_key_pressed(KeyboardKey::KEY_K),
            dodge: rl.is_key_pressed(KeyboardKey::KEY_L),
            jump: rl.is_key_pressed(KeyboardKey::KEY_SPACE),
        },
        Input {
            x: axis(KeyboardKey::KEY_RIGHT, KeyboardKey::KEY_LEFT),
            y: axis(KeyboardKey::KEY_DOWN, KeyboardKey::KEY_UP),
            guard: rl.is_key_down(KeyboardKey::KEY_Y),
            grab: rl.is_key_pressed(KeyboardKey::KEY_P),
            punch: rl.is_key_pressed(KeyboardKey::KEY_U),
            kick: rl.is_key_pressed(KeyboardKey::KEY_I),
            dodge: rl.is_key_pressed(KeyboardKey::KEY_O),
            jump: rl.is_key_pressed(KeyboardKey::KEY_RIGHT_SHIFT),
        },
    ]
}

/// Plays each fight through normal movement, guard, pickup and attack inputs.
pub fn autoplay(g: &Game) -> [Input; 2] {
    let mut inputs = [Input::default(); 2];
    for (idx, p) in g.players.iter().enumerate().filter(|(_, p)| p.alive()) {
        let distance = |x: f32, y: f32| (x - p.x).hypot((y - p.y) * 1.5);
        let Some(e) = g
            .enemies
            .iter()
            .filter(|e| e.alive())
            .min_by(|a, b| distance(a.x, a.y).total_cmp(&distance(b.x, b.y)))
        else {
            continue;
        };
        let i = &mut inputs[idx];
        let holding = g.props.iter().any(|v| v.state == PropState::Held(idx));
        let boss = e.kind == Kind::Hammer;
        let pickup = if boss && !holding {
            g.props
                .iter()
                .filter(|v| v.state == PropState::Ready)
                .min_by(|a, b| distance(a.x, a.y).total_cmp(&distance(b.x, b.y)))
        } else {
            None
        };
        let (tx, ty) = pickup.map_or((e.x, e.y), |v| (v.x, v.y));
        let (dx, dy) = (tx - p.x, ty - p.y);
        if dx.abs() > 4.0 {
            i.x = dx.signum();
        }
        if dy.abs() > 5.0 {
            i.y = dy.signum();
        }
        if g.stage == Stage::LivingRoom && pickup.is_none() && !holding {
            // Run to the opposite lane when the boss closes in.
            i.x = if p.x < 125.0 {
                1.0
            } else if p.x > 835.0 {
                -1.0
            } else {
                -dx.signum()
            };
            if dx.abs() < 160.0 {
                i.y = if e.y < 387.0 { 1.0 } else { -1.0 };
            }
        }
        if let Some(v) = pickup {
            i.grab = v.reachable(p);
        } else if (!boss || holding)
            && dx.abs() < if boss { 138.0 } else { 94.0 }
            && dy.abs() < 24.0
        {
            i.kick = true;
        }
        let warn = matches!(e.action,Action::Strike{attack:Attack::Punch,time,fired:false}
            if time>windup(e.kind,Attack::Punch,e.enraged())-0.14);
        if warn && distance(e.x, e.y) < if boss { 145.0 } else { 72.0 } {
            i.x = (e.x - p.x).signum();
            i.guard = true;
        }
        i.jump = g.waves.iter().any(|w| {
            let ahead = (p.x - w.x).hypot((p.y - w.y) * 1.8) - w.radius;
            (0.0..78.0).contains(&ahead)
        }) || matches!(e.action,Action::Strike{attack:Attack::Slam,time,fired:false} if distance(e.x,e.y)<75.0 && time>windup(e.kind,Attack::Slam,e.enraged())-0.16);
        if i.jump {
            i.guard = false;
            i.kick = false;
        }
    }
    inputs
}
struct Sounds<'a> {
    hit: Sound<'a>,
    slam: Sound<'a>,
    clear: Sound<'a>,
}
impl<'a> Sounds<'a> {
    fn load(audio: &'a RaylibAudio, path: &Path) -> Result<Self, Error> {
        let tone = |name: &str, freq: f32, seconds: f32| -> Result<Sound<'a>, Error> {
            let file = path.join(format!("{name}.wav"));
            let rate = 22050u32;
            let count = (rate as f32 * seconds) as u32;
            let mut data = Vec::with_capacity(44 + count as usize * 2);
            data.extend_from_slice(b"RIFF");
            data.extend_from_slice(&(36 + count * 2).to_le_bytes());
            data.extend_from_slice(b"WAVEfmt ");
            data.extend_from_slice(&16u32.to_le_bytes());
            data.extend_from_slice(&1u16.to_le_bytes());
            data.extend_from_slice(&1u16.to_le_bytes());
            data.extend_from_slice(&rate.to_le_bytes());
            data.extend_from_slice(&(rate * 2).to_le_bytes());
            data.extend_from_slice(&2u16.to_le_bytes());
            data.extend_from_slice(&16u16.to_le_bytes());
            data.extend_from_slice(b"data");
            data.extend_from_slice(&(count * 2).to_le_bytes());
            for n in 0..count {
                let t = n as f32 / rate as f32;
                let phase = t * freq * std::f32::consts::TAU * (1.0 - t * 0.5 / seconds);
                let sample = (phase.sin().signum() * 7000.0 * (1.0 - t / seconds).powi(2)) as i16;
                data.extend_from_slice(&sample.to_le_bytes());
            }
            std::fs::write(&file, data)?;
            let s = audio.new_sound(&file.to_string_lossy())?;
            s.set_volume(0.25);
            Ok(s)
        };
        Ok(Self {
            hit: tone("hit", 160.0, 0.08)?,
            slam: tone("slam", 70.0, 0.3)?,
            clear: tone("clear", 880.0, 0.5)?,
        })
    }
}
