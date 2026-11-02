//! Hidden-window development verification, using the host's real input/UI loop.
use crate::{
    graphics::resources::StaticModel,
    minigames::{
        common::fps::CursorCapture,
        minecraft::{MinecraftScene, SurfacePose, SurfaceState},
    },
};
use raylib::{ffi, prelude::*};
use std::path::Path;

pub const LAST_FRAME: u32 = 135;
#[derive(Default)]
pub struct Smoke {
    initial: Option<SurfacePose>,
    modal: Option<SurfacePose>,
    checks: Vec<&'static str>,
}
fn key(code: KeyboardKey, down: bool) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: if down { 2 } else { 1 },
            params: [code as i32, 0, 0, 0],
        });
    }
}
fn mouse(rl: &RaylibHandle) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: 7,
            params: [
                rl.get_screen_width() / 2 + 100,
                rl.get_screen_height() / 2 - 30,
                0,
                0,
            ],
        });
    }
}
fn click_button(rl: &RaylibHandle, index: usize) {
    let rect = super::buttons(rl.get_screen_width(), rl.get_screen_height())[index];
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: 7,
            params: [
                (rect.x + rect.width * 0.5) as i32,
                (rect.y + rect.height * 0.5) as i32,
                0,
                0,
            ],
        });
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: 6,
            params: [0, 0, 0, 0],
        });
    }
}
fn release_click() {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: 5,
            params: [0, 0, 0, 0],
        });
    }
}
fn pose(scene: &mut MinecraftScene, feet: Vector3, yaw: f32, pitch: f32) {
    scene.player.feet = feet;
    scene.player.yaw = yaw;
    scene.player.pitch = pitch;
}
fn mine_pose(scene: &mut MinecraftScene) {
    pose(
        scene,
        Vector3::new(8.5, 0.0, -22.5),
        -std::f32::consts::FRAC_PI_2,
        0.0,
    );
}
fn same(a: SurfacePose, b: SurfacePose) -> bool {
    (a.feet - b.feet).length() < 0.0001
        && (a.yaw - b.yaw).abs() < 0.0001
        && (a.pitch - b.pitch).abs() < 0.0001
}
impl Smoke {
    pub fn before_frame(&mut self, f: u32, s: &mut MinecraftScene, rl: &RaylibHandle) {
        match f {
            0 => self.initial = Some(s.pose()),
            1 => {
                key(KeyboardKey::KEY_W, true);
                key(KeyboardKey::KEY_LEFT_SHIFT, true);
            }
            10 => mouse(rl),
            20 => {
                key(KeyboardKey::KEY_W, false);
                key(KeyboardKey::KEY_LEFT_SHIFT, false);
            }
            22 | 120 | 130 => mine_pose(s),
            24 | 35 | 127 => key(KeyboardKey::KEY_E, true),
            26 => {
                key(KeyboardKey::KEY_W, true);
                mouse(rl);
            }
            30 => click_button(rl, 1),
            31 => release_click(),
            39 | 43 | 45 => key(KeyboardKey::KEY_ESCAPE, true),
            40 | 44 | 46 => key(KeyboardKey::KEY_ESCAPE, false),
            32 => {
                key(KeyboardKey::KEY_E, false);
                key(KeyboardKey::KEY_W, false);
            }
            36 | 128 => key(KeyboardKey::KEY_E, false),
            37 => click_button(rl, 0),
            38 => release_click(),
            48 => pose(
                s,
                Vector3::new(15.5, 0.0, 15.0),
                -std::f32::consts::FRAC_PI_2,
                0.0,
            ),
            49 | 89 => {
                key(KeyboardKey::KEY_W, true);
                key(KeyboardKey::KEY_LEFT_SHIFT, true);
            }
            86 | 117 => {
                key(KeyboardKey::KEY_W, false);
                key(KeyboardKey::KEY_LEFT_SHIFT, false);
            }
            88 => pose(s, Vector3::new(21.0, 0.0, -0.5), 0.0, 0.0),
            125 => pose(
                s,
                Vector3::new(3.5, 0.0, -23.0),
                -std::f32::consts::FRAC_PI_2,
                -0.4,
            ),
            _ => {}
        }
    }
    pub fn after_frame(
        &mut self,
        f: u32,
        s: &MinecraftScene,
        c: &CursorCapture,
        rl: &RaylibHandle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut check = |ok: bool, name: &'static str| -> Result<(), Box<dyn std::error::Error>> {
            if ok {
                self.checks.push(name);
                Ok(())
            } else {
                Err(format!("Minecraft smoke frame {f}: {name}").into())
            }
        };
        match f {
            20 => {
                let initial = self.initial.unwrap();
                check(
                    (s.player.feet - initial.feet).length() > 1.0,
                    "WASD and Shift move the grounded FPS player",
                )?;
                check(
                    (s.player.yaw - initial.yaw).abs() > 0.01,
                    "Mouse changes camera orientation through real input",
                )?;
            }
            24 => {
                check(
                    s.state == SurfaceState::MineMenu,
                    "E raycast opens the mine menu",
                )?;
                check(
                    !c.captured && !rl.is_cursor_hidden(),
                    "Mine menu releases the native cursor",
                )?;
                self.modal = Some(s.pose());
            }
            29 => check(
                same(self.modal.unwrap(), s.pose()),
                "Modal menu blocks movement and mouse look",
            )?,
            31 => {
                check(
                    s.state == SurfaceState::Exploring,
                    "Cancel closes the menu; held E does not reopen it",
                )?;
                check(
                    c.captured && rl.is_cursor_hidden(),
                    "Cancel restores native cursor capture",
                )?;
            }
            37 => check(
                s.state == SurfaceState::ChallengePending && s.pending_request.is_some(),
                "Start displays pending confirmation and reserves a return pose",
            )?,
            40 => check(
                s.state == SurfaceState::Exploring && s.pending_request.is_none(),
                "Pending confirmation returns to exploration",
            )?,
            43 => check(
                !c.captured && !rl.is_cursor_hidden(),
                "Escape releases cursor during exploration",
            )?,
            45 => check(
                c.captured && rl.is_cursor_hidden(),
                "Escape recaptures cursor during exploration",
            )?,
            86 => check(
                s.player.feet.z >= 13.349 && s.player.feet.z < 13.36,
                "Sprinting collides with the actual house",
            )?,
            117 => check(
                s.player.feet.x <= 22.651 && s.player.feet.x > 22.64,
                "Sprinting collides with the actual oak trunk",
            )?,
            122 => check(
                s.selected == Some(0),
                "Mine entrance remains targetable after exploration",
            )?,
            128 => check(
                s.confirmation.is_some(),
                "Existing crafting table supports an occluded/ranged inspection ray",
            )?,
            _ => {}
        }
        if !s.world.can_occupy(
            s.player.feet,
            s.player.config.radius,
            s.player.config.body_height,
        ) || s.player.feet.y.abs() > 0.001
        {
            return Err(format!("Minecraft smoke: invalid player body at frame {f}").into());
        }
        Ok(())
    }
    pub fn finish(
        &self,
        dir: &Path,
        model: &StaticModel,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pixels = model
            .pixel_material_audit()
            .ok_or("Missing pixel render audit")?;
        if pixels["point_filtered_textures"] != 27
            || pixels["alpha_mask_materials"] != 4
            || model.audit.triangles != 39176
        {
            return Err("Minecraft model render audit differs from phase 1".into());
        }
        let report = serde_json::json!({"status":"passed","frames":LAST_FRAME+1,"checks":self.checks,
            "model":model.audit,"pixel_materials":pixels,
            "method":"Hidden OpenGL window; real PlayerInput, automation events, UI loop, native cursor and renderer",
            "limitations":["No human keyboard/mouse playtest; injected input and scene fixtures exercise the runtime.",
                            "No mining, underground scene, timer or solar integration."]});
        std::fs::create_dir_all(dir)?;
        std::fs::write(
            dir.join("runtime-validation.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!("MINECRAFT_SMOKE {}", report);
        Ok(())
    }
}
pub fn capture_name(frame: u32) -> Option<&'static str> {
    match frame {
        0 => Some("village.png"),
        25 => Some("mine-menu.png"),
        38 => Some("challenge-pending.png"),
        86 => Some("house-collision.png"),
        117 => Some("tree-collision.png"),
        122 => Some("mine.png"),
        128 => Some("crafting-table.png"),
        _ => None,
    }
}
