//! Runtime evidence for the development mine, without extracting any block.
use crate::{
    graphics::resources::StaticModel,
    minigames::{common::fps::CursorCapture, minecraft::mine::MineScene},
};
use raylib::{ffi, prelude::*};
use std::path::Path;
pub const LAST_FRAME: u32 = 129;
#[derive(Default)]
pub struct Smoke {
    checks: Vec<&'static str>,
    modal: Option<(Vector3, f32, f32)>,
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
fn pose(s: &mut MineScene, p: Vector3, yaw: f32, pitch: f32) {
    s.player.feet = p;
    s.player.yaw = yaw;
    s.player.pitch = pitch;
}
fn mouse(rl: &RaylibHandle) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: 7,
            params: [
                rl.get_screen_width() / 2 + 100,
                rl.get_screen_height() / 2 - 20,
                0,
                0,
            ],
        });
    }
}
impl Smoke {
    pub fn before(&mut self, f: u32, s: &mut MineScene, rl: &RaylibHandle) {
        match f {
            1 | 46 | 86 => {
                key(KeyboardKey::KEY_W, true);
                key(KeyboardKey::KEY_LEFT_SHIFT, true);
            }
            10 => mouse(rl),
            20 | 81 | 106 => {
                key(KeyboardKey::KEY_W, false);
                key(KeyboardKey::KEY_LEFT_SHIFT, false);
            }
            24 => pose(s, Vector3::new(0.0, 0.0, -31.0), 0.0, 0.0),
            30 => pose(
                s,
                Vector3::new(0.0, 0.0, -48.0),
                (-6.0_f32).atan2(-2.0),
                -0.1,
            ),
            35 => pose(s, Vector3::new(23.5, 0.0, -48.5), 0.0, -0.2),
            38 => pose(s, Vector3::new(21.5, 0.0, -48.5), 0.0, -0.08),
            36 | 113 | 127 => key(KeyboardKey::KEY_E, true),
            37 | 128 => key(KeyboardKey::KEY_E, false),
            45 => pose(s, Vector3::new(3.5, 0.0, -1.0), 0.0, 0.0),
            85 => pose(s, Vector3::new(-7.0, 0.0, -32.5), std::f32::consts::PI, 0.0),
            110 => pose(s, Vector3::zero(), std::f32::consts::FRAC_PI_2, 0.0),
            114 => {
                key(KeyboardKey::KEY_W, true);
                mouse(rl);
            }
            117 | 121 | 123 => key(KeyboardKey::KEY_ESCAPE, true),
            118 => {
                key(KeyboardKey::KEY_E, false);
                key(KeyboardKey::KEY_W, false);
                key(KeyboardKey::KEY_ESCAPE, false);
            }
            122 | 124 => key(KeyboardKey::KEY_ESCAPE, false),
            129 => key(KeyboardKey::KEY_ENTER, true),
            _ => {}
        }
    }
    pub fn after(
        &mut self,
        f: u32,
        s: &MineScene,
        c: &CursorCapture,
        rl: &RaylibHandle,
        menu: bool,
        returning: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut check = |ok: bool, name: &'static str| -> Result<(), Box<dyn std::error::Error>> {
            if ok {
                self.checks.push(name);
                Ok(())
            } else {
                Err(format!("Mine smoke {f}: {name}").into())
            }
        };
        match f {
            20 => check(
                s.player.feet.length() > 1.0 && s.player.pitch.abs() > 0.001,
                "WASD, sprint and native mouse input control grounded mine FPS",
            )?,
            36 => check(
                s.selected
                    .is_some_and(|i| s.targets[i].id == "ORE_DIAMOND_11")
                    && s.data.interactive_blocks.len() >= 26,
                "Diamond raycast selects its independent block; E does not extract it",
            )?,
            81 => check(
                (s.player.feet.x - 4.6499).abs() < 0.002,
                "Swept sprint stops at mine wall",
            )?,
            106 => check(
                (s.player.feet.x + 7.6499).abs() < 0.002,
                "Prepared passage blocks remain solid during sprint",
            )?,
            113 => {
                check(
                    menu && !c.captured && !rl.is_cursor_hidden(),
                    "Exit E opens development menu and releases native cursor",
                )?;
                self.modal = Some((s.player.feet, s.player.yaw, s.player.pitch));
            }
            116 => {
                let p = self.modal.unwrap();
                check(
                    s.player.feet == p.0 && s.player.yaw == p.1 && s.player.pitch == p.2,
                    "Exit menu freezes movement and camera",
                )?;
            }
            119 => check(
                !menu && c.captured && rl.is_cursor_hidden(),
                "Cancel resumes mine exploration and native cursor capture",
            )?,
            121 => check(
                !c.captured && !rl.is_cursor_hidden(),
                "Escape releases mine cursor",
            )?,
            123 => check(
                c.captured && rl.is_cursor_hidden(),
                "Escape recaptures mine cursor",
            )?,
            129 => check(
                returning,
                "Development exit requests a normal return to village",
            )?,
            _ => {}
        }
        if !s.grounded() {
            return Err(format!("Mine player penetrated an obstacle at frame {f}").into());
        }
        Ok(())
    }
    pub fn finish(
        &self,
        out: &Path,
        s: &MineScene,
        m: &StaticModel,
        frames: u32,
        returned: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let intact = s.accessibility(false);
        let future = s.accessibility(true);
        if frames != 130
            || !returned
            || intact.safe_reachable_diamonds.len() < 8
            || future.safe_reachable_diamonds.len() != 14
            || m.independent_groups().len() != s.data.interactive_blocks.len()
        {
            return Err("Mine runtime/layout audit incomplete".into());
        }
        let report = serde_json::json!({"status":"passed","frames":frames,"checks":self.checks,"model":m.audit,
            "pixel_materials":m.pixel_material_audit(),"independent_groups":m.independent_groups(),"intact_layout":intact,"future_layout":future,
            "lights":s.data.lights.len(),"lava_sensors":s.data.lava_regions.len(),"note":"Real hidden OpenGL window, raylib automation and pose fixtures; no hardware playtest or mining."});
        std::fs::create_dir_all(out)?;
        std::fs::write(
            out.join("runtime-validation.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("MINE_SMOKE {}", report);
        Ok(())
    }
}
pub fn capture(f: u32) -> Option<&'static str> {
    match f {
        0 => Some("runtime_entrance.png"),
        25 => Some("runtime_galleries.png"),
        31 => Some("runtime_depths.png"),
        38 => Some("runtime_diamonds.png"),
        106 => Some("runtime_gate.png"),
        114 => Some("runtime_exit.png"),
        _ => None,
    }
}
