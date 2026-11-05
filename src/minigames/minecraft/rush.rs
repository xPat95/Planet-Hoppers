//! Challenge rules; independent of graphics, audio devices and the surface host.
use super::{mine::MineScene, mining::MiningSession};
use crate::minigames::common::manifest::{from_blender, from_blender_bounds};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RushConfig {
    pub initial_seconds: f32,
    pub required_diamonds: u32,
    pub available_diamonds: u32,
    pub lava_penalty_seconds: f32,
    pub lava_protection_seconds: f32,
    pub warning_seconds: f32,
}
impl RushConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if ![
            c.initial_seconds,
            c.lava_penalty_seconds,
            c.lava_protection_seconds,
            c.warning_seconds,
        ]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
            || c.required_diamonds == 0
            || c.available_diamonds < c.required_diamonds
            || c.warning_seconds >= c.initial_seconds
        {
            return Err("Invalid Diamond Rush config".into());
        }
        Ok(c)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RushState {
    Loading,
    Running,
    Paused,
    Victory,
    Defeat,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum RushCue {
    Danger,
    Success,
    Victory,
    GameOver,
    Reset,
}
pub struct RushSession {
    pub config: RushConfig,
    pub state: RushState,
    pub remaining: f32,
    pub warning: bool,
    pub objective_met: bool,
    pub lava_protection: f32,
    pub notice: Option<(String, f32)>,
    cues: Vec<RushCue>,
}
impl crate::minigames::common::host::AttemptOutcome for RushSession {
    fn attempt_outcome(&self) -> Option<crate::minigames::common::host::MinigameResult> {
        use crate::minigames::common::host::MinigameResult;
        match self.state {
            RushState::Victory => Some(MinigameResult::Victory),
            RushState::Defeat => Some(MinigameResult::Defeat),
            _ => None,
        }
    }
}
impl RushSession {
    pub fn new(config: RushConfig) -> Self {
        Self {
            remaining: config.initial_seconds,
            config,
            state: RushState::Loading,
            warning: false,
            objective_met: false,
            lava_protection: 0.0,
            notice: None,
            cues: vec![],
        }
    }
    pub fn ready(&mut self) {
        if self.state == RushState::Loading {
            self.state = RushState::Running;
        }
    }
    pub fn pause(&mut self) {
        if self.state == RushState::Running {
            self.state = RushState::Paused;
        }
    }
    pub fn resume(&mut self) {
        if self.state == RushState::Paused {
            self.state = RushState::Running;
        }
    }
    pub fn clock(&self) -> String {
        let n = self.remaining.max(0.0).ceil() as u32;
        format!("{:02}:{:02}", n / 60, n % 60)
    }
    fn alert(&mut self) {
        if self.remaining > 0.0 && self.remaining <= self.config.warning_seconds && !self.warning {
            self.warning = true;
            self.cues.push(RushCue::Danger);
            self.notice = Some(("¡Queda poco tiempo!".into(), 2.5));
        }
    }
    fn lose(&mut self) {
        if self.state == RushState::Running {
            self.remaining = 0.0;
            self.state = RushState::Defeat;
            self.cues.push(RushCue::GameOver);
        }
    }
    pub fn advance(&mut self, dt: f32) {
        if self.state != RushState::Running {
            return;
        }
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        self.remaining = (self.remaining - dt).max(0.0);
        self.lava_protection = (self.lava_protection - dt).max(0.0);
        if let Some((_, t)) = &mut self.notice {
            *t -= dt;
            if *t <= 0.0 {
                self.notice = None;
            }
        }
        if self.remaining <= 0.0 {
            self.lose();
        } else {
            self.alert();
        }
    }
    pub fn inspect_world(
        &mut self,
        scene: &mut MineScene,
        mining: &mut MiningSession,
        exit_requested: bool,
    ) {
        if self.state != RushState::Running {
            return;
        }
        if self.lava_protection <= 0.0 {
            let body = scene.world.body(
                scene.player.feet,
                scene.player.config.radius,
                scene.player.config.body_height,
            );
            if let Some(r) = scene
                .data
                .lava_regions
                .iter()
                .find(|r| r.damage_enabled && from_blender_bounds(r.min, r.max).overlaps(body))
            {
                if let Some(safe) = r.safe_return {
                    scene.player.feet = from_blender(safe);
                }
                self.remaining = (self.remaining - self.config.lava_penalty_seconds).max(0.0);
                self.lava_protection = self.config.lava_protection_seconds;
                self.notice = Some((
                    format!(
                        "¡Lava! -{} segundos",
                        self.config.lava_penalty_seconds as u32
                    ),
                    2.5,
                ));
                mining.update(scene, false, false, 0.0);
                if self.remaining <= 0.0 {
                    self.lose();
                    return;
                }
                self.alert();
            }
        }
        if mining.diamonds >= self.config.required_diamonds && !self.objective_met {
            self.objective_met = true;
            self.cues.push(RushCue::Success);
            self.notice = Some(("¡Objetivo completado! Regresá a la salida.".into(), 3.0));
        }
        if exit_requested {
            if mining.diamonds >= self.config.required_diamonds && self.remaining > 0.0 {
                self.state = RushState::Victory;
                self.cues.push(RushCue::Victory);
            } else {
                self.notice = Some((
                    format!(
                        "Necesitás {} diamantes para salir victorioso. Faltan {}.",
                        self.config.required_diamonds,
                        self.config
                            .required_diamonds
                            .saturating_sub(mining.diamonds)
                    ),
                    2.5,
                ));
            }
        }
    }
    pub fn restart(&mut self, scene: &mut MineScene, mining: &mut MiningSession) {
        mining.reset_mining_session(scene);
        *self = Self::new(self.config.clone());
        self.ready();
        self.cues.push(RushCue::Reset);
    }
    pub fn drain_cues(&mut self) -> Vec<RushCue> {
        std::mem::take(&mut self.cues)
    }
}
