mod music;
mod render;
mod ship;
mod simulation;
mod video;
use raylib::prelude::*;
use simulation::*;
use std::path::Path;
fn save_capture(rl: &RaylibHandle, path: &Path) {
    let name = std::ffi::CString::new(path.to_string_lossy().as_bytes()).unwrap();
    unsafe {
        let width = rl.get_render_width();
        let height = rl.get_render_height();
        let pixels = raylib::ffi::rlReadScreenPixels(width, height);
        if pixels.is_null() {
            return;
        }
        let image = raylib::ffi::Image {
            data: pixels.cast(),
            width,
            height,
            mipmaps: 1,
            format: raylib::ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
        };
        raylib::ffi::ExportImage(image, name.as_ptr());
        raylib::ffi::UnloadImage(image);
    }
}
fn capture_scene(g: &mut Game, stage: usize) {
    g.start();
    if stage >= 14 {
        if stage >= 29 {
            g.distance = BULKHEADS[[0, 1, 2, 4][stage - 29]].0 - 30.0;
        } else if stage >= 27 {
            g.distance = if stage == 27 { 2180.0 } else { 2310.0 };
        } else if stage == 26 {
            g.distance = 2185.0;
            g.pos = [-3.0, 2.0];
            g.camera = g.pos;
            for o in &mut g.obstacles {
                if o.pos[2] == 2210.0
                    && o.material == simulation::Material::Cargo
                    && o.pos[0] == -3.0
                {
                    o.hp = 0;
                }
            }
        } else {
            let family = (stage - 14) / 2;
            let near = (stage - 14) % 2 == 1;
            g.distance = 155.0 + family as f32 * 76.0 - if near { 26.0 } else { 65.0 };
        }
        g.travel = g.distance;
        return;
    }
    if stage >= 10 {
        let target = match stage {
            10 | 11 => 420.0,
            12 => 715.0,
            _ => 865.0,
        };
        for _ in 0..120 * 100 {
            if g.distance >= target || g.phase != Phase::Running {
                break;
            }
            let input = diagnostic_pilot(g);
            g.update(STEP, input);
        }
        if stage < 12 {
            g.pos = [0.0; 2];
            g.camera = [0.0; 2];
            g.rear_attack = Some(RearAttack {
                target: [0.0; 2],
                age: if stage == 10 {
                    0.9
                } else {
                    REAR_WARNING + 0.45
                },
                resolved: false,
            });
        }
        return;
    }
    let target = match stage {
        0 => {
            g.phase = Phase::Briefing;
            return;
        }
        1 => 80.0,
        2 => 195.0,
        3 => 345.0,
        4 => 470.0,
        5 => 2160.0,
        6 => ATTACK_Z,
        _ => ATTACK_Z,
    };
    for _ in 0..120 * 100 {
        if g.distance >= target || g.phase != Phase::Running {
            break;
        }
        let mut input = diagnostic_pilot(g);
        if stage == 5 && g.distance > 2050.0 {
            input.fire = false;
        }
        g.update(STEP, input);
    }
    if stage == 2 {
        g.pos = [4.0, 2.8];
        g.camera = [3.5, 2.6];
        g.bank = -0.3;
    }
    if stage == 3 {
        for _ in 0..30 {
            let input = diagnostic_pilot(g);
            g.update(STEP, input);
        }
    }
    if stage == 6 {
        g.pos = [0.0, 0.0];
        g.camera = [0.0, 0.0];
        g.cooldown = 0.0;
        g.update(
            STEP,
            Controls {
                fire: true,
                ..Controls::default()
            },
        );
        for _ in 0..12 {
            g.update(STEP, Controls::default());
        }
    }
    if stage >= 7 {
        for _ in 0..120 * 10 {
            g.update(
                STEP,
                Controls {
                    fire: true,
                    ..Controls::default()
                },
            );
            if g.phase == Phase::Explosion {
                break;
            }
        }
        g.end_time = 0.35;
    }
}
pub fn run(
    project_root: &Path,
    selected: &crate::ships::ShipModel,
    args: impl Iterator<Item = String>,
) -> Result<crate::minigames::common::host::MinigameExit, Box<dyn std::error::Error>> {
    run_with_results(project_root, selected, args, &mut |_| Ok(()))
}

pub fn run_with_results(
    project_root: &Path,
    selected: &crate::ships::ShipModel,
    args: impl Iterator<Item = String>,
    observer: &mut crate::minigames::common::host::ResultObserver<'_>,
) -> Result<crate::minigames::common::host::MinigameExit, Box<dyn std::error::Error>> {
    use crate::minigames::common::host::{MinigameExit, MinigameResult};
    let mut reporter = crate::minigames::common::host::AttemptReporter::default();
    let args: Vec<_> = args.collect();
    if args.iter().any(|a| a == "--help") {
        println!("Star Wars: nave activa de la sesión. --capture --smoke-hidden --check-startup --check-ships");
        return Ok(MinigameExit::ReturnedWithoutResult);
    }
    let root = project_root.join("assets/minigames/starwars");
    let check_ships = args.iter().any(|a| a == "--check-ships");
    let check_startup = args.iter().any(|a| a == "--check-startup");
    let check_menus = args.iter().any(|a| a == "--check-menus");
    let capture = args.iter().any(|a| a == "--capture");
    let smoke_hidden = args.iter().any(|a| a == "--smoke-hidden");
    let smoke = smoke_hidden || args.iter().any(|a| a == "--smoke");
    let demo = smoke || args.iter().any(|a| a == "--demo");
    if args.iter().any(|a| a == "--check-music") {
        music::verify(&root)?;
        return Ok(MinigameExit::ReturnedWithoutResult);
    }
    let captures = project_root.join("target/starwars/capturas");
    if capture || smoke {
        std::fs::create_dir_all(&captures)?;
    }
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .title("Planet Hoppers | Asalto al nucleo - Vuelo libre")
        .resizable()
        .msaa_4x()
        .build();
    if smoke_hidden || check_startup || check_ships || check_menus {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    rl.set_exit_key(None);
    rl.set_window_min_size(960, 600);
    rl.set_target_fps(60);
    rl.set_trace_log(TraceLogLevel::LOG_WARNING);
    let audio = RaylibAudio::init_audio_device().ok();
    let mut soundtrack = if capture {
        None
    } else {
        audio
            .as_ref()
            .map(|device| music::FlightMusic::load(device, &root))
            .transpose()?
    };
    let mut presentation =
        render::Presentation::load(&mut rl, &thread, &root, project_root, selected)?;
    let mut clip: Option<video::Video<'_>> = None;
    let mut media_error: Option<String> = None;
    let mut game = Game {
        ship_half: presentation.ship.half(),
        ..Game::default()
    };
    if check_ships {
        let config = crate::ships::ShipsConfig::load(&project_root.join("config/ships.json"))?;
        let catalog = crate::gameplay::selection::ShipCatalog::load(
            &project_root.join("config/ship-selection.json"),
            &config,
        )?;
        let captures = project_root.join("target/starwars/ships");
        std::fs::create_dir_all(&captures)?;
        for choice in &catalog.choices {
            let model = catalog.model(choice, &config);
            presentation.ship = ship::TunnelShip::load(&mut rl, &thread, project_root, &model)?;
            let half = presentation.ship.half();
            let mut collision = Game {
                ship_half: half,
                ..Game::default()
            };
            collision.start();
            collision.obstacles.clear();
            collision.obstacles.push(Obstacle {
                pos: [0.0, 0.0, 35.0],
                half: [0.6, 0.6, 0.35],
                material: simulation::Material::Armor,
                hp: 3,
                flash: 0.0,
                contacted: false,
                seed: 0,
                shape: Shape::Duct,
                angle: 0.0,
                group: 0,
            });
            for _ in 0..180 {
                collision.update(STEP, Controls::default());
            }
            if collision.hull != 2 {
                return Err(format!("Choque de {} no restó una barra", model.id).into());
            }
            collision.start();
            collision.obstacles.clear();
            for _ in 0..300 {
                collision.update(
                    STEP,
                    Controls {
                        x: 1.0,
                        ..Controls::default()
                    },
                );
            }
            if collision.hull == 3 {
                return Err(format!("Pared no daña a {}", model.id).into());
            }

            for boost in [false, true] {
                game = Game::default();
                game.ship_half = half;
                game.start();
                for _ in 0..120 * 110 {
                    let mut controls = diagnostic_pilot(&game);
                    controls.boost = boost;
                    game.update(STEP, controls);
                    if matches!(game.phase, Phase::Outro | Phase::Lost) {
                        break;
                    }
                }
                if game.phase != Phase::Outro || game.hull != 3 || game.rear_encounters != 4 {
                    return Err(format!(
                        "Ruta de {} falló: phase={:?} hull={} z={} turbo={boost}",
                        model.id, game.phase, game.hull, game.distance
                    )
                    .into());
                }
            }
            game.start();
            game.distance = 125.0;
            game.travel = 125.0;
            for _ in 0..4 {
                let mut draw = rl.begin_drawing(&thread);
                presentation.draw(&mut draw, &thread, &game, 0.0, 1280.0, 800.0);
            }
            save_capture(&rl, &captures.join(format!("{}.png", model.id)));
            println!(
                "SHIP_QA_PASS id={} model={} half={half:?} normal_hull=3 turbo_hull=3 obstacle_damage=1 wall_damage=true",
                model.id, model.model
            );
        }
        println!("ALL_SHIPS_OK count={}", catalog.choices.len());
        return Ok(MinigameExit::ReturnedWithoutResult);
    }
    if check_menus {
        std::fs::create_dir_all(&captures)?;
        for (width, height) in [(1280, 800), (960, 600)] {
            rl.set_window_size(width, height);
            for (name, phase) in [
                ("briefing", Phase::Briefing),
                ("pause", Phase::Paused),
                ("defeat", Phase::Lost),
                ("victory", Phase::Won),
            ] {
                game.phase = phase;
                for _ in 0..4 {
                    let mut draw = rl.begin_drawing(&thread);
                    presentation.draw(&mut draw, &thread, &game, 1.0, width as f32, height as f32);
                }
                save_capture(&rl, &captures.join(format!("menu-{name}-{width}.png")));
            }
        }
        println!("STARWARS_MENUS_OK");
        return Ok(MinigameExit::ReturnedWithoutResult);
    }
    if check_startup {
        // Exercise font allocation, accented text, menu rendering and teardown
        // while the real soundtrack is loaded but not yet playing.
        for _ in 0..8 {
            assert_eq!(presentation.font.chars().len(), 223);
            for (glyph, expected) in presentation.font.chars().iter().zip(32..255) {
                assert_eq!(glyph.value, expected);
            }
            for ch in "NÚCLEO MISIÓN AÑADIDA".chars() {
                assert_eq!(presentation.font.get_glyph_info(ch).value, ch as i32);
            }
            {
                let mut draw = rl.begin_drawing(&thread);
                presentation.draw(&mut draw, &thread, &game, 0.0, 1280.0, 800.0);
            }
            presentation =
                render::Presentation::load(&mut rl, &thread, &root, project_root, selected)?;
        }
        println!("STARTUP_OK loads=9 menu_frames=8 glyphs=223 accents=true");
        return Ok(MinigameExit::ReturnedWithoutResult);
    }
    let (mut accumulator, mut ambient) = (0.0, 0.0);
    let (mut capture_stage, mut capture_frame) = (0, 0);
    let names = [
        "01_menu.png",
        "02_centro.png",
        "03_vuelo_lateral.png",
        "04_cargas.png",
        "05_estructuras.png",
        "06_compuerta.png",
        "07_reactor.png",
        "08_explosion.png",
        "09_video_inicio.png",
        "10_video_final.png",
        "11_alerta_cazas.png",
        "12_salva_popa.png",
        "13_portico.png",
        "14_marco_mantenimiento.png",
        "15_cercha_aproximacion.png",
        "16_cercha_cerca.png",
        "17_portico_aproximacion.png",
        "18_portico_cerca.png",
        "19_tuberias_aproximacion.png",
        "20_tuberias_cerca.png",
        "21_ductos_aproximacion.png",
        "22_ductos_cerca.png",
        "23_bastidor_aproximacion.png",
        "24_bastidor_cerca.png",
        "25_collar_aproximacion.png",
        "26_collar_cerca.png",
        "27_compuerta_abierta.png",
        "28_compuerta_tres_paneles.png",
        "29_compuerta_final_tres_paneles.png",
        "30_disco_1350.png",
        "31_disco_1800.png",
        "32_disco_2130.png",
        "33_disco_2275.png",
    ];
    let mut menu_frames = 0;
    let mut victory_frames = 0;
    let mut smoke_replay_verified = false;
    let mut previous_phase = game.phase;
    let (mut intro_saved, mut outro_saved) = (false, false);
    let (mut frames, mut frame_time) = (0u32, 0.0f32);
    if !capture {
        game.intro();
    }
    let mut exit = MinigameExit::WindowClosed;
    while !rl.window_should_close() {
        // Consume press events as well as held states. Very short taps can begin
        // and end between two rendered frames, especially at lower frame rates.
        let mut pressed = Vec::new();
        while let Some(key) = rl.get_key_pressed() {
            pressed.push(key);
        }
        if smoke {
            pressed.retain(|key| *key == KeyboardKey::KEY_ESCAPE);
            if game.phase == Phase::Won && victory_frames >= 4 {
                pressed.push(if smoke_replay_verified {
                    KeyboardKey::KEY_ESCAPE
                } else {
                    KeyboardKey::KEY_ENTER
                });
            }
        }
        // Mouse and keyboard share the final screen's two actions.
        if game.phase == Phase::Won && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let buttons = render::victory_buttons(
                rl.get_screen_width() as f32,
                rl.get_screen_height() as f32,
            );
            if buttons[0].check_collision_point_rec(rl.get_mouse_position()) {
                pressed.push(KeyboardKey::KEY_ENTER);
            }
            if buttons[1].check_collision_point_rec(rl.get_mouse_position()) {
                pressed.push(KeyboardKey::KEY_ESCAPE);
            }
        }
        if pressed.contains(&KeyboardKey::KEY_ESCAPE) {
            if smoke && game.phase == Phase::Won {
                println!(
                    "SMOKE_OK hull={} average_fps={:.1} intro={} outro={} results=true replay={}",
                    game.hull,
                    frames as f32 / frame_time,
                    intro_saved,
                    outro_saved,
                    smoke_replay_verified
                );
            }
            exit = MinigameExit::Returned(match game.phase {
                Phase::Won => MinigameResult::Victory,
                Phase::Lost => MinigameResult::Defeat,
                _ => MinigameResult::Abandoned,
            });
            break;
        }
        let dt = rl.get_frame_time().clamp(0.0, 0.1);
        ambient += dt;
        if pressed.contains(&KeyboardKey::KEY_F11) {
            rl.toggle_borderless_windowed();
        }
        if capture {
            if capture_frame == 0 {
                if !(8..10).contains(&capture_stage) {
                    capture_scene(&mut game, capture_stage);
                    clip = None;
                } else {
                    let stem = if capture_stage == 8 {
                        "inicio"
                    } else {
                        "final"
                    };
                    let mut v = video::Video::load(&mut rl, &thread, None, &root, stem)?;
                    v.seek_frame(if capture_stage == 8 { 8.0 } else { 15.0 })?;
                    v.elapsed = 5.0;
                    clip = Some(v);
                }
            }
        } else {
            let enter = pressed.contains(&KeyboardKey::KEY_ENTER);
            let pause = pressed.contains(&KeyboardKey::KEY_P);
            if game.phase == Phase::Won && (enter || pressed.contains(&KeyboardKey::KEY_R)) {
                let half = game.ship_half;
                game.start();
                clip = None;
                media_error = None;
                accumulator = 0.0;
                if smoke {
                    if game.distance != 0.0
                        || game.score != 0
                        || game.hull != 3
                        || game.reactor_hp != 6
                        || game.shots_fired != 0
                        || game.ship_half != half
                    {
                        return Err("La revancha no reinició correctamente la misión".into());
                    }
                    smoke_replay_verified = true;
                    println!(
                        "REPLAY_AFTER_OUTRO phase=Running distance=0 score=0 hull=3 same_ship=true"
                    );
                }
            } else if pressed.contains(&KeyboardKey::KEY_R) {
                clip = None;
                media_error = None;
                game.intro();
                accumulator = 0.0;
            } else if let Some(v) = &mut clip {
                if pause || (enter && v.paused) {
                    v.toggle_pause();
                } else if enter {
                    println!("VIDEO_SKIP {:?}", game.phase);
                    clip = None;
                    game.finish_video();
                    accumulator = 0.0;
                }
            } else if enter {
                if game.phase == Phase::Paused {
                    game.toggle_pause();
                } else if matches!(game.phase, Phase::Briefing | Phase::Lost) {
                    game.start();
                    media_error = None;
                }
                accumulator = 0.0;
            } else if pause {
                game.toggle_pause();
                accumulator = 0.0;
            }
            if !rl.is_window_focused() && !demo {
                if game.phase == Phase::Running {
                    game.phase = Phase::Paused;
                    accumulator = 0.0;
                }
                if let Some(v) = &mut clip {
                    if !v.paused {
                        v.toggle_pause();
                    }
                }
            }
            if matches!(game.phase, Phase::Intro | Phase::Outro)
                && clip.is_none()
                && media_error.is_none()
            {
                let stem = if game.phase == Phase::Intro {
                    "inicio"
                } else {
                    "final"
                };
                match video::Video::load(&mut rl, &thread, audio.as_ref(), &root, stem) {
                    Ok(v) => {
                        println!("VIDEO_START {stem}");
                        clip = Some(v);
                    }
                    Err(e) => {
                        eprintln!("VIDEO_ERROR {e}");
                        media_error = Some(e.to_string());
                    }
                }
            }
            if let Some(v) = &mut clip {
                match v.update(dt) {
                    Ok(true) => {
                        println!("VIDEO_END {:?} elapsed={:.3}", game.phase, v.elapsed);
                        clip = None;
                        game.finish_video();
                        accumulator = 0.0;
                    }
                    Ok(false) => {}
                    Err(e) => {
                        media_error = Some(e.to_string());
                        clip = None;
                    }
                }
            } else if media_error.is_none() {
                let key = |a, b| {
                    rl.is_key_down(a)
                        || rl.is_key_down(b)
                        || pressed.contains(&a)
                        || pressed.contains(&b)
                };
                let input = Controls {
                    x: key(KeyboardKey::KEY_A, KeyboardKey::KEY_LEFT) as i32 as f32
                        - key(KeyboardKey::KEY_D, KeyboardKey::KEY_RIGHT) as i32 as f32,
                    y: key(KeyboardKey::KEY_W, KeyboardKey::KEY_UP) as i32 as f32
                        - key(KeyboardKey::KEY_S, KeyboardKey::KEY_DOWN) as i32 as f32,
                    boost: key(KeyboardKey::KEY_LEFT_SHIFT, KeyboardKey::KEY_RIGHT_SHIFT),
                    fire: rl.is_key_down(KeyboardKey::KEY_SPACE)
                        || pressed.contains(&KeyboardKey::KEY_SPACE),
                };
                accumulator += dt * if smoke { 5.0 } else { 1.0 };
                while accumulator >= STEP {
                    let input = if demo { diagnostic_pilot(&game) } else { input };
                    game.update(STEP, input);
                    accumulator -= STEP;
                }
            }
        }
        // Captures stage synthetic scenes and must never award progress.
        if !capture {
            reporter.observe(&game, observer)?;
        }
        if let Some(music) = &mut soundtrack {
            music.update(if media_error.is_none() {
                game.phase
            } else {
                Phase::Briefing
            });
        }
        let hide_cursor = matches!(
            game.phase,
            Phase::Running | Phase::Intro | Phase::Outro | Phase::Explosion
        );
        if hide_cursor != rl.is_cursor_hidden() {
            if hide_cursor {
                rl.hide_cursor();
            } else {
                rl.show_cursor();
            }
        }
        let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
        {
            let mut d = rl.begin_drawing(&thread);
            if let Some(v) = &clip {
                v.draw(&mut d, w, h);
            } else {
                presentation.draw(&mut d, &thread, &game, ambient, w, h);
            }
            if media_error.is_some() {
                d.clear_background(Color::new(6, 12, 19, 255));
                d.draw_text("NO SE PUDO ABRIR LA CINEMATICA", 50, 100, 28, Color::WHITE);
                d.draw_text(
                    "Verifica assets/minigames/starwars/videos. R: reintentar. ESC: salir.",
                    50,
                    150,
                    20,
                    Color::LIGHTGRAY,
                );
                d.draw_text(
                    "Detalles en la consola. Preparar con tools/starwars/prepare_videos.py",
                    50,
                    190,
                    18,
                    Color::GRAY,
                );
            }
        }
        if capture {
            capture_frame += 1;
            if capture_frame == 4 {
                save_capture(&rl, &captures.join(names[capture_stage]));
                capture_stage += 1;
                capture_frame = 0;
                if capture_stage == names.len() {
                    break;
                }
            }
        } else if pressed.contains(&KeyboardKey::KEY_F12) {
            std::fs::create_dir_all(&captures)?;
            save_capture(
                &rl,
                &captures.join(format!("vuelo_{:.0}.png", ambient * 1000.0)),
            );
        }
        if smoke {
            frames += 1;
            frame_time += dt;
            if let Some(v) = &clip {
                if game.phase == Phase::Intro && v.elapsed > 3.0 && !intro_saved {
                    save_capture(&rl, &captures.join("smoke_intro.png"));
                    intro_saved = true;
                }
                if game.phase == Phase::Outro && v.elapsed > 4.0 && !outro_saved {
                    save_capture(&rl, &captures.join("smoke_final.png"));
                    outro_saved = true;
                }
            }
            if game.phase != previous_phase {
                println!(
                    "PHASE {:?} distance={:.1} hull={} destroyed={}",
                    game.phase, game.distance, game.hull, game.destroyed
                );
                previous_phase = game.phase;
            }
            if game.phase == Phase::Lost || media_error.is_some() || ambient > 240.0 {
                return Err("La prueba integral no completó la misión".into());
            }
        }
        if smoke && game.phase == Phase::Won {
            victory_frames += 1;
            if victory_frames == 4 {
                save_capture(&rl, &captures.join("smoke_victory.png"));
                println!("VICTORY_MENU_AFTER_OUTRO replay_verified={smoke_replay_verified}");
            }
        } else {
            victory_frames = 0;
        }
        // Render a few menu frames before the diagnostic confirms it, so the
        // screenshot reads a presented menu buffer instead of the last video frame.
        if demo && game.phase == Phase::Briefing {
            menu_frames += 1;
            if menu_frames == 4 {
                println!("MENU_AFTER_INTRO");
                if smoke {
                    save_capture(&rl, &captures.join("smoke_menu.png"));
                }
                game.start();
                accumulator = 0.0;
            }
        } else {
            menu_frames = 0;
        }
    }
    if smoke && (game.phase != Phase::Won || !smoke_replay_verified) {
        return Err("Prueba integral interrumpida antes del cierre final".into());
    }
    Ok(exit)
}
