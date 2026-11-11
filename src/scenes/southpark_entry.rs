//! Visit lifetime, independent of match restarts. A future solar host owns Visits.
use crate::minigames::{common::collision::CollisionWorld, southpark::SouthParkScene};
use raylib::prelude::*;
use serde::Deserialize;
use std::path::Path;

#[derive(Default)]
pub struct SouthParkVisits {
    pub intro_seen: bool,
    pub chef_intro_seen: bool,
}
/// Kept as an alias for existing callers; all minigames share this contract.
pub use crate::minigames::common::host::MinigameExit as SouthParkExit;

/// Rebuild only CPU gameplay state. GPU resources and the visit/intro lifetime stay alive.
pub fn restart_attempt(
    scene: &mut SouthParkScene,
    session: &mut crate::minigames::southpark::session::SaveKennySession,
    manifest: &crate::minigames::common::manifest::SceneManifest,
    config: &crate::minigames::southpark::DevConfig,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut fresh_scene = SouthParkScene::load(manifest, config.clone())?;
    let mut fresh_session = crate::minigames::southpark::session::SaveKennySession::new(session.config.clone());
    if let Some(locations) = session.locations.clone() {
        fresh_session.enable_world(locations, seed);
        fresh_scene.kenny_hint = false;
        fresh_session.place_kenny_near_player(&mut fresh_scene)?;
    }
    fresh_session.adventure = session.adventure.clone();
    fresh_session.start(&mut fresh_scene, seed)?;
    *scene = fresh_scene;
    *session = fresh_session;
    Ok(())
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryConfig {
    pub fallback_intro_seconds: f32,
    pub return_transition_seconds: f32,
}
impl EntryConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if ![c.fallback_intro_seconds, c.return_transition_seconds]
            .iter()
            .all(|s| s.is_finite() && *s > 0.0)
        {
            return Err("Invalid South Park entry durations".into());
        }
        Ok(c)
    }
}

pub struct SouthParkEntry {
    pub elapsed: f32,
    pub duration: f32,
    pub full_intro: bool,
    pub active: bool,
    points: Vec<(Vector3, Vector3)>,
    fov: f32,
}
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
impl SouthParkEntry {
    pub fn new(
        visits: &SouthParkVisits,
        config: EntryConfig,
        audio_length: Option<f32>,
        scene: &SouthParkScene,
    ) -> Result<Self, String> {
        let eye = scene.player.eye();
        let full_intro = !visits.intro_seen;
        // The video is the only first-visit presentation. The return fade uses
        // the exact FPS pose; no cinematic camera path runs behind the clip.
        let points = vec![(eye, eye + scene.player.direction()); 2];
        let entry = Self {
            elapsed: 0.0,
            duration: if full_intro {
                audio_length
                    .filter(|v| v.is_finite() && *v > 0.0)
                    .unwrap_or(config.fallback_intro_seconds)
                    .max(0.1)
            } else {
                config.return_transition_seconds
            },
            full_intro,
            active: true,
            points,
            fov: scene.player.config.fov_degrees,
        };
        entry.validate(&scene.world)?;
        Ok(entry)
    }
    pub fn camera_at(&self, fraction: f32) -> Camera3D {
        let t = fraction.clamp(0.0, 1.0) * (self.points.len() - 1) as f32;
        let i = (t.floor() as usize).min(self.points.len() - 2);
        let u = smooth(t - i as f32);
        let (a, at) = self.points[i];
        let (b, bt) = self.points[i + 1];
        Camera3D::perspective(a + (b - a) * u, at + (bt - at) * u, Vector3::up(), self.fov)
    }
    pub fn camera(&self) -> Camera3D {
        self.camera_at(self.elapsed / self.duration)
    }
    pub fn validate(&self, world: &CollisionWorld) -> Result<(), String> {
        for i in 0..=1000 {
            let c = self.camera_at(i as f32 / 1000.0);
            if c.position.y <= world.ground_y
                || world
                    .obstacles
                    .iter()
                    .filter(|o| {
                        !o.id.starts_with("boundary-")
                            && o.id != crate::minigames::southpark::kenny::KENNY_ID
                    })
                    .any(|o| {
                        o.bounds
                            .expanded(Vector3::one() * 0.15)
                            .ray(c.position, Vector3::up(), 0.0)
                            .is_some()
                    })
            {
                return Err(format!(
                    "Cinematic camera intersects scenery at {:?}",
                    c.position
                ));
            }
        }
        Ok(())
    }
    pub fn update(&mut self, dt: f32, skip: bool, visits: &mut SouthParkVisits) {
        self.update_audio(dt, skip, visits, crate::audio::VoiceClock::default());
    }
    /// Use the video audio clock for playback and completion. No fixed cap.
    pub fn update_audio(
        &mut self,
        dt: f32,
        skip: bool,
        visits: &mut SouthParkVisits,
        clock: crate::audio::VoiceClock,
    ) {
        if !self.active {
            return;
        }
        let audio_position = clock.position.filter(|p| p.is_finite() && *p >= 0.0);
        if self.full_intro {
            if let Some(position) = audio_position {
                self.elapsed = position.min(self.duration);
            } else {
                self.elapsed += if dt.is_finite() {
                    dt.clamp(0.0, 0.1)
                } else {
                    0.0
                };
            }
        } else {
            self.elapsed += if dt.is_finite() {
                dt.clamp(0.0, 0.1)
            } else {
                0.0
            };
        }
        let finished = if self.full_intro && audio_position.is_some() {
            clock.finished
        } else {
            self.elapsed >= self.duration
        };
        if skip || finished {
            self.active = false;
            visits.intro_seen = true;
        }
    }
    pub fn draw(&self, d: &mut RaylibDrawHandle, w: i32, h: i32) {
        if !self.active {
            return;
        }
        // The full intro is drawn by VideoIntro, with no legacy letterbox/title.
        if self.full_intro { return; }
        let alpha = 1.0 - self.elapsed / self.duration;
        if alpha > 0.0 {
            d.draw_rectangle(0, 0, w, h, Color::new(12, 24, 38, (alpha * 255.0) as u8));
        }
    }
}
