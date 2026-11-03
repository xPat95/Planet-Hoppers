//! Native-input state and audio regression; camera positions/time presets are test fixtures.
use super::{buttons, MineRun};
use crate::minigames::{
    common::{fps::CursorCapture, manifest::from_blender},
    minecraft::{rush::RushState, sound::MinecraftAudio, MinecraftScene, SurfaceState},
};
use raylib::{ffi, prelude::*};
use std::path::Path;
#[derive(Default)]
pub struct Smoke {
    phase: u8,
    tick: u32,
    index: usize,
    targets: Vec<String>,
    checks: Vec<&'static str>,
    file: Option<&'static str>,
    paused_time: f32,
    done: bool,
    victory_return: bool,
}
fn key(k: KeyboardKey, down: bool) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: if down { 2 } else { 1 },
            params: [k as i32, 0, 0, 0],
        });
    }
}
fn click(down: bool) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: if down { 6 } else { 5 },
            params: [0, 0, 0, 0],
        });
    }
}
fn aim(m: &mut MineRun, id: &str) {
    let b = m
        .scene
        .data
        .interactive_blocks
        .iter()
        .find(|b| b.id == id)
        .unwrap();
    m.scene.player.feet = from_blender(b.approach.unwrap());
    let center = (from_blender(b.min) + from_blender(b.max)) * 0.5;
    let d = (center - m.scene.player.eye()).normalized();
    m.scene.player.yaw = d.z.atan2(d.x);
    m.scene.player.pitch = d.y.asin();
}
impl Smoke {
    pub fn winning_return() -> Self {
        Self {
            victory_return: true,
            ..Self::default()
        }
    }
    pub fn before(
        &mut self,
        frame: u32,
        s: &mut MinecraftScene,
        m: Option<&mut MineRun>,
        _: bool,
        rl: &RaylibHandle,
    ) {
        self.file = None;
        if self.phase == 0 {
            match frame {
                0 => {
                    key(KeyboardKey::KEY_W, true);
                    self.file = Some("runtime_surface.png");
                }
                2 => {
                    key(KeyboardKey::KEY_W, false);
                    s.player.feet = Vector3::new(8.5, 0.0, -23.0);
                    s.player.yaw = -std::f32::consts::FRAC_PI_2;
                    s.player.pitch = 0.0;
                    key(KeyboardKey::KEY_E, true);
                }
                3 => key(KeyboardKey::KEY_E, false),
                5 => self.file = Some("runtime_start_menu.png"),
                7 => key(KeyboardKey::KEY_ENTER, true),
                8 => key(KeyboardKey::KEY_ENTER, false),
                _ => {}
            }
            if let Some(m) = m {
                match frame {
                    10 => {
                        m.scene.player.feet = Vector3::zero();
                        m.scene.player.yaw = std::f32::consts::FRAC_PI_2;
                    }
                    12 => key(KeyboardKey::KEY_E, true),
                    13 => key(KeyboardKey::KEY_E, false),
                    14 => self.file = Some("runtime_need_diamonds.png"),
                    16 => {
                        m.scene.player.feet = Vector3::new(11.0, 0.0, -65.0);
                        m.scene.player.yaw = (-13.0_f32).atan2(13.0);
                        m.scene.player.pitch = 0.2;
                        key(KeyboardKey::KEY_W, true);
                    }
                    17 => {
                        key(KeyboardKey::KEY_W, false);
                        self.file = Some("runtime_cavern.png");
                    }
                    20 => m.scene.player.feet = Vector3::new(-2.5, 0.0, -52.5),
                    22 => self.file = Some("runtime_lava.png"),
                    25 => {
                        key(KeyboardKey::KEY_ESCAPE, true);
                        self.paused_time = m.game.remaining;
                    }
                    26 => key(KeyboardKey::KEY_ESCAPE, false),
                    27 => self.file = Some("runtime_pause.png"),
                    30 => key(KeyboardKey::KEY_ESCAPE, true),
                    31 => key(KeyboardKey::KEY_ESCAPE, false),
                    35 => {
                        self.targets = m
                            .scene
                            .accessibility(false)
                            .safe_reachable_diamonds
                            .into_iter()
                            .take(8)
                            .collect();
                        self.phase = 1;
                        self.tick = 0;
                    }
                    _ => {}
                }
                if self.phase == 1 {
                    click(false);
                    aim(m, &self.targets[0]);
                }
            }
            return;
        }
        let Some(m) = m else {
            return;
        };
        match self.phase {
            1 => {
                if self.tick == 0 {
                    click(false);
                    aim(m, &self.targets[self.index]);
                }
                if self.tick == 2 {
                    click(true);
                }
            }
            2 => match self.tick {
                0 => {
                    click(false);
                    m.game.remaining = 30.02;
                }
                2 => self.file = Some("runtime_low_time.png"),
                3 => {
                    m.scene.player.feet = Vector3::zero();
                    m.scene.player.yaw = std::f32::consts::FRAC_PI_2;
                    m.scene.player.pitch = 0.0;
                    key(KeyboardKey::KEY_E, true);
                }
                4 => {
                    key(KeyboardKey::KEY_E, false);
                    self.file = Some("runtime_victory.png");
                }
                6 if self.victory_return => click_button(rl, 2, 1),
                7 if self.victory_return => click(false),
                6 => key(KeyboardKey::KEY_ENTER, true),
                7 => key(KeyboardKey::KEY_ENTER, false),
                _ => {}
            },
            3 => match self.tick {
                0 => m.game.remaining = 0.02,
                2 => self.file = Some("runtime_defeat.png"),
                4 => key(KeyboardKey::KEY_ENTER, true),
                5 => key(KeyboardKey::KEY_ENTER, false),
                _ => {}
            },
            4 => match self.tick {
                0 => key(KeyboardKey::KEY_BACKSPACE, true),
                1 => key(KeyboardKey::KEY_BACKSPACE, false),
                2 | 10 => click_button(rl, 3, 2),
                3 | 11 => click(false),
                4 => self.file = Some("runtime_abandon_warning.png"),
                5 => key(KeyboardKey::KEY_ESCAPE, true),
                6 => key(KeyboardKey::KEY_ESCAPE, false),
                8 => key(KeyboardKey::KEY_BACKSPACE, true),
                9 => key(KeyboardKey::KEY_BACKSPACE, false),
                12 => key(KeyboardKey::KEY_ENTER, true),
                13 => key(KeyboardKey::KEY_ENTER, false),
                14 => self.file = Some("runtime_return.png"),
                _ => {}
            },
            _ => {}
        }
    }
    pub fn after(
        &mut self,
        frame: u32,
        s: &MinecraftScene,
        m: Option<&MineRun>,
        in_mine: bool,
        c: &CursorCapture,
        a: &MinecraftAudio,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut check = |ok: bool, label: &'static str| -> Result<(), Box<dyn std::error::Error>> {
            if ok {
                self.checks.push(label);
                Ok(())
            } else {
                Err(format!("Rush smoke frame {frame}: {label}").into())
            }
        };
        if self.phase == 0 {
            match frame {
                5 => check(
                    !in_mine && s.state == SurfaceState::MineMenu,
                    "E opens invitation without starting a timer",
                )?,
                7 => {
                    let m = m.unwrap();
                    check(
                        in_mine && m.game.remaining == 150.0,
                        "Loaded scene starts at 02:30 after resources are ready",
                    )?;
                }
                14 => {
                    let m = m.unwrap();
                    check(
                        m.game.state == RushState::Running && m.game.notice.is_some(),
                        "Exit with insufficient diamonds gives feedback and keeps gameplay active",
                    )?;
                }
                21 => {
                    let m = m.unwrap();
                    check(
                        m.game.lava_protection > 0.0
                            && m.scene.player.feet == Vector3::new(0.0, 0.0, -48.0)
                            && m.game.remaining < 140.0,
                        "Lava removes ten seconds and teleports to safe ground once",
                    )?;
                }
                29 => check(
                    m.unwrap().game.remaining == self.paused_time,
                    "Real pause menu freezes countdown",
                )?,
                _ => {}
            }
            return Ok(());
        }
        let m = m.unwrap();
        match self.phase {
            1 => {
                if !m
                    .mining
                    .blocks
                    .iter()
                    .find(|b| b.id == self.targets[self.index])
                    .unwrap()
                    .alive
                {
                    self.index += 1;
                    self.tick = 0;
                    if self.index == 8 {
                        check(m.mining.diamonds==8&&m.game.state==RushState::Running,"Eight actual mined diamonds require a physical return, not automatic victory")?;
                        self.phase = 2;
                    }
                    return Ok(());
                }
                if self.tick > 160 {
                    return Err(format!("Could not extract {}", self.targets[self.index]).into());
                }
            }
            2 => {
                if self.tick == 2 {
                    check(
                        m.game.warning,
                        "Low-time threshold activates visual and one-shot alert",
                    )?;
                }
                if self.tick == 4 {
                    check(m.game.state==RushState::Victory&&!c.captured,"E at actual exit with time and diamonds produces victory and releases cursor")?;
                }
                if self.tick == 7 {
                    if self.victory_return {
                        check(
                            !in_mine && s.state == SurfaceState::Exploring && c.captured,
                            "Victory returns to village without turning into abandonment",
                        )?;
                        self.done = true;
                        return Ok(());
                    }
                    check(
                        m.game.state == RushState::Running
                            && m.game.remaining > 149.9
                            && m.mining.diamonds == 0
                            && m.mining
                                .blocks
                                .iter()
                                .all(|b| b.alive && m.model.group_visible(&b.id) == Some(true)),
                        "Victory retry restores timer, every block, collision state and HUD",
                    )?;
                    self.phase = 3;
                    self.tick = 0;
                    return Ok(());
                }
            }
            3 => {
                if self.tick == 2 {
                    check(
                        m.game.state == RushState::Defeat && m.game.remaining == 0.0 && !c.captured,
                        "Timeout ends gameplay, clamps clock and opens defeat screen",
                    )?;
                }
                if self.tick == 5 {
                    check(
                        m.game.state == RushState::Running && m.mining.diamonds == 0,
                        "Defeat retry starts a fresh attempt without reopening application",
                    )?;
                    self.phase = 4;
                    self.tick = 0;
                    return Ok(());
                }
            }
            4 => {
                if self.tick == 4 {
                    check(
                        in_mine && m.game.state == RushState::Paused && !c.captured,
                        "Active attempt requests abandonment confirmation before leaving mine",
                    )?;
                }
                if self.tick == 6 {
                    check(
                        in_mine && m.game.state == RushState::Running && c.captured,
                        "Cancel abandonment resumes the same active attempt",
                    )?;
                }
                if self.tick == 14 {
                    check(
                        !in_mine && s.state == SurfaceState::Exploring && c.captured,
                        "Return restores surface FPS in the same graphics context",
                    )?;
                    check(
                        a.music_starts == 1 && a.music_playing(),
                        "One looping Minecraft music stream survives all scene changes and retries",
                    )?;
                    check(
                        a.audit.loaded.len() == 9 && a.audit.warnings.is_empty(),
                        "All nine exact audio resources decode successfully",
                    )?;
                    check(
                        a.audit.plays.get("success") == Some(&1)
                            && a.audit.plays.get("victory") == Some(&1)
                            && a.audit.plays.get("game_over") == Some(&1),
                        "Success, victory and defeat sounds each fire once in their attempt",
                    )?;
                    check(["footsteps","pickaxe_hit","block_break","diamond_collect","danger"].iter().all(|k|a.audit.plays.get(*k).is_some_and(|n|*n>0)),"Movement, mining, collection and warning cues play through the real audio device")?;
                    self.done = true;
                }
            }
            _ => {}
        }
        self.tick += 1;
        Ok(())
    }
    pub fn capture(&mut self) -> Option<&'static str> {
        self.file.take()
    }
    pub fn done(&self) -> bool {
        self.done
    }
    pub fn finish(
        &self,
        out: &Path,
        a: &MinecraftAudio,
        m: Option<&MineRun>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.done {
            return Err("Rush smoke ended early".into());
        }
        let report = serde_json::json!({"status":"passed","checks":self.checks,"audio":a.audit,"music_starts":a.music_starts,"model":m.unwrap().model.audit,
            "method":"Real hidden OpenGL/audio; native events and explicit pose/time presets. No manual hardware playtest."});
        std::fs::create_dir_all(out)?;
        std::fs::write(
            out.join("runtime-validation.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("RUSH_SMOKE_PASS {}", report);
        Ok(())
    }
}

fn click_button(rl: &RaylibHandle, count: usize, index: usize) {
    let r = buttons(rl.get_screen_width(), rl.get_screen_height(), count)[index];
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: 7,
            params: [
                (r.x + r.width * 0.5) as i32,
                (r.y + r.height * 0.5) as i32,
                0,
                0,
            ],
        });
    }
    click(true);
}
