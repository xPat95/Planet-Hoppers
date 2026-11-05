//! Main menu and a single selector view shared by initial selection and future station visits.
use crate::{
    gameplay::selection::{
        GameState, SelectionMode, SelectionTransition, ShipCatalog, ShipSelection,
    },
    graphics::{
        hangar::{Hangar, HangarStyle},
        resources::save_frame,
        space::{Sky, SKY_BACKGROUND},
    },
    ships::ShipsConfig,
};
use raylib::{ffi, prelude::*};
use std::{collections::BTreeSet, path::PathBuf};

#[path = "front_end_ui.rs"]
mod ui;

type Error = Box<dyn std::error::Error>;
const INK: Color = Color::new(221, 234, 244, 255);
const MUTED: Color = Color::new(133, 157, 179, 255);
const CYAN: Color = Color::new(105, 213, 234, 255);
const MORPH_DURATION: f32 = 0.42;
const HANGAR_WIDE_HOLD: f32 = 0.35;
const HANGAR_APPROACH: f32 = 1.25;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Controls,
    Hangar,
}

pub fn run() -> Result<(), Error> {
    let mut smoke = false;
    let mut returns = false;
    let mut docking = false;
    let mut minecraft = false;
    let mut starwars = false;
    let mut goal = "minecraft".to_string();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--smoke-docking" => { smoke = true; docking = true; }
            "--smoke-starwars" => { smoke=true;starwars=true; }
            "--smoke-minecraft" => { smoke = true; minecraft = true; }
            "--smoke-returns" => {
                smoke = true;
                returns = true;
            }
            "--ship" => goal = args.next().ok_or("--ship necesita un identificador")?,
            "--help" | "-h" => {
                println!("cargo run --locked -- [--smoke | --smoke-returns | --smoke-docking | --smoke-minecraft | --smoke-starwars] [--ship halloween|starwars|smilingfriends|minecraft|regularshow|southpark]");
                println!("Sin opciones: menú principal. Desarrollo: cargo run --bin solarsystem o --bin southpark.");
                return Ok(());
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let selected = view(
        Screen::Main,
        SelectionMode::Initial,
        None,
        smoke.then_some(goal.clone()),
    )?;
    if let Some(SelectionTransition::StartAdventure(game)) = selected {
        // Initial abduction cinematic goes HERE, before solar_system::run_with_game.
        // A future station consumes ChangedShip inside its existing solar Session instead.
        let flags = if docking {
            vec!["--smoke-docking".into()]
        } else if starwars {
            vec!["--smoke-starwars".into()]
        } else if minecraft {
            vec!["--smoke-minecraft".into()]
        } else if returns {
            vec!["--smoke-returns".into()]
        } else if smoke {
            vec!["--smoke-launch".into()]
        } else {
            vec![]
        };
        let game = super::solar_system::run_with_game(game, flags.into_iter())?;
        if smoke {
            if game.first_ship() != goal || (!docking && (game.active_ship() != goal || game.unlocked_count() != 1))
            {
                return Err("El sistema solar no conservó la selección inicial".into());
            }
            println!(
                "FRONT_END_SMOKE_PASS active={} initial={} unlocked={}",
                game.active_ship(),
                game.first_ship(),
                game.unlocked_count()
            );
        }
    }
    Ok(())
}

/// Future station entry: returns a choice to the caller, never restarts the solar system.
/// Back returns None and leaves the caller's original GameState untouched.
pub fn select(
    mode: SelectionMode,
    game: Option<GameState>,
) -> Result<Option<SelectionTransition>, Error> {
    if mode != SelectionMode::Initial && game.is_none() {
        return Err("Cambiar nave requiere una partida activa".into());
    }
    view(Screen::Hangar, mode, game, None)
}

/// Called only after the flight window and its GPU resources have been released.
pub(crate) fn dock(game: GameState, smoke_goal: Option<String>) -> Result<Option<SelectionTransition>, Error> {
    view(Screen::Hangar, SelectionMode::Docked, Some(game), smoke_goal)
}

const MENU_BUTTON_GAP: f32 = 14.0;
/// Recorte del logo dentro de `newertitle.png` (1920x1080, el resto es transparente).
const TITLE_SOURCE: (f32, f32, f32, f32) = (338.0, 101.0, 1332.0, 926.0);
const TITLE_ASPECT: f32 = TITLE_SOURCE.2 / TITLE_SOURCE.3;
/// Entrada del menú: el logo crece desde chico (con un rebote) y después suben los botones, uno tras otro.
const TITLE_INTRO: f32 = 0.9;
const BUTTONS_DELAY: f32 = 0.55;
const BUTTON_STAGGER: f32 = 0.12;
const BUTTON_INTRO: f32 = 0.45;

/// Sube rápido, se pasa un poco y vuelve (0 → 1).
fn ease_out_back(t: f32) -> f32 {
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

struct MenuGalaxy {
    shader: Shader,
    resolution: i32,
    time: i32,
}

impl MenuGalaxy {
    fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Error> {
        let shader = rl.load_shader_from_memory(
            thread,
            None,
            Some(include_str!("../graphics/shaders/menu_galaxy.fs")),
        );
        let resolution = shader.get_shader_location("resolution");
        let time = shader.get_shader_location("time");
        if !shader.is_shader_valid() || resolution < 0 || time < 0 {
            return Err("Menu galaxy shader failed to compile".into());
        }
        Ok(Self {
            shader,
            resolution,
            time,
        })
    }

    fn draw(&mut self, d: &mut RaylibDrawHandle, width: i32, height: i32, time: f32) {
        self.shader
            .set_shader_value(self.resolution, Vector2::new(width as f32, height as f32));
        self.shader.set_shader_value(self.time, time);
        let mut layer = d.begin_shader_mode(&mut self.shader);
        layer.draw_rectangle(0, 0, width, height, Color::WHITE);
    }
}

struct MainMenuLayout {
    title_y: f32,
    title_height: f32,
    buttons_y: f32,
    button_height: f32,
}
impl MainMenuLayout {
    fn new(width: i32, height: i32) -> Self {
        // The artwork rises to the right, leaving room for the buttons below that end.
        let title_height = (height as f32 * 0.77).min((width as f32 - 70.0) / TITLE_ASPECT);
        let title_y = ((height as f32 - title_height) * 0.30 + height as f32 * 0.035).max(0.0);
        let button_height = (height as f32 * 0.06).clamp(48.0, 66.0);
        let buttons_y = height as f32 * 0.76;
        Self {
            title_y,
            title_height,
            buttons_y,
            button_height,
        }
    }
}

fn menu_button(w: i32, h: i32, row: usize) -> Rectangle {
    let row_width = w as f32 * 0.42;
    let width = (row_width - 2.0 * MENU_BUTTON_GAP) / 3.0;
    let layout = MainMenuLayout::new(w, h);
    Rectangle::new(
        w as f32 * 0.52 + row as f32 * (width + MENU_BUTTON_GAP),
        layout.buttons_y,
        width,
        layout.button_height,
    )
}

/// Degradé del logo, de arriba (t = 0) hacia abajo (t = 1): cielo azul que se aclara hasta casi blanco, un
/// horizonte marcado y después naranja que se vuelve amarillo claro.
const CHROME: [(f32, (u8, u8, u8)); 6] = [
    (0.0, (38, 128, 226)),
    (0.46, (226, 246, 255)),
    (0.5, (160, 58, 30)),
    (0.64, (238, 104, 48)),
    (0.84, (255, 196, 74)),
    (1.0, (255, 244, 196)),
];
/// Inclinación de los botones (como las letras del logo): cuánto se corre el borde de arriba a la derecha.
const BUTTON_SLANT: f32 = 0.32;

fn chrome(t: f32, lift: f32) -> Color {
    let i = CHROME.iter().rposition(|(at, _)| *at <= t).unwrap_or(0).min(CHROME.len() - 2);
    let ((a, ca), (b, cb)) = (CHROME[i], CHROME[i + 1]);
    let k = ((t - a) / (b - a)).clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| {
        let v = x as f32 + (y as f32 - x as f32) * k;
        (v + (255.0 - v) * lift) as u8
    };
    Color::new(mix(ca.0, cb.0), mix(ca.1, cb.1), mix(ca.2, cb.2), 255)
}

/// Paralelogramo inclinado, línea por línea: `color(t)` da el color de cada fila (t de 0 arriba a 1 abajo).
fn slanted(d: &mut RaylibDrawHandle, x: f32, y: f32, w: f32, h: f32, color: impl Fn(f32) -> Color) {
    let rows = h.round().max(1.0) as i32;
    for k in 0..rows {
        let t = (k as f32 + 0.5) / rows as f32;
        let shift = (1.0 - t) * h * BUTTON_SLANT;
        d.draw_rectangle((x + shift).round() as i32, (y + k as f32).round() as i32, w.round() as i32, 1, color(t));
    }
}

fn draw_menu_button(
    d: &mut RaylibDrawHandle,
    font: &Font,
    rect: Rectangle,
    label: &str,
    mouse: Vector2,
) {
    let hovered = rect.check_collision_point_rec(mouse);
    let lift = if hovered { 3.0 } else { 0.0 };
    // El paralelogramo entra en el rectángulo del botón (que es el que recibe los clics)
    let skew = rect.height * BUTTON_SLANT;
    let (x, y, w, h) = (rect.x, rect.y - lift, rect.width - skew, rect.height);
    let cyan = Color::new(99, 205, 255, 255);
    let cobalt = Color::new(19, 4, 146, 255);
    // Sombra dura hacia abajo, borde celeste y borde azul oscuro, como el logo
    slanted(d, x + 3.0, y + 6.0 + lift, w, h, |_| Color::new(10, 2, 70, 255));
    slanted(d, x - 3.0, y - 3.0, w + 6.0, h + 6.0, |_| cyan);
    slanted(d, x, y, w, h, |_| cobalt);
    slanted(d, x + 3.0, y + 3.0, w - 6.0, h - 6.0, |t| chrome(t, if hovered { 0.18 } else { 0.0 }));
    // Mismo tamaño en los tres botones: el más grande que deja entrar "CONTROLES" (la palabra más larga)
    let fit = font.measure_text("CONTROLES", 100.0, 1.0).x.max(font.measure_text(label, 100.0, 1.0).x) / 100.0;
    let size = (h * 0.66).min((w - 30.0) / fit).max(12.0);
    let text = font.measure_text(label, size, 1.0);
    let at = Vector2::new(
        x + skew * 0.5 + (w - text.x) * 0.5,
        y + (h - text.y) * 0.5,
    );
    // Letras blancas con contorno celeste completo (el texto repetido en un círculo alrededor)
    let outline = Color::new(38, 150, 235, 255);
    let r = (size * 0.09).max(2.0);
    for k in 0..16 {
        let a = k as f32 / 16.0 * std::f32::consts::TAU;
        d.draw_text_ex(font, label, at + Vector2::new(a.cos() * r, a.sin() * r), size, 1.0, outline);
    }
    d.draw_text_ex(font, label, at, size, 1.0, Color::WHITE);
}

fn menu_random(index: u32, salt: u32) -> f32 {
    let mut x = index.wrapping_mul(0x9e37_79b9) ^ salt;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    (x >> 8) as f32 / 16_777_216.0
}

fn draw_moving_stars(d: &mut RaylibDrawHandle, width: i32, height: i32, time: f32) {
    for i in 0..130 {
        let speed = 2.0 + menu_random(i, 0x2345) * 5.0;
        let x = (menu_random(i, 0x7183) * width as f32 + time * speed).rem_euclid(width as f32);
        let y = (menu_random(i, 0x9e12) * height as f32 + time * speed * 0.28)
            .rem_euclid(height as f32);
        let twinkle = 0.72 + 0.28 * (time * (0.8 + speed * 0.1) + i as f32).sin();
        let alpha = (135.0 * twinkle) as u8;
        let tint = if i % 3 == 0 {
            Color::new(131, 222, 255, alpha)
        } else {
            Color::new(169, 209, 255, alpha)
        };
        d.draw_circle_v(Vector2::new(x, y), if i % 7 == 0 { 1.5 } else { 0.8 }, tint);
    }
}
fn back_button(w: i32) -> Rectangle {
    Rectangle::new(w as f32 - 182.0, 24.0, 150.0, 42.0)
}
fn eased(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn blended_camera(a: Camera3D, b: Camera3D, t: f32) -> Camera3D {
    let t = eased(t);
    Camera3D::perspective(
        a.position + (b.position - a.position) * t,
        a.target + (b.target - a.target) * t,
        Vector3::up(),
        a.fovy + (b.fovy - a.fovy) * t,
    )
}

fn draw_ship_morph(d: &mut RaylibDrawHandle, base: Vector2, top: Vector2, center: Vector2, progress: f32) {
    let peak = (std::f32::consts::PI * progress).sin().max(0.0);
    let mut light = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
    for step in 0..=10 {
        let t = step as f32 / 10.0;
        let at = base + (top - base) * t;
        let fade = peak * (0.55 + 0.45 * (std::f32::consts::PI * t).sin());
        for (radius, alpha) in [(105.0, 7.0), (55.0, 11.0), (24.0, 13.0)] {
            light.draw_circle_gradient(at.x as i32, at.y as i32, radius,
                Color::new(85, 186, 255, (alpha * fade) as u8), Color::new(40, 131, 255, 0));
        }
    }
    let flash = (1.0 - (progress - 0.5).abs() / 0.14).clamp(0.0, 1.0);
    light.draw_circle_gradient(center.x as i32, center.y as i32, 65.0 + 55.0 * flash,
        Color::new(226, 251, 255, (155.0 * flash) as u8), Color::new(70, 178, 255, 0));
}
fn arrow(w: i32, h: i32, right: bool) -> Rectangle {
    Rectangle::new(
        if right { w as f32 - 90.0 } else { 32.0 },
        h as f32 * 0.49,
        58.0,
        72.0,
    )
}
fn launch_button(w: i32, h: i32) -> Rectangle {
    Rectangle::new(w as f32 * 0.5 - 170.0, h as f32 - 92.0, 340.0, 52.0)
}

fn view(
    mut screen: Screen,
    mode: SelectionMode,
    game: Option<GameState>,
    smoke_goal: Option<String>,
) -> Result<Option<SelectionTransition>, Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json"))?;
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships)?;
    let goal_index = smoke_goal
        .as_ref()
        .filter(|id| id.as_str() != "@cancel")
        .map(|id| {
            catalog
                .choices
                .iter()
                .position(|c| &c.id == id)
                .ok_or("Nave de prueba desconocida")
        })
        .transpose()?;
    let embedded = screen == Screen::Hangar;
    let mut selection = ShipSelection::new(mode, game.as_ref(), &catalog);
    let entry_ship = selection.index;
    let mut shown = None;
    let mut hangar_elapsed = 0.0f32;
    let mut morph: Option<f32> = None;
    let mut close_focus: Option<Camera3D> = None;
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .msaa_4x()
        .title("Planet Hoppers")
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(960, 640);
    rl.enable_cursor();
    rl.set_target_fps(if smoke_goal.is_some() { 0 } else { 60 });
    if smoke_goal.is_some() {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    let sky = Sky::new();
    let title_card = rl.load_texture(&thread, &root.join("newertitle.png").to_string_lossy())?;
    let mut galaxy = MenuGalaxy::load(&mut rl, &thread)?;
    let menu_font = rl.load_font_ex(
        &thread,
        &root
            .join("assets/fonts/Bungee-Regular.ttf")
            .to_string_lossy(),
        64,
        None,
    )?;
    // Fuente nítida al achicarla (se carga grande y se escala según el botón)
    unsafe { ffi::SetTextureFilter(menu_font.texture, ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
    let secondary_title = ui::load_font(&root.join("assets/fonts/Bungee-Regular.ttf"))?;
    let body_font = ui::load_font(&root.join("assets/fonts/TitilliumWeb-Bold.ttf"))?;
    let mut hangar = Hangar::load(&mut rl, &thread, &root, HangarStyle::default())?;
    let captures = root.join("target/front-end");
    if smoke_goal.is_some() {
        std::fs::create_dir_all(&captures)?;
    }
    let mut seen = BTreeSet::new();
    let mut controls_seen = false;
    let mut back_seen = false;
    let mut frame = 0u32;
    // Segundos en el menú principal (la entrada se ve una vez, al abrir el juego). En las pruebas ya terminó.
    let mut intro = if smoke_goal.is_some() { 10.0f32 } else { 0.0 };
    loop {
        if rl.window_should_close()
            || ((rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                && rl.is_key_pressed(KeyboardKey::KEY_Q))
        {
            return Ok(if mode==SelectionMode::Docked {Some(SelectionTransition::Quit)} else {None});
        }
        // Exercise the same menu layout on real window resizes before the existing smoke flow.
        if smoke_goal.is_some() {
            match frame {
                0 => rl.set_window_size(960, 640),
                1 => rl.set_window_size(1920, 1080),
                2 | 4 => rl.set_window_size(1280, 800),
                3 => rl.set_window_size(1024, 1024),
                6 | 30 if !embedded => rl.set_window_size(960, 640),
                7 | 31 if !embedded => rl.set_window_size(1920, 1080),
                9 | 32 if !embedded => rl.set_window_size(1024, 1024),
                8 | 10 | 33 if !embedded => rl.set_window_size(1280, 800),
                _ => {}
            }
        }
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        if embedded && smoke_goal.is_some() {
            for key in [KeyboardKey::KEY_ENTER,KeyboardKey::KEY_ESCAPE,KeyboardKey::KEY_RIGHT] {
                unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent { frame:0,type_:1,params:[key as i32,0,0,0] }); }
            }
            let steps=(goal_index.unwrap_or(entry_ship)+6-entry_ship)%6;
            let key=if frame>=12 && frame<12+steps as u32*4 && (frame-12)%4==0 {
                Some(KeyboardKey::KEY_RIGHT)
            } else if frame==44 {
                Some(if smoke_goal.as_deref()==Some("@cancel") {KeyboardKey::KEY_ESCAPE} else {KeyboardKey::KEY_ENTER})
            } else {None};
            if let Some(key)=key { unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {frame:0,type_:2,params:[key as i32,0,0,0]}); } }
        } else if let Some(goal) = goal_index {
            inject_smoke(frame, w, h, goal);
        }
        let mouse = rl.get_mouse_position();
        let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let hit = |r: Rectangle| clicked && r.check_collision_point_rec(mouse);
        let escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
        match screen {
            Screen::Main => {
                if enter || hit(menu_button(w, h, 0)) {
                    screen = Screen::Hangar;
                    selection = ShipSelection::new(mode, game.as_ref(), &catalog);
                    shown = None;
                    morph = None;
                    hangar_elapsed = 0.0;
                    close_focus = None;
                } else if hit(menu_button(w, h, 1)) {
                    screen = Screen::Controls;
                } else if hit(menu_button(w, h, 2)) || escape {
                    return Ok(None);
                }
            }
            Screen::Controls => {
                if escape || hit(back_button(w)) {
                    screen = Screen::Main;
                }
            }
            Screen::Hangar => {
                if escape || hit(back_button(w)) {
                    if embedded {
                        return Ok(None);
                    }
                    screen = Screen::Main;
                } else {
                    if rl.is_key_pressed(KeyboardKey::KEY_LEFT) || hit(arrow(w, h, false)) {
                        selection.navigate(-1, &catalog);
                    }
                    if rl.is_key_pressed(KeyboardKey::KEY_RIGHT) || hit(arrow(w, h, true)) {
                        selection.navigate(1, &catalog);
                    }
                    if selection.can_confirm(&catalog, game.as_ref())
                        && (enter || hit(launch_button(w, h)))
                    {
                        let transition = selection.confirm(&catalog, game.as_ref())?;
                        if smoke_goal.is_some() {
                            if embedded && Some(selection.index)!=goal_index {return Err("El hangar no confirmó la nave solicitada".into());}
                            if !embedded && (!controls_seen
                                || !back_seen
                                || seen.len() != 6
                                || Some(selection.index) != goal_index)
                            {
                                return Err("Recorrido de menú/selector incompleto".into());
                            }
                            let report = serde_json::json!({"status":"PASS", "previews":seen, "controls":controls_seen, "back_to_menu":back_seen, "selected":catalog.choices[selection.index].id, "physical_input_verified":false});
                            std::fs::write(
                                captures.join("selection-validation.json"),
                                serde_json::to_string_pretty(&report)?,
                            )?;
                            println!("{}_SMOKE_PASS {report}",if embedded {"DOCK_SELECTION"} else {"SELECTION"});
                        }
                        return Ok(Some(transition));
                    }
                }
            }
        }
        let dt = if smoke_goal.is_some() {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.1)
        };
        if screen == Screen::Hangar {
            hangar_elapsed += dt;
            if shown.is_none() {
                hangar.exhibit(&mut rl, &thread, &root,
                    &catalog.model(&catalog.choices[selection.index], &ships));
                shown = Some(selection.index);
            } else if morph.is_none() && shown != Some(selection.index) {
                morph = Some(0.0);
            }
            if let Some(progress) = morph {
                let next = progress + dt / MORPH_DURATION;
                if progress < 0.5 && next >= 0.5 {
                    hangar.exhibit(&mut rl, &thread, &root,
                        &catalog.model(&catalog.choices[selection.index], &ships));
                    shown = Some(selection.index);
                }
                morph = (next < 1.0).then_some(next);
            }
            if smoke_goal.is_some() && morph.is_none() && shown == Some(selection.index)
                && hangar.placeholder != catalog.choices[selection.index].ship_id.is_none() {
                return Err("Hangar exhibit did not load".into());
            }
        }
        hangar.update(dt);
        if screen == Screen::Main {
            intro += dt;
        }
        if smoke_goal.is_some() {
            match (catalog.choices[selection.index].id.as_str(), frame) {
                ("halloween", 117) => hangar.set_preview_angle(115.0),
                ("halloween", 118) => hangar.set_preview_angle(205.0),
                ("starwars", 157) => hangar.set_preview_angle(90.0),
                ("smilingfriends", 197) | ("minecraft", 237) => hangar.set_preview_angle(115.0),
                ("smilingfriends", 198) | ("minecraft", 238) => hangar.set_preview_angle(205.0),
                _ => {}
            }
        }
        let camera = if screen == Screen::Hangar {
            let approach = ((hangar_elapsed - HANGAR_WIDE_HOLD) / HANGAR_APPROACH).clamp(0.0, 1.0);
            let destination = hangar.close_camera(&catalog.choices[selection.index].id);
            let focus = close_focus.get_or_insert(destination);
            let current = *focus;
            let follow = 1.0 - (-dt * 7.0).exp();
            *focus = Camera3D::perspective(
                current.position + (destination.position - current.position) * follow,
                current.target + (destination.target - current.target) * follow,
                Vector3::up(),
                current.fovy + (destination.fovy - current.fovy) * follow,
            );
            blended_camera(hangar.wide_camera(), *focus, approach)
        } else {
            let time = rl.get_time() as f32;
            Camera3D::perspective(
                Vector3::zero(),
                Vector3::new(
                    (time * 0.07).sin() * 0.025,
                    (time * 0.05).sin() * 0.016,
                    -1.0,
                ),
                Vector3::up(),
                50.0,
            )
        };
        let morph_points = if screen == Screen::Hangar && morph.is_some() {
            Some((
                rl.get_world_to_screen(Vector3::new(0.0, 0.9, 0.0), camera),
                rl.get_world_to_screen(Vector3::new(0.0, 4.5, 0.0), camera),
                rl.get_world_to_screen(hangar.ship_center(), camera),
            ))
        } else { None };
        let sky_frame = sky.project(&rl, camera);
        let menu_time = rl.get_time() as f32;
        let left_active = rl.is_key_down(KeyboardKey::KEY_LEFT);
        let right_active = rl.is_key_down(KeyboardKey::KEY_RIGHT);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(SKY_BACKGROUND);
        if matches!(screen, Screen::Main | Screen::Controls) {
            galaxy.draw(&mut d, w, h, menu_time);
        }
        if screen == Screen::Hangar {
            hangar.draw_backdrop(&mut d, camera, Vector2::new(w as f32, h as f32));
        } else {
            sky.draw(&mut d, &sky_frame);
        }
        if matches!(screen, Screen::Main | Screen::Controls) {
            draw_moving_stars(&mut d, w, h, menu_time);
        }
        match screen {
            Screen::Main => {
                let layout = MainMenuLayout::new(w, h);
                // El logo crece desde su centro, del 12 % a su tamaño, y aparece de a poco
                let t = (intro / TITLE_INTRO).min(1.0);
                let grow = 0.12 + 0.88 * ease_out_back(t);
                let title_height = layout.title_height * grow;
                let title_width = title_height * TITLE_ASPECT;
                let center_y = layout.title_y + layout.title_height * 0.5;
                let (sx, sy, sw, sh) = TITLE_SOURCE;
                d.draw_texture_pro(
                    &title_card,
                    Rectangle::new(sx, sy, sw, sh),
                    Rectangle::new(
                        (w as f32 - title_width) * 0.5,
                        center_y - title_height * 0.5,
                        title_width,
                        title_height,
                    ),
                    Vector2::zero(),
                    0.0,
                    Color::new(255, 255, 255, ((t * 3.0).min(1.0) * 255.0) as u8),
                );
                for (i, label) in ["JUGAR", "CONTROLES", "SALIR"].iter().enumerate() {
                    // Cada botón sube desde abajo de la pantalla cuando el logo ya creció
                    let bt = ((intro - BUTTONS_DELAY - i as f32 * BUTTON_STAGGER) / BUTTON_INTRO).clamp(0.0, 1.0);
                    let mut rect = menu_button(w, h, i);
                    rect.y += (1.0 - bt).powi(3) * h as f32 * 0.45;
                    draw_menu_button(&mut d, &menu_font, rect, label, mouse);
                }
            }
            Screen::Controls => {
                ui::controls(&mut d, &secondary_title, &body_font, w, h);
                ui::action(&mut d, &secondary_title, back_button(w), "VOLVER", mouse, true);
            }
            Screen::Hangar => {
                {
                    let mut d3 = d.begin_mode3D(camera);
                    hangar.draw(&mut d3, camera, morph);
                }
                if let (Some(progress), Some((base, top, center))) = (morph, morph_points) {
                    draw_ship_morph(&mut d, base, top, center, progress);
                }
                ui::hangar_shade(&mut d, w, h);
                let choice = &catalog.choices[selection.index];
                let model = catalog.model(choice, &ships);
                let provisional = format!("NAVE PROVISIONAL / {}", choice.name);
                ui::hangar_title(
                    &mut d, &secondary_title, &body_font, &choice.name,
                    if hangar.placeholder { &provisional } else { &model.name },
                    if mode == SelectionMode::Initial { "CIUDADELA / TU PRIMERA NAVE" }
                    else { "CIUDADELA / HANGAR DEL PRESIDIUM" }, w,
                );
                ui::action(&mut d, &secondary_title, back_button(w), "VOLVER", mouse, true);
                ui::arrow(&mut d, arrow(w, h, false), false, mouse, left_active);
                ui::arrow(&mut d, arrow(w, h, true), true, mouse, right_active);
                let status = if mode == SelectionMode::Initial {
                    "Elige libremente tu nave inicial"
                } else if mode == SelectionMode::Docked {
                    "Flota de la estación / cambio libre durante esta sesión"
                } else if selection.can_confirm(&catalog, game.as_ref()) {
                    "Nave desbloqueada"
                } else {
                    "Nave bloqueada"
                };
                ui::centered(&mut d, &body_font, status, w, h as f32 - 123.0, 18.0, INK);
                let enabled = selection.can_confirm(&catalog, game.as_ref());
                ui::action(
                    &mut d, &secondary_title, launch_button(w, h),
                    if !enabled { "BLOQUEADA" }
                    else if mode == SelectionMode::Initial { "DESPEGAR" }
                    else if mode == SelectionMode::Docked { "DESPEGAR CON ESTA NAVE" }
                    else { "ACTIVAR NAVE" }, mouse, enabled,
                );
                let count = format!("{:02} / {:02}", selection.index + 1, catalog.choices.len());
                ui::text(&mut d, &secondary_title, &count, 38.0, h as f32 - 34.0, 18.0, CYAN);
                ui::centered(&mut d, &body_font, "FLECHAS / elegir     ENTER / despegar     ESC / volver", w, h as f32 - 26.0, 16.0, INK);
            }
        }
        if smoke_goal.is_some() {
            if !embedded && ((frame == 24 && hangar_elapsed >= HANGAR_WIDE_HOLD)
                || (frame == 116 && hangar_elapsed < HANGAR_WIDE_HOLD + HANGAR_APPROACH)) {
                return Err("La cámara del hangar no completó su acercamiento".into());
            }
            if embedded && frame==6 {
                if selection.index!=entry_ship { return Err("La entrada al hangar cambió la nave".into()); }
                save_frame(&captures.join("docked-entry.png"),w,h)?;
            }
            if embedded && frame==40 { save_frame(&captures.join("docked-selected.png"),w,h)?; }
            if !embedded && frame == 355 && (screen != Screen::Hangar || selection.index != 4) {
                return Err("La flecha izquierda no cambió la nave".into());
            }
            let name = match frame {
                0 | 1 | 3 => Some(format!("main-menu-{w}x{h}.png")),
                2 => Some("main-menu.png".to_string()),
                8 => {
                    controls_seen = screen == Screen::Controls;
                    Some("controls.png".into())
                }
                6 | 7 | 9 if !embedded => Some(format!("controls-{w}x{h}.png")),
                30 | 31 | 32 if !embedded => Some(format!("hangar-{w}x{h}.png")),
                24 if !embedded => {
                    seen.insert(catalog.choices[selection.index].id.clone());
                    save_frame(&captures.join("hangar-halloween.png"), w, h)?;
                    Some("hangar-complete.png".into())
                }
                70 if !embedded => Some("hangar-approach.png".into()),
                116 if !embedded => {
                    if catalog.choices[selection.index].id == "halloween" {
                        save_frame(&captures.join("hangar-halloween-close.png"), w, h)?;
                    }
                    Some("hangar-arrival.png".into())
                }
                117 | 118 if screen == Screen::Hangar
                    && catalog.choices[selection.index].id == "halloween" =>
                {
                    Some(format!("hangar-halloween-{}.png",
                        if frame == 117 { "side" } else { "back" }))
                }
                133 if !embedded => Some("hangar-transform.png".into()),
                356 if !embedded => Some("hangar-close-up.png".into()),
                155 | 195 | 235 | 275 | 315 if screen == Screen::Hangar => {
                    seen.insert(catalog.choices[selection.index].id.clone());
                    Some(format!(
                        "hangar-{}.png",
                        catalog.choices[selection.index].id
                    ))
                }
                157 if screen == Screen::Hangar
                    && catalog.choices[selection.index].id == "starwars" =>
                {
                    Some("hangar-starwars-sway.png".into())
                }
                197 | 198
                    if screen == Screen::Hangar
                        && catalog.choices[selection.index].id == "smilingfriends" =>
                {
                    Some(format!(
                        "hangar-smilingfriends-{}.png",
                        if frame == 197 { "side" } else { "back" }
                    ))
                }
                237 | 238
                    if screen == Screen::Hangar
                        && catalog.choices[selection.index].id == "minecraft" =>
                {
                    Some(format!(
                        "hangar-minecraft-{}.png",
                        if frame == 237 { "side" } else { "back" }
                    ))
                }
                364 => {
                    back_seen = screen == Screen::Main;
                    Some("back-to-menu.png".into())
                }
                _ => None,
            };
            if let Some(name) = name {
                save_frame(&captures.join(name), w, h)?;
            }
            if frame > 560 {
                return Err("La prueba del menú no llegó al lanzamiento".into());
            }
        }
        drop(d);
        frame += 1;
    }
}

fn inject_smoke(frame: u32, w: i32, h: i32, goal: usize) {
    fn event(kind: u32, params: [i32; 4]) {
        unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: kind,
                params,
            });
        }
    }
    for key in [
        KeyboardKey::KEY_ESCAPE,
        KeyboardKey::KEY_ENTER,
        KeyboardKey::KEY_LEFT,
        KeyboardKey::KEY_RIGHT,
    ] {
        event(1, [key as i32, 0, 0, 0]);
    }
    event(5, [0, 0, 0, 0]);
    let key = |key| event(2, [key as i32, 0, 0, 0]);
    let click = |r: Rectangle| {
        event(
            7,
            [
                (r.x + r.width * 0.5) as i32,
                (r.y + r.height * 0.5) as i32,
                0,
                0,
            ],
        );
        event(6, [0, 0, 0, 0]);
    };
    match frame {
        5 => click(menu_button(w, h, 1)),
        12 => key(KeyboardKey::KEY_ESCAPE),
        16 | 370 => click(menu_button(w, h, 0)),
        120 | 160 | 200 | 240 | 280 => click(arrow(w, h, true)),
        320 => click(arrow(w, h, false)),
        360 => click(back_button(w)),
        f if f >= 470 && f < 470 + goal as u32 * 4 && (f - 470) % 4 == 0 => {
            key(KeyboardKey::KEY_RIGHT)
        }
        510 => click(launch_button(w, h)),
        _ => {}
    }
}
