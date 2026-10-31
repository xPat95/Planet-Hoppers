//! Match rules, not a second threat controller. Rendering and audio consume
//! explicit states/cues; a result is consumed only when leaving Active.
use super::{
    kenny::KENNY_ID,
    threats::{ThreatKind, ThreatResult, ThreatState, PREFIX},
    SouthParkScene,
};
use raylib::prelude::Vector3;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum SessionState {
    Exploring,
    Briefing,
    Preparing,
    Active,
    Success,
    DeathAnimation,
    DeathDialogue,
    Restoring,
    Victory,
    Defeat,
    Paused,
}
pub use crate::audio::{AudioCue as SessionCue, MusicMode, VoiceClock};
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionConfig {
    pub death_animation_seconds: f32,
    pub voice_fallback_seconds: f32,
    pub kyle_switch_seconds: f32,
    pub success_hold_seconds: f32,
    pub text_characters_per_second: f32,
}
impl Default for SessionConfig {
    fn default() -> Self {
        serde_json::from_str(include_str!("../../../config/southpark-session.json"))
            .expect("Bundled session config")
    }
}
impl SessionConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if ![
            c.death_animation_seconds,
            c.voice_fallback_seconds,
            c.kyle_switch_seconds,
            c.success_hold_seconds,
            c.text_characters_per_second,
        ]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
            || c.death_animation_seconds < 1.4
            || c.kyle_switch_seconds >= c.voice_fallback_seconds
        {
            return Err("Invalid dialogue/session timings".into());
        }
        Ok(c)
    }
}

pub struct SaveKennySession {
    pub state: SessionState,
    pub lives: u8,
    pub saves: u8,
    pub order: [ThreatKind; 5],
    pub elapsed: f32,
    pub voice_time: f32,
    pub config: SessionConfig,
    pub error: Option<String>,
    index: usize,
    cues: Vec<SessionCue>,
    paused: Option<(SessionState, f32)>,
}
impl Default for SaveKennySession {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}
impl SaveKennySession {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            state: SessionState::Exploring,
            lives: 3,
            saves: 0,
            order: ThreatKind::ALL,
            elapsed: 0.0,
            voice_time: 0.0,
            config,
            error: None,
            index: 0,
            cues: Vec::new(),
            paused: None,
        }
    }
    pub fn in_match(&self) -> bool {
        self.state != SessionState::Exploring
    }
    pub fn current(&self) -> ThreatKind {
        self.order[self.index.min(4)]
    }
    pub fn blocks_controls(&self) -> bool {
        !matches!(
            self.state,
            SessionState::Exploring | SessionState::Preparing | SessionState::Active
        )
    }
    pub fn permits_interaction(&self) -> bool {
        matches!(self.state, SessionState::Exploring | SessionState::Active)
    }
    pub fn music(&self) -> MusicMode {
        match self.state {
            SessionState::Exploring => MusicMode::Exploration,
            SessionState::DeathAnimation
            | SessionState::DeathDialogue
            | SessionState::Restoring
            | SessionState::Victory
            | SessionState::Defeat
            | SessionState::Paused => MusicMode::Silent,
            _ => MusicMode::Challenge,
        }
    }
    fn enter(&mut self, state: SessionState) {
        self.state = state;
        self.elapsed = 0.0;
        self.error = None;
    }
    pub fn take_cues(&mut self) -> Vec<SessionCue> {
        std::mem::take(&mut self.cues)
    }
    /// Fisher-Yates with local xorshift64*. No global RNG or new dependency.
    pub fn shuffled(mut seed: u64) -> [ThreatKind; 5] {
        let mut order = ThreatKind::ALL;
        if seed == 0 {
            seed = 0x9e3779b97f4a7c15;
        }
        for i in (1..5).rev() {
            seed ^= seed >> 12;
            seed ^= seed << 25;
            seed ^= seed >> 27;
            let random = seed.wrapping_mul(0x2545f4914f6cdd1d);
            order.swap(i, (random % (i as u64 + 1)) as usize);
        }
        order
    }
    pub fn start(&mut self, scene: &mut SouthParkScene, seed: u64) -> Result<(), String> {
        // A new match is explicit; debug results never enter this accounting.
        self.stage_player(scene)?;
        scene.clear_threat()?;
        scene.close_dialogue();
        self.lives = 3;
        self.saves = 0;
        self.index = 0;
        self.order = Self::shuffled(seed);
        self.voice_time = 0.0;
        self.cues.clear();
        self.paused = None;
        scene.kenny.pause();
        self.enter(SessionState::Briefing);
        Ok(())
    }
    fn stage_player(&self, scene: &mut SouthParkScene) -> Result<(), String> {
        let mut clean = scene.world.clone();
        clean.obstacles.retain(|o| !o.id.starts_with(PREFIX));
        let spawn = Vector3::new(-43.2, 0.0, -42.9);
        let feet = [
            Vector3::zero(),
            Vector3::new(0.0, 0.0, -2.0),
            Vector3::new(0.0, 0.0, 2.0),
            -Vector3::right() * 4.0,
        ]
        .iter()
        .map(|d| spawn + *d)
        .find(|p| {
            clean.can_occupy(
                *p,
                scene.player.config.radius,
                scene.player.config.body_height,
            )
        })
        .ok_or("No hay un acceso seguro a North Avenue. Volvé a explorar e intentá de nuevo.")?;
        scene.player.feet = feet;
        scene.player.yaw = 0.0;
        scene.player.pitch = 0.0;
        Ok(())
    }
    fn briefing(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        self.stage_player(scene)?;
        scene.clear_threat()?;
        scene.kenny.pause();
        self.enter(SessionState::Briefing);
        Ok(())
    }
    pub fn continue_match(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        match self.state {
            SessionState::Briefing => {
                scene.start_threat(self.current())?;
                self.enter(SessionState::Preparing);
                self.cues.push(SessionCue::Danger);
            }
            SessionState::Success
                if self.saves < 5 && self.elapsed >= self.config.success_hold_seconds =>
            {
                self.index += 1;
                // Retain the same index on failure; only success reaches here.
                if let Err(e) = self.briefing(scene) {
                    self.index -= 1;
                    return Err(e);
                }
            }
            SessionState::Paused => {
                if let Some((state, elapsed)) = self.paused.take() {
                    self.state = state;
                    self.elapsed = elapsed;
                }
            }
            _ => {}
        }
        Ok(())
    }
    pub fn pause(&mut self) {
        if matches!(self.state, SessionState::Preparing | SessionState::Active) {
            self.paused = Some((self.state, self.elapsed));
            self.enter(SessionState::Paused);
        }
    }
    pub fn explore(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        scene.clear_threat()?;
        scene.close_dialogue();
        scene.kenny.resume();
        self.cues.clear();
        self.paused = None;
        self.enter(SessionState::Exploring);
        Ok(())
    }
    pub fn dialogue(&self) -> (&'static str, &'static str, usize) {
        let (name, text, start) = if self.voice_time < self.config.kyle_switch_seconds {
            ("Stan", "Oh my God! They killed Kenny!", 0.0)
        } else {
            ("Kyle", "You bastards!", self.config.kyle_switch_seconds)
        };
        (
            name,
            text,
            ((self.voice_time - start).max(0.0) * self.config.text_characters_per_second) as usize,
        )
    }
    pub fn update(
        &mut self,
        dt: f32,
        scene: &mut SouthParkScene,
        voice: VoiceClock,
    ) -> Result<(), String> {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.1)
        } else {
            0.0
        };
        if self.state != SessionState::Paused {
            self.elapsed += dt;
        }
        match self.state {
            SessionState::Exploring => scene.update_kenny(dt),
            SessionState::Preparing | SessionState::Active => {
                scene.update_kenny(dt);
                if let Some(result) = scene.threats.result {
                    scene.kenny.pause();
                    if result == ThreatResult::Survived {
                        self.saves += 1;
                        self.enter(SessionState::Success);
                        self.cues.push(SessionCue::Success);
                    } else {
                        self.enter(SessionState::DeathAnimation);
                    }
                } else if scene.threats.state == ThreatState::Active {
                    self.state = SessionState::Active;
                }
            }
            SessionState::Success => {
                if self.saves == 5 && self.elapsed >= self.config.success_hold_seconds {
                    scene.clear_threat()?;
                    scene.kenny.pause();
                    self.enter(SessionState::Victory);
                    self.cues.push(SessionCue::Victory);
                }
            }
            SessionState::DeathAnimation => {
                scene.update_kenny(dt);
                if self.elapsed >= self.config.death_animation_seconds {
                    self.voice_time = 0.0;
                    self.enter(SessionState::DeathDialogue);
                    self.cues.push(SessionCue::DeathVoice);
                }
            }
            SessionState::DeathDialogue => {
                self.voice_time = voice.position.unwrap_or(self.elapsed).max(self.voice_time);
                let done = if voice.position.is_some() {
                    voice.finished
                } else {
                    self.elapsed >= self.config.voice_fallback_seconds
                };
                if done {
                    self.lives = self.lives.saturating_sub(1);
                    self.enter(SessionState::Restoring);
                }
            }
            SessionState::Restoring => {
                if self.lives == 0 {
                    scene.clear_threat()?;
                    scene.kenny.pause();
                    self.enter(SessionState::Defeat);
                    self.cues.push(SessionCue::GameOver);
                } else {
                    self.briefing(scene)?;
                }
            }
            _ => {}
        }
        if self.blocks_controls() {
            scene.kenny.animation.update(dt, 0.0);
        }
        Ok(())
    }
    pub fn assert_exploration_clean(&self, scene: &SouthParkScene) -> bool {
        self.state == SessionState::Exploring
            && !scene.kenny.is_paused()
            && !scene
                .world
                .obstacles
                .iter()
                .any(|o| o.id.starts_with(PREFIX))
            && scene.targets.iter().any(|t| t.id == KENNY_ID)
    }
}
