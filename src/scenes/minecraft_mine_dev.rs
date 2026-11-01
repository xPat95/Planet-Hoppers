//! Independent mine preview. Return closes its context normally, then opens the village.
use crate::{
    graphics::resources::{save_frame, StaticModel},
    minigames::{
        common::fps::{CursorCapture, PlayerInput},
        minecraft::{mine::MineScene, MinecraftConfig},
    },
};
use raylib::{ffi, prelude::*};
use std::path::{Path, PathBuf};
#[path = "minecraft_mine_smoke.rs"]
mod smoke;
type Error = Box<dyn std::error::Error>;

pub fn run(args: Vec<String>) -> Result<(), Error> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config = root.join("config/minecraft-dev.json");
    let mut output = root.join("docs/references/minecraft/phase3");
    let mut test = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mine" => {}
            "--smoke" => test = true,
            "--config" => config = PathBuf::from(args.next().ok_or("Missing config path")?),
            "--capture-dir" => {
                output = PathBuf::from(args.next().ok_or("Missing capture directory")?)
            }
            "--help" | "-h" => {
                println!("cargo run --bin minecraft -- --mine [--smoke] [--capture-dir DIR]");
                println!("WASD/mouse/Shift: FPS. E at SALIDA or Backspace: development exit. Esc: cursor/menu. F12: capture.");
                return Ok(());
            }
            _ => return Err(format!("Unknown mine option: {arg}").into()),
        }
    }
    let return_to_surface = window(&root, &config, &output, test)?;
    if return_to_surface {
        println!("MINE_DEV_RETURN_SURFACE");
        if test {
            let temp = std::env::temp_dir().join("minecraft_phase3_surface_smoke");
            super::minecraft_dev::run_with_args(
                vec![
                    "--smoke".into(),
                    "--capture-dir".into(),
                    temp.to_string_lossy().into_owned(),
                ]
                .into_iter(),
            )?;
        } else {
            super::minecraft_dev::run_with_args(std::iter::empty())?;
        }
    }
    Ok(())
}

fn window(root: &Path, config: &Path, output: &Path, test: bool) -> Result<bool, Error> {
    let manifest = root.join("assets/models/minigames/minecraft/mine/minecraft_mine.scene.json");
    let mut scene = MineScene::load(&manifest, MinecraftConfig::load(config)?)?;
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .title("Planet-Hoppers | DIAMOND RUSH | Mina - desarrollo")
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(800, 600);
    rl.set_target_fps(if test { 0 } else { 120 });
    if test {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    unsafe {
        ffi::rlSetClipPlanes(0.05, 100.0);
    }
    let mut model =
        StaticModel::load_mine(&mut rl, &thread, &scene.manifest.visual_path(&manifest)?)?;
    let (positions, colors, radii) = scene.light_arrays();
    model.set_mine_lights(&positions, &colors, &radii)?;
    if model.independent_groups().len() != scene.data.interactive_blocks.len() {
        return Err("Mine renderer merged independent blocks".into());
    }
    let mut cursor = CursorCapture::default();
    cursor.apply(&mut rl);
    let mut menu = false;
    let mut return_to_surface = false;
    let mut frame = 0;
    let mut smoke = smoke::Smoke::default();
    while !rl.window_should_close() {
        if test {
            smoke.before(frame, &mut scene, &rl);
        }
        let focused = test || rl.is_window_focused();
        let was_menu = menu;
        let mut input = PlayerInput::read(&rl);
        if menu {
            if focused && rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                menu = false;
            }
            let [back, stay] = buttons(rl.get_screen_width(), rl.get_screen_height());
            let click = focused && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
            let mouse = rl.get_mouse_position();
            if focused
                && (rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                    || (click && back.check_collision_point_rec(mouse)))
            {
                return_to_surface = true;
            }
            if click && stay.check_collision_point_rec(mouse) {
                menu = false;
            }
        } else {
            if focused
                && !cursor.captured
                && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            {
                cursor.captured = true;
                cursor.apply(&mut rl);
            }
            let enabled = cursor.update(&mut rl, &mut input, focused);
            let exit = scene.update(
                input,
                if test {
                    1.0 / 60.0
                } else {
                    rl.get_frame_time()
                },
                enabled,
            );
            if exit || (focused && rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)) {
                menu = true;
            }
        }
        if menu != was_menu {
            cursor.captured = !menu;
            cursor.apply(&mut rl);
        }
        if test {
            smoke.after(frame, &scene, &cursor, &rl, menu, return_to_surface)?;
        }
        let camera = scene.player.camera();
        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        let capture = if test {
            smoke::capture(frame)
        } else if rl.is_key_pressed(KeyboardKey::KEY_F12) {
            Some("runtime_manual.png")
        } else {
            None
        };
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::new(6, 9, 13, 255));
            {
                let mut view = d.begin_mode3D(camera);
                model.draw(&mut view, camera.position);
            }
            d.draw_rectangle(0, 0, w, 62, Color::new(8, 13, 18, 235));
            d.draw_text(
                "DIAMOND RUSH  |  MINA",
                20,
                12,
                24,
                Color::new(112, 233, 236, 255),
            );
            d.draw_text(
                "Exploración de desarrollo - minerales y lava sin mecánicas",
                20,
                41,
                16,
                Color::LIGHTGRAY,
            );
            d.draw_rectangle(0, h - 35, w, 35, Color::new(8, 13, 18, 235));
            d.draw_text("WASD / Mouse / Shift    E: salida    Backspace: volver    Esc: cursor    F12: captura",14,h-25,16,Color::WHITE);
            d.draw_line(w / 2 - 5, h / 2, w / 2 + 5, h / 2, Color::WHITE);
            d.draw_line(w / 2, h / 2 - 5, w / 2, h / 2 + 5, Color::WHITE);
            if !menu {
                if let Some(i) = scene.selected.filter(|_| cursor.captured) {
                    let text = if i == 0 {
                        "[E] Salir del modo de prueba".to_string()
                    } else {
                        scene.targets[i].label.clone()
                    };
                    let width = d.measure_text(&text, 21);
                    d.draw_rectangle(
                        (w - width) / 2 - 10,
                        h / 2 + 24,
                        width + 20,
                        34,
                        Color::new(8, 13, 18, 235),
                    );
                    d.draw_text(&text, (w - width) / 2, h / 2 + 31, 21, Color::YELLOW);
                }
                if !cursor.captured {
                    d.draw_text(
                        "Cursor libre. Clic o Esc para continuar.",
                        20,
                        78,
                        20,
                        Color::WHITE,
                    );
                }
            } else {
                draw_menu(&mut d, w, h);
            }
            if let Some(file) = capture {
                std::fs::create_dir_all(output)?;
                save_frame(&output.join(file), w, h)?;
            }
        }
        frame += 1;
        if return_to_surface {
            break;
        }
        if test && frame > smoke::LAST_FRAME {
            return Err("Mine smoke did not choose development exit".into());
        }
    }
    rl.enable_cursor();
    if test {
        smoke.finish(output, &scene, &model, frame, return_to_surface)?;
    }
    Ok(return_to_surface)
}
fn buttons(w: i32, h: i32) -> [Rectangle; 2] {
    let left = (w as f32 - 680.0) * 0.5;
    let top = h as f32 * 0.5 + 45.0;
    [
        Rectangle::new(left + 20.0, top, 310.0, 48.0),
        Rectangle::new(left + 350.0, top, 310.0, 48.0),
    ]
}
fn draw_menu(d: &mut RaylibDrawHandle, w: i32, h: i32) {
    d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 150));
    let x = (w - 680) / 2;
    let y = h / 2 - 130;
    d.draw_rectangle(x, y, 680, 260, Color::new(20, 31, 37, 255));
    d.draw_text(
        "SALIR DE LA PRUEBA",
        x + 24,
        y + 24,
        32,
        Color::new(112, 233, 236, 255),
    );
    d.draw_text(
        "Podés volver a la aldea sin iniciar un desafío.",
        x + 24,
        y + 90,
        21,
        Color::WHITE,
    );
    d.draw_text(
        "Enter: volver a la aldea    Esc: seguir en la mina",
        x + 24,
        y + 127,
        18,
        Color::LIGHTGRAY,
    );
    for (r, label) in buttons(w, h)
        .into_iter()
        .zip(["Volver a la aldea", "Seguir explorando"])
    {
        d.draw_rectangle_rec(r, Color::new(47, 85, 91, 255));
        d.draw_text(label, r.x as i32 + 18, r.y as i32 + 14, 21, Color::WHITE);
    }
}
