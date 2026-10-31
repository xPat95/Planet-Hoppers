//! Match rules, not a second threat controller. Rendering and audio consume
//! explicit states/cues; a result is consumed only when leaving Active.
use super::{
    kenny::KENNY_ID,
    locations::{random_step, EventSite, WorldLocations},
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
    Interlude,
    Relocating,
    Searching,
    MapOpen,
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
    pub world_interval_seconds: f32,
    pub arrival_distance: f32,
    pub relocation_seconds: f32,
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
            c.world_interval_seconds,
            c.arrival_distance,
            c.relocation_seconds,
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
    pub locations: Option<WorldLocations>,
    pub site: Option<EventSite>,
    pub objective_distance: f32,
    last_site: Option<String>,
    random: u64,
    returning: bool,
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
            locations: None,
            site: None,
            last_site: None,
            random: 1,
            returning: false,
            objective_distance: 0.0,
        }
    }
    pub fn in_match(&self) -> bool {
        self.state != SessionState::Exploring
            && !(self.state == SessionState::MapOpen
                && self.paused.is_some_and(|s| s.0 == SessionState::Exploring))
            && !(self.state == SessionState::Relocating && self.returning)
    }
    pub fn dynamic(&self) -> bool {
        self.locations.is_some()
    }
    pub fn enable_world(&mut self, locations: WorldLocations, seed: u64) {
        self.locations = Some(locations);
        self.random = seed;
    }
    pub fn randomize_exploration(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        let Some(locations) = &self.locations else {
            return Ok(());
        };
        let count = locations.exploration_spawns.len();
        let start = random_step(&mut self.random) as usize % count;
        for i in 0..count {
            let p = locations.exploration_spawns[(start + i) % count];
            let point = Vector3::new(p[0], p[1], p[2]);
            if (point - scene.kenny.position()).length() > 10.0 && scene.relocate_kenny(point) {
                scene.kenny.resume();
                return Ok(());
            }
        }
        Err("No safe exploration spawn available".into())
    }
    pub fn current(&self) -> ThreatKind {
        self.order[self.index.min(4)]
    }
    pub fn blocks_controls(&self) -> bool {
        if self.dynamic()
            && matches!(
                self.state,
                SessionState::Interlude | SessionState::Searching | SessionState::Success
            )
        {
            return false;
        }
        !matches!(
            self.state,
            SessionState::Exploring | SessionState::Preparing | SessionState::Active
        )
    }
    pub fn permits_interaction(&self) -> bool {
        matches!(self.state, SessionState::Exploring | SessionState::Active)
    }
    pub fn music(&self) -> MusicMode {
        if self.state == SessionState::MapOpen {
            return if self.paused.is_some_and(|s| s.0 == SessionState::Exploring) {
                MusicMode::Exploration
            } else {
                MusicMode::Challenge
            };
        }
        if self.state == SessionState::Relocating && self.returning {
            return MusicMode::Exploration;
        }
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
        if !self.dynamic() {
            self.stage_player(scene)?;
        }
        scene.clear_threat()?;
        scene.close_dialogue();
        self.lives = 3;
        self.saves = 0;
        self.index = 0;
        self.order = Self::shuffled(seed);
        self.voice_time = 0.0;
        self.cues.clear();
        self.paused = None;
        self.returning = false;
        self.site = None;
        if self.dynamic() {
            scene.kenny.resume();
            self.enter(SessionState::Interlude);
        } else {
            scene.kenny.pause();
            self.enter(SessionState::Briefing);
        }
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
                if !self.dynamic()
                    && self.saves < 5
                    && self.elapsed >= self.config.success_hold_seconds =>
            {
                self.index += 1;
                // Retain the same index on failure; only success reaches here.
                if let Err(e) = self.briefing(scene) {
                    self.index -= 1;
                    return Err(e);
                }
            }
            SessionState::Paused | SessionState::MapOpen => {
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
        if matches!(
            self.state,
            SessionState::Preparing
                | SessionState::Active
                | SessionState::Searching
                | SessionState::Interlude
        ) {
            self.paused = Some((self.state, self.elapsed));
            self.enter(SessionState::Paused);
        }
    }
    pub fn open_map(&mut self) -> bool {
        if matches!(
            self.state,
            SessionState::Exploring
                | SessionState::Interlude
                | SessionState::Searching
                | SessionState::Preparing
                | SessionState::Active
        ) || (self.dynamic() && self.state == SessionState::Success)
        {
            self.paused = Some((self.state, self.elapsed));
            self.enter(SessionState::MapOpen);
            true
        } else {
            false
        }
    }
    pub fn location_visible(&self) -> bool {
        let state = if self.state == SessionState::MapOpen {
            self.paused.map(|s| s.0).unwrap_or(self.state)
        } else {
            self.state
        };
        matches!(
            state,
            SessionState::Searching | SessionState::Preparing | SessionState::Active
        )
    }
    pub fn explore(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        scene.clear_threat()?;
        scene.close_dialogue();
        scene.kenny.resume();
        self.cues.clear();
        self.paused = None;
        self.site = None;
        if self.dynamic() {
            self.returning = true;
            self.enter(SessionState::Relocating);
        } else {
            self.enter(SessionState::Exploring);
        }
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
        if !matches!(self.state, SessionState::Paused | SessionState::MapOpen) {
            self.elapsed += dt;
        }
        match self.state {
            SessionState::Exploring => scene.update_kenny(dt),
            SessionState::Interlude => {
                if scene.threats.kind.is_none() {
                    scene.update_kenny(dt);
                }
                if self.elapsed >= self.config.world_interval_seconds {
                    self.enter(SessionState::Relocating);
                }
            }
            SessionState::Relocating => {
                if self.elapsed >= self.config.relocation_seconds {
                    if self.returning {
                        self.randomize_exploration(scene)?;
                        self.returning = false;
                        self.enter(SessionState::Exploring);
                    } else {
                        self.prepare_search(scene)?;
                    }
                }
            }
            SessionState::Searching => {
                let delta = scene.kenny.position() - scene.player.feet;
                self.objective_distance = delta.length();
                let eye = scene.player.eye();
                let toward = scene.kenny.position() + Vector3::up() - eye;
                if delta.length() <= self.config.arrival_distance
                    && !scene
                        .world
                        .ray_distance_ignoring(
                            eye,
                            toward.normalized(),
                            toward.length(),
                            Some(KENNY_ID),
                        )
                        .is_some_and(|d| d < toward.length() - 0.1)
                {
                    self.enter(SessionState::Preparing);
                }
            }
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
                } else if self.dynamic() && self.elapsed >= self.config.success_hold_seconds {
                    self.index += 1;
                    self.enter(SessionState::Interlude);
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
                    if self.dynamic() {
                        self.enter(SessionState::Relocating);
                    } else {
                        self.briefing(scene)?;
                    }
                }
            }
            _ => {}
        }
        if self.blocks_controls() && self.state != SessionState::MapOpen {
            scene.kenny.animation.update(dt, 0.0);
        }
        Ok(())
    }
    fn prepare_search(&mut self, scene: &mut SouthParkScene) -> Result<(), String> {
        scene.clear_threat()?;
        let locations = self.locations.as_ref().ok_or("No location catalog")?;
        let mut candidates: Vec<_> = locations
            .sites
            .iter()
            .filter(|s| {
                s.compatible.contains(&self.current())
                    && Some(s.id.as_str()) != self.last_site.as_deref()
            })
            .cloned()
            .collect();
        let start = random_step(&mut self.random) as usize % candidates.len().max(1);
        candidates.rotate_left(start);
        for site in candidates {
            if scene
                .start_threat_at(self.current(), site.translation())
                .is_ok()
            {
                self.last_site = Some(site.id.clone());
                self.site = Some(site);
                self.objective_distance = (scene.kenny.position() - scene.player.feet).length();
                self.enter(SessionState::Searching);
                self.cues.push(SessionCue::Danger);
                return Ok(());
            }
        }
        Err("Las zonas están ocupadas; alejate un poco y se reintentará la preparación.".into())
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
