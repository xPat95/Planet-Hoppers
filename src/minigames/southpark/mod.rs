use crate::minigames::common::{
    collision::{Aabb, CollisionWorld, Obstacle},
    fps::{FpsConfig, FpsController},
    interaction::InteractionTarget,
    manifest::{from_blender, SceneManifest},
};
use raylib::prelude::Vector3;
use serde::Deserialize;
use std::path::Path;
pub mod kenny;
pub mod navigation;
pub mod threats;
use kenny::{Kenny, KENNY_ID};
use navigation::RoadNetwork;
use threats::{ThreatDirector, ThreatKind, PREFIX};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KennyDialogue {
    Closed,
    Invitation,
    Preparing,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevConfig {
    pub player: FpsConfig,
    pub interaction_distance: f32,
    pub test_objects: bool,
    #[serde(default = "default_hint")]
    pub kenny_hint: bool,
}
fn default_hint() -> bool {
    true
}
impl DevConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        let p = &config.player;
        if ![
            p.walk_speed,
            p.run_speed,
            p.mouse_sensitivity,
            p.radius,
            p.body_height,
            p.fov_degrees,
            p.pitch_limit_degrees,
            config.interaction_distance,
        ]
        .iter()
        .all(|n| n.is_finite() && *n > 0.0)
            || p.run_speed < p.walk_speed
            || p.pitch_limit_degrees >= 89.9
            || !(30.0..=110.0).contains(&p.fov_degrees)
            || p.radius > 1.0
            || p.body_height > 3.0
        {
            return Err("Invalid FPS/interaction configuration".into());
        }
        Ok(config)
    }
}

pub struct SouthParkScene {
    pub world: CollisionWorld,
    pub player: FpsController,
    pub targets: Vec<InteractionTarget>,
    pub interaction_distance: f32,
    pub confirmation: Option<(String, f32)>,
    pub interaction_count: u32,
    pub kenny: Kenny,
    pub roads: RoadNetwork,
    pub dialogue: KennyDialogue,
    pub kenny_hint: bool,
    pub threats: ThreatDirector,
    kenny_obstacle: usize,
}

impl SouthParkScene {
    pub fn load(
        manifest: &SceneManifest,
        config: DevConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut world = manifest.collision_world()?;
        if config.player.body_height < manifest.spawn.eye_height {
            return Err("Player body must contain the eye height".into());
        }
        let player = FpsController::new(
            from_blender(manifest.spawn.position),
            from_blender(manifest.spawn.forward),
            manifest.spawn.eye_height,
            config.player,
        );
        if !world.can_occupy(player.feet, player.config.radius, player.config.body_height) {
            return Err("Player spawn intersects an obstacle".into());
        }
        let mut targets = Vec::new();
        if config.test_objects {
            for (index, (label, offset)) in [
                ("Caja", Vector3::new(3.2, 0.0, 0.0)),
                ("Cartel", Vector3::new(1.8, 0.0, -2.0)),
                ("Interruptor", Vector3::new(1.8, 0.0, 2.0)),
            ]
            .into_iter()
            .enumerate()
            {
                let base = player.feet + offset;
                let bounds = Aabb::new(
                    base + Vector3::new(-0.35, 0.02, -0.35),
                    base + Vector3::new(0.35, 1.95, 0.35),
                );
                if world.obstacles.iter().any(|o| o.bounds.overlaps(bounds)) {
                    return Err("Temporary test object intersects scenery".into());
                }
                let id = format!("dev-probe-{index}");
                world.obstacles.push(Obstacle {
                    id: id.clone(),
                    bounds,
                });
                targets.push(InteractionTarget {
                    id,
                    label: label.into(),
                    bounds,
                    action: "dev-confirm".into(),
                });
            }
        }
        let roads = RoadNetwork::build(&manifest.roads, &world)?;
        // The initial approach must not target the occupied player spawn.
        world.obstacles.push(Obstacle {
            id: "fps-player".into(),
            bounds: world.body(player.feet, player.config.radius, player.config.body_height),
        });
        let kenny = Kenny::new(player.feet + Vector3::new(3.4, 0.0, -1.3), &roads, &world);
        world.obstacles.pop();
        let kenny = kenny?;
        let kenny_obstacle = world.obstacles.len();
        world.obstacles.push(Obstacle {
            id: KENNY_ID.into(),
            bounds: kenny.body(&world),
        });
        targets.push(InteractionTarget {
            id: KENNY_ID.into(),
            label: "Kenny".into(),
            bounds: kenny.body(&world),
            action: "talk-kenny".into(),
        });
        Ok(Self {
            world,
            player,
            targets,
            interaction_distance: config.interaction_distance,
            confirmation: None,
            interaction_count: 0,
            kenny,
            roads,
            dialogue: KennyDialogue::Closed,
            kenny_hint: config.kenny_hint,
            kenny_obstacle,
            threats: ThreatDirector::new(threats::ThreatConfig::load(Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/config/southpark-threats.json"
            )))?),
        })
    }
    pub fn interact(&mut self, index: usize) {
        if let Some(target) = self.targets.get(index) {
            if target.id.starts_with(PREFIX) {
                self.threats.interact(&target.id, &mut self.kenny);
                return;
            }
            if target.action == "talk-kenny" {
                if self.threats.owns_kenny() {
                    return;
                }
                self.dialogue = KennyDialogue::Invitation;
                self.kenny.pause();
                return;
            }
            if target.action == "dev-confirm" {
                self.interaction_count += 1;
                let text = format!(
                    "Interaccion confirmada: {} ({})",
                    target.label, self.interaction_count
                );
                println!("{} [{}]", text, target.id);
                self.confirmation = Some((text, 2.5));
            }
        }
    }
    pub fn tick_confirmation(&mut self, dt: f32) {
        if let Some((_, remaining)) = &mut self.confirmation {
            *remaining -= dt;
            if *remaining <= 0.0 {
                self.confirmation = None;
            }
        }
    }
    pub fn update_kenny(&mut self, dt: f32) {
        let player_body = self.world.body(
            self.player.feet,
            self.player.config.radius,
            self.player.config.body_height,
        );
        self.world.obstacles.push(Obstacle {
            id: "fps-player".into(),
            bounds: player_body,
        });
        if self.threats.owns_kenny() {
            self.threats.update(dt, &mut self.kenny);
        } else {
            self.kenny.update(dt, &self.roads, &self.world);
            self.threats.update(dt, &mut self.kenny);
        }
        self.world.obstacles.pop();
        if self.threats.result.is_some()
            && self.threats.state_time >= 6.0
            && !self.threats.released
            && matches!(
                self.threats.kind,
                Some(ThreatKind::Piano | ThreatKind::Snowman)
            )
        {
            let mut clean = self.world.clone();
            clean.obstacles.retain(|o| !o.id.starts_with(PREFIX));
            clean.obstacles.push(Obstacle {
                id: "fps-player".into(),
                bounds: player_body,
            });
            self.threats.release_completed(&mut self.kenny, &clean);
        }
        self.world.obstacles[self.kenny_obstacle].bounds = self.kenny.body(&self.world);
        self.sync_threats();
    }
    pub fn kenny_target(&self) -> usize {
        self.targets
            .iter()
            .position(|t| t.id == KENNY_ID)
            .expect("Kenny target is registered")
    }
    pub fn accept_challenge(&mut self) {
        if self.dialogue == KennyDialogue::Invitation {
            self.dialogue = KennyDialogue::Preparing;
        }
    }
    pub fn close_dialogue(&mut self) {
        self.dialogue = KennyDialogue::Closed;
        self.kenny.resume();
    }
    fn sync_threats(&mut self) {
        self.targets.retain(|t| !t.id.starts_with(PREFIX));
        self.targets.extend(self.threats.targets());
        self.world.obstacles.retain(|o| !o.id.starts_with(PREFIX));
        self.world.obstacles.extend(self.threats.obstacles());
        // Moving bus pushes a player standing in its lane to a checked roadside
        // point. It cannot trap the FPS body inside its next-frame volume.
        if !self.world.can_occupy(
            self.player.feet,
            self.player.config.radius,
            self.player.config.body_height,
        ) {
            for dz in [-2.5, 2.5, -4.0, 4.0] {
                let candidate = self.player.feet + Vector3::new(0.0, 0.0, dz);
                if self.world.can_occupy(
                    candidate,
                    self.player.config.radius,
                    self.player.config.body_height,
                ) {
                    self.player.feet = candidate;
                    break;
                }
            }
        }
    }
    pub fn start_threat(&mut self, kind: ThreatKind) -> Result<(), String> {
        if self.dialogue != KennyDialogue::Closed {
            return Err("Cierra el diálogo antes de activar una amenaza".into());
        }
        let mut clean = self.world.clone();
        clean.obstacles.retain(|o| !o.id.starts_with(PREFIX));
        clean.obstacles.push(Obstacle {
            id: "fps-player".into(),
            bounds: clean.body(
                self.player.feet,
                self.player.config.radius,
                self.player.config.body_height,
            ),
        });
        self.threats
            .start(kind, &mut self.kenny, &clean, &self.roads)?;
        self.world.obstacles[self.kenny_obstacle].bounds = self.kenny.body(&self.world);
        self.sync_threats();
        Ok(())
    }
    pub fn restart_threat(&mut self) -> Result<(), String> {
        let mut clean = self.world.clone();
        clean.obstacles.retain(|o| !o.id.starts_with(PREFIX));
        clean.obstacles.push(Obstacle {
            id: "fps-player".into(),
            bounds: clean.body(
                self.player.feet,
                self.player.config.radius,
                self.player.config.body_height,
            ),
        });
        self.threats.restart(&mut self.kenny, &clean, &self.roads)?;
        self.world.obstacles[self.kenny_obstacle].bounds = self.kenny.body(&self.world);
        self.sync_threats();
        Ok(())
    }
    pub fn clear_threat(&mut self) -> Result<(), String> {
        let mut clean = self.world.clone();
        clean.obstacles.retain(|o| !o.id.starts_with(PREFIX));
        clean.obstacles.push(Obstacle {
            id: "fps-player".into(),
            bounds: clean.body(
                self.player.feet,
                self.player.config.radius,
                self.player.config.body_height,
            ),
        });
        self.threats.clear(&mut self.kenny, &clean)?;
        self.world.obstacles[self.kenny_obstacle].bounds = self.kenny.body(&self.world);
        self.sync_threats();
        Ok(())
    }
}
