//! Standalone development host. No solar-system, progress or minigame rules.
use crate::{
    graphics::resources::{save_frame, StaticModel},
    minigames::{
        common::{
            fps::{CursorCapture, PlayerInput},
            interaction::select_target,
            manifest::SceneManifest,
        },
        southpark::{DevConfig, SouthParkScene},
    },
};
use raylib::{ffi, prelude::*};
use serde_json::json;
use std::{path::PathBuf, time::Instant};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = root.join("config/southpark-dev.json");
    let mut smoke = false;
    let mut no_probes = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--no-test-objects" => no_probes = true,
            "--config" => {
                config_path = PathBuf::from(args.next().ok_or("--config requires a JSON path")?)
            }
            "--help" | "-h" => {
                println!("cargo run --bin southpark -- [--config PATH] [--no-test-objects] [--smoke]\nWASD: caminar | Shift: correr | mouse: mirar | E: interactuar | Esc: cursor\nF1: ayuda | F2: colisionadores | F12: captura | Ctrl+Q / cerrar ventana: salir");
                return Ok(());
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let mut config = DevConfig::load(&config_path)?;
    if no_probes {
        config.test_objects = false;
    }
    if smoke && !config.test_objects {
        return Err("--smoke requires the three technical interaction objects".into());
    }
    let manifest_path = root.join("assets/models/minigames/southpark/southpark_town.scene.json");
    let manifest = SceneManifest::load(&manifest_path)?;
    let mut scene = SouthParkScene::load(&manifest, config)?;
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .msaa_4x()
        .title("Planet-Hoppers | South Park | Fase 2")
        .build();
    rl.set_exit_key(None);
    if smoke {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    rl.set_target_fps(if smoke { 0 } else { 120 });
    unsafe {
        ffi::rlSetClipPlanes(0.05, 1500.0);
    }
    let mut model = StaticModel::load(
        &mut rl,
        &thread,
        &manifest.visual_path(&manifest_path)?,
        Some("SP_Snow"),
    )?;
    println!(
        "SCENE {} | {} solid proxies | {:?}",
        manifest.id,
        manifest
            .colliders
            .iter()
            .filter(|c| c.role == "solid")
            .count(),
        model.audit
    );
    if (model.audit.bounds_min[0] - scene.world.terrain.min.x).abs() > 0.01
        || (model.audit.bounds_max[0] - scene.world.terrain.max.x).abs() > 0.01
        || (model.audit.bounds_min[2] - scene.world.terrain.min.z).abs() > 0.01
        || (model.audit.bounds_max[2] - scene.world.terrain.max.z).abs() > 0.01
    {
        return Err("GLB and collision manifest use different bounds/coordinates".into());
    }
    let captures = root.join("docs/references/southpark/phase2");
    if smoke {
        std::fs::create_dir_all(&captures)?;
    }
    let mut cursor = CursorCapture::default();
    cursor.apply(&mut rl);
    let mut debug = false;
    let mut help = true;
    let mut frame = 0;
    let mut released = false;
    let mut recaptured = false;
    let mut mouse_changed = false;
    let mut running_read = false;
    let initial_feet = scene.player.feet;
    let mut run_distance = 0.0;
    let mut render_times = Vec::new();
    while !rl.window_should_close() {
        if smoke {
            inject_smoke_input(&rl, frame);
        }
        let frame_start = Instant::now();
        let dt = if smoke {
            1.0 / 60.0
        } else {
            rl.get_frame_time().min(0.1)
        };
        if (rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL))
            && rl.is_key_pressed(KeyboardKey::KEY_Q)
        {
            break;
        }
        let mut input = PlayerInput::read(&rl);
        if smoke && (3..=50).contains(&frame) {
            running_read |= input.forward > 0.0 && input.sprint;
        }
        let old_yaw = scene.player.yaw;
        let focused = smoke || rl.is_window_focused();
        let active = cursor.update(&mut rl, &mut input, focused);
        if active {
            scene.player.update(input, dt, &scene.world);
        }
        if smoke && frame == 52 {
            mouse_changed = (scene.player.yaw - old_yaw).abs() > 0.001;
        }
        if smoke && frame == 55 {
            released = !cursor.captured && !rl.is_cursor_hidden();
        }
        if smoke && frame == 58 {
            recaptured = cursor.captured && rl.is_cursor_hidden();
        }
        let selected = if active {
            select_target(
                &scene.targets,
                &scene.world,
                scene.player.eye(),
                scene.player.direction(),
                scene.interaction_distance,
            )
        } else {
            None
        };
        if active && input.interact {
            if let Some(index) = selected {
                scene.interact(index);
            }
        }
        scene.tick_confirmation(dt);
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            help = !help;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F2) {
            debug = !debug;
        }
        if smoke && frame == 51 {
            run_distance = (scene.player.feet - initial_feet).length();
            if !scene.world.can_occupy(
                scene.player.feet,
                scene.player.config.radius,
                scene.player.config.body_height,
            ) {
                return Err("Smoke runner penetrated a collider".into());
            }
        }
        let camera = if smoke && frame == 75 {
            Camera3D::perspective(
                Vector3::new(260.0, 260.0, 310.0),
                Vector3::zero(),
                Vector3::up(),
                65.0,
            )
        } else {
            scene.player.camera()
        };
        let manual_capture = !smoke && rl.is_key_pressed(KeyboardKey::KEY_F12);
        {
            let width = rl.get_screen_width();
            let height = rl.get_screen_height();
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(Color::new(184, 212, 237, 255));
            {
                let mut view = d.begin_mode3D(camera);
                model.draw(&mut view, camera.position);
                for (index, target) in scene.targets.iter().enumerate() {
                    let color = [
                        Color::new(202, 134, 72, 255),
                        Color::new(65, 167, 176, 255),
                        Color::new(162, 104, 195, 255),
                    ][index % 3];
                    let p = target.bounds.center();
                    let s = target.bounds.size();
                    view.draw_cube(p, s.x, s.y, s.z, color);
                    view.draw_cube_wires(
                        p,
                        s.x + 0.01,
                        s.y + 0.01,
                        s.z + 0.01,
                        if selected == Some(index) {
                            Color::YELLOW
                        } else {
                            Color::DARKGRAY
                        },
                    );
                }
                if debug {
                    for obstacle in &scene.world.obstacles {
                        if obstacle.id.starts_with("boundary-") {
                            continue;
                        }
                        let s = obstacle.bounds.size();
                        view.draw_cube_wires(
                            obstacle.bounds.center(),
                            s.x,
                            s.y,
                            s.z,
                            Color::new(230, 80, 90, 150),
                        );
                    }
                }
            }
            d.draw_text(
                "SOUTH PARK - prueba tecnica / fase 2",
                16,
                14,
                20,
                Color::DARKGRAY,
            );
            if !smoke {
                d.draw_fps(width - 100, 14);
            }
            if active && !(smoke && frame == 75) {
                let color = if selected.is_some() {
                    Color::YELLOW
                } else {
                    Color::WHITE
                };
                d.draw_circle(width / 2, height / 2, 3.0, Color::BLACK);
                d.draw_line(width / 2 - 7, height / 2, width / 2 + 7, height / 2, color);
                d.draw_line(width / 2, height / 2 - 7, width / 2, height / 2 + 7, color);
                if let Some(index) = selected {
                    let text = format!("[E] Interactuar - {}", scene.targets[index].label);
                    let x = (width - d.measure_text(&text, 22)) / 2;
                    d.draw_rectangle(
                        x - 10,
                        height / 2 + 24,
                        d.measure_text(&text, 22) + 20,
                        34,
                        Color::new(20, 30, 40, 220),
                    );
                    d.draw_text(&text, x, height / 2 + 30, 22, Color::YELLOW);
                }
            } else if !active {
                d.draw_rectangle(0, height / 2 - 30, width, 60, Color::new(20, 30, 40, 210));
                d.draw_text(
                    "Cursor libre - pulsa ESC para retomar",
                    (width - 480) / 2,
                    height / 2 - 10,
                    24,
                    Color::WHITE,
                );
            }
            if let Some((text, _)) = &scene.confirmation {
                if !(smoke && frame == 75) {
                    d.draw_text(text, 16, 46, 20, Color::DARKGREEN);
                }
            }
            if help {
                d.draw_rectangle(0, height - 55, width, 55, Color::new(20, 30, 40, 210));
                d.draw_text(
                    "WASD caminar | Shift correr | Mouse mirar | E interactuar | ESC cursor",
                    16,
                    height - 48,
                    18,
                    Color::WHITE,
                );
                d.draw_text(
                    "F1 ayuda | F2 colisiones | F12 captura | Ctrl+Q / cerrar ventana: salir",
                    16,
                    height - 25,
                    18,
                    Color::LIGHTGRAY,
                );
            }
            if manual_capture {
                let path = root.join("target/southpark.png");
                save_frame(&path, width, height)?;
                println!("Captura: {}", path.display());
            }
            if smoke && [0, 55, 65, 75].contains(&frame) {
                let name = match frame {
                    0 => "street.png",
                    55 => "cursor-released.png",
                    65 => "interaction.png",
                    _ => "overview.png",
                };
                save_frame(&captures.join(name), width, height)?;
            }
        }
        render_times.push(frame_start.elapsed().as_secs_f64() * 1000.0);
        if smoke && frame == 60 {
            scene.player.feet = initial_feet;
            scene.player.yaw = 0.0;
            scene.player.pitch = 0.0;
        }
        frame += 1;
        if smoke && frame >= 90 {
            break;
        }
    }
    rl.enable_cursor();
    if smoke {
        if frame < 90
            || scene.interaction_count == 0
            || !released
            || !recaptured
            || !mouse_changed
            || !running_read
            || !(2.0..2.51).contains(&run_distance)
        {
            return Err(format!("Smoke input failed: frames={frame}, E={}, release={released}, capture={recaptured}, mouse={mouse_changed}, sprint={running_read}, distance={run_distance}",scene.interaction_count).into());
        }
        let report = json!({"status":"PASS","model":model.audit,"scene_solids":manifest.colliders.iter().filter(|c|c.role=="solid").count(),"frames":frame,
            "raylib_automation":{"W_Shift":running_read,"mouse_look":mouse_changed,"E_confirmations":scene.interaction_count,"Escape_release":released,"Escape_recapture":recaptured,"run_blocked_at_m":run_distance},
            "render_mean_ms":render_times.iter().skip(5).sum::<f64>()/(render_times.len()-5) as f64,
            "manual_hardware_input_tested":false,"notes":"Real OpenGL render and raylib automation events. Physical keyboard/mouse and long play sessions require manual verification."});
        std::fs::write(
            captures.join("runtime-validation.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("SMOKE_PASS {report}");
    }
    Ok(())
}

fn inject_smoke_input(rl: &RaylibHandle, frame: u32) {
    // raylib 5.5 AutomationEventType: key up=1, down=2, mouse position=7.
    fn event(kind: u32, params: [i32; 4]) {
        unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: kind,
                params,
            });
        }
    }
    fn key(key: KeyboardKey, down: bool) {
        event(if down { 2 } else { 1 }, [key as i32, 0, 0, 0]);
    }
    match frame {
        1 => key(KeyboardKey::KEY_E, true),
        2 => key(KeyboardKey::KEY_E, false),
        3 => {
            key(KeyboardKey::KEY_W, true);
            key(KeyboardKey::KEY_LEFT_SHIFT, true);
        }
        51 => {
            key(KeyboardKey::KEY_W, false);
            key(KeyboardKey::KEY_LEFT_SHIFT, false);
        }
        52 => {
            let p = rl.get_mouse_position();
            event(7, [p.x as i32 + 50, p.y as i32 + 15, 0, 0]);
        }
        55 | 58 => key(KeyboardKey::KEY_ESCAPE, true),
        56 | 59 => key(KeyboardKey::KEY_ESCAPE, false),
        _ => {}
    }
}
