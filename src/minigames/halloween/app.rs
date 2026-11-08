//! Ventana del minijuego de Halloween: dibujar glifos con el mouse, pausa, victoria y derrota, regreso al sistema
//! solar (contrato de `docs/minigames/host-return.md`) y una prueba automática que juega sola (`--smoke`).
use super::render::{self as art, Art, Fx, View, FRIGHT};
use super::sound::{Sounds, Track};
use super::story::{self, Story, ENDING};
use super::{recognize, Balance, Cast, Glyph, Kind, Phase, Session, State};
use crate::minigames::common::host::{AttemptReporter, MinigameExit, MinigameResult, ResultObserver};
use raylib::{ffi, prelude::*};
use std::path::Path;

type Error = Box<dyn std::error::Error>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Screen {
    /// La historieta de la intro (es la pantalla de inicio).
    Story,
    /// La bruja baila sola en la luz mientras se aprenden los glifos (dibujando cada uno) y después la cámara gira
    /// hasta sus ojos.
    Intro,
    Play,
    Pause,
    Confirm,
    /// La victoria: la bruja baila feliz con luces de disco (después aparece la pantalla de victoria encima).
    Ending,
    Over,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Goal {
    Victory,
    Defeat,
    Abandoned,
}

pub fn run_visit(root: &Path, args: impl Iterator<Item = String>, observer: &mut ResultObserver<'_>)
    -> Result<MinigameExit, Error> {
    let mut goal = None;
    let mut mute = false;
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => goal = Some(Goal::Victory),
            "--smoke-return" => {
                goal = Some(match args.next().as_deref() {
                    Some("victory") => Goal::Victory,
                    Some("defeat") => Goal::Defeat,
                    Some("abandoned") => Goal::Abandoned,
                    _ => return Err("--smoke-return necesita victory, defeat o abandoned".into()),
                })
            }
            "--mute" => mute = true,
            "--help" | "-h" => {
                println!("cargo run --bin halloween -- [--smoke] [--smoke-return victory|defeat|abandoned]");
                println!("Mantener el botón izquierdo y dibujar el glifo; Esc o P pausa; Retroceso volver al sistema solar.");
                return Ok(MinigameExit::ReturnedWithoutResult);
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let smoke = goal.is_some();
    let captures = root.join("target/halloween");
    if smoke {
        std::fs::create_dir_all(&captures)?;
    }
    let (mut rl, thread) = raylib::init().size(1280, 720).resizable().title("Hexbound").build();
    rl.set_exit_key(None);
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);
    rl.set_window_min_size(960, 540);
    if smoke {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    rl.set_target_fps(if smoke { 0 } else { 60 });
    let mut art = Art::load(&mut rl, &thread, root)?;
    // Sin dispositivo de audio, el juego sigue en silencio; la prueba automática va muda
    let device = RaylibAudio::init_audio_device().ok();
    let mut sounds = Sounds::load(device.as_ref(), root, mute || smoke);
    let mut fx = Fx::default();
    let mut seed = if smoke { 5 } else {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.as_nanos() as u64)
    };
    let mut s = Session::new(Balance::default(), seed);
    let mut reporter = AttemptReporter::default();
    let mut screen = Screen::Story;
    let mut story = Story::default();
    // Desde dónde se pausó (la historieta, el baile o el juego), y cuánto hace que terminó la partida ganada
    let mut paused_from = Screen::Play;
    let mut ending = 0.0f32;
    let mut selected = 0usize;
    let mut buttons: Vec<Rectangle> = Vec::new();
    let mut stroke: Vec<Vector2> = Vec::new();
    let mut drawing = false;
    let mut last = (s.phase, s.health);
    let mut auto = goal.map(Auto::new);
    let mut intro = 0.0f32;
    // Los glifos que faltan aprender en la intro, y cuánto hace que se aprendieron todos (después viene la
    // advertencia y el giro de la cámara)
    let mut learn: Vec<Glyph> = Glyph::ALL.to_vec();
    let mut after: Option<f32> = None;
    // Los sustos de mentira: cuándo toca el próximo, y cuánto hace del último susto (de mentira o de verdad)
    let mut frights = Frights::new(seed);
    // Hacia dónde mira la bruja (grados): clic derecho y arrastrar, o flechas / A y D
    let mut look = (0.0f32, 0.0f32);
    loop {
        if rl.window_should_close()
            || (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) && rl.is_key_pressed(KeyboardKey::KEY_Q)) {
            return Ok(MinigameExit::WindowClosed);
        }
        let dt = if smoke { 1.0 / 30.0 } else { rl.get_frame_time().min(0.1) };
        let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
        let mut i = Input {
            mouse: rl.get_mouse_position(),
            pressed: rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT),
            down: rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
            released: rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT),
            enter: rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE),
            back: rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) || rl.is_key_pressed(KeyboardKey::KEY_P),
            leave: rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE),
            up: rl.is_key_pressed(KeyboardKey::KEY_UP) || rl.is_key_pressed(KeyboardKey::KEY_W),
            down_key: rl.is_key_pressed(KeyboardKey::KEY_DOWN) || rl.is_key_pressed(KeyboardKey::KEY_S),
        };
        if let Some(a) = &mut auto {
            let turn = after.map(|a| a - TURN_AT).filter(|t| *t > -LEARNED_PAUSE);
            a.drive(dt, screen, &s, (w, h), &mut i, &mut selected, (intro, &learn, turn, warn_amount(after)),
                    (fx.scaring(), fx.frightening()), (story.progress(), ending))?;
        }
        // Menús: flechas y mouse eligen, Enter o clic confirman
        let mut choice = None;
        if matches!(screen, Screen::Pause | Screen::Confirm | Screen::Over) && !buttons.is_empty() {
            if i.up {
                selected = (selected + buttons.len() - 1) % buttons.len();
            }
            if i.down_key {
                selected = (selected + 1) % buttons.len();
            }
            if let Some(k) = buttons.iter().position(|r| r.check_collision_point_rec(i.mouse)) {
                if auto.is_none() {
                    selected = k;
                }
                if i.pressed {
                    choice = Some(k);
                }
            }
            if i.enter {
                choice = Some(selected);
            }
        }
        match screen {
            Screen::Story => {
                story.update(dt);
                let on_pause = i.mouse.distance_to(art::pause_button(w)) < 26.0;
                let on_skip = story::skip_button(w, h).check_collision_point_rec(i.mouse);
                if i.back || (i.pressed && on_pause) {
                    paused_from = Screen::Story;
                    screen = Screen::Pause;
                    selected = 0;
                } else if (i.pressed && on_skip) || ((i.pressed || i.enter) && story.click()) {
                    screen = Screen::Intro;
                    intro = 0.0;
                    learn = Glyph::ALL.to_vec();
                    after = None;
                }
            }
            Screen::Intro if i.back || (i.pressed && i.mouse.distance_to(art::pause_button(w)) < 26.0) => {
                paused_from = Screen::Intro;
                screen = Screen::Pause;
                selected = 0;
                drawing = false;
                stroke.clear();
            }
            Screen::Intro => {
                intro += dt;
                if let Some(done) = track_stroke(&i, &mut drawing, &mut stroke) {
                    let pts: Vec<_> = done.iter().map(|p| (p.x, p.y)).collect();
                    match recognize(&pts) {
                        Some(g) if learn.contains(&g) => {
                            let (at, size) = art::learn_spot(g, w, h);
                            fx.learned(done, at, size);
                            learn.retain(|&x| x != g);
                        }
                        Some(g) => fx.miss(done, &format!("You already know the {} glyph", g.name())),
                        None if pts.len() > 2 => fx.fizzle(done, None),
                        None => {}
                    }
                }
                // Con todos aprendidos: un momento para ver las chispas, la bruja advierte (con un clic se sigue),
                // vuelve a bailar y la cámara gira hasta sus ojos
                if learn.is_empty() && after.is_none() {
                    after = Some(0.0);
                }
                if let Some(a) = &mut after {
                    *a += dt;
                    let hold = (LEARNED_PAUSE + WARN_IN + 0.8, LEARNED_PAUSE + WARN_IN + WARN_HOLD);
                    if i.pressed && *a > hold.0 && *a < hold.1 {
                        *a = hold.1;
                    }
                }
                fx.warn = warn_amount(after);
                if after.is_some_and(|a| a >= TURN_AT + INTRO_TURN) || i.enter {
                    fx.warn = 0.0;
                    screen = Screen::Play;
                    drawing = false;
                    stroke.clear();
                    fx.banner("WAVE 1", "Devils and ghosts are crashing the party");
                    sounds.crowd();
                }
            }
            Screen::Play => {
                let on_pause = i.mouse.distance_to(art::pause_button(w)) < 26.0;
                if i.back || (i.pressed && on_pause) {
                    paused_from = Screen::Play;
                    screen = Screen::Pause;
                    selected = 0;
                    drawing = false;
                    stroke.clear();
                } else if i.leave {
                    screen = Screen::Confirm;
                    selected = 1;
                    drawing = false;
                    stroke.clear();
                } else {
                    look = turn_head(&rl, look, dt);
                    if let Some(done) = track_stroke(&i, &mut drawing, &mut stroke) {
                        let pts: Vec<_> = done.iter().map(|p| (p.x, p.y)).collect();
                        match recognize(&pts) {
                            Some(g) => match s.cast(g) {
                                Cast::Hit { id, calmed } =>
                                    fx.spell(&art, &s, id, g, calmed, done, art.camera(View::Pov, fx.time, look)),
                                Cast::Miss => fx.fizzle(done, Some(g)),
                            },
                            None if pts.len() > 2 => fx.fizzle(done, None),
                            None => {}
                        }
                    }
                    let coming: Vec<(u32, Kind)> = s.creatures.iter()
                        .filter(|c| c.state == State::Approaching).map(|c| (c.id, c.kind)).collect();
                    s.update(dt);
                    // La que llegó a la bruja ya no está: se tira encima de la cámara
                    if let Some(&(_, kind)) = coming.iter().find(|(id, _)| !s.creatures.iter().any(|c| c.id == *id)) {
                        fx.scare(kind, (s.health < last.1).then_some(s.health));
                        sounds.roar(kind, true);
                        frights.scared();
                    }
                    if let Some(kind) = frights.update(dt, &s, &fx) {
                        fx.fright(kind);
                        sounds.roar(kind, false);
                    }
                    if s.phase != last.0 {
                        match s.phase {
                            Phase::Wave(2) => {
                                fx.banner("WAVE 2", "The sleep paralysis demons woke up");
                                sounds.crowd();
                            }
                            Phase::Victory => fx.calm_all(&s),
                            _ => {}
                        }
                    }
                    if s.health > last.1 {
                        fx.gain(s.health - 1);
                    }
                    last = (s.phase, s.health);
                    reporter.observe(&s, observer)?;
                    if s.phase == Phase::Victory {
                        // Ganó: baila feliz antes de la pantalla de victoria
                        screen = Screen::Ending;
                        ending = 0.0;
                        selected = 0;
                    } else if s.finished() {
                        screen = Screen::Over;
                        selected = 0;
                    }
                }
            }
            Screen::Ending => {
                ending += dt;
                if ending >= ENDING || i.pressed || i.enter {
                    screen = Screen::Over;
                    selected = 0;
                }
            }
            // Pausado en la historieta o en el baile: seguir o volver al sistema solar
            Screen::Pause if paused_from != Screen::Play => {
                if i.back {
                    screen = paused_from;
                }
                match choice {
                    Some(0) => screen = paused_from,
                    Some(1) => {
                        screen = Screen::Confirm;
                        selected = 1;
                    }
                    Some(_) => return give_up(observer),
                    None => {}
                }
            }
            Screen::Pause => {
                if i.back {
                    screen = Screen::Play;
                }
                match choice {
                    Some(0) => screen = Screen::Play,
                    Some(1) => {
                        seed += 1;
                        restart(&mut s, &mut fx, &mut last, seed);
                        sounds.crowd();
                        frights = Frights::new(seed);
                        screen = Screen::Play;
                    }
                    Some(2) => {
                        screen = Screen::Confirm;
                        selected = 1;
                    }
                    Some(_) => return give_up(observer),
                    None => {}
                }
            }
            Screen::Confirm => {
                if i.back {
                    screen = Screen::Pause;
                    selected = 0;
                }
                match choice {
                    Some(0) => return Ok(MinigameExit::Returned(MinigameResult::Abandoned)),
                    Some(_) => {
                        screen = Screen::Pause;
                        selected = 0;
                    }
                    None => {}
                }
            }
            Screen::Over => match choice {
                Some(0) => {
                    seed += 1;
                    restart(&mut s, &mut fx, &mut last, seed);
                    sounds.crowd();
                    frights = Frights::new(seed);
                    reporter.observe(&s, observer)?;
                    screen = Screen::Play;
                }
                Some(_) => {
                    return Ok(MinigameExit::Returned(if s.phase == Phase::Victory {
                        MinigameResult::Victory
                    } else {
                        MinigameResult::Defeat
                    }))
                }
                None => {}
            },
        }
        if screen != Screen::Pause && screen != Screen::Confirm {
            fx.update(dt);
        }
        if screen == Screen::Over {
            ending += dt;
        }
        // Lo que se ve atrás (pausado: lo que estaba)
        let base = if matches!(screen, Screen::Pause | Screen::Confirm) { paused_from } else { screen };
        // "Exploring" en la historieta, la práctica y el baile del final (con la victoria encima); "Conspiracy Theory"
        // en el juego
        let dancing = matches!(base, Screen::Ending | Screen::Over) && s.phase == Phase::Victory;
        let track = if matches!(base, Screen::Story | Screen::Intro) || dancing { Track::Intro } else { Track::Game };
        sounds.update(track, matches!(screen, Screen::Pause | Screen::Confirm), dt);
        sounds.approach(&s, look.0, dt, screen != Screen::Play);
        let view = match base {
            Screen::Intro => match after {
                Some(a) if a > TURN_AT => View::Turn((a - TURN_AT) / INTRO_TURN),
                _ => View::Show(warn_amount(after)),
            },
            _ => View::Pov,
        };
        art.prepare(&mut rl, &thread)?;
        let mut d = rl.begin_drawing(&thread);
        let won = s.phase == Phase::Victory;
        match base {
            Screen::Story => story::draw_story(&mut d, &thread, &mut art, &story, fx.time),
            Screen::Ending | Screen::Over if won => story::draw_ending(&mut d, &thread, &mut art, ending),
            _ => art::draw_game(&mut d, &thread, &mut art, &fx, &s, &stroke, view, look, base != Screen::Intro),
        }
        let font = &art.font;
        if base == Screen::Intro {
            // Los glifos aparecen al ratito de empezar el baile y se van cuando gira la cámara
            art::draw_learn(&mut d, font, &fx, &learn, &stroke, intro, 1.0);
            art::draw_warning(&mut d, font, warn_amount(after));
        }
        if matches!(screen, Screen::Story | Screen::Intro) {
            art::draw_pause_icon(&mut d, w);
        }
        buttons = match screen {
            Screen::Story | Screen::Intro | Screen::Play | Screen::Ending => Vec::new(),
            Screen::Pause if paused_from != Screen::Play => art::draw_menu(&mut d, font, "Paused", "The party can wait a moment.",
                &["Continue", "Return to the solar system", "I give up :("], selected),
            Screen::Pause => art::draw_menu(&mut d, font, "Paused", "The party can wait a moment.",
                &["Continue", "Restart", "Return to the solar system", "I give up :("], selected),
            Screen::Confirm => art::draw_menu(&mut d, font, "Leave the party?",
                "Going back to the solar system ends this attempt.", &["Return", "Cancel"], selected),
            Screen::Over if s.phase == Phase::Victory => art::draw_menu(&mut d, font, "Quiet at last",
                &format!("You sent {} of them back into the dark.", s.calmed_count()),
                &["Play again", "Return to the solar system"], selected),
            Screen::Over => art::draw_menu(&mut d, font, "They got you",
                "The dark was too crowded. Try again?", &["Retry", "Return to the solar system"], selected),
        };
        if let Some(a) = &mut auto {
            if let Some(name) = a.shot.take() {
                capture(&captures.join(format!("{name}.png")))?;
            }
        }
    }
}

/// La intro: la bruja baila sola en la luz mientras se aprenden los glifos (entran como dice `render::draw_learn`).
/// Con todos aprendidos, después de `LEARNED_PAUSE`: la cámara se acerca y la bruja levanta el dedo (`WARN_IN`), la
/// advertencia se queda `WARN_HOLD` (o hasta un clic), vuelve a bailar (`WARN_OUT`), baila un poco más
/// (`DANCE_AGAIN`) y la cámara gira hasta sus ojos en `INTRO_TURN`.
const LEARNED_PAUSE: f32 = 0.7;
const WARN_IN: f32 = 0.6;
const WARN_HOLD: f32 = 4.5;
const WARN_OUT: f32 = 0.6;
const DANCE_AGAIN: f32 = 0.8;
const TURN_AT: f32 = LEARNED_PAUSE + WARN_IN + WARN_HOLD + WARN_OUT + DANCE_AGAIN;
const INTRO_TURN: f32 = 1.5;

/// Cuánto está advirtiendo la bruja (0 a 1), según cuánto hace que se aprendieron todos los glifos.
fn warn_amount(after: Option<f32>) -> f32 {
    let Some(a) = after else { return 0.0 };
    let up = (a - LEARNED_PAUSE) / WARN_IN;
    let down = (LEARNED_PAUSE + WARN_IN + WARN_HOLD + WARN_OUT - a) / WARN_OUT;
    up.min(down).clamp(0.0, 1.0)
}

/// Los sustos de mentira: una criatura pegada a la cara un instante, sin daño. Cada oleada tiene por lo menos
/// `FRIGHTS_MIN` (a veces uno más), repartidos a lo largo de la oleada y a destiempo, nunca al empezar ni con el cartel
/// ni con el trío final. Esperan a que nada pueda llegar a la bruja en el próximo segundo y medio, pero si un susto ya
/// se atrasó `FRIGHT_LATE` segundos sale igual (siempre antes de que termine la oleada), respetando un segundo sin
/// sustos después del anterior.
struct Frights {
    rng: u64,
    /// La oleada que se está contando y cuándo toca cada susto que le queda (segundos de oleada; el próximo, primero).
    wave: u8,
    due: Vec<f32>,
    /// Cuánto hace del último susto, de mentira o de verdad.
    since: f32,
}

const FRIGHTS_MIN: usize = 3;
const FRIGHT_LATE: f32 = 5.0;

impl Frights {
    fn new(seed: u64) -> Self {
        Self { rng: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1, wave: 0, due: Vec::new(), since: 10.0 }
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    fn scared(&mut self) {
        self.since = 0.0;
    }

    /// Reparte los sustos de una oleada de `len` segundos, para que hasta el último atrasado salga antes del trío.
    fn schedule(&mut self, len: f32) {
        let n = FRIGHTS_MIN + usize::from(self.rand() < 0.5);
        let (first, span) = (8.0, len - 6.0 - 8.0 - FRIGHT_LATE - 2.0);
        self.due = (0..n).map(|i| first + span * (i as f32 + 0.15 + 0.7 * self.rand()) / n as f32).collect();
    }

    /// Avanza el reloj; devuelve la criatura del susto cuando toca uno.
    fn update(&mut self, dt: f32, s: &Session, fx: &Fx) -> Option<Kind> {
        self.since += dt;
        let Phase::Wave(n) = s.phase else { return None };
        if n != self.wave {
            self.wave = n;
            self.schedule(s.balance.wave_time[n as usize - 1]);
        }
        let due = *self.due.first()?;
        if s.phase_time < due || !fx.quiet() || s.creatures.iter().any(|c| c.boss) {
            return None;
        }
        // Nada llega a la bruja hasta un segundo después de que termine el susto; o ya se atrasó demasiado
        let clear = self.since >= 2.5 && s.creatures.iter().filter(|c| c.state == State::Approaching)
            .all(|c| (c.distance() - s.balance.hit_radius) / c.speed > FRIGHT + 1.0);
        let late = s.phase_time >= due + FRIGHT_LATE && self.since >= 1.0 + FRIGHT;
        if !clear && !late {
            return None;
        }
        self.due.remove(0);
        self.since = 0.0;
        let r = self.rand();
        Some(match n {
            1 if r < 0.5 => Kind::Devil,
            1 => Kind::Ghost,
            _ if r < 0.34 => Kind::Devil,
            _ if r < 0.67 => Kind::Ghost,
            _ => Kind::Demon,
        })
    }
}

/// El trazo con el botón izquierdo: lo junta mientras se arrastra y lo devuelve al soltar.
fn track_stroke(i: &Input, drawing: &mut bool, stroke: &mut Vec<Vector2>) -> Option<Vec<Vector2>> {
    if i.pressed {
        *drawing = true;
        *stroke = vec![i.mouse];
    } else if i.down && *drawing && stroke.last().is_none_or(|p| p.distance_to(i.mouse) > 3.0) {
        stroke.push(i.mouse);
    }
    if i.released && *drawing {
        *drawing = false;
        return Some(std::mem::take(stroke));
    }
    None
}

/// Girar la cabeza: arrastrar con el botón derecho, o flechas / A y D (y W y S para arriba y abajo). Con límites
/// cómodos: hasta 75° a los costados, 25° abajo y 55° arriba (para ver a las criaturas, que la superan).
fn turn_head(rl: &RaylibHandle, (mut yaw, mut pitch): (f32, f32), dt: f32) -> (f32, f32) {
    if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
        let m = rl.get_mouse_delta();
        yaw -= m.x * 0.18;
        pitch -= m.y * 0.18;
    }
    let key = |a: KeyboardKey, b: KeyboardKey| (rl.is_key_down(a) || rl.is_key_down(b)) as i32 as f32;
    yaw += (key(KeyboardKey::KEY_LEFT, KeyboardKey::KEY_A) - key(KeyboardKey::KEY_RIGHT, KeyboardKey::KEY_D)) * 80.0 * dt;
    pitch += (key(KeyboardKey::KEY_UP, KeyboardKey::KEY_W) - key(KeyboardKey::KEY_DOWN, KeyboardKey::KEY_S)) * 60.0 * dt;
    (yaw.clamp(-75.0, 75.0), pitch.clamp(-25.0, 55.0))
}

/// "I give up": cuenta como si hubiera ganado (así lo pidió Cami), y vuelve al sistema solar.
fn give_up(observer: &mut ResultObserver<'_>) -> Result<MinigameExit, Error> {
    observer(MinigameResult::Victory)?;
    Ok(MinigameExit::Returned(MinigameResult::Victory))
}

fn restart(s: &mut Session, fx: &mut Fx, last: &mut (Phase, u8), seed: u64) {
    *s = Session::new(Balance::default(), seed);
    fx.clear();
    fx.banner("WAVE 1", "Devils and ghosts are crashing the party");
    *last = (s.phase, s.health);
}

/// Guarda lo que hay en la pantalla: primero se dibuja lo que raylib tiene en cola (el HUD 2D) y se lee el tamaño
/// real del dibujo (no el de la ventana escalada por la pantalla).
fn capture(path: &Path) -> Result<(), Error> {
    let path = std::ffi::CString::new(path.to_string_lossy().as_bytes())?;
    unsafe {
        ffi::rlDrawRenderBatchActive();
        let (w, h) = (ffi::GetRenderWidth(), ffi::GetRenderHeight());
        let data = ffi::rlReadScreenPixels(w, h);
        let img = ffi::Image { data: data.cast(), width: w, height: h, mipmaps: 1,
                               format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32 };
        let ok = ffi::ExportImage(img, path.as_ptr());
        ffi::MemFree(data.cast());
        if !ok {
            return Err("No se pudo guardar la captura".into());
        }
    }
    Ok(())
}

struct Input {
    mouse: Vector2,
    pressed: bool,
    down: bool,
    released: bool,
    enter: bool,
    back: bool,
    leave: bool,
    up: bool,
    down_key: bool,
}

/// Puntos de pantalla de un glifo dibujado a mano, centrado en `c` con medio ancho `size`.
pub fn glyph_points(g: Glyph, c: Vector2, size: f32, wobble: f32) -> Vec<Vector2> {
    let corners: Vec<(f32, f32)> = match g {
        Glyph::Line => vec![(-1.0, 0.0), (1.0, 0.05)],
        Glyph::Vee => vec![(-0.8, -0.7), (0.0, 0.8), (0.8, -0.65)],
        Glyph::Circle => (0..=40).map(|k| {
            let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::TAU * k as f32 / 40.0;
            (0.8 * a.cos(), 0.8 * a.sin())
        }).collect(),
        Glyph::Bolt => vec![(0.3, -1.0), (-0.35, 0.1), (0.35, -0.05), (-0.3, 1.0)],
    };
    let mut out = Vec::new();
    let mut k = 0.0f32;
    for seg in corners.windows(2) {
        let (a, b) = (Vector2::new(c.x + seg[0].0 * size, c.y + seg[0].1 * size),
                      Vector2::new(c.x + seg[1].0 * size, c.y + seg[1].1 * size));
        let n = (a.distance_to(b) / 6.0).ceil().max(1.0) as usize;
        for i in 0..n {
            k += 1.0;
            let j = wobble * (k * 1.7).sin();
            out.push(a.lerp(b, i as f32 / n as f32) + Vector2::new(j, -j));
        }
    }
    let (l, _) = corners[corners.len() - 1];
    out.push(Vector2::new(c.x + l * size, c.y + corners[corners.len() - 1].1 * size));
    out
}

/// Jugador automático de la prueba: dibuja trazos de verdad (los mismos eventos que el mouse) y toma capturas.
struct Auto {
    goal: Goal,
    t: f32,
    pts: Vec<Vector2>,
    fed: usize,
    wait: f32,
    taken: Vec<&'static str>,
    shot: Option<&'static str>,
    scribbled: bool,
}

impl Auto {
    fn new(goal: Goal) -> Self {
        Self { goal, t: 0.0, pts: Vec::new(), fed: 0, wait: 0.0, taken: Vec::new(), shot: None, scribbled: false }
    }

    fn snap(&mut self, name: &'static str) {
        if !self.taken.contains(&name) {
            self.taken.push(name);
            self.shot = Some(name);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn drive(&mut self, dt: f32, screen: Screen, s: &Session, (w, h): (f32, f32), i: &mut Input, selected: &mut usize,
             (intro, learn, turn, warn): (f32, &[Glyph], Option<f32>, f32), (scaring, frightening): (bool, bool),
             ((frame, typed), ending): ((usize, bool), f32)) -> Result<(), Error> {
        self.t += dt;
        if self.t > 420.0 {
            return Err("HALLOWEEN_SMOKE_FAIL: tiempo agotado".into());
        }
        i.mouse = Vector2::new(w * 0.5, h * 0.62);
        i.pressed = false;
        i.down = false;
        i.released = false;
        i.enter = false;
        i.back = false;
        match screen {
            Screen::Story => {
                // Cada viñeta con su texto completo, y un clic cada tanto
                if typed {
                    self.snap(["story1", "story2", "story3", "story4"][frame]);
                }
                // Lee: espera a que se escriba el texto y un ratito más
                self.wait -= dt;
                if !typed {
                    self.wait = self.wait.max(0.6);
                } else if self.wait <= 0.0 && self.shot.is_none() {
                    i.pressed = true;
                    self.wait = 0.6;
                }
            }
            Screen::Ending => {
                if ending > 1.0 {
                    self.snap("ending");
                }
                if ending > 3.0 {
                    self.snap("disco");
                }
            }
            Screen::Intro => {
                if intro > 1.5 {
                    self.snap("entrance");
                }
                if intro > 2.8 {
                    self.snap("intro");
                }
                if warn > 0.99 {
                    self.snap("warn");
                }
                if turn.is_some_and(|t| t > INTRO_TURN * 0.55) {
                    self.snap("turn");
                }
                if !self.taken.contains(&"intro") {
                    return Ok(());
                }
                // Aprende los glifos dibujándolos (primero un garabato, para ver el aviso suave)
                self.wait -= dt;
                if self.pts.is_empty() && self.wait <= 0.0 {
                    let c = Vector2::new(w * 0.5, h * 0.5);
                    if !self.scribbled {
                        self.scribbled = true;
                        self.pts = (0..30).map(|k| c + Vector2::new((k as f32 * 1.3).sin() * 90.0, (k as f32 * 2.9).cos() * 40.0)).collect();
                    } else if let Some(g) = learn.first() {
                        self.pts = glyph_points(*g, art::learn_spot(*g, w, h).0, 90.0, 2.0);
                    }
                    self.fed = 0;
                }
                if self.feed(i, "learn", !self.scribbled || learn.len() == 2) {
                    self.wait = 0.5;
                }
            }
            Screen::Pause => {
                self.snap("pause");
                *selected = 2;
                i.enter = true;
            }
            Screen::Confirm => {
                self.snap("confirm");
                *selected = 0;
                i.enter = true;
            }
            Screen::Over => {
                self.snap(if s.phase == Phase::Victory { "victory" } else { "defeat" });
                if self.taken.len() > 1 && self.shot.is_none() {
                    *selected = 1;
                    i.enter = true;
                }
            }
            Screen::Play => {
                if self.goal == Goal::Abandoned && self.t > 6.0 {
                    i.back = true;
                    return Ok(());
                }
                if scaring {
                    self.snap("scare");
                }
                // Varios cuadros seguidos del susto de mentira (en algunos la criatura no está, a propósito)
                if frightening {
                    let next = ["fright_a", "fright_b", "fright_c", "fright_d"].into_iter().find(|n| !self.taken.contains(n));
                    if let Some(name) = next {
                        self.snap(name);
                    }
                }
                if s.creatures.iter().any(|c| c.state == State::Approaching && c.distance() < 2.5) {
                    self.snap("close");
                }
                // El trío final de cada oleada
                if s.creatures.iter().any(|c| c.boss && c.state == State::Approaching && c.distance() < 7.4) {
                    self.snap(if s.phase == Phase::Wave(1) { "boss1" } else { "boss2" });
                }
                // Cómo vienen: el diablo y el fantasma, a saltos
                for c in s.creatures.iter().filter(|c| c.state == State::Approaching && c.distance() < 6.0) {
                    match c.kind {
                        Kind::Devil => self.snap("devil"),
                        Kind::Ghost => self.snap("ghost"),
                        Kind::Demon => {}
                    }
                }
                if s.phase == Phase::Wave(1) {
                    for c in &s.creatures {
                        match c.state {
                            State::Calmed { wait, .. } if wait < 0.8 && wait > 0.6 => self.snap("melt"),
                            State::Calmed { wait, .. } if wait < 0.25 => self.snap("puddle"),
                            _ => {}
                        }
                    }
                }
                // El cartel de la oleada: al caer y ya quieto
                if s.phase == Phase::Wave(1) && s.phase_time > 0.07 {
                    self.snap("slam");
                }
                if s.phase == Phase::Wave(1) && s.phase_time > 0.6 {
                    self.snap("banner");
                }
                match s.phase {
                    Phase::Wave(1) if s.phase_time > 20.0 => self.snap("wave1"),
                    Phase::Wave(2) if s.phase_time > 25.0 => self.snap("wave2"),
                    _ => {}
                }
                if self.goal != Goal::Victory {
                    return Ok(());
                }
                self.wait -= dt;
                if self.pts.is_empty() && self.wait <= 0.0 {
                    let c = Vector2::new(w * 0.5, h * 0.62);
                    let near = s.creatures.iter()
                        .filter(|c| c.state == State::Approaching)
                        .filter(|c| s.seen(c) && c.distance() < 7.5)
                        .min_by(|a, b| a.distance().total_cmp(&b.distance()));
                    if let Some(target) = near {
                        self.pts = glyph_points(target.glyphs[0], c, 110.0, 2.0);
                    }
                    self.fed = 0;
                }
                if self.feed(i, "", false) {
                    self.wait = 0.2;
                }
            }
        }
        Ok(())
    }

    /// Mueve el mouse por el trazo que tiene preparado; `true` cuando lo suelta. Con `shot`, una captura a mitad
    /// del trazo (si `snap`).
    fn feed(&mut self, i: &mut Input, shot: &'static str, snap: bool) -> bool {
        if self.pts.is_empty() {
            return false;
        }
        let k = self.fed.min(self.pts.len() - 1);
        i.mouse = self.pts[k];
        if self.fed == 0 {
            i.pressed = true;
        } else if self.fed >= self.pts.len() {
            i.released = true;
            self.pts.clear();
            return true;
        }
        i.down = true;
        if snap && !shot.is_empty() && self.fed > 12 {
            self.snap(shot);
        }
        self.fed += 4;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cuenta los sustos de cada oleada. `kill`: la jugadora las maldice a todas; si no, la llenan de golpes.
    fn frights_per_wave(seed: u64, kill: bool) -> [u32; 2] {
        let mut s = Session::new(Balance::default(), seed);
        let (mut f, mut fx) = (Frights::new(seed), Fx::default());
        let mut count = [0; 2];
        while let Phase::Wave(n) = s.phase {
            if kill {
                let glyph = s.creatures.iter().filter(|c| c.state == State::Approaching && s.seen(c))
                    .min_by(|a, b| a.distance().total_cmp(&b.distance())).map(|c| c.glyphs[0]);
                if let Some(g) = glyph {
                    s.cast(g);
                }
            }
            s.health = 200;
            let before = s.creatures.len();
            s.update(1.0 / 30.0);
            if !kill && s.creatures.len() < before {
                f.scared();
                fx.scare(Kind::Devil, None);
            }
            if let Some(kind) = f.update(1.0 / 30.0, &s, &fx) {
                fx.fright(kind);
                // Nunca durante el trío final
                assert!(!s.creatures.iter().any(|c| c.boss));
                count[n as usize - 1] += 1;
            }
            fx.update(1.0 / 30.0);
        }
        count
    }

    #[test]
    fn cada_oleada_tiene_por_lo_menos_tres_sustos() {
        for seed in 1..40 {
            for kill in [true, false] {
                let count = frights_per_wave(seed, kill);
                assert!(count.iter().all(|&c| c >= FRIGHTS_MIN as u32), "semilla {seed}, maldice {kill}: {count:?}");
            }
        }
    }
}
