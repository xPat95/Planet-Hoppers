//! Standalone host; match rules, UI and audio live in their own modules.
use super::kenny_dialogue::{self, DialogueAction};
use super::session_ui::{self, SessionAction, SessionUi};
use super::world_map::WorldMap;
use crate::minigames::southpark::threats::{ThreatKind, PREFIX};
use crate::minigames::southpark::{locations::WorldLocations, session::SessionState};
use crate::{
    audio::{AudioConfig, AudioController},
    minigames::southpark::session::{SaveKennySession, SessionConfig},
};
use crate::{
    graphics::billboard::SpriteBillboard,
    graphics::resources::{save_frame, StaticModel},
    graphics::threats::{draw_status, ThreatModels},
    minigames::{
        common::{
            fps::{CursorCapture, PlayerInput},
            interaction::select_target_with,
            manifest::SceneManifest,
        },
        southpark::{
            kenny::KENNY_ID, navigation::NPC_HEIGHT, DevConfig, KennyDialogue, SouthParkScene,
        },
    },
};
use raylib::{ffi, prelude::*};
use serde_json::json;
use std::{path::PathBuf, time::Instant};

use super::southpark_entry::{EntryConfig, SouthParkEntry, SouthParkExit, SouthParkVisits};
use crate::minigames::common::host::{MinigameResult, ReturnConfirmation};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    run_visit(&mut SouthParkVisits::default()).map(|_| ())
}
/// Standalone lifetime adapter. A future host retains `visits` across entries.
pub fn run_visit(
    visits: &mut SouthParkVisits,
) -> Result<SouthParkExit, Box<dyn std::error::Error>> {
    run_visit_with_args(visits, std::env::args().skip(1))
}
/// Host entry (solar system): the host's own command-line options are not South Park's.
pub fn run_visit_with_args(
    visits: &mut SouthParkVisits,
    mut args: impl Iterator<Item = String>,
) -> Result<SouthParkExit, Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = root.join("config/southpark-dev.json");
    let mut smoke = false;
    let mut kenny_smoke = false;
    let mut no_probes = false;
    let mut initial_threat = None;
    let mut threat_smoke = false;
    let mut session_smoke = false;
    let mut world_smoke = false;
    let mut entry_smoke = false;
    let mut return_smoke_goal = None;
    let mut dev_threats = false;
    let mut muted = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke-return" => {
                return_smoke_goal = Some(match args.next().as_deref() {
                    Some("victory") => MinigameResult::Victory,
                    Some("defeat") => MinigameResult::Defeat,
                    Some("abandoned") => MinigameResult::Abandoned,
                    _ => return Err("--smoke-return needs victory, defeat or abandoned".into()),
                });
                no_probes = true;
            }
            "--smoke-entry" => entry_smoke = true,
            "--smoke-world" => {
                world_smoke = true;
                session_smoke = true;
                no_probes = true;
            }
            "--smoke" => smoke = true,
            "--smoke-session" => {
                session_smoke = true;
                no_probes = true;
            }
            "--dev-threats" => {
                dev_threats = true;
                no_probes = true;
            }
            "--mute" => muted = true,
            "--smoke-threats" => {
                smoke = true;
                threat_smoke = true;
                no_probes = true;
            }
            "--smoke-kenny" => {
                smoke = true;
                kenny_smoke = true;
                no_probes = true;
            }
            "--no-test-objects" => no_probes = true,
            "--threat" => {
                initial_threat = Some(
                    ThreatKind::parse(
                        &args
                            .next()
                            .ok_or("--threat needs bus, ufo, sewer, piano or snowman")?,
                    )
                    .ok_or("Unknown threat: use bus, ufo, sewer, piano or snowman")?,
                );
                no_probes = true;
                dev_threats = true;
            }
            "--config" => {
                config_path = PathBuf::from(args.next().ok_or("--config requires a JSON path")?)
            }
            "--help" | "-h" => {
                println!("Intro: Espacio/Enter saltar. Backspace: confirmar regreso al sistema solar durante exploracion. --smoke-entry: validar intro y visitas.\nM: mapa ampliado (pausa el mundo). Arrastrar: mover; rueda: zoom; R: jugador; M/Esc: cerrar.\n--smoke-world: prueba gráfica del mundo dinámico y los mapas.\ncargo run --bin southpark-locations: auditoría de zonas sin ventana.");
                println!("Partida normal: hablá con Kenny y elegí Comenzar desafío. Esc/P: pausa, reiniciar o regresar al sistema solar.\n--dev-threats: habilitar F5/F6/F7/F10/F11 para pruebas individuales.\n--smoke-session: partida automatizada con audio y capturas de fase 5.\n--mute: cargar y verificar audio con volúmenes en cero.");
                println!("cargo run --bin southpark -- [--config PATH] [--no-test-objects] [--threat bus|ufo|sewer|piano|snowman] [--smoke] [--smoke-kenny] [--smoke-threats]\nWASD: caminar | Shift: correr | mouse: mirar | E: interactuar\nEsc: pausa / cerrar mapa o dialogo | Enter: aceptar / continuar\nF5: autobús | F6: OVNI | F7: alcantarilla | F10: piano | F11: nieve | F8: reiniciar | F9: explorar\nF1: ayuda | F2: colisionadores | F3: indicador Kenny | F12: captura | Ctrl+Q: salir");
                return Ok(SouthParkExit::WindowClosed);
            }
            _ => return Err(format!("Unknown option: {arg}").into()),
        }
    }
    let mut config = DevConfig::load(&config_path)?;
    if no_probes || !smoke {
        config.test_objects = false;
    }
    if smoke && !kenny_smoke && !threat_smoke && !config.test_objects {
        return Err("--smoke requires the three technical interaction objects".into());
    }
    let manifest_path = root.join("assets/models/minigames/southpark/southpark_town.scene.json");
    let manifest = SceneManifest::load(&manifest_path)?;
    let mut scene = SouthParkScene::load(&manifest, config.clone())?;
    let dynamic_mode = !smoke && !dev_threats && (!session_smoke || world_smoke);
    let locations = if dynamic_mode {
        Some(WorldLocations::build(
            &manifest,
            &scene.world,
            &scene.roads,
        )?)
    } else {
        None
    };
    if dynamic_mode {
        scene.kenny_hint = false;
    }
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .msaa_4x()
        .title("Planet-Hoppers | South Park | SAVE KENNY!")
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(640, 480);
    if smoke || session_smoke || entry_smoke || return_smoke_goal.is_some() {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    // The session smoke uses the real audio clock, so do not exhaust its frame
    // budget faster than the combined voice file can physically play.
    rl.set_target_fps(if session_smoke || entry_smoke || return_smoke_goal.is_some() {
        240
    } else if smoke {
        0
    } else {
        120
    });
    let device = RaylibAudio::init_audio_device().ok();
    let mut audio_config = AudioConfig::load(&root.join("config/southpark-audio.json"))
        .unwrap_or_else(|e| {
            eprintln!("AUDIO_CONFIG_WARNING {e}");
            AudioConfig::default()
        });
    if muted {
        audio_config.music_volume = 0.0;
        audio_config.sfx_volume = 0.0;
        audio_config.voice_volume = 0.0;
        audio_config.footsteps_volume = 0.0;
    }
    let mut audio = AudioController::load(device.as_ref(), &root, audio_config);
    let mut session = SaveKennySession::new(SessionConfig::load(
        &root.join("config/southpark-session.json"),
    )?);
    if let Some(length) = audio.voice_duration() {
        session.config.voice_fallback_seconds = length;
        session.config.kyle_switch_seconds = session
            .config
            .kyle_switch_seconds
            .min((length - 0.15).max(0.0));
    }
    let session_ui = SessionUi::load(&mut rl, &thread, &root);
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos() as u64;
    if let Some(locations) = locations {
        session.enable_world(locations, seed);
        session.place_kenny_near_player(&mut scene)?;
    }
    let mut world_map = WorldMap::load(&mut rl, &thread, &root, &scene)?;
    unsafe {
        ffi::rlSetClipPlanes(0.05, 1500.0);
    }
    let mut model = StaticModel::load(
        &mut rl,
        &thread,
        &manifest.visual_path(&manifest_path)?,
        Some("SP_Snow"),
    )?;
    let mut threat_models = ThreatModels::load(
        &mut rl,
        &thread,
        &root.join("assets/models/minigames/southpark/threats"),
    )?;
    if let Some(kind) = initial_threat {
        scene.start_threat(kind)?;
    }
    // The repository's actual asset directory contains a space. Never move or
    // rewrite the sole source PNG to match a guessed path.
    let sprite = SpriteBillboard::load(
        &mut rl,
        &thread,
        &root.join("assets/textures/minigames/South Park/KennyMcCormick.png"),
        NPC_HEIGHT,
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
    let entry_config = EntryConfig::load(&root.join("config/southpark-entry.json"))?;
    let mut entry =
        SouthParkEntry::new(visits, entry_config.clone(), audio.intro_duration(), &scene)?;
    entry.active = !smoke && !session_smoke && !dev_threats;
    let mut entry_case = 0;
    let mut entry_checks = Vec::new();
    let mut entry_captures = std::collections::BTreeSet::new();
    let mut exit = SouthParkExit::WindowClosed;
    let mut return_confirmation = ReturnConfirmation::default();
    let captures = if let Some(goal) = return_smoke_goal {
        root.join(format!("target/minigame-return-{goal:?}"))
    } else { root.join(if entry_smoke || world_smoke {
        "docs/references/southpark/phase6b"
    } else if session_smoke {
        "target/southpark-session-smoke"
    } else if threat_smoke {
        "target/southpark-threats-smoke"
    } else if kenny_smoke {
        "target/southpark-kenny-smoke"
    } else {
        "target/southpark-smoke"
    }) };
    if smoke || session_smoke || entry_smoke || return_smoke_goal.is_some() {
        std::fs::create_dir_all(&captures)?;
    }
    let mut cursor = CursorCapture::default();
    cursor.apply(&mut rl);
    let mut debug = false;
    let mut help = dev_threats || smoke;
    let mut frame = 0;
    let mut released = false;
    let mut recaptured = false;
    let mut mouse_changed = false;
    let mut running_read = false;
    let initial_feet = scene.player.feet;
    let mut run_distance = 0.0;
    let mut render_times = Vec::new();
    let mut threat_test = super::threat_smoke::ThreatSmoke::new();
    let mut session_test = super::session_smoke::SessionSmoke::new();
    let mut return_test = return_smoke_goal.map(super::return_smoke::ReturnSmoke::new);
    let mut dialogue_opened = false;
    let mut dialogue_paused = false;
    let mut cancelled_and_resumed = false;
    let mut challenge_accepted = false;
    let mut accepted_and_closed = false;
    let mut held_position = scene.kenny.position();
    let mut held_player = scene.player.feet;
    let mut held_look = (scene.player.yaw, scene.player.pitch);
    let mut modal_fps_frozen = false;
    let mut modal_cursor_free = false;
    let kenny_start = scene.kenny.position();
    let initial_look = (scene.player.yaw, scene.player.pitch);
    while !rl.window_should_close() {
        audio.update();
        if let Some(test) = &mut return_test {
            test.before_frame(&rl, &mut scene, &session, &return_confirmation, entry.active)?;
        }
        if session_smoke {
            session_test.before_frame(&rl, &mut scene, &session)?;
        }
        if smoke {
            if threat_smoke {
                threat_test.before_frame(&mut scene)?;
            } else if kenny_smoke {
                inject_kenny_input(&rl, frame);
                if frame <= 40 || (109..=110).contains(&frame) {
                    let direction = (scene.kenny.position() + Vector3::up() * (NPC_HEIGHT * 0.5)
                        - scene.player.eye())
                    .normalized();
                    scene.player.yaw = direction.z.atan2(direction.x);
                    scene.player.pitch = direction.y.asin();
                }
            } else {
                inject_smoke_input(&rl, frame);
            }
        }
        let frame_start = Instant::now();
        let dt = if threat_smoke || session_smoke || return_test.is_some() {
            0.1
        } else if smoke {
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
        let intro_was_active = entry.active;
        if entry_smoke && entry.active {
            if scene.player.feet != initial_feet
                || (scene.player.yaw, scene.player.pitch) != initial_look
                || scene.kenny.position() != kenny_start
                || session.state != SessionState::Exploring
            {
                return Err("World/FPS changed during cinematic".into());
            }
            for key in [KeyboardKey::KEY_W, KeyboardKey::KEY_M, KeyboardKey::KEY_E] {
                unsafe {
                    ffi::PlayAutomationEvent(ffi::AutomationEvent {
                        frame: 0,
                        type_: 2,
                        params: [key as i32, 0, 0, 0],
                    });
                }
            }
        }
        if entry_smoke
            && entry.active
            && entry.elapsed >= 0.6
            && (entry_case == 1 || entry_case == 2)
        {
            unsafe {
                ffi::PlayAutomationEvent(ffi::AutomationEvent {
                    frame: 0,
                    type_: 2,
                    params: [
                        if entry_case == 1 {
                            KeyboardKey::KEY_SPACE as i32
                        } else {
                            KeyboardKey::KEY_ENTER as i32
                        },
                        0,
                        0,
                        0,
                    ],
                });
            }
        }
        let intro_clock = audio.intro_clock();
        if entry.active {
            entry.update_audio(
                dt,
                rl.is_key_pressed(KeyboardKey::KEY_SPACE)
                    || rl.is_key_pressed(KeyboardKey::KEY_ENTER),
                visits,
                intro_clock,
            );
            if !entry.active {
                cursor.captured = true;
                cursor.apply(&mut rl);
            }
        }
        if !intro_was_active
            && session.state == SessionState::Exploring
            && scene.dialogue == KennyDialogue::Closed
            && rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
        {
            session.pause();
            return_confirmation.request();
        }
        let mut input = PlayerInput::read(&rl);
        let session_was_modal = session.blocks_controls();
        let was_paused = session.state == SessionState::Paused;
        let had_dialogue = scene.dialogue != KennyDialogue::Closed;
        let map_was_open = session.state == SessionState::MapOpen;
        if !smoke && !intro_was_active && !had_dialogue {
            if map_was_open {
                world_map.input(&rl, scene.player.feet);
                if rl.is_key_pressed(KeyboardKey::KEY_M)
                    || rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
                {
                    session.continue_match(&mut scene)?;
                }
            } else if rl.is_key_pressed(KeyboardKey::KEY_M) && session.open_map() {
                world_map.view.fit();
            }
        }
        let map_consumed = map_was_open || session.state == SessionState::MapOpen;
        let mut restarted = false;
        if !smoke && !intro_was_active && !map_consumed && !had_dialogue {
            let action = session_ui::input(&rl, &session, &return_confirmation);
            let result = match action {
                SessionAction::Continue => session.continue_match(&mut scene),
                SessionAction::Explore => {
                    audio.stop_effects();
                    audio.silence_voice();
                    session.explore(&mut scene)
                }
                SessionAction::Replay => {
                    audio.stop_effects();
                    audio.silence_voice();
                    audio.set_paused(false);
                    return_confirmation.cancel();
                    seed = seed.wrapping_add(0x9e3779b97f4a7c15);
                    match super::southpark_entry::restart_attempt(&mut scene, &mut session, &manifest, &config, seed) {
                        Ok(()) => { restarted = true; world_map.view.fit(); Ok(()) }
                        Err(error) => Err(error.to_string()),
                    }
                }
                SessionAction::Pause => {
                    session.pause();
                    Ok(())
                }
                SessionAction::None => Ok(()),
                SessionAction::Return(result) => {
                    exit = SouthParkExit::Returned(result);
                    break;
                }
                SessionAction::AskReturn => { return_confirmation.request(); Ok(()) }
                SessionAction::CancelReturn => { return_confirmation.cancel(); Ok(()) }
                SessionAction::ConfirmReturn => {
                    if let Some(result) = return_confirmation.confirm() {
                        exit = SouthParkExit::Returned(result);
                        break;
                    }
                    Ok(())
                }
            };
            if let Err(error) = result {
                session.error = Some(error);
            }
        }
        if !smoke
            && !intro_was_active
            && !map_consumed
            && dev_threats
            && !session.in_match()
            && scene.dialogue == KennyDialogue::Closed
        {
            let requested = if rl.is_key_pressed(KeyboardKey::KEY_F5) {
                Some(ThreatKind::Bus)
            } else if rl.is_key_pressed(KeyboardKey::KEY_F6) {
                Some(ThreatKind::Ufo)
            } else if rl.is_key_pressed(KeyboardKey::KEY_F7) {
                Some(ThreatKind::Sewer)
            } else if rl.is_key_pressed(KeyboardKey::KEY_F10) {
                Some(ThreatKind::Piano)
            } else if rl.is_key_pressed(KeyboardKey::KEY_F11) {
                Some(ThreatKind::Snowman)
            } else {
                None
            };
            let action = if let Some(kind) = requested {
                scene.start_threat(kind)
            } else if rl.is_key_pressed(KeyboardKey::KEY_F8) {
                scene.restart_threat()
            } else if rl.is_key_pressed(KeyboardKey::KEY_F9) {
                scene.clear_threat()
            } else {
                Ok(())
            };
            if let Err(message) = action {
                scene.confirmation = Some((message, 4.0));
            }
        }
        let modal_at_start = intro_was_active
            || had_dialogue
            || session_was_modal
            || scene.dialogue != KennyDialogue::Closed
            || session.blocks_controls();
        if scene.dialogue != KennyDialogue::Closed {
            match kenny_dialogue::input(&rl, scene.dialogue) {
                DialogueAction::Accept => {
                    if smoke {
                        scene.accept_challenge();
                    } else {
                        seed = seed.wrapping_add(0x9e3779b97f4a7c15);
                        if let Err(error) = session.start(&mut scene, seed) {
                            scene.confirmation = Some((error, 4.0));
                        }
                    }
                }
                DialogueAction::Close => {
                    scene.close_dialogue();
                    cursor.captured = true;
                    cursor.apply(&mut rl);
                }
                DialogueAction::None => {}
            }
            input.toggle_cursor = false;
        }
        if session_was_modal
            && !session.blocks_controls()
            && scene.dialogue == KennyDialogue::Closed
        {
            cursor.captured = true;
            cursor.apply(&mut rl);
        }
        if modal_at_start {
            input.toggle_cursor = false;
        }
        if smoke && (3..=50).contains(&frame) {
            running_read |= input.forward > 0.0 && input.sprint;
        }
        let old_yaw = scene.player.yaw;
        let focused = smoke || session_smoke || return_test.is_some() || rl.is_window_focused();
        let active = !modal_at_start && cursor.update(&mut rl, &mut input, focused);
        let previous_feet = scene.player.feet;
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
        let walk_speed = if active {
            (scene.player.feet - previous_feet).length() / dt.max(0.0001)
        } else {
            0.0
        };
        audio.set_paused(session.state == SessionState::Paused);
        if intro_was_active || restarted || (was_paused && session.state != SessionState::Paused) {
            // Freeze world simulation until the FPS handoff, including Kenny.
        } else if smoke {
            scene.update_kenny(dt);
        } else if let Err(error) = session.update(
            if session_smoke || return_test.is_some() {
                dt
            } else {
                rl.get_frame_time()
            },
            &mut scene,
            audio.voice_clock(),
        ) {
            session.error = Some(error);
        }
        // Automatic relocation/search transitions also leave modal input.
        if (session_was_modal || had_dialogue)
            && !session.blocks_controls()
            && scene.dialogue == KennyDialogue::Closed
            && !cursor.captured
        {
            cursor.captured = true;
            cursor.apply(&mut rl);
        }
        audio.set_music(if entry.active {
            if entry.full_intro {
                crate::audio::MusicMode::Intro
            } else {
                crate::audio::MusicMode::Silent
            }
        } else {
            session.music()
        });
        for cue in session.take_cues() {
            audio.cue(cue);
        }
        audio.footsteps(if session.blocks_controls() {
            0.0
        } else {
            walk_speed
        });
        let threat_capture = if threat_smoke {
            threat_test.capture(&scene)
        } else {
            None
        };
        let camera = if entry.active {
            entry.camera()
        } else if threat_capture.is_some() {
            threat_test.camera(&scene)
        } else if smoke && !kenny_smoke && !threat_smoke && frame == 75 {
            Camera3D::perspective(
                Vector3::new(260.0, 260.0, 310.0),
                Vector3::zero(),
                Vector3::up(),
                65.0,
            )
        } else if kenny_smoke && frame == 195 {
            Camera3D::perspective(
                scene.kenny.position() + Vector3::new(-2.8, 1.7, 2.0),
                scene.kenny.position() + Vector3::up() * 0.7,
                Vector3::up(),
                65.0,
            )
        } else {
            scene.player.camera()
        };
        let pose = sprite.pose(
            scene.kenny.position(),
            camera.position,
            scene.kenny.animation.roll() + scene.threats.sprite_roll(),
            scene.kenny.animation.bounce(),
        );
        let mut pose = pose;
        pose.height *= scene.threats.sprite_height_scale();
        let kenny_index = scene.kenny_target();
        scene.targets[kenny_index].bounds = pose.bounds();
        let selected = if active && session.permits_interaction() && !session.blocks_controls() {
            select_target_with(
                &scene.targets,
                &scene.world,
                scene.player.eye(),
                scene.player.direction(),
                scene.interaction_distance.max(scene.threats.range()),
                |index, origin, direction, range, broad| {
                    if index == kenny_index {
                        if session.in_match()
                            || scene.threats.owns_kenny()
                            || !scene.threats.visible_kenny()
                        {
                            None
                        } else {
                            sprite.hit(
                                &pose,
                                origin,
                                direction,
                                range.min(scene.interaction_distance),
                            )
                        }
                    } else if scene.targets[index].id.starts_with(PREFIX) {
                        scene.threats.refine(
                            &scene.targets[index].id,
                            origin,
                            direction,
                            scene.threats.range(),
                            broad,
                        )
                    } else {
                        (broad <= scene.interaction_distance).then_some(broad)
                    }
                },
            )
        } else {
            None
        };
        if active && input.interact {
            if let Some(index) = selected {
                scene.interact(index);
            }
        }
        if (scene.dialogue != KennyDialogue::Closed || session.blocks_controls()) && cursor.captured
        {
            cursor.captured = false;
            cursor.apply(&mut rl);
        }
        let active = active
            && !intro_was_active
            && scene.dialogue == KennyDialogue::Closed
            && !session.blocks_controls();
        if kenny_smoke {
            if frame == 40 {
                dialogue_opened = scene.dialogue == KennyDialogue::Invitation;
                held_position = scene.kenny.position();
                held_player = scene.player.feet;
                held_look = (scene.player.yaw, scene.player.pitch);
            }
            if frame == 65 {
                modal_fps_frozen = (scene.player.feet - held_player).length() < 0.0001
                    && (scene.player.yaw - held_look.0).abs() < 0.0001
                    && (scene.player.pitch - held_look.1).abs() < 0.0001;
                modal_cursor_free = !cursor.captured && !rl.is_cursor_hidden();
                dialogue_paused = (scene.kenny.position() - held_position).length() < 0.0001
                    && scene.kenny.animation.weight < 0.02;
            }
            if frame == 90 {
                cancelled_and_resumed = scene.dialogue == KennyDialogue::Closed
                    && (scene.kenny.position() - held_position).length() > 0.1;
            }
            if frame == 118 {
                challenge_accepted = scene.dialogue == KennyDialogue::Preparing;
            }
            if frame == 133 {
                accepted_and_closed = scene.dialogue == KennyDialogue::Closed && cursor.captured;
            }
        }
        if session.state != SessionState::Paused { scene.tick_confirmation(dt); }
        if rl.is_key_pressed(KeyboardKey::KEY_F1) {
            help = !help;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F2) {
            debug = !debug;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_F3) {
            scene.kenny_hint = !scene.kenny_hint;
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
                    if target.id == KENNY_ID || target.id.starts_with(PREFIX) {
                        continue;
                    }
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
                threat_models.draw(&mut view, camera.position, &scene.threats);
                if scene.threats.visible_kenny() {
                    sprite.draw(&mut view, &pose);
                }
            }
            d.draw_text("SOUTH PARK - SAVE KENNY!", 16, 14, 20, Color::DARKGRAY);
            if !entry.active
                && !smoke
                && !dev_threats
                && session.state == SessionState::Exploring
                && scene.dialogue == KennyDialogue::Closed
            {
                d.draw_text(
                    "E: hablar | M: mapa | Retroceso: salir de South Park",
                    16,
                    46,
                    16,
                    Color::DARKBLUE,
                );
            }
            if !smoke && dev_threats {
                d.draw_fps(width - 100, 14);
            }
            if active
                && threat_capture.is_none()
                && !(smoke && frame == 75)
                && !(kenny_smoke && frame == 195)
            {
                let color = if selected.is_some() {
                    Color::YELLOW
                } else {
                    Color::WHITE
                };
                d.draw_circle(width / 2, height / 2, 3.0, Color::BLACK);
                d.draw_line(width / 2 - 7, height / 2, width / 2 + 7, height / 2, color);
                d.draw_line(width / 2, height / 2 - 7, width / 2, height / 2 + 7, color);
                if let Some(index) = selected {
                    let text = if index == kenny_index {
                        "[E] Hablar con Kenny".into()
                    } else if scene.targets[index].id.starts_with(PREFIX) {
                        format!("[E] {}", scene.targets[index].label)
                    } else {
                        format!("[E] Interactuar - {}", scene.targets[index].label)
                    };
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
            } else if !active
                && !cursor.captured
                && scene.dialogue == KennyDialogue::Closed
                && !session.blocks_controls()
            {
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
                if !(smoke && frame == 75) && !session.in_match() {
                    d.draw_text(text, 16, 46, 20, Color::DARKGREEN);
                }
            }
            if help && !session.blocks_controls() {
                let help_x = if smoke { 16 } else { 220 };
                d.draw_rectangle(
                    help_x - 16,
                    height - 55,
                    width - help_x + 16,
                    55,
                    Color::new(20, 30, 40, 210),
                );
                d.draw_text(
                    if smoke {
                        "WASD caminar | Shift correr | Mouse mirar | E interactuar | ESC cursor"
                    } else {
                        "WASD: mover | Mouse: mirar | E: usar"
                    },
                    help_x,
                    height - 48,
                    if smoke { 18 } else { 14 },
                    Color::WHITE,
                );
                d.draw_text(
                    if smoke {
                        "F1 ayuda | F2 colisiones | F3 Kenny | F12 captura | Ctrl+Q salir"
                    } else {
                        "Shift: correr | M: mapa | Esc/P: pausa"
                    },
                    help_x,
                    height - 25,
                    if smoke { 18 } else { 14 },
                    Color::LIGHTGRAY,
                );
            }
            if scene.kenny_hint
                && !session.in_match()
                && scene.dialogue == KennyDialogue::Closed
                && !(kenny_smoke && frame == 195)
            {
                d.draw_text(
                    &format!(
                        "Kenny: {:.1} m | F3 ocultar indicador",
                        (scene.kenny.position() - scene.player.feet).length()
                    ),
                    16,
                    73,
                    18,
                    Color::new(130, 65, 15, 255),
                );
            }
            if scene.dialogue != KennyDialogue::Closed {
                kenny_dialogue::draw(&mut d, scene.dialogue, width, height);
            }
            if !session.in_match() {
                draw_status(&mut d, &scene.threats, width);
            }
            session_ui.draw(&mut d, &session, &scene.threats, width, height, &return_confirmation);
            if let Some(test) = &mut return_test {
                if let Some(name) = test.capture(&session, &return_confirmation) {
                    save_frame(&captures.join(name), width, height)?;
                }
            }
            if !entry.active && scene.dialogue == KennyDialogue::Closed {
                world_map.draw(&mut d, &scene, &session, width, height);
            }
            if help && scene.confirmation.is_none() && (dev_threats || smoke) && !session.in_match()
            {
                d.draw_text(
                    "F5 bus | F6 OVNI | F7 tapa | F10 piano | F11 nieve | F8 reiniciar | F9 explorar",
                    16,
                    50,
                    18,
                    Color::DARKGRAY,
                );
            }
            entry.draw(&mut d, width, height);
            if entry_smoke && entry_case == 0 && entry.active {
                for (at, name) in [
                    (3.0, "intro-30.png"),
                    (12.0, "intro-120.png"),
                    (22.0, "intro-220.png"),
                ] {
                    if entry.elapsed >= at && entry_captures.insert(name) {
                        save_frame(&captures.join(name), width, height)?;
                    }
                }
            }
            if manual_capture {
                let path = root.join("target/southpark.png");
                save_frame(&path, width, height)?;
                println!("Captura: {}", path.display());
            }
            if smoke && !kenny_smoke && !threat_smoke && [0, 55, 65, 75].contains(&frame) {
                let name = match frame {
                    0 => "street.png",
                    55 => "cursor-released.png",
                    65 => "interaction.png",
                    _ => "overview.png",
                };
                save_frame(&captures.join(name), width, height)?;
            }
            if kenny_smoke && [34, 65, 125, 195].contains(&frame) {
                let name = match frame {
                    34 => "kenny-walking.png",
                    65 => "kenny-dialogue.png",
                    125 => "kenny-accepted.png",
                    _ => "kenny-side.png",
                };
                save_frame(&captures.join(name), width, height)?;
            }
            if let Some(name) = threat_capture {
                save_frame(&captures.join(name), width, height)?;
                threat_test.captured = true;
            }
            if session_smoke {
                if let Some(name) = session_test.capture(&session) {
                    save_frame(&captures.join(name), width, height)?;
                }
            }
        }
        render_times.push(frame_start.elapsed().as_secs_f64() * 1000.0);
        if smoke && !kenny_smoke && !threat_smoke && frame == 60 {
            scene.player.feet = initial_feet;
            scene.player.yaw = 0.0;
            scene.player.pitch = 0.0;
        }
        frame += 1;
        if let Some(test) = &mut return_test {
            test.after_frame(&scene, &session, &audio)?;
            if frame > 12000 { return Err("Return smoke timed out".into()); }
        }
        if entry_smoke && intro_was_active && !entry.active {
            if !visits.intro_seen || !cursor.captured || !rl.is_cursor_hidden() {
                return Err("Entry did not restore visit/cursor".into());
            }
            if entry_case == 0 && !intro_clock.finished {
                return Err("Intro ended before its stream finished".into());
            }
            entry_checks.push(json!({"case":entry_case,"elapsed":entry.elapsed,
                "full_intro":entry.full_intro,"cursor_captured":cursor.captured,
                "audio_finished":intro_clock.finished,"audio_position":intro_clock.position,"duration":entry.duration,
                "kenny_distance":(scene.kenny.position()-scene.player.feet).length()}));
            if entry_case == 3 {
                let report = json!({"status":"PASS","cases":entry_checks,"audio":audio.audit,
                    "manual_listening_verified":false,"physical_input_verified":false,"clock":"stream playback clock; first intro waits for natural audio completion"});
                std::fs::write(
                    captures.join("intro-validation.json"),
                    serde_json::to_string_pretty(&report)?,
                )?;
                println!("ENTRY_SMOKE_PASS {report}");
                break;
            }
            entry_case += 1;
            visits.intro_seen = entry_case == 3;
            for key in [KeyboardKey::KEY_SPACE, KeyboardKey::KEY_ENTER] {
                unsafe {
                    ffi::PlayAutomationEvent(ffi::AutomationEvent {
                        frame: 0,
                        type_: 1,
                        params: [key as i32, 0, 0, 0],
                    });
                }
            }
            entry =
                SouthParkEntry::new(visits, entry_config.clone(), audio.intro_duration(), &scene)?;
        }
        if session_smoke {
            session_test.after_frame(&scene, &session, &audio)?;
            if session_test.done() {
                break;
            }
            if frame > 12000 {
                return Err(format!(
                    "Session smoke timed out: {:?}, lives={}, saves={}",
                    session.state, session.lives, session.saves
                )
                .into());
            }
        }
        if threat_smoke {
            threat_test.after_frame(&mut scene)?;
            if threat_test.done() {
                break;
            }
            if frame > 3000 {
                return Err("Threat graphical test timed out".into());
            }
        }
        if smoke && !threat_smoke && frame >= if kenny_smoke { 210 } else { 90 } {
            break;
        }
    }
    audio.set_music(crate::audio::MusicMode::Silent);
    audio.stop_effects();
    audio.silence_voice();
    rl.enable_cursor();
    if let Some(test) = &return_test {
        test.finish(exit, &captures)?;
    } else if session_smoke {
        if world_smoke && !(world_map.zoomed && world_map.dragged) {
            return Err("Map zoom/drag not exercised".into());
        }
        session_test.finish(
            &scene,
            &session,
            &audio,
            session_ui.loaded_portraits,
            &captures,
        )?;
    } else if threat_smoke {
        if !threat_test.done() {
            return Err("Threat test interrupted before all ten cases".into());
        }
        let report = json!({"status":"PASS","cases":threat_test.results,"frames":frame,"model_loading_and_dimensions":"PASS","manual_hardware_input_tested":false,"note":"Real OpenGL and shared raycasting/E input; automated player placement, aiming and cinematic capture cameras."});
        std::fs::write(
            captures.join("runtime-validation.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("THREATS_SMOKE_PASS {report}");
    } else if kenny_smoke {
        if !dialogue_opened
            || !dialogue_paused
            || !cancelled_and_resumed
            || !challenge_accepted
            || !accepted_and_closed
            || !modal_fps_frozen
            || !modal_cursor_free
            || (scene.kenny.position() - kenny_start).length() < 0.5
        {
            return Err(format!("Kenny smoke failed: opened={dialogue_opened}, paused={dialogue_paused}, resumed={cancelled_and_resumed}, accepted={challenge_accepted}, closed={accepted_and_closed}").into());
        }
        let report = json!({"status":"PASS","frames":frame,"texture":{"width":sprite.pixel_width,"height":sprite.pixel_height,"transparent_pixels":sprite.alpha.iter().filter(|a|**a==0).count(),"opaque_pixels":sprite.alpha.iter().filter(|a|**a==255).count()},"road_nodes":scene.roads.nodes.len(),"road_edges":scene.roads.neighbors.iter().map(Vec::len).sum::<usize>()/2,"dialogue_opened_with_E":dialogue_opened,"stopped_and_settled":dialogue_paused,"cancel_resumed_walk":cancelled_and_resumed,"accepted_preparation_by_mouse":challenge_accepted,"closed_and_recaptured":accepted_and_closed,"fps_frozen_during_dialogue":modal_fps_frozen,"dialogue_cursor_free":modal_cursor_free,"manual_hardware_input_tested":false});
        std::fs::write(
            captures.join("runtime-validation.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("KENNY_SMOKE_PASS {report}");
    } else if smoke {
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
    println!(
        "AUDIO_RUNTIME_AUDIT {}",
        serde_json::to_string(&audio.audit)?
    );
    Ok(exit)
}

fn inject_kenny_input(rl: &RaylibHandle, frame: u32) {
    fn key(code: KeyboardKey, down: bool) {
        unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: if down { 2 } else { 1 },
                params: [code as i32, 0, 0, 0],
            });
        }
    }
    match frame {
        40 | 110 => key(KeyboardKey::KEY_E, true),
        41 | 111 => key(KeyboardKey::KEY_E, false),
        45 => key(KeyboardKey::KEY_W, true),
        50 => unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: 7,
                params: [
                    rl.get_screen_width() / 2 + 150,
                    rl.get_screen_height() / 2 - 100,
                    0,
                    0,
                ],
            });
        },
        60 => key(KeyboardKey::KEY_W, false),
        70 => key(KeyboardKey::KEY_ESCAPE, true),
        71 => key(KeyboardKey::KEY_ESCAPE, false),
        75 => key(KeyboardKey::KEY_W, true),
        105 => key(KeyboardKey::KEY_W, false),
        118 => unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: 7,
                params: [
                    rl.get_screen_width() / 2 - 120,
                    rl.get_screen_height() / 2 + 57,
                    0,
                    0,
                ],
            });
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: 6,
                params: [0, 0, 0, 0],
            });
        },
        119 => unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: 5,
                params: [0, 0, 0, 0],
            });
        },
        132 => key(KeyboardKey::KEY_ENTER, true),
        133 => key(KeyboardKey::KEY_ENTER, false),
        _ => {}
    }
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
