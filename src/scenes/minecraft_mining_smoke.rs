//! Real input/render loop checks. Poses are test fixtures; no production mining shortcuts.
use crate::{
    graphics::resources::StaticModel,
    minigames::{
        common::manifest::from_blender,
        minecraft::{
            mine::MineScene,
            mining::{BlockKind, MiningEvent, MiningSession},
        },
    },
};
use raylib::{ffi, prelude::*};
use std::path::Path;
pub struct Smoke {
    phase: u8,
    tick: u32,
    tasks: Vec<String>,
    task: usize,
    checks: Vec<&'static str>,
    events: Vec<MiningEvent>,
    capture_file: Option<&'static str>,
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
fn aim(scene: &mut MineScene, id: &str) {
    let b = scene
        .data
        .interactive_blocks
        .iter()
        .find(|b| b.id == id)
        .unwrap();
    let target = from_blender([
        (b.min[0] + b.max[0]) * 0.5,
        (b.min[1] + b.max[1]) * 0.5,
        (b.min[2] + b.max[2]) * 0.5,
    ]);
    let mut feet = if let Some(p) = b.approach {
        from_blender(p)
    } else {
        from_blender([b.max[0] + 1.0, b.min[1] + 0.5, 0.0])
    };
    if b.approach.is_some() {
        let horizontal = Vector3::new(target.x - feet.x, 0.0, target.z - feet.z).normalized();
        let farther = feet - horizontal * 1.5;
        if scene.world.can_occupy(farther, 0.35, 1.8) {
            feet = farther;
        }
    }
    scene.player.feet = feet;
    let d = (target - scene.player.eye()).normalized();
    scene.player.yaw = d.z.atan2(d.x);
    scene.player.pitch = d.y.asin();
}
impl Smoke {
    pub fn new(s: &MineScene) -> Self {
        let mut tasks = vec![];
        for kind in ["coal", "iron", "lapis", "redstone", "gold"] {
            tasks.push(
                s.data
                    .interactive_blocks
                    .iter()
                    .find(|b| b.kind == kind && b.occluded_by.is_empty())
                    .unwrap()
                    .id
                    .clone(),
            );
        }
        tasks.extend(
            s.accessibility(false)
                .safe_reachable_diamonds
                .into_iter()
                .filter(|id| id != "ORE_DIAMOND_11")
                .take(8),
        );
        tasks.extend(
            s.data
                .interactive_blocks
                .iter()
                .filter(|b| b.kind == "breakable")
                .map(|b| b.id.clone()),
        );
        Self {
            phase: 0,
            tick: 0,
            tasks,
            task: 0,
            checks: vec![],
            events: vec![],
            capture_file: None,
        }
    }
    pub fn before(&mut self, frame: u32, s: &mut MineScene, _: &MiningSession, rl: &RaylibHandle) {
        self.capture_file = None;
        if self.phase == 0 {
            match frame {
                0 => self.capture_file = Some("pickaxe_idle.png"),
                4 => aim(s, "ORE_DIAMOND_11"),
                6 | 60 => click(true),
                30 => self.capture_file = Some("pickaxe_swing.png"),
                55 | 171 => click(false),
                100 => self.capture_file = Some("cracks.png"),
                169 => self.capture_file = Some("diamond_collected.png"),
                180 => {
                    self.phase = 1;
                    self.tick = 0;
                }
                _ => {}
            }
        }
        match self.phase {
            1 => {
                if self.tick == 0 {
                    click(false);
                    aim(s, &self.tasks[self.task]);
                }
                if self.tick == 2 {
                    click(true);
                }
            }
            2 => match self.tick {
                0 => {
                    click(false);
                    s.player.feet = Vector3::new(-7.0, 0.0, -32.5);
                    s.player.yaw = std::f32::consts::PI;
                    s.player.pitch = 0.0;
                }
                2 => {
                    key(KeyboardKey::KEY_W, true);
                    key(KeyboardKey::KEY_LEFT_SHIFT, true);
                }
                41 => {
                    key(KeyboardKey::KEY_W, false);
                    key(KeyboardKey::KEY_LEFT_SHIFT, false);
                    self.capture_file = Some("passage_open.png");
                }
                _ => {}
            },
            3 => match self.tick {
                0 => aim(s, "ORE_DIAMOND_12"),
                2 => click(true),
                16 => key(KeyboardKey::KEY_BACKSPACE, true),
                17 => {
                    key(KeyboardKey::KEY_BACKSPACE, false);
                    self.capture_file = Some("mining_cancelled_menu.png");
                }
                18 => click(false),
                20 => {
                    let r = super::buttons(rl.get_screen_width(), rl.get_screen_height())[1];
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
                26 => click(false),
                _ => {}
            },
            4 => match self.tick {
                0 => key(KeyboardKey::KEY_R, true),
                1 => key(KeyboardKey::KEY_R, false),
                3 => self.capture_file = Some("reset.png"),
                _ => {}
            },
            5 => match self.tick {
                0 => key(KeyboardKey::KEY_BACKSPACE, true),
                1 => key(KeyboardKey::KEY_BACKSPACE, false),
                2 => key(KeyboardKey::KEY_ENTER, true),
                _ => {}
            },
            _ => {}
        }
    }
    pub fn after(
        &mut self,
        frame: u32,
        s: &MineScene,
        m: &MiningSession,
        model: &StaticModel,
        menu: bool,
        returning: bool,
        events: &[MiningEvent],
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.events.extend_from_slice(events);
        for event in events {
            if let MiningEvent::BlockDestroyed { block, .. } = event {
                if model.group_visible(block) != Some(false)
                    || s.world.obstacles.iter().any(|c| c.id == *block)
                    || s.targets.iter().any(|t| t.id == *block)
                {
                    return Err(format!("Destruction incomplete: {block}").into());
                }
            }
        }
        if !s.grounded() {
            return Err(format!("Mining smoke penetrated geometry at {frame}").into());
        }
        let mut check = |ok: bool, msg: &'static str| -> Result<(), Box<dyn std::error::Error>> {
            if ok {
                self.checks.push(msg);
                Ok(())
            } else {
                Err(format!("Mining smoke frame {frame}: {msg}").into())
            }
        };
        if self.phase == 0 {
            match frame {
                30 => check(
                    m.active.is_some() && m.crack_stage() > 0,
                    "Held native left mouse advances mining, cracks and pickaxe swing",
                )?,
                56 => check(
                    m.active.is_none() && m.crack_stage() == 0,
                    "Mouse release removes all progress and cracks",
                )?,
                169 => check(
                    m.diamonds == 1
                        && !m
                            .blocks
                            .iter()
                            .find(|b| b.id == "ORE_DIAMOND_11")
                            .unwrap()
                            .alive,
                    "Diamond disappears, removes collider and updates HUD exactly once",
                )?,
                _ => {}
            }
            return Ok(());
        }
        match self.phase {
            1 => {
                let alive = m
                    .blocks
                    .iter()
                    .find(|b| b.id == self.tasks[self.task])
                    .unwrap()
                    .alive;
                if !alive {
                    self.task += 1;
                    self.tick = 0;
                    if self.task == self.tasks.len() {
                        check(
                            m.diamonds == 9,
                            "Nine diamonds can be collected without triggering victory",
                        )?;
                        check(
                            self.events
                                .iter()
                                .filter(|e| matches!(e, MiningEvent::CommonOreCollected { .. }))
                                .count()
                                == 5,
                            "All five common minerals produce feedback without awarding diamonds",
                        )?;
                        check(
                            self.events
                                .iter()
                                .filter(|e| matches!(e, MiningEvent::PassageUnlocked { .. }))
                                .count()
                                == 2,
                            "Both passages emit a physical-clearance unlock event",
                        )?;
                        self.capture_file = Some("diamonds_nine.png");
                        self.phase = 2;
                    }
                    return Ok(());
                }
                if self.tick > 160 {
                    return Err(format!("Could not mine {}", self.tasks[self.task]).into());
                }
            }
            2 => {
                if self.tick == 41 {
                    check(
                        s.player.feet.x < -10.0,
                        "Player walks through the genuinely cleared gate",
                    )?;
                    self.phase = 3;
                    self.tick = 0;
                    return Ok(());
                }
            }
            3 => {
                if self.tick == 17 {
                    check(
                        menu && m.active.is_none(),
                        "Opening exit menu cancels mining immediately",
                    )?;
                }
                if self.tick == 25 {
                    check(
                        !menu && m.active.is_none(),
                        "A UI click held after recapture cannot strike a block behind the menu",
                    )?;
                }
                if self.tick == 27 {
                    self.phase = 4;
                    self.tick = 0;
                    return Ok(());
                }
            }
            4 => {
                if self.tick == 3 {
                    check(m.diamonds==0&&m.active.is_none()&&m.blocks.iter().all(|b|b.alive&&model.group_visible(&b.id)==Some(true)&&s.world.obstacles.iter().any(|c|c.id==b.id)),"R restores every visual, collider, target and session counter without restarting the app")?;
                    self.phase = 5;
                    self.tick = 0;
                    return Ok(());
                }
            }
            5 => {
                if self.tick == 2 {
                    check(
                        returning,
                        "Development exit returns normally to the Overworld",
                    )?;
                }
            }
            _ => {}
        }
        self.tick += 1;
        Ok(())
    }
    pub fn capture(&mut self, _: u32, _: &MiningSession) -> Option<&'static str> {
        self.capture_file.take()
    }
    pub fn finish(
        &self,
        out: &Path,
        s: &MineScene,
        m: &MiningSession,
        model: &StaticModel,
        frames: u32,
        returned: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !returned
            || self.checks.len() < 11
            || m.diamonds != 0
            || m.blocks.iter().any(|b| !b.alive)
        {
            return Err("Mining runtime verification incomplete".into());
        }
        let report = serde_json::json!({"status":"passed","frames":frames,"checks":self.checks,"model":model.audit,"extractable_blocks":m.blocks.len(),
            "events":self.events,"restored_diamonds":m.blocks.iter().filter(|b|b.kind==BlockKind::Diamond).count(),
            "restored_colliders":s.world.obstacles.len(),"manual_hardware_playtest":false});
        std::fs::create_dir_all(out)?;
        std::fs::write(
            out.join("mining-runtime.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("MINING_SMOKE_PASS {}", report);
        Ok(())
    }
}
