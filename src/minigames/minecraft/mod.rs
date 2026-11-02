//! Surface exploration only. FPS, collision and target selection stay shared.
use crate::minigames::common::{
    collision::CollisionWorld,
    fps::{FpsConfig, FpsController, PlayerInput},
    interaction::{select_target, InteractionTarget},
    manifest::{from_blender, from_blender_bounds, SceneManifest},
};
use raylib::prelude::Vector3;
use serde::Deserialize;
use std::path::Path;
pub mod mine;
pub mod mining;
pub mod rush;
pub mod sound;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinecraftConfig {
    pub player: FpsConfig,
    pub interaction_distance: f32,
}
impl MinecraftConfig {
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
        .all(|v| v.is_finite() && *v > 0.0)
            || p.run_speed < p.walk_speed
            || p.radius > 1.0
            || p.body_height > 3.0
            || p.pitch_limit_degrees >= 89.9
            || !(30.0..=110.0).contains(&p.fov_degrees)
            || config.interaction_distance > 6.0
        {
            return Err("Invalid Minecraft FPS/interaction configuration".into());
        }
        Ok(config)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceState {
    Exploring,
    MineMenu,
    ChallengePending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    None,
    Start,
    Cancel,
}

#[derive(Clone, Copy, Debug)]
pub struct SurfacePose {
    pub feet: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

/// Phase 3 can consume this request when an actual mine scene exists.
/// Phase 2 only displays a pending confirmation and never loads another scene.
#[derive(Clone, Copy, Debug)]
pub struct ChallengeRequest {
    pub return_pose: SurfacePose,
    pub entrance: Vector3,
}

#[derive(Deserialize)]
struct Landmark {
    name: String,
    position: Option<[f32; 3]>,
}
#[derive(Deserialize)]
struct Landmarks {
    landmarks: Vec<Landmark>,
}

pub struct MinecraftScene {
    pub manifest: SceneManifest,
    pub world: CollisionWorld,
    pub player: FpsController,
    pub targets: Vec<InteractionTarget>,
    pub state: SurfaceState,
    pub selected: Option<usize>,
    pub pending_request: Option<ChallengeRequest>,
    pub confirmation: Option<String>,
    pub interaction_distance: f32,
}
impl MinecraftScene {
    pub fn load(path: &Path, config: MinecraftConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest = SceneManifest::load(path)?;
        if manifest.id != "minecraft.diamond-rush.environment" {
            return Err("Not a DIAMOND RUSH surface manifest".into());
        }
        let world = manifest.collision_world()?;
        let player = FpsController::new(
            from_blender(manifest.spawn.position),
            from_blender(manifest.spawn.forward),
            manifest.spawn.eye_height,
            config.player,
        );
        if player.eye_height > player.config.body_height
            || !world.can_occupy(player.feet, player.config.radius, player.config.body_height)
        {
            return Err("Minecraft spawn is blocked or has an invalid eye height".into());
        }
        let landmarks: Landmarks = serde_json::from_slice(&std::fs::read(path)?)?;
        let entrance = landmarks
            .landmarks
            .iter()
            .find(|l| l.name == "Mine_interaction_reserved")
            .and_then(|l| l.position)
            .ok_or("Missing reserved mine interaction marker")?;
        if !entrance.iter().all(|n| n.is_finite()) {
            return Err("Invalid mine marker".into());
        }
        let mut targets = vec![InteractionTarget {
            id: "minecraft-mine-entrance".into(),
            label: "Explorar mina".into(),
            action: "mine-menu".into(),
            bounds: from_blender_bounds(
                [entrance[0] - 2.3, entrance[1] - 0.1, 0.0],
                [entrance[0] + 2.3, entrance[1] + 0.9, 3.8],
            ),
        }];
        // Reuse the existing table and its exact collider, without new geometry.
        if let Some(table) = world.obstacles.iter().find(|o| o.id == "crafting_table") {
            targets.push(InteractionTarget {
                id: table.id.clone(),
                label: "Inspeccionar mesa".into(),
                bounds: table.bounds,
                action: "inspect-table".into(),
            });
        }
        Ok(Self {
            manifest,
            world,
            player,
            targets,
            state: SurfaceState::Exploring,
            selected: None,
            pending_request: None,
            confirmation: None,
            interaction_distance: config.interaction_distance,
        })
    }
    pub fn pose(&self) -> SurfacePose {
        SurfacePose {
            feet: self.player.feet,
            yaw: self.player.yaw,
            pitch: self.player.pitch,
        }
    }
    pub fn update(&mut self, input: PlayerInput, dt: f32, controls_enabled: bool) {
        self.selected = None;
        if self.state != SurfaceState::Exploring || !controls_enabled {
            return;
        }
        self.player.update(input, dt, &self.world);
        self.selected = select_target(
            &self.targets,
            &self.world,
            self.player.eye(),
            self.player.direction(),
            self.interaction_distance,
        );
        if input.interact {
            if let Some(index) = self.selected {
                match self.targets[index].action.as_str() {
                    "mine-menu" => {
                        self.state = SurfaceState::MineMenu;
                        self.confirmation = None;
                    }
                    "inspect-table" => {
                        self.confirmation = Some("Mesa inspeccionada. Crafting pendiente.".into())
                    }
                    _ => {}
                }
            }
        }
    }
    pub fn menu_action(&mut self, action: MenuAction) {
        match (self.state, action) {
            (SurfaceState::MineMenu, MenuAction::Start) => {
                self.pending_request = Some(ChallengeRequest {
                    return_pose: self.pose(),
                    entrance: self.targets[0].bounds.center(),
                });
                self.state = SurfaceState::ChallengePending;
            }
            (SurfaceState::MineMenu | SurfaceState::ChallengePending, MenuAction::Cancel) => {
                self.state = SurfaceState::Exploring;
                self.pending_request = None;
                self.selected = None;
            }
            _ => {}
        }
    }
}
