//! Minecraft visit and standalone adapter: one graphics context and one music stream.
use crate::{
    graphics::{
        mining::MiningVisuals,
        resources::{save_frame, StaticModel},
    },
    minigames::{
        common::{
            fps::{CursorCapture, PlayerInput},
            host::{MinigameExit, MinigameResult, ReturnConfirmation},
            manifest::from_blender,
        },
        minecraft::{
            mine::MineScene,
            mining::{MiningConfig, MiningEvent, MiningSession},
            rush::{RushConfig, RushSession, RushState},
            sound::MinecraftAudio,
            visit::{MinecraftVisits, VillageAction, VillageMenu},
            MenuAction, MinecraftConfig, MinecraftScene, SurfacePose, SurfaceState,
        },
    },
};
use raylib::{ffi, prelude::*};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
#[path = "minecraft_rush_smoke.rs"]
mod smoke;
type Error = Box<dyn std::error::Error>;
pub(super) struct MineRun {
    pub scene: MineScene,
    pub mining: MiningSession,
    pub game: RushSession,
    pub model: StaticModel,
}
impl MineRun {
    fn load(
        root: &Path,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        config: RushConfig,
    ) -> Result<Self, Error> {
        let path = root.join("assets/models/minigames/minecraft/rush/minecraft_mine.scene.json");
        let scene = MineScene::load(
            &path,
            MinecraftConfig::load(&root.join("config/minecraft-dev.json"))?,
        )?;
        if scene
            .data
            .interactive_blocks
            .iter()
            .filter(|b| b.kind == "diamond")
            .count()
            != config.available_diamonds as usize
        {
            return Err("Diamond configuration differs from expanded scene".into());
        }
        let mut mining = MiningSession::new(
            &scene,
            MiningConfig::load(&root.join("config/minecraft-mining.json"))?,
        );
        mining.config.target_diamonds = config.required_diamonds;
        let mut model = StaticModel::load_mine(rl, thread, &scene.manifest.visual_path(&path)?)?;
        if model.independent_groups().len() != mining.blocks.len() {
            return Err("Expanded interactive mesh groups missing".into());
        }
        let (p, c, r) = scene.light_arrays();
        model.set_mine_lights(&p, &c, &r)?;
        Ok(Self {
            scene,
            mining,
            game: RushSession::new(config),
            model,
        })
    }
    fn restart(&mut self) {
        self.game.restart(&mut self.scene, &mut self.mining);
    }
    fn sync(&mut self, events: &[MiningEvent]) {
        for e in events {
            match e {
                MiningEvent::BlockDestroyed { block, .. } => {
                    self.model.set_group_visible(block, false);
                }
                MiningEvent::SessionReset => {
                    for b in &self.mining.blocks {
                        self.model.set_group_visible(&b.id, true);
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn run(args: Vec<String>) -> Result<(), Error> {
    run_visit(args).map(|_| ())
}

pub fn run_visit(args: Vec<String>) -> Result<MinigameExit, Error> {
    run_visit_with_state(args, &mut MinecraftVisits::default())
}

pub fn run_visit_with_state(
    args: Vec<String>,
    visits: &mut MinecraftVisits,
) -> Result<MinigameExit, Error> {
    run_visit_with_results(args, visits, &mut |_| Ok(()))
}

pub fn run_visit_with_results(
    args: Vec<String>,
    visits: &mut MinecraftVisits,
    observer: &mut crate::minigames::common::host::ResultObserver<'_>,
) -> Result<MinigameExit, Error> {
    let result = run_inner(args, visits, observer);
    // Window close/error also ends a genuinely active attempt; completed victories stay intact.
    visits.finish(MinigameResult::Abandoned);
    result
}

fn run_inner(args: Vec<String>, visits: &mut MinecraftVisits, observer: &mut crate::minigames::common::host::ResultObserver<'_>) -> Result<MinigameExit, Error> {
    let mut reporter = crate::minigames::common::host::AttemptReporter::default();
    let visit_start = visits.completed().len();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut direct = false;
    let mut test = false;
    let mut mute = false;
    let mut host_smoke = false;
    let mut exploration_smoke = false;
    let mut output = root.join("docs/references/minecraft/phase5");
    let mut config = root.join("config/minecraft-rush.json");
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mine" => direct = true,
            "--smoke-rush" => {
                test = true;
                mute = true;
            }
            "--smoke-host" => {
                test = true;
                mute = true;
                host_smoke = true;
            }
            "--smoke-exploration-exit" => {
                test = true;
                mute = true;
                host_smoke = true;
                exploration_smoke = true;
            }
            "--mute" => mute = true,
            "--capture-dir" => {
                output = PathBuf::from(args.next().ok_or("Missing capture directory")?)
            }
            "--rush-config" => config = PathBuf::from(args.next().ok_or("Missing rush config")?),
            "--help" | "-h" => {
                println!("cargo run --bin minecraft -- [--mine] [--mute] [--smoke-rush] [--rush-config FILE] [--capture-dir DIR]");
                return Ok(MinigameExit::WindowClosed);
            }
            _ => return Err(format!("Unknown Minecraft option {arg}").into()),
        }
    }
    let settings = RushConfig::load(&config)?;
    let path = root.join("assets/models/minigames/minecraft/minecraft_village.scene.json");
    let mut surface = MinecraftScene::load(
        &path,
        MinecraftConfig::load(&root.join("config/minecraft-dev.json"))?,
    )?;
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
        ffi::rlSetClipPlanes(0.05, 240.0);
    }
    let device = RaylibAudio::init_audio_device().ok();
    let mut audio = MinecraftAudio::load(device.as_ref(), &root, mute);
    let mut surface_model =
        StaticModel::load_overworld(&mut rl, &thread, &surface.manifest.visual_path(&path)?)?;
    let mut mine: Option<MineRun> = None;
    let mut in_mine = false;
    let mut return_pose: SurfacePose = surface.pose();
    let visuals = MiningVisuals::default();
    let mut cursor = CursorCapture::default();
    cursor.apply(&mut rl);
    let mut last = Instant::now();
    let mut frame = 0;
    let mut smoke = if host_smoke && !exploration_smoke {
        smoke::Smoke::winning_return()
    } else {
        smoke::Smoke::default()
    };
    let mut village_menu = VillageMenu::Closed;
    let mut exit = MinigameExit::WindowClosed;
    let mut abandonment = ReturnConfirmation::default();
    let mut host_tick = 0;
    let mut host_frozen = None;
    let mut host_mine_time = None;
    while !rl.window_should_close() {
        if test && !smoke.done() && !exploration_smoke {
            smoke.before(frame, &mut surface, mine.as_mut(), in_mine, &rl);
        }
        if host_smoke && (smoke.done() || exploration_smoke) {
            host_tick += 1;
            let event = match host_tick {
                1 => Some((KeyboardKey::KEY_ESCAPE, true)),
                2 => Some((KeyboardKey::KEY_ESCAPE, false)),
                3 => Some((KeyboardKey::KEY_R, true)),
                _ => None,
            };
            if let Some((key, down)) = event {
                unsafe {
                    ffi::PlayAutomationEvent(ffi::AutomationEvent {
                        frame: 0,
                        type_: if down { 2 } else { 1 },
                        params: [key as i32, 0, 0, 0],
                    });
                }
            }
            if host_tick == 2 {
                host_frozen = Some(surface.pose());
                host_mine_time = mine.as_ref().map(|m| m.game.remaining);
            }
            if host_tick > 2 {
                let held = host_frozen.unwrap();
                if surface.player.feet != held.feet
                    || surface.player.yaw != held.yaw
                    || surface.player.pitch != held.pitch
                {
                    return Err("Minecraft host smoke: village moved during exit menu".into());
                }
                if mine.as_ref().map(|m| m.game.remaining) != host_mine_time {
                    return Err("Minecraft host smoke: mine timer advanced in village".into());
                }
            }
            if visits.active_attempt().is_some() {
                return Err("Minecraft host smoke: challenge still active in village".into());
            }
        }
        let now = Instant::now();
        let dt = if test {
            1.0 / 60.0
        } else {
            (now - last).as_secs_f32()
        };
        last = now;
        let focused = test || rl.is_window_focused();
        let mut input = PlayerInput::read(&rl);
        let mut load = direct;
        direct = false;
        let mut returned = false;
        let mut skip = false;
        let old_mine = in_mine;
        let old_state = mine.as_ref().map(|m| m.game.state);
        let old_surface = surface.state;
        let old_village_menu = village_menu;
        let before = if in_mine {
            mine.as_ref().unwrap().scene.player.feet
        } else {
            surface.player.feet
        };
        if !in_mine {
            if village_menu != VillageMenu::Closed {
                if focused && village_menu.apply(village_input(&rl, village_menu)) {
                    exit = visits.exit_since(visit_start);
                    break;
                }
            } else if surface.state == SurfaceState::Exploring && focused && input.toggle_cursor {
                village_menu.apply(VillageAction::Pause);
                surface.selected = None;
                audio.reset_effects();
            } else if surface.state == SurfaceState::Exploring {
                if focused
                    && !cursor.captured
                    && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                {
                    cursor.captured = true;
                    cursor.apply(&mut rl);
                }
                let enabled = cursor.update(&mut rl, &mut input, focused);
                surface.update(input, dt, enabled);
            } else if surface.state == SurfaceState::MineMenu && focused {
                let action = two_buttons_input(&rl, true);
                if action == 1 {
                    surface.menu_action(MenuAction::Start);
                    load = true;
                }
                if action == 2 {
                    surface.menu_action(MenuAction::Cancel);
                }
            }
            if load {
                return_pose = surface.pose();
                {
                    let mut d = rl.begin_drawing(&thread);
                    d.clear_background(Color::new(13, 23, 28, 255));
                    d.draw_text("Preparando DIAMOND RUSH...", 35, 35, 28, Color::WHITE);
                }
                if let Some(m) = &mut mine {
                    m.restart();
                } else {
                    mine = Some(MineRun::load(&root, &mut rl, &thread, settings.clone())?);
                }
                mine.as_mut().unwrap().game.ready();
                visits.start_attempt();
                in_mine = true;
                skip = true;
                last = Instant::now();
            }
        } else {
            let m = mine.as_mut().unwrap();
            if m.game.state == RushState::Running
                && (!focused
                    || input.toggle_cursor
                    || rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE))
            {
                m.game.pause();
                skip = true;
            }
            if matches!(
                m.game.state,
                RushState::Paused | RushState::Victory | RushState::Defeat
            ) && focused
                && !skip
            {
                let action = if abandonment.pending {
                    match two_buttons_input(&rl, true) {
                        1 => {
                            abandonment.confirm();
                            visits.finish(MinigameResult::Abandoned);
                            returned = true;
                        }
                        2 => {
                            abandonment.cancel();
                            m.game.resume();
                            skip = true;
                        }
                        _ => {}
                    }
                    0
                } else if m.game.state == RushState::Paused {
                    pause_input(&rl)
                } else {
                    two_buttons_input(&rl, false)
                };
                match (m.game.state, action) {
                    (RushState::Paused, 1) => {
                        m.game.resume();
                        skip = true;
                    }
                    (RushState::Paused, 2) | (RushState::Victory | RushState::Defeat, 1) => {
                        m.restart();
                        visits.start_attempt();
                        skip = true;
                    }
                    (RushState::Paused, 3) if visits.active_attempt().is_some() => {
                        abandonment.request();
                    }
                    (RushState::Victory | RushState::Defeat, 2) => returned = true,
                    _ => {}
                }
            }
            if m.game.state == RushState::Running && !skip && !returned {
                m.game.advance(dt);
                if m.game.state == RushState::Running {
                    input.toggle_cursor = false;
                    let enabled = cursor.update(&mut rl, &mut input, focused);
                    let exit = m.scene.update(input, dt, enabled);
                    m.mining.update(
                        &mut m.scene,
                        rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT),
                        enabled,
                        dt,
                    );
                    m.game.inspect_world(&mut m.scene, &mut m.mining, exit);
                }
            }
            if m.game.state != RushState::Running || skip || returned {
                m.mining.update(&mut m.scene, false, false, 0.0);
                last = Instant::now();
            }
            if returned {
                // Retained resources may be reused, but no active countdown survives in the village.
                m.game.pause();
                audio.reset_effects();
                in_mine = false;
                surface.state = SurfaceState::Exploring;
                surface.pending_request = None;
                surface.selected = None;
                surface.player.feet = return_pose.feet;
                surface.player.yaw = return_pose.yaw;
                surface.player.pitch = return_pose.pitch;
                if !surface.world.can_occupy(
                    surface.player.feet,
                    surface.player.config.radius,
                    surface.player.config.body_height,
                ) {
                    surface.player.feet = from_blender(surface.manifest.spawn.position)
                        - Vector3::up() * surface.player.eye_height;
                }
            }
        }
        if let Some(m) = &mut mine {
            let events = m.mining.drain_events();
            m.sync(&events);
            audio.mining(&events);
            let cues = m.game.drain_cues();
            visits.record_cues(&cues);
            reporter.observe(&m.game, observer)?;
            audio.cues(&cues);
        }
        let state = if in_mine {
            Some(mine.as_ref().unwrap().game.state)
        } else {
            None
        };
        let captured = if in_mine {
            state == Some(RushState::Running)
        } else {
            surface.state == SurfaceState::Exploring && village_menu == VillageMenu::Closed
        };
        if old_mine != in_mine
            || old_state != state && in_mine
            || old_surface != surface.state && !in_mine
            || old_village_menu != village_menu
        {
            cursor.captured = captured;
            cursor.apply(&mut rl);
            last = Instant::now();
        }
        let after = if in_mine {
            mine.as_ref().unwrap().scene.player.feet
        } else {
            surface.player.feet
        };
        audio.update(
            dt,
            (input.forward != 0.0 || input.right != 0.0)
                && (after - before).length() > 0.005
                && cursor.captured
                && focused
                && !skip
                && !returned,
            input.sprint,
            state,
            in_mine && mine.as_ref().unwrap().game.warning,
        );
        if test && !smoke.done() && !exploration_smoke {
            smoke.after(frame, &surface, mine.as_ref(), in_mine, &cursor, &audio)?;
        }
        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        {
            let mut d = rl.begin_drawing(&thread);
            d.clear_background(if in_mine {
                Color::new(6, 9, 13, 255)
            } else {
                Color::new(158, 201, 240, 255)
            });
            if in_mine {
                let m = mine.as_mut().unwrap();
                let camera = m.scene.player.camera();
                {
                    let mut view = d.begin_mode3D(camera);
                    m.model.draw(&mut view, camera.position);
                    visuals.draw_cracks(&mut view, &m.mining);
                }
                if m.game.state == RushState::Running {
                    visuals.draw_pickaxe(&mut d, w, h, &m.mining);
                    visuals.draw_hud(&mut d, w, h, &m.mining);
                }
                draw_header(&mut d, w, "DIAMOND RUSH | MINA AMPLIADA");
                let time = m.game.clock();
                let color = if m.game.warning {
                    Color::new(255, 108, 83, 255)
                } else {
                    Color::WHITE
                };
                d.draw_text(&time, w / 2 - 45, 15, 32, color);
                let objective = if m.game.objective_met {
                    "¡Regresá a la salida!".to_string()
                } else {
                    format!("Recolectá {} diamantes", m.game.config.required_diamonds)
                };
                d.draw_text(&objective, 20, 125, 22, Color::new(121, 232, 225, 255));
                if let Some((text, _)) = &m.game.notice {
                    d.draw_text(text, 20, 158, 19, Color::YELLOW);
                }
                if m.game.objective_met && m.game.state == RushState::Running {
                    let delta = m.scene.targets[0].bounds.center() - m.scene.player.eye();
                    let mut angle = delta.z.atan2(delta.x) - m.scene.player.yaw;
                    angle = (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
                        - std::f32::consts::PI;
                    let direction = if angle.abs() < 0.35 {
                        "al frente"
                    } else if angle > 0.0 {
                        "a la derecha"
                    } else {
                        "a la izquierda"
                    };
                    let text = format!("Salida {} - {} m", direction, delta.length() as i32);
                    d.draw_text(&text, w / 2 - 150, 80, 18, Color::new(138, 226, 213, 255));
                }
                if matches!(
                    m.game.state,
                    RushState::Paused | RushState::Victory | RushState::Defeat
                ) {
                    if abandonment.pending {
                        panel(
                            &mut d,
                            w,
                            h,
                            "ABANDONAR DIAMOND RUSH",
                            &[
                                "Perderás el progreso de la partida actual.".into(),
                                "¿Querés volver a la aldea?".into(),
                            ],
                            &["Abandonar", "Cancelar"],
                        );
                    } else {
                        draw_result(&mut d, w, h, m);
                    }
                }
            } else {
                let camera = surface.player.camera();
                {
                    let mut view = d.begin_mode3D(camera);
                    surface_model.draw(&mut view, camera.position);
                }
                draw_header(&mut d, w, "MINECRAFT | OVERWORLD");
                d.draw_text(
                    "WASD / Mouse / Shift   E: interactuar   Esc: pausa / salir",
                    20,
                    h - 32,
                    18,
                    Color::WHITE,
                );
                if let Some(i) = surface.selected {
                    d.draw_text(
                        &format!("[E] {}", surface.targets[i].label),
                        w / 2 - 110,
                        h / 2 + 28,
                        22,
                        Color::YELLOW,
                    );
                }
                if surface.state == SurfaceState::MineMenu {
                    draw_surface_menu(&mut d, w, h, settings.required_diamonds);
                }
                if let Some(text) = &surface.confirmation {
                    d.draw_text(text, 20, 110, 20, Color::YELLOW);
                }
                if !cursor.captured
                    && surface.state == SurfaceState::Exploring
                    && village_menu == VillageMenu::Closed
                {
                    d.draw_text("Clic para retomar | Esc: pausa", 20, 85, 20, Color::WHITE);
                }
                if village_menu != VillageMenu::Closed {
                    draw_village_menu(&mut d, w, h, village_menu);
                }
            }
            if cursor.captured {
                d.draw_line(w / 2 - 5, h / 2, w / 2 + 5, h / 2, Color::WHITE);
                d.draw_line(w / 2, h / 2 - 5, w / 2, h / 2 + 5, Color::WHITE);
            }
            if test {
                if let Some(file) = smoke.capture() {
                    std::fs::create_dir_all(&output)?;
                    save_frame(&output.join(file), w, h)?;
                }
                if host_smoke && matches!(host_tick, 2 | 4 | 7) {
                    std::fs::create_dir_all(&output)?;
                    save_frame(&output.join(format!("host-menu-{host_tick}.png")), w, h)?;
                }
            } else if d.is_key_pressed(KeyboardKey::KEY_F12) {
                std::fs::create_dir_all(&output)?;
                save_frame(&output.join("manual.png"), w, h)?;
            }
        }
        frame += 1;
        if test && smoke.done() && !host_smoke {
            break;
        }
        if test && frame > 5000 {
            return Err("Rush smoke timed out".into());
        }
    }
    rl.enable_cursor();
    audio.stop();
    if test && !exploration_smoke {
        smoke.finish(&output, &audio, mine.as_ref())?;
    }
    println!("MINECRAFT_AUDIO {}", serde_json::to_string(&audio.audit)?);
    if host_smoke {
        let expected = if exploration_smoke {
            MinigameExit::ReturnedWithoutResult
        } else {
            MinigameExit::Returned(MinigameResult::Victory)
        };
        if exit != expected
            || visits.active_attempt().is_some()
            || visits.completed().len() != visit_start + usize::from(!exploration_smoke)
        {
            return Err(
                "Minecraft host smoke: incorrect or duplicated result on village exit".into(),
            );
        }
        println!("MINECRAFT_HOST_PASS exit={exit:?} village_warning=false attempts_total={} audio_stopped={}", visits.completed().len(), !audio.music_playing());
    }
    Ok(exit)
}

fn village_input(rl: &RaylibHandle, menu: VillageMenu) -> VillageAction {
    let b = buttons(rl.get_screen_width(), rl.get_screen_height(), 2);
    let click = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let mouse = rl.get_mouse_position();
    let first = click && b[0].check_collision_point_rec(mouse);
    let second = click && b[1].check_collision_point_rec(mouse);
    match menu {
        VillageMenu::Paused
            if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
                || rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                || first =>
        {
            VillageAction::Continue
        }
        VillageMenu::Paused if rl.is_key_pressed(KeyboardKey::KEY_R) || second => {
            VillageAction::RequestReturn
        }
        _ => VillageAction::None,
    }
}

fn draw_village_menu(d: &mut RaylibDrawHandle, w: i32, h: i32, menu: VillageMenu) {
    if menu == VillageMenu::Paused {
        panel(
            d,
            w,
            h,
            "PAUSA | OVERWORLD",
            &["La exploración está en pausa.".into()],
            &["Continuar", "Regresar al sistema solar"],
        );
    }
}
fn draw_header(d: &mut RaylibDrawHandle, w: i32, title: &str) {
    d.draw_rectangle(0, 0, w, 62, Color::new(8, 16, 20, 235));
    d.draw_text(title, 20, 16, 23, Color::new(132, 238, 225, 255));
}
pub(super) fn buttons(w: i32, h: i32, count: usize) -> Vec<Rectangle> {
    let width = 680.0;
    let x = (w as f32 - width) * 0.5;
    (0..count)
        .map(|i| {
            Rectangle::new(
                x + 20.0 + i as f32 * (640.0 / count as f32),
                h as f32 * 0.5 + 95.0,
                620.0 / count as f32,
                46.0,
            )
        })
        .collect()
}
fn two_buttons_input(rl: &RaylibHandle, surface: bool) -> u8 {
    let b = buttons(rl.get_screen_width(), rl.get_screen_height(), 2);
    let p = rl.get_mouse_position();
    let click = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    if rl.is_key_pressed(KeyboardKey::KEY_ENTER) || click && b[0].check_collision_point_rec(p) {
        1
    } else if surface && rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
        || click && b[1].check_collision_point_rec(p)
    {
        2
    } else {
        0
    }
}
fn pause_input(rl: &RaylibHandle) -> u8 {
    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) || rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
        return 1;
    }
    let b = buttons(rl.get_screen_width(), rl.get_screen_height(), 3);
    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        for (i, r) in b.iter().enumerate() {
            if r.check_collision_point_rec(rl.get_mouse_position()) {
                return i as u8 + 1;
            }
        }
    }
    0
}
fn panel(d: &mut RaylibDrawHandle, w: i32, h: i32, title: &str, lines: &[String], labels: &[&str]) {
    d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 160));
    let x = (w - 760) / 2;
    let y = h / 2 - 155;
    d.draw_rectangle(x, y, 760, 330, Color::new(20, 36, 38, 255));
    d.draw_text(title, x + 24, y + 25, 29, Color::new(132, 238, 225, 255));
    for (i, line) in lines.iter().enumerate() {
        d.draw_text(line, x + 24, y + 90 + i as i32 * 31, 20, Color::WHITE);
    }
    for (r, label) in buttons(w, h, labels.len()).into_iter().zip(labels) {
        d.draw_rectangle_rec(r, Color::new(52, 96, 78, 255));
        d.draw_text(label, r.x as i32 + 12, r.y as i32 + 14, 19, Color::WHITE);
    }
}
fn draw_surface_menu(d: &mut RaylibDrawHandle, w: i32, h: i32, required: u32) {
    panel(
        d,
        w,
        h,
        "DIAMOND RUSH",
        &[
            format!("Recolectá {} diamantes y regresá a la salida", required),
            "antes de que termine el tiempo.".into(),
        ],
        &["Iniciar desafío", "Cancelar"],
    );
}
fn draw_result(d: &mut RaylibDrawHandle, w: i32, h: i32, m: &MineRun) {
    match m.game.state {
        RushState::Victory => panel(
            d,
            w,
            h,
            "¡DIAMOND RUSH COMPLETADO!",
            &[
                format!(
                    "Diamantes: {} / {}",
                    m.mining.diamonds, m.game.config.required_diamonds
                ),
                format!("Tiempo restante: {}", m.game.clock()),
                "¡Buen trabajo! Lograste volver a la superficie.".into(),
            ],
            &["Jugar otra vez", "Regresar a la aldea"],
        ),
        RushState::Defeat => panel(
            d,
            w,
            h,
            "¡SE ACABÓ EL TIEMPO!",
            &[
                format!(
                    "Diamantes obtenidos: {} / {}",
                    m.mining.diamonds, m.game.config.required_diamonds
                ),
                "Explorá otra ruta y volvé antes del límite.".into(),
            ],
            &["Reintentar", "Regresar a la aldea"],
        ),
        _ => panel(
            d,
            w,
            h,
            "PAUSA",
            &["El temporizador y la minería están detenidos.".into()],
            &["Continuar", "Reiniciar", "Aldea"],
        ),
    }
}
