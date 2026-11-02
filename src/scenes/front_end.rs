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

type Error = Box<dyn std::error::Error>;
const INK: Color = Color::new(221, 234, 244, 255);
const MUTED: Color = Color::new(133, 157, 179, 255);
const CYAN: Color = Color::new(105, 213, 234, 255);
const HANGAR_HEADER: f32 = 151.0;
const HANGAR_FOOTER: f32 = 156.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Main,
    Controls,
    Hangar,
}

pub fn run() -> Result<(), Error> {
    let mut smoke = false;
    let mut returns = false;
    let mut goal = "minecraft".to_string();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--smoke-returns" => {
                smoke = true;
                returns = true;
            }
            "--ship" => goal = args.next().ok_or("--ship necesita un identificador")?,
            "--help" | "-h" => {
                println!("cargo run --locked -- [--smoke | --smoke-returns] [--ship halloween|starwars|smilingfriends|minecraft|regularshow|southpark]");
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
        let flags = if returns {
            vec!["--smoke-returns".into()]
        } else if smoke {
            vec!["--smoke-launch".into()]
        } else {
            vec![]
        };
        let game = super::solar_system::run_with_game(game, flags.into_iter())?;
        if smoke {
            if game.active_ship() != goal || game.first_ship() != goal || game.unlocked_count() != 1
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
    if mode == SelectionMode::Change && game.is_none() {
        return Err("Cambiar nave requiere una partida activa".into());
    }
    view(Screen::Hangar, mode, game, None)
}

const MENU_BUTTON_HEIGHT: f32 = 48.0;
const MENU_BUTTON_GAP: f32 = 14.0;

struct MainMenuLayout {
    top: f32,
    planet_y: f32,
    hoppers_y: f32,
    tagline_y: f32,
    buttons_y: f32,
    bottom: f32,
}
impl MainMenuLayout {
    fn new(height: i32) -> Self {
        let accent_height = 3.0;
        let planet_size = 40.0;
        let hoppers_size = 76.0;
        let tagline_size = 18.0;
        let title_gap = 18.0;
        let title_line_gap = 6.0;
        let button_top_gap = 30.0;
        let title_height = accent_height + title_gap + planet_size + title_line_gap
            + hoppers_size + title_gap + tagline_size;
        let buttons_height = 3.0 * MENU_BUTTON_HEIGHT + 2.0 * MENU_BUTTON_GAP;
        let content_height = title_height + button_top_gap + buttons_height;
        let top = (height as f32 - content_height) * 0.5;
        let planet_y = top + accent_height + title_gap;
        let hoppers_y = planet_y + planet_size + title_line_gap;
        let tagline_y = hoppers_y + hoppers_size + title_gap;
        let buttons_y = top + title_height + button_top_gap;
        Self { top, planet_y, hoppers_y, tagline_y, buttons_y, bottom: top + content_height }
    }
}

fn centered_text(d: &mut RaylibDrawHandle, text: &str, width: i32, y: i32, size: i32, color: Color) {
    d.draw_text(text, (width - d.measure_text(text, size)) / 2, y, size, color);
}

fn menu_button(w: i32, h: i32, row: usize) -> Rectangle {
    let width = 310.0_f32.min((w as f32 - 64.0).max(1.0));
    let layout = MainMenuLayout::new(h);
    Rectangle::new(
        (w as f32 - width) * 0.5,
        layout.buttons_y + row as f32 * (MENU_BUTTON_HEIGHT + MENU_BUTTON_GAP),
        width,
        MENU_BUTTON_HEIGHT,
    )
}
fn back_button(w: i32) -> Rectangle {
    Rectangle::new(w as f32 - 182.0, 24.0, 150.0, 42.0)
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

fn button(
    d: &mut RaylibDrawHandle,
    rect: Rectangle,
    label: &str,
    mouse: Vector2,
    enabled: bool,
    primary: bool,
) {
    let hovered = enabled && rect.check_collision_point_rec(mouse);
    let fill = if !enabled {
        Color::new(26, 35, 46, 255)
    } else if primary {
        if hovered {
            Color::new(94, 204, 225, 255)
        } else {
            Color::new(65, 165, 190, 255)
        }
    } else if hovered {
        Color::new(38, 65, 85, 245)
    } else {
        Color::new(20, 37, 53, 235)
    };
    d.draw_rectangle_rec(rect, fill);
    d.draw_rectangle_lines_ex(
        rect,
        1.0,
        if hovered || primary {
            CYAN
        } else {
            Color::new(47, 74, 95, 255)
        },
    );
    let size = 20;
    d.draw_text(
        label,
        (rect.x + (rect.width - d.measure_text(label, size) as f32) * 0.5) as i32,
        (rect.y + (rect.height - size as f32) * 0.5) as i32,
        size,
        if primary && enabled {
            Color::new(4, 21, 30, 255)
        } else if enabled {
            INK
        } else {
            MUTED
        },
    );
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
    let mut shown = None;
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
    let mut hangar = Hangar::load(&mut rl, &thread, &root, HangarStyle::default())?;
    let captures = root.join("target/front-end");
    if smoke_goal.is_some() {
        std::fs::create_dir_all(&captures)?;
    }
    let mut seen = BTreeSet::new();
    let mut controls_seen = false;
    let mut back_seen = false;
    let mut frame = 0u32;
    loop {
        if rl.window_should_close()
            || ((rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
                || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
                && rl.is_key_pressed(KeyboardKey::KEY_Q))
        {
            return Ok(None);
        }
        // Exercise the same menu layout on real window resizes before the existing smoke flow.
        if smoke_goal.is_some() {
            match frame {
                0 => rl.set_window_size(960, 640),
                1 => rl.set_window_size(1920, 1080),
                2 | 4 => rl.set_window_size(1280, 800),
                3 => rl.set_window_size(1024, 1024),
                _ => {}
            }
        }
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        if let Some(goal) = goal_index {
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
                            if !controls_seen
                                || !back_seen
                                || seen.len() != 6
                                || Some(selection.index) != goal_index
                            {
                                return Err("Recorrido de menú/selector incompleto".into());
                            }
                            let report = serde_json::json!({"status":"PASS", "previews":seen, "controls":controls_seen, "back_to_menu":back_seen, "selected":catalog.choices[selection.index].id, "physical_input_verified":false});
                            std::fs::write(
                                captures.join("selection-validation.json"),
                                serde_json::to_string_pretty(&report)?,
                            )?;
                            println!("SELECTION_SMOKE_PASS {report}");
                        }
                        return Ok(Some(transition));
                    }
                }
            }
        }
        if screen == Screen::Hangar && shown != Some(selection.index) {
            hangar.exhibit(
                &mut rl,
                &thread,
                &root,
                &catalog.model(&catalog.choices[selection.index], &ships),
            );
            shown = Some(selection.index);
            if smoke_goal.is_some()
                && hangar.placeholder != catalog.choices[selection.index].ship_id.is_none()
            {
                return Err("No cargó un modelo integrado del hangar".into());
            }
        }
        let dt = if smoke_goal.is_some() {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.1)
        };
        hangar.update(dt);
        let camera = if screen == Screen::Hangar {
            hangar.camera(
                Vector2::new(w as f32, h as f32),
                Vector2::new(HANGAR_HEADER, HANGAR_FOOTER),
            )
        } else {
            Camera3D::perspective(
                Vector3::zero(),
                Vector3::new(0.0, 0.0, -1.0),
                Vector3::up(),
                50.0,
            )
        };
        let sky_frame = sky.project(&rl, camera);
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(SKY_BACKGROUND);
        sky.draw(&mut d, &sky_frame);
        match screen {
            Screen::Main => {
                let layout = MainMenuLayout::new(h);
                d.draw_rectangle((w - 44) / 2, layout.top as i32, 44, 3, CYAN);
                centered_text(&mut d, "PLANET", w, layout.planet_y as i32, 40, MUTED);
                centered_text(&mut d, "HOPPERS", w, layout.hoppers_y as i32, 76, INK);
                centered_text(&mut d, "TU VIAJE EMPIEZA AQUÍ", w, layout.tagline_y as i32, 18, CYAN);
                for (i, label) in ["Nueva partida", "Controles", "Salir"].iter().enumerate() {
                    button(&mut d, menu_button(w, h, i), label, mouse, true, i == 0);
                }
                centered_text(&mut d, "ELIGE TU NAVE.  EXPLORA SEIS MUNDOS.  UNA AVENTURA TE ESPERA.", w, (layout.bottom + 28.0) as i32, 14, MUTED);
                centered_text(&mut d, "ESTACIÓN ORBITAL  /  SECTOR 06", w, h - 54, 16, MUTED);
            }
            Screen::Controls => {
                d.draw_text("CONTROLES", 70, 75, 44, INK);
                d.draw_text("VUELO ESPACIAL", 74, 150, 18, CYAN);
                for (i, (keys, action)) in [
                    ("W / S", "Acelerar / frenar"),
                    ("Mouse / A / D / R / F", "Orientar la nave"),
                    ("Shift", "Turbo"),
                    ("Tab / B", "Elegir objetivo / piloto automático"),
                    ("Espacio", "Subir (ascenso vertical)"),
                    ("E / Enter", "Explorar planeta cercano"),
                    ("C / O", "Vista general / órbitas"),
                    ("Esc", "Soltar o recapturar mouse en el espacio"),
                    (
                        "South Park: Esc / P",
                        "Pausa, reinicio y regreso al sistema solar",
                    ),
                    ("Ctrl + Q", "Salir del juego"),
                ]
                .iter()
                .enumerate()
                {
                    d.draw_text(keys, 74, 194 + i as i32 * 34, 19, INK);
                    d.draw_text(action, 360, 194 + i as i32 * 34, 18, MUTED);
                }
                button(&mut d, back_button(w), "Volver", mouse, true, false);
            }
            Screen::Hangar => {
                {
                    let mut d3 = d.begin_mode3D(camera);
                    hangar.draw(&mut d3, camera);
                }
                d.draw_rectangle(0, 0, w, HANGAR_HEADER as i32, Color::new(4, 12, 21, 220));
                d.draw_text(
                    if mode == SelectionMode::Initial {
                        "HANGAR / TU PRIMERA NAVE"
                    } else {
                        "HANGAR / CAMBIAR NAVE"
                    },
                    36,
                    28,
                    18,
                    CYAN,
                );
                let choice = &catalog.choices[selection.index];
                d.draw_text(&choice.name, 36, 65, 38, INK);
                let model = catalog.model(choice, &ships);
                let provisional = format!("NAVE PROVISIONAL / {}", choice.name);
                d.draw_text(
                    if hangar.placeholder {
                        &provisional
                    } else {
                        &model.name
                    },
                    38,
                    116,
                    17,
                    if hangar.placeholder {
                        Color::new(255, 178, 88, 255)
                    } else {
                        MUTED
                    },
                );
                button(&mut d, back_button(w), "Volver", mouse, true, false);
                button(&mut d, arrow(w, h, false), "<", mouse, true, false);
                button(&mut d, arrow(w, h, true), ">", mouse, true, false);
                d.draw_rectangle(
                    0,
                    h - HANGAR_FOOTER as i32,
                    w,
                    HANGAR_FOOTER as i32,
                    Color::new(4, 12, 21, 238),
                );
                let status = if mode == SelectionMode::Initial {
                    "Elige libremente tu nave inicial"
                } else if selection.can_confirm(&catalog, game.as_ref()) {
                    "Nave desbloqueada"
                } else {
                    "Nave bloqueada"
                };
                d.draw_text(
                    status,
                    (w - d.measure_text(status, 20)) / 2,
                    h - 137,
                    20,
                    INK,
                );
                let enabled = selection.can_confirm(&catalog, game.as_ref());
                button(
                    &mut d,
                    launch_button(w, h),
                    if !enabled {
                        "Bloqueada"
                    } else if mode == SelectionMode::Initial {
                        "Comenzar aventura"
                    } else {
                        "Activar nave"
                    },
                    mouse,
                    enabled,
                    true,
                );
                let count = format!(
                    "{:02} / 06    Flechas: elegir    Enter: confirmar    Esc: volver",
                    selection.index + 1
                );
                d.draw_text(
                    &count,
                    (w - d.measure_text(&count, 16)) / 2,
                    h - 26,
                    16,
                    MUTED,
                );
            }
        }
        if smoke_goal.is_some() {
            if frame == 94 && (screen != Screen::Hangar || selection.index != 4) {
                return Err("La flecha izquierda no cambió la nave".into());
            }
            let name = match frame {
                0 | 1 | 3 => Some(format!("main-menu-{w}x{h}.png")),
                2 => Some("main-menu.png".to_string()),
                8 => {
                    controls_seen = screen == Screen::Controls;
                    Some("controls.png".into())
                }
                24 | 34 | 46 | 58 | 70 | 82 if screen == Screen::Hangar => {
                    seen.insert(catalog.choices[selection.index].id.clone());
                    Some(format!(
                        "hangar-{}.png",
                        catalog.choices[selection.index].id
                    ))
                }
                102 => {
                    back_seen = screen == Screen::Main;
                    Some("back-to-menu.png".into())
                }
                _ => None,
            };
            if let Some(name) = name {
                save_frame(&captures.join(name), w, h)?;
            }
            if frame > 180 {
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
        16 | 108 => click(menu_button(w, h, 0)),
        28 | 40 | 52 | 64 | 76 => click(arrow(w, h, true)),
        90 => click(arrow(w, h, false)),
        98 => click(back_button(w)),
        f if f >= 112 && f < 112 + goal as u32 * 4 && (f - 112) % 4 == 0 => {
            key(KeyboardKey::KEY_RIGHT)
        }
        140 => click(launch_button(w, h)),
        _ => {}
    }
}
