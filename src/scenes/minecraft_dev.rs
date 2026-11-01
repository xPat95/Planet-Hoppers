//! Standalone development host. The default front end and solar system are unchanged.
use crate::{
    graphics::resources::{save_frame, StaticModel},
    minigames::{
        common::fps::{CursorCapture, PlayerInput},
        minecraft::{MenuAction, MinecraftConfig, MinecraftScene, SurfaceState},
    },
};
use raylib::{ffi, prelude::*};
use std::path::{Path, PathBuf};

#[path = "minecraft_smoke.rs"]
mod smoke;

type Error = Box<dyn std::error::Error>;
const SKY: Color = Color::new(158, 201, 240, 255);

pub fn run() -> Result<(), Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = root.join("config/minecraft-dev.json");
    let mut captures = root.join("docs/references/minecraft/phase2");
    let mut test = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => test = true,
            "--config" => config_path = PathBuf::from(args.next().ok_or("Missing --config path")?),
            "--capture-dir" => {
                captures = PathBuf::from(args.next().ok_or("Missing --capture-dir path")?)
            }
            "--help" | "-h" => {
                println!(
                    "cargo run --bin minecraft -- [--smoke] [--config FILE] [--capture-dir DIR]"
                );
                println!("WASD: caminar | Mouse: mirar | Shift: correr | E: interactuar | Esc: cursor/menu | F12: captura");
                return Ok(());
            }
            _ => return Err(format!("Unknown Minecraft option: {arg}").into()),
        }
    }
    let manifest_path = root.join("assets/models/minigames/minecraft/minecraft_village.scene.json");
    let mut scene = MinecraftScene::load(&manifest_path, MinecraftConfig::load(&config_path)?)?;
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .title("Planet-Hoppers | Minecraft | DIAMOND RUSH")
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(800, 600);
    rl.set_target_fps(if test { 0 } else { 120 });
    if test {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    unsafe {
        ffi::rlSetClipPlanes(0.05, 220.0);
    }
    let mut model = StaticModel::load_overworld(
        &mut rl,
        &thread,
        &scene.manifest.visual_path(&manifest_path)?,
    )?;
    println!(
        "MINECRAFT_MODEL_AUDIT {}",
        serde_json::to_string(&model.audit)?
    );
    let mut cursor = CursorCapture::default();
    cursor.apply(&mut rl);
    let mut smoke = smoke::Smoke::default();
    let mut frame = 0;
    while !rl.window_should_close() {
        if test {
            smoke.before_frame(frame, &mut scene, &rl);
        }
        let mut input = PlayerInput::read(&rl);
        let previous_state = scene.state;
        let focused = test || rl.is_window_focused();
        if scene.state == SurfaceState::Exploring {
            if focused
                && !cursor.captured
                && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            {
                cursor.captured = true;
                cursor.apply(&mut rl);
            }
            let enabled = cursor.update(&mut rl, &mut input, focused);
            scene.update(
                input,
                if test {
                    1.0 / 60.0
                } else {
                    rl.get_frame_time()
                },
                enabled,
            );
        } else {
            scene.menu_action(menu_input(&rl, scene.state, focused));
        }
        if scene.state != previous_state {
            cursor.captured = scene.state == SurfaceState::Exploring;
            cursor.apply(&mut rl);
        }
        if test {
            smoke.after_frame(frame, &scene, &cursor, &rl)?;
        }
        let camera = scene.player.camera();
        let width = rl.get_screen_width();
        let height = rl.get_screen_height();
        let capture = if test {
            smoke::capture_name(frame)
        } else if rl.is_key_pressed(KeyboardKey::KEY_F12) {
            Some("manual.png")
        } else {
            None
        };
        {
            let mut draw = rl.begin_drawing(&thread);
            draw.clear_background(SKY);
            {
                let mut view = draw.begin_mode3D(camera);
                model.draw(&mut view, camera.position);
            }
            draw.draw_rectangle(0, 0, width, 66, Color::new(16, 28, 20, 210));
            draw.draw_text("MINECRAFT  |  DIAMOND RUSH", 20, 12, 24, Color::WHITE);
            draw.draw_text(
                "Overworld - exploración libre",
                20,
                42,
                16,
                Color::new(189, 225, 166, 255),
            );
            draw.draw_rectangle(0, height - 38, width, 38, Color::new(16, 28, 20, 210));
            draw.draw_text("WASD: caminar   Mouse: mirar   Shift: correr   E: interactuar   Esc: cursor   F12: captura", 16, height-27, 16, Color::WHITE);
            if scene.state == SurfaceState::Exploring {
                draw.draw_line(
                    width / 2 - 6,
                    height / 2,
                    width / 2 + 6,
                    height / 2,
                    Color::WHITE,
                );
                draw.draw_line(
                    width / 2,
                    height / 2 - 6,
                    width / 2,
                    height / 2 + 6,
                    Color::WHITE,
                );
                if let Some(index) = scene.selected.filter(|_| cursor.captured) {
                    let text = format!("[E] {}", scene.targets[index].label);
                    let x = (width - draw.measure_text(&text, 23)) / 2;
                    draw.draw_rectangle(
                        x - 12,
                        height / 2 + 22,
                        draw.measure_text(&text, 23) + 24,
                        40,
                        Color::new(14, 24, 17, 230),
                    );
                    draw.draw_text(&text, x, height / 2 + 30, 23, Color::YELLOW);
                }
                if !cursor.captured {
                    draw.draw_text(
                        "Cursor libre. Clic o Esc para continuar.",
                        20,
                        80,
                        20,
                        Color::WHITE,
                    );
                }
                if let Some(text) = &scene.confirmation {
                    draw.draw_text(text, 20, 110, 20, Color::YELLOW);
                }
            } else {
                draw_menu(&mut draw, width, height, scene.state);
            }
            if let Some(name) = capture {
                capture_frame(&captures, name, width, height)?;
            }
        }
        frame += 1;
        if test && frame > smoke::LAST_FRAME {
            break;
        }
    }
    rl.enable_cursor();
    if test {
        if frame <= smoke::LAST_FRAME {
            return Err("Minecraft smoke window closed early".into());
        }
        smoke.finish(&captures, &model)?;
    }
    Ok(())
}

fn panel(w: i32, h: i32) -> Rectangle {
    let width = (w as f32 - 48.0).min(760.0);
    Rectangle::new(
        (w as f32 - width) * 0.5,
        (h as f32 - 320.0) * 0.5,
        width,
        320.0,
    )
}
pub(super) fn buttons(w: i32, h: i32) -> [Rectangle; 2] {
    let p = panel(w, h);
    [
        Rectangle::new(p.x + 24.0, p.y + 235.0, (p.width - 64.0) * 0.5, 48.0),
        Rectangle::new(
            p.x + p.width * 0.5 + 8.0,
            p.y + 235.0,
            (p.width - 64.0) * 0.5,
            48.0,
        ),
    ]
}
fn menu_input(rl: &RaylibHandle, state: SurfaceState, focused: bool) -> MenuAction {
    if !focused {
        return MenuAction::None;
    }
    let click = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let [start, cancel] = buttons(rl.get_screen_width(), rl.get_screen_height());
    let mouse = rl.get_mouse_position();
    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
        || (click && cancel.check_collision_point_rec(mouse))
    {
        return MenuAction::Cancel;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
        || (click && start.check_collision_point_rec(mouse))
    {
        return if state == SurfaceState::MineMenu {
            MenuAction::Start
        } else {
            MenuAction::Cancel
        };
    }
    MenuAction::None
}
fn draw_menu(d: &mut RaylibDrawHandle, w: i32, h: i32, state: SurfaceState) {
    d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 150));
    let p = panel(w, h);
    d.draw_rectangle_rec(p, Color::new(24, 39, 29, 255));
    d.draw_rectangle_lines_ex(p, 3.0, Color::new(126, 182, 87, 255));
    d.draw_text(
        "DIAMOND RUSH",
        p.x as i32 + 24,
        p.y as i32 + 24,
        36,
        Color::new(114, 231, 231, 255),
    );
    if state == SurfaceState::MineMenu {
        d.draw_text(
            "Explorá la mina, recolectá diamantes y regresá a la",
            p.x as i32 + 24,
            p.y as i32 + 94,
            21,
            Color::WHITE,
        );
        d.draw_text(
            "superficie antes de que se agote el tiempo.",
            p.x as i32 + 24,
            p.y as i32 + 126,
            21,
            Color::WHITE,
        );
        d.draw_text(
            "Enter: iniciar desafío    Esc: cancelar",
            p.x as i32 + 24,
            p.y as i32 + 183,
            18,
            Color::LIGHTGRAY,
        );
    } else {
        d.draw_text(
            "El desafío está pendiente de implementación.",
            p.x as i32 + 24,
            p.y as i32 + 94,
            22,
            Color::WHITE,
        );
        d.draw_text(
            "La exploración de la aldea está lista para continuar.",
            p.x as i32 + 24,
            p.y as i32 + 132,
            20,
            Color::LIGHTGRAY,
        );
    }
    let rects = buttons(w, h);
    for (rect, label) in rects.into_iter().zip(if state == SurfaceState::MineMenu {
        ["Iniciar desafío", "Cancelar"]
    } else {
        ["Volver a explorar", "Cerrar"]
    }) {
        d.draw_rectangle_rec(rect, Color::new(74, 111, 49, 255));
        let x = rect.x as i32 + (rect.width as i32 - d.measure_text(label, 21)) / 2;
        d.draw_text(label, x, rect.y as i32 + 14, 21, Color::WHITE);
    }
}
fn capture_frame(dir: &Path, name: &str, w: i32, h: i32) -> Result<(), Error> {
    std::fs::create_dir_all(dir)?;
    save_frame(&dir.join(name), w, h)
}
