//! One development event at a time. Logic/results are independent of rendering,
//! score, lives and solar-system state. All coordinates are metres, Y up.
use super::{
    kenny::{Kenny, KENNY_ID},
    navigation::{NPC_HEIGHT, NPC_RADIUS},
};
use crate::minigames::common::{
    collision::{Aabb, CollisionWorld, Obstacle},
    interaction::InteractionTarget,
};
use raylib::prelude::Vector3;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PREFIX: &str = "threat:";
pub const LANE_Z: f32 = -42.9;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ThreatKind {
    Bus,
    Ufo,
    Sewer,
}
impl ThreatKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "bus" => Some(Self::Bus),
            "ufo" => Some(Self::Ufo),
            "sewer" => Some(Self::Sewer),
            _ => None,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Bus => "Autobús: busca el panel rojo y activa el freno",
            Self::Ufo => "OVNI: desactiva sus tres luces desde el suelo",
            Self::Sewer => "Alcantarilla: recoge la tapa y cubre el agujero",
        }
    }
    pub fn stage(self) -> Vector3 {
        Vector3::new(
            match self {
                Self::Bus => -24.0,
                Self::Ufo => -28.0,
                Self::Sewer => -26.0,
            },
            0.0,
            LANE_Z,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ThreatState {
    Inactive,
    Preparing,
    Active,
    Resolving,
    Succeeded,
    Failed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ThreatResult {
    Survived,
    Died,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreatConfig {
    pub preparation_seconds: f32,
    pub bus_seconds: f32,
    pub ufo_seconds: f32,
    pub sewer_seconds: f32,
    pub ufo_max_lift: f32,
    pub interaction_distance: f32,
    pub ufo_light_range: f32,
}
impl Default for ThreatConfig {
    fn default() -> Self {
        serde_json::from_str(include_str!("../../../config/southpark-threats.json"))
            .expect("Bundled threat configuration")
    }
}
impl ThreatConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if ![
            c.preparation_seconds,
            c.bus_seconds,
            c.ufo_seconds,
            c.sewer_seconds,
            c.ufo_max_lift,
            c.interaction_distance,
            c.ufo_light_range,
        ]
        .iter()
        .all(|n| n.is_finite() && *n > 0.0)
            || c.bus_seconds < 6.0
            || c.ufo_seconds < 8.0
            || c.sewer_seconds < 10.0
            || c.ufo_max_lift > 4.8
            || c.ufo_light_range < 8.0
        {
            return Err(
                "Invalid threat configuration: insufficient time/range or invalid lift".into(),
            );
        }
        Ok(c)
    }
}

pub struct ThreatDirector {
    pub config: ThreatConfig,
    pub kind: Option<ThreatKind>,
    pub state: ThreatState,
    pub result: Option<ThreatResult>,
    pub kenny_id: &'static str,
    pub elapsed: f32,
    pub state_time: f32,
    pub lights: [bool; 3],
    pub carrying_lid: bool,
    pub covered: bool,
    saved_position: Option<Vector3>,
    pub progress: f32,
}
impl Default for ThreatDirector {
    fn default() -> Self {
        Self::new(ThreatConfig::default())
    }
}
impl ThreatDirector {
    pub fn new(config: ThreatConfig) -> Self {
        Self {
            config,
            kind: None,
            state: ThreatState::Inactive,
            result: None,
            kenny_id: KENNY_ID,
            elapsed: 0.0,
            state_time: 0.0,
            lights: [true; 3],
            carrying_lid: false,
            covered: false,
            saved_position: None,
            progress: 0.0,
        }
    }
    pub fn running(&self) -> bool {
        matches!(
            self.state,
            ThreatState::Preparing | ThreatState::Active | ThreatState::Resolving
        )
    }
    pub fn owns_kenny(&self) -> bool {
        self.running() || self.state == ThreatState::Failed
    }
    pub fn visible_kenny(&self) -> bool {
        self.state != ThreatState::Failed || self.state_time < 1.4
    }
    pub fn duration(&self) -> f32 {
        match self.kind {
            Some(ThreatKind::Bus) => self.config.bus_seconds,
            Some(ThreatKind::Ufo) => self.config.ufo_seconds,
            _ => self.config.sewer_seconds,
        }
    }
    pub fn remaining(&self) -> f32 {
        if self.state == ThreatState::Preparing {
            (self.config.preparation_seconds - self.state_time).max(0.0)
        } else {
            (self.duration() - self.elapsed).max(0.0)
        }
    }
    pub fn panel_position(&self) -> Vector3 {
        Vector3::new(-33.0, 0.0, LANE_Z + 2.0)
    }
    pub fn cover_position(&self) -> Vector3 {
        Vector3::new(-32.0, 0.0, LANE_Z + 2.0)
    }
    pub fn hole_position(&self) -> Vector3 {
        Vector3::new(-12.0, 0.0, LANE_Z)
    }
    pub fn ufo_position(&self) -> Vector3 {
        ThreatKind::Ufo.stage() + Vector3::up() * (6.2 + (self.elapsed * 0.8).sin() * 0.10)
    }
    pub fn light_position(&self, i: usize) -> Vector3 {
        let a = (30.0 + i as f32 * 120.0).to_radians();
        self.ufo_position() + Vector3::new(2.25 * a.cos(), -0.45, 2.25 * a.sin())
    }
    pub fn bus_position(&self) -> Vector3 {
        ThreatKind::Bus.stage() + Vector3::new(48.0 - 52.85 * self.progress, 0.0, 0.0)
    }
    pub fn bus_bounds(&self) -> Aabb {
        let p = self.bus_position();
        Aabb::new(
            p + Vector3::new(-3.1, 0.0, -1.22),
            p + Vector3::new(3.1, 2.7, 1.22),
        )
    }
    pub fn lid_position(&self) -> Option<Vector3> {
        if self.covered || self.state == ThreatState::Resolving {
            Some(
                self.hole_position()
                    + Vector3::up()
                        * if self.state == ThreatState::Resolving {
                            (1.0 - self.state_time / 0.5).max(0.0) * 0.6
                        } else {
                            0.0
                        },
            )
        } else if self.carrying_lid {
            None
        } else {
            Some(self.cover_position())
        }
    }
    pub fn sprite_roll(&self) -> f32 {
        if self.state == ThreatState::Failed && self.kind == Some(ThreatKind::Bus) {
            (self.state_time / 0.7).min(1.0) * 80.0_f32.to_radians()
        } else {
            0.0
        }
    }
    pub fn sprite_height_scale(&self) -> f32 {
        if self.state == ThreatState::Failed && self.kind == Some(ThreatKind::Bus) {
            1.0 - (self.state_time / 0.7).min(1.0) * 0.7
        } else {
            1.0
        }
    }

    /// Validate the whole staging corridor, not only the endpoint. Bus sweep is
    /// an expanded static AABB; these derived vehicle dimensions match Blender.
    pub fn validate_zone(&self, kind: ThreatKind, world: &CollisionWorld) -> Result<(), String> {
        let origin = kind.stage();
        let end = match kind {
            ThreatKind::Bus => origin - Vector3::right() * 8.0,
            ThreatKind::Sewer => self.hole_position(),
            ThreatKind::Ufo => origin,
        };
        for n in 0..=100 {
            let p = origin + (end - origin) * (n as f32 / 100.0);
            let body = world.body(p, NPC_RADIUS, NPC_HEIGHT);
            if world
                .obstacles
                .iter()
                .filter(|o| o.id != KENNY_ID && o.id != "fps-player" && !o.id.starts_with(PREFIX))
                .any(|o| body.overlaps(o.bounds))
            {
                return Err("Kenny event route intersects an obstacle".into());
            }
        }
        if kind == ThreatKind::Bus {
            let half = Vector3::new(3.1, 1.35, 1.22);
            let start = origin + Vector3::new(48.0, 1.35, 0.0);
            let delta = Vector3::new(-52.85, 0.0, 0.0);
            if world
                .obstacles
                .iter()
                .filter(|o| o.id != KENNY_ID && o.id != "fps-player" && !o.id.starts_with(PREFIX))
                .any(|o| o.bounds.expanded(half).ray(start, delta, 1.0).is_some())
            {
                return Err("Bus corridor is obstructed".into());
            }
        }
        let prop = if kind == ThreatKind::Bus {
            self.panel_position()
        } else {
            self.cover_position()
        };
        let approach = world.body(prop + Vector3::new(-1.0, 0.0, 0.0), 0.35, 1.8);
        if world
            .obstacles
            .iter()
            .filter(|o| o.id != KENNY_ID && o.id != "fps-player" && !o.id.starts_with(PREFIX))
            .any(|o| approach.overlaps(o.bounds))
        {
            return Err("Interaction approach is blocked".into());
        }
        Ok(())
    }
    pub fn start(
        &mut self,
        kind: ThreatKind,
        kenny: &mut Kenny,
        world: &CollisionWorld,
    ) -> Result<(), String> {
        if self.running() {
            return Err("Ya hay una amenaza en curso. F8 reinicia; F9 vuelve a explorar.".into());
        }
        self.validate_zone(kind, world)?;
        let original = kenny.position();
        if !kenny.reposition_safe(kind.stage(), world) {
            return Err("Event stage is occupied; step off the marked lane".into());
        }
        if self.saved_position.is_none() {
            self.saved_position = Some(original);
        }
        self.kind = Some(kind);
        self.state = ThreatState::Preparing;
        self.result = None;
        self.elapsed = 0.0;
        self.state_time = 0.0;
        self.lights = [true; 3];
        self.carrying_lid = false;
        self.covered = false;
        self.progress = 0.0;
        kenny.pause();
        kenny.take_threat_contact();
        kenny.request_danger_animation(true);
        Ok(())
    }
    pub fn restart(&mut self, kenny: &mut Kenny, world: &CollisionWorld) -> Result<(), String> {
        let kind = self.kind.ok_or("No hay amenaza seleccionada")?;
        let previous = self.state;
        self.state = ThreatState::Inactive;
        let result = self.start(kind, kenny, world);
        if result.is_err() {
            self.state = previous;
        }
        result
    }
    pub fn clear(&mut self, kenny: &mut Kenny, world: &CollisionWorld) -> Result<(), String> {
        if self.kind.is_none() {
            return Ok(());
        }
        let anchor = self.saved_position.unwrap_or(ThreatKind::Ufo.stage());
        let mut restored = false;
        for offset in [
            Vector3::zero(),
            Vector3::new(0.0, 0.0, 1.7),
            Vector3::new(0.0, 0.0, -1.7),
            Vector3::new(-4.0, 0.0, 0.0),
            Vector3::new(4.0, 0.0, 0.0),
        ] {
            if kenny.reposition_safe(anchor + offset, world) {
                restored = true;
                break;
            }
        }
        if !restored {
            return Err("No safe point to restore Kenny; move away and retry F9".into());
        }
        kenny.request_danger_animation(false);
        kenny.resume();
        let cfg = self.config.clone();
        kenny.take_threat_contact();
        *self = Self::new(cfg);
        Ok(())
    }
    fn success(&mut self, kenny: &mut Kenny) {
        self.state = ThreatState::Succeeded;
        self.state_time = 0.0;
        self.result = Some(ThreatResult::Survived);
        kenny.request_danger_animation(false);
        kenny.resume();
    }
    fn fail(&mut self, kenny: &mut Kenny) {
        self.state = ThreatState::Failed;
        self.state_time = 0.0;
        self.result = Some(ThreatResult::Died);
        kenny.notify_threat_contact(match self.kind {
            Some(ThreatKind::Bus) => "bus",
            Some(ThreatKind::Ufo) => "ufo",
            _ => "sewer",
        });
        kenny.pause();
    }
    pub fn update(&mut self, dt: f32, kenny: &mut Kenny) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.state_time += dt;
        let Some(kind) = self.kind else {
            return;
        };
        if self.state == ThreatState::Preparing {
            kenny.event_pose(kind.stage(), dt);
            if self.state_time >= self.config.preparation_seconds {
                self.state = ThreatState::Active;
                self.state_time = 0.0;
            }
        } else if self.state == ThreatState::Active {
            self.elapsed = (self.elapsed + dt).min(self.duration());
            self.progress = self.elapsed / self.duration();
            let p = match kind {
                ThreatKind::Bus => kind.stage() - Vector3::right() * (8.0 * self.progress),
                ThreatKind::Ufo => {
                    kind.stage() + Vector3::up() * (self.config.ufo_max_lift * self.progress)
                }
                ThreatKind::Sewer => {
                    kind.stage() + (self.hole_position() - kind.stage()) * self.progress
                }
            };
            kenny.event_pose(p, dt);
            let contact = kind == ThreatKind::Bus
                && self.bus_bounds().overlaps(Aabb::new(
                    p + Vector3::new(-NPC_RADIUS, 0.0, -NPC_RADIUS),
                    p + Vector3::new(NPC_RADIUS, NPC_HEIGHT, NPC_RADIUS),
                ));
            if self.progress >= 1.0 || contact {
                self.fail(kenny);
            }
        } else if self.state == ThreatState::Resolving {
            if kind == ThreatKind::Ufo {
                let mut p = kenny.position();
                p.y = (p.y - 2.0 * dt).max(0.0);
                kenny.event_pose(p, dt);
                if p.y <= 0.0 {
                    self.success(kenny);
                }
            } else {
                kenny.event_pose(kenny.position(), dt);
                if self.state_time >= 0.5 {
                    self.carrying_lid = false;
                    self.covered = true;
                    self.success(kenny);
                }
            }
        } else if self.state == ThreatState::Failed {
            let y = match kind {
                ThreatKind::Sewer => -3.0 * (self.state_time / 1.2).min(1.0),
                ThreatKind::Ufo => {
                    self.config.ufo_max_lift + 2.0 * (self.state_time / 1.2).min(1.0)
                }
                ThreatKind::Bus => {
                    if self.state_time >= 1.4 {
                        -3.0
                    } else {
                        0.0
                    }
                }
            };
            let mut p = kenny.position();
            p.y = y;
            kenny.event_pose(p, dt);
        }
    }
    pub fn targets(&self) -> Vec<InteractionTarget> {
        if self.state != ThreatState::Active {
            return vec![];
        }
        let mut result = Vec::new();
        let mut add = |id: &str, label: &str, p: Vector3, half: Vector3| {
            result.push(InteractionTarget {
                id: format!("{PREFIX}{id}"),
                label: label.into(),
                bounds: Aabb::new(p - half, p + half),
                action: format!("{PREFIX}{id}"),
            })
        };
        match self.kind {
            Some(ThreatKind::Bus) => add(
                "brake",
                "Activar freno",
                self.panel_position() + Vector3::up() * 1.5,
                Vector3::new(0.34, 0.34, 0.18),
            ),
            Some(ThreatKind::Ufo) => {
                for i in 0..3 {
                    if self.lights[i] {
                        add(
                            &format!("light-{i}"),
                            &format!("Desactivar luz {}", i + 1),
                            self.light_position(i),
                            Vector3::new(0.25, 0.25, 0.25),
                        );
                    }
                }
            }
            Some(ThreatKind::Sewer) => {
                if self.carrying_lid {
                    add(
                        "place",
                        "Colocar tapa",
                        self.hole_position() + Vector3::up() * 0.12,
                        Vector3::new(0.86, 0.15, 0.86),
                    );
                } else {
                    add(
                        "pickup",
                        "Recoger tapa",
                        self.cover_position() + Vector3::up() * 0.12,
                        Vector3::new(0.86, 0.15, 0.86),
                    );
                }
            }
            None => {}
        }
        result
    }
    pub fn range(&self) -> f32 {
        if self.kind == Some(ThreatKind::Ufo) && self.state == ThreatState::Active {
            self.config.ufo_light_range
        } else {
            self.config.interaction_distance
        }
    }
    pub fn refine(
        &self,
        id: &str,
        origin: Vector3,
        direction: Vector3,
        range: f32,
        broad: f32,
    ) -> Option<f32> {
        if id == "threat:pickup" || id == "threat:place" {
            if direction.y.abs() < 0.00001 {
                return None;
            }
            let center = if id == "threat:pickup" {
                self.cover_position()
            } else {
                self.hole_position()
            } + Vector3::up() * 0.12;
            let t = (center.y - origin.y) / direction.y;
            let p = origin + direction * t - center;
            return (t >= 0.0 && t <= range && p.x * p.x + p.z * p.z <= 0.86 * 0.86).then_some(t);
        }
        if let Some(i) = id
            .strip_prefix("threat:light-")
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|i| *i < 3)
        {
            let m = origin - self.light_position(i);
            let b = m.dot(direction);
            let discriminant = b * b - (m.dot(m) - 0.25 * 0.25);
            if discriminant < 0.0 {
                return None;
            }
            let t = (-b - discriminant.sqrt()).max(0.0);
            return (t <= range).then_some(t);
        }
        (broad <= range).then_some(broad)
    }
    pub fn interact(&mut self, id: &str, kenny: &mut Kenny) -> bool {
        if self.state != ThreatState::Active {
            return false;
        }
        match (self.kind, id) {
            (Some(ThreatKind::Bus), "threat:brake") => {
                self.success(kenny);
                true
            }
            (Some(ThreatKind::Ufo), _) => {
                let Some(i) = id
                    .strip_prefix("threat:light-")
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|i| *i < 3)
                else {
                    return false;
                };
                if !self.lights[i] {
                    return false;
                }
                self.lights[i] = false;
                if self.lights.iter().all(|on| !*on) {
                    self.state = ThreatState::Resolving;
                    self.state_time = 0.0;
                }
                true
            }
            (Some(ThreatKind::Sewer), "threat:pickup") if !self.carrying_lid => {
                self.carrying_lid = true;
                true
            }
            (Some(ThreatKind::Sewer), "threat:place") if self.carrying_lid => {
                self.state = ThreatState::Resolving;
                self.state_time = 0.0;
                true
            }
            _ => false,
        }
    }
    pub fn obstacles(&self) -> Vec<Obstacle> {
        if self.kind == Some(ThreatKind::Bus) && self.state != ThreatState::Inactive {
            vec![Obstacle {
                id: "threat:bus-body".into(),
                bounds: self.bus_bounds(),
            }]
        } else {
            vec![]
        }
    }
}
