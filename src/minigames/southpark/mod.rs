use crate::minigames::common::{
    collision::{Aabb, CollisionWorld, Obstacle},
    fps::{FpsConfig, FpsController},
    interaction::InteractionTarget,
    manifest::{from_blender, SceneManifest},
};
use raylib::prelude::Vector3;
use serde::Deserialize;
use std::path::Path;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevConfig {
    pub player: FpsConfig,
    pub interaction_distance: f32,
    pub test_objects: bool,
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
        Ok(Self {
            world,
            player,
            targets,
            interaction_distance: config.interaction_distance,
            confirmation: None,
            interaction_count: 0,
        })
    }
    pub fn interact(&mut self, index: usize) {
        if let Some(target) = self.targets.get(index) {
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
}
