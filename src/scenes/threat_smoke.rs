//! Automated graphical smoke, not production navigation. Positions the test
//! player and aims at live targets; E travels through raylib's normal input API.
use crate::minigames::southpark::{
    threats::{ThreatKind, ThreatResult, ThreatState, PREFIX},
    SouthParkScene,
};
use raylib::{ffi, prelude::*};
use serde_json::{json, Value};
pub struct ThreatSmoke {
    case: usize,
    started: bool,
    cooldown: f32,
    pub captured: bool,
    detail_captured: bool,
    presses: usize,
    pub results: Vec<Value>,
}
impl ThreatSmoke {
    pub fn new() -> Self {
        Self {
            case: 0,
            started: false,
            cooldown: 0.0,
            captured: false,
            detail_captured: false,
            presses: 0,
            results: Vec::new(),
        }
    }
    pub fn done(&self) -> bool {
        self.case >= 10
    }
    fn kind(&self) -> ThreatKind {
        ThreatKind::ALL[self.case / 2]
    }
    fn success(&self) -> bool {
        self.case.is_multiple_of(2)
    }
    pub fn before_frame(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        unsafe {
            ffi::PlayAutomationEvent(ffi::AutomationEvent {
                frame: 0,
                type_: 1,
                params: [KeyboardKey::KEY_E as i32, 0, 0, 0],
            });
        }
        if self.done() {
            return Ok(());
        }
        if !self.started {
            scene.player.feet = Vector3::new(-43.2, 0.0, -42.9);
            scene.player.yaw = 0.0;
            scene.player.pitch = 0.0;
            scene.start_threat(self.kind())?;
            self.started = true;
            self.cooldown = 0.0;
            self.captured = false;
            self.detail_captured = false;
            self.presses = 0;
        }
        self.cooldown -= 0.1;
        if !self.success() || scene.threats.state != ThreatState::Active || self.cooldown > 0.0 {
            return Ok(());
        }
        let target = match self.kind() {
            ThreatKind::Piano if scene.threats.elapsed >= 8.0 => {
                scene.threats.targets().into_iter().next()
            }
            ThreatKind::Snowman if scene.threats.elapsed >= 3.0 => {
                scene.threats.targets().into_iter().next()
            }
            ThreatKind::Bus if scene.threats.elapsed >= 8.0 => {
                scene.threats.targets().into_iter().next()
            }
            ThreatKind::Ufo if scene.threats.elapsed >= 3.0 => {
                scene.threats.targets().into_iter().next()
            }
            ThreatKind::Sewer
                if scene.threats.elapsed >= 2.0
                    && (!scene.threats.carrying_lid || scene.threats.elapsed >= 5.0) =>
            {
                scene.threats.targets().into_iter().next()
            }
            _ => None,
        };
        if let Some(target) = target {
            let feet = match self.kind() {
                ThreatKind::Piano | ThreatKind::Snowman => {
                    target.bounds.center() + Vector3::new(-2.5, -target.bounds.center().y, -1.0)
                }
                ThreatKind::Bus => scene.threats.panel_position() + Vector3::new(-2.5, 0.0, -0.8),
                ThreatKind::Ufo => ThreatKind::Ufo.stage() - Vector3::right() * 5.0,
                ThreatKind::Sewer => {
                    if scene.threats.carrying_lid {
                        scene.threats.hole_position() + Vector3::new(-2.0, 0.0, 2.0)
                    } else {
                        scene.threats.cover_position() + Vector3::new(-2.0, 0.0, -0.8)
                    }
                }
            };
            if !scene.world.can_occupy(
                feet,
                scene.player.config.radius,
                scene.player.config.body_height,
            ) {
                return Err("Smoke interaction approach intersects scenery".into());
            }
            scene.player.feet = feet;
            let direction = (target.bounds.center() - scene.player.eye()).normalized();
            scene.player.yaw = direction.z.atan2(direction.x);
            scene.player.pitch = direction.y.asin();
            unsafe {
                ffi::PlayAutomationEvent(ffi::AutomationEvent {
                    frame: 0,
                    type_: 2,
                    params: [KeyboardKey::KEY_E as i32, 0, 0, 0],
                });
            }
            self.presses += 1;
            self.cooldown = 0.6;
        }
        Ok(())
    }
    pub fn capture(&mut self, scene: &SouthParkScene) -> Option<&'static str> {
        if !self.detail_captured {
            let detail = match (self.kind(), self.success(), scene.threats.state) {
                (ThreatKind::Piano, false, ThreatState::Resolving)
                    if scene.threats.state_time >= 0.3 =>
                {
                    Some("piano-fall.png")
                }
                (ThreatKind::Piano, true, ThreatState::Succeeded)
                    if scene.threats.state_time >= 0.3 =>
                {
                    Some("piano-secured.png")
                }
                (ThreatKind::Snowman, true, ThreatState::Resolving)
                    if scene.threats.state_time >= 0.8 =>
                {
                    Some("snowman-melt.png")
                }
                (ThreatKind::Snowman, false, ThreatState::Failed)
                    if scene.threats.state_time >= 0.5 =>
                {
                    Some("snowman-contact.png")
                }
                _ => None,
            };
            if detail.is_some() {
                self.detail_captured = true;
                return detail;
            }
        }
        if !self.success() || self.captured || scene.threats.state != ThreatState::Active {
            return None;
        }
        match self.kind() {
            ThreatKind::Bus if scene.threats.elapsed >= 6.0 => Some("bus.png"),
            ThreatKind::Ufo if scene.threats.elapsed >= 2.0 => Some("ufo.png"),
            ThreatKind::Sewer if scene.threats.carrying_lid => Some("sewer.png"),
            ThreatKind::Piano if scene.threats.elapsed >= 6.0 => Some("piano.png"),
            ThreatKind::Snowman if scene.threats.elapsed >= 2.0 => Some("snowman.png"),
            _ => None,
        }
    }
    pub fn camera(&self, scene: &SouthParkScene) -> Camera3D {
        let (eye, target) = match self.kind() {
            ThreatKind::Piano => (
                scene.threats.piano_impact() + Vector3::new(-28.0, 4.0, -2.5),
                scene.threats.piano_impact() + Vector3::new(-8.0, 3.5, 0.0),
            ),
            ThreatKind::Snowman => (
                scene.threats.snow_position() + Vector3::new(18.0, 3.3, -2.5),
                scene.threats.snow_position() + Vector3::new(5.0, 1.4, 0.0),
            ),
            ThreatKind::Bus => {
                let middle = (scene.threats.bus_position() + scene.kenny.position()) * 0.5;
                (
                    middle + Vector3::new(-2.0, 9.0, 25.0),
                    middle + Vector3::up(),
                )
            }
            ThreatKind::Ufo => (
                ThreatKind::Ufo.stage() + Vector3::new(-8.0, 4.0, 10.0),
                ThreatKind::Ufo.stage() + Vector3::up() * 3.3,
            ),
            ThreatKind::Sewer => (
                Vector3::new(-34.0, 4.5, -41.1),
                Vector3::new(-18.0, 0.2, -42.9),
            ),
        };
        Camera3D::perspective(eye, target, Vector3::up(), 65.0)
    }
    pub fn after_frame(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        let Some(result) = scene.threats.result else {
            return Ok(());
        };
        if result
            != if self.success() {
                ThreatResult::Survived
            } else {
                ThreatResult::Died
            }
        {
            return Err(format!(
                "Unexpected {:?} outcome: {:?}",
                self.kind(),
                result
            ));
        }
        if scene.threats.state_time < 1.6 {
            return Ok(());
        }
        if self.success()
            && (!self.captured
                || self.presses
                    < match self.kind() {
                        ThreatKind::Bus => 1,
                        ThreatKind::Ufo => 3,
                        ThreatKind::Sewer => 2,
                        ThreatKind::Piano => 1,
                        ThreatKind::Snowman => 3,
                    })
        {
            return Err("An event succeeded without its expected raycast actions/capture".into());
        }
        if !self.success() && scene.threats.visible_kenny() {
            return Err("Failure animation did not hide Kenny".into());
        }
        let kind = self.kind();
        let presses = self.presses;
        scene.restart_threat()?;
        if scene.threats.state != ThreatState::Preparing
            || scene.threats.result.is_some()
            || !scene.threats.visible_kenny()
            || scene.kenny.position().y.abs() > 0.001
            || scene.threats.carrying_lid
            || scene.threats.lights != [true; 3]
            || scene.threats.heaters != [false; 3]
            || scene.threats.piano_locked
            || scene.threats.released
        {
            return Err("Incomplete event reset".into());
        }
        scene.clear_threat()?;
        if scene.threats.state != ThreatState::Inactive
            || scene.targets.iter().any(|t| t.id.starts_with(PREFIX))
            || scene
                .world
                .obstacles
                .iter()
                .any(|o| o.id.starts_with(PREFIX))
            || scene.kenny.is_paused()
        {
            return Err("Stale event after returning to exploration".into());
        }
        self.results.push(json!({"kind":kind,"result":result,"E_presses":presses,"restart":"PASS","return_to_exploration":"PASS"}));
        self.case += 1;
        self.started = false;
        Ok(())
    }
}
