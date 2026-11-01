//! Independent mine development scene; interactive blocks are data, never extracted here.
use super::MinecraftConfig;
use crate::minigames::common::{
    collision::{Aabb, CollisionWorld},
    fps::{FpsController, PlayerInput},
    interaction::{select_target, InteractionTarget},
    manifest::{from_blender, from_blender_bounds, SceneManifest},
};
use raylib::prelude::Vector3;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeSet, VecDeque},
    path::Path,
};

#[derive(Deserialize)]
pub struct MineBlock {
    pub id: String,
    pub object: String,
    pub collider_id: String,
    pub kind: String,
    pub room: String,
    pub sector: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub approach: Option<[f32; 3]>,
    pub initial_visible: bool,
    pub initial_solid: bool,
}
#[derive(Deserialize)]
pub struct LavaRegion {
    pub id: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub damage_enabled: bool,
}
#[derive(Deserialize)]
pub struct MineLight {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub radius: f32,
}
#[derive(Deserialize)]
pub struct FloorCell {
    pub x: i32,
    pub y: i32,
    pub ceiling: f32,
}
#[derive(Deserialize)]
pub struct Chamber {
    pub id: String,
    pub sector: String,
    pub center: [f32; 3],
}
#[derive(Deserialize)]
pub struct Tunnel {
    pub id: String,
    pub a: String,
    pub b: String,
    pub requires_break: Vec<String>,
}
#[derive(Deserialize)]
pub struct MineGraph {
    pub nodes: Vec<Chamber>,
    pub edges: Vec<Tunnel>,
}
#[derive(Deserialize)]
pub struct MineExit {
    pub id: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub surface_position: [f32; 3],
    pub surface_forward: [f32; 3],
}
#[derive(Deserialize)]
pub struct MineData {
    pub interactive_blocks: Vec<MineBlock>,
    pub lava_regions: Vec<LavaRegion>,
    pub lights: Vec<MineLight>,
    pub floor_cells: Vec<FloorCell>,
    pub graph: MineGraph,
    pub exit: MineExit,
}
#[derive(Serialize)]
pub struct LayoutAudit {
    pub future_gates_open: bool,
    pub safe_reachable_diamonds: Vec<String>,
    pub reachable_chambers: Vec<String>,
    pub visited_cells: usize,
}

pub struct MineScene {
    pub manifest: SceneManifest,
    pub data: MineData,
    pub world: CollisionWorld,
    pub player: FpsController,
    pub targets: Vec<InteractionTarget>,
    pub selected: Option<usize>,
    pub interaction_distance: f32,
}
impl MineScene {
    pub fn load(path: &Path, config: MinecraftConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let manifest = SceneManifest::load(path)?;
        if manifest.id != "minecraft.diamond-rush.mine" {
            return Err("Not a Minecraft mine manifest".into());
        }
        let data: MineData = serde_json::from_slice(&std::fs::read(path)?)?;
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
            return Err("Mine spawn is blocked or eye height is invalid".into());
        }
        let mut ids = BTreeSet::new();
        let mut targets = vec![InteractionTarget {
            id: data.exit.id.clone(),
            label: "Salir del modo de prueba".into(),
            action: "mine-dev-exit".into(),
            bounds: from_blender_bounds(data.exit.min, data.exit.max),
        }];
        for b in &data.interactive_blocks {
            if !ids.insert(&b.id)
                || !matches!(b.kind.as_str(), "diamond" | "breakable")
                || !b.initial_visible
                || !b.initial_solid
                || !b.min.iter().chain(b.max.iter()).all(|n| n.is_finite())
                || b.min.iter().zip(b.max).any(|(a, b)| *a >= b)
            {
                return Err(format!("Invalid prepared mine block: {}", b.id).into());
            }
            let source = manifest
                .colliders
                .iter()
                .find(|c| c.name == b.collider_id)
                .ok_or("Mine block collider missing")?;
            if source.min != b.min || source.max != b.max {
                return Err("Mine block collider bounds differ".into());
            }
            targets.push(InteractionTarget {
                id: b.collider_id.clone(),
                label: if b.kind == "diamond" {
                    "Mineral de diamante preparado"
                } else {
                    "Piedra rompible preparada"
                }
                .into(),
                action: "prepared-only".into(),
                bounds: from_blender_bounds(b.min, b.max),
            });
        }
        if data.lights.is_empty()
            || data.lights.len() > 24
            || data.lights.iter().any(|l| {
                !l.radius.is_finite()
                    || l.radius <= 0.0
                    || !l
                        .position
                        .iter()
                        .chain(l.color.iter())
                        .all(|n| n.is_finite())
            })
        {
            return Err("Invalid mine lighting metadata".into());
        }
        if data.lava_regions.iter().any(|r| {
            r.damage_enabled
                || !r.min.iter().chain(r.max.iter()).all(|v| v.is_finite())
                || r.min.iter().zip(r.max).any(|(a, b)| *a >= b)
        }) {
            return Err("Mine lava must remain a valid inactive sensor".into());
        }
        Ok(Self {
            manifest,
            data,
            world,
            player,
            targets,
            selected: None,
            interaction_distance: config.interaction_distance,
        })
    }
    /// Returns only a development exit request. E never removes or awards a block.
    pub fn update(&mut self, input: PlayerInput, dt: f32, enabled: bool) -> bool {
        self.selected = None;
        if !enabled {
            return false;
        }
        self.player.update(input, dt, &self.world);
        self.selected = select_target(
            &self.targets,
            &self.world,
            self.player.eye(),
            self.player.direction(),
            self.interaction_distance,
        );
        input.interact && self.selected == Some(0)
    }
    /// Hypothetical layout check on a clone; the running scene always keeps gates intact.
    pub fn accessibility(&self, open_gates: bool) -> LayoutAudit {
        let mut world = self.world.clone();
        if open_gates {
            let gates: BTreeSet<_> = self
                .data
                .interactive_blocks
                .iter()
                .filter(|b| b.kind == "breakable")
                .map(|b| b.collider_id.as_str())
                .collect();
            world.obstacles.retain(|o| !gates.contains(o.id.as_str()));
        }
        let radius = self.player.config.radius;
        let height = self.player.config.body_height;
        let cells: BTreeSet<_> = self
            .data
            .floor_cells
            .iter()
            .filter_map(|c| {
                let feet = from_blender([c.x as f32 + 0.5, c.y as f32 + 0.5, 0.0]);
                (c.ceiling >= height
                    && world.can_occupy(feet, radius, height)
                    && !self.data.lava_regions.iter().any(|r| {
                        from_blender_bounds(r.min, r.max).overlaps(world.body(feet, radius, height))
                    }))
                .then_some((c.x, c.y))
            })
            .collect();
        let start = (
            self.manifest.spawn.position[0].floor() as i32,
            self.manifest.spawn.position[1].floor() as i32,
        );
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::new();
        if cells.contains(&start) {
            visited.insert(start);
            queue.push_back(start);
        }
        while let Some((x, y)) = queue.pop_front() {
            for next in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
                if !cells.contains(&next) || visited.contains(&next) {
                    continue;
                }
                let a = from_blender([x as f32 + 0.5, y as f32 + 0.5, 0.0]);
                let b = from_blender([next.0 as f32 + 0.5, next.1 as f32 + 0.5, 0.0]);
                if (world.move_body(a, b - a, radius, height) - b).length() < 0.002 {
                    visited.insert(next);
                    queue.push_back(next);
                }
            }
        }
        LayoutAudit {
            future_gates_open: open_gates,
            visited_cells: visited.len(),
            safe_reachable_diamonds: self
                .data
                .interactive_blocks
                .iter()
                .filter(|b| {
                    b.kind == "diamond"
                        && b.approach.is_some_and(|p| {
                            visited.contains(&(p[0].floor() as i32, p[1].floor() as i32))
                        })
                })
                .map(|b| b.id.clone())
                .collect(),
            reachable_chambers: self
                .data
                .graph
                .nodes
                .iter()
                .filter(|n| {
                    visited.contains(&(n.center[0].floor() as i32, n.center[1].floor() as i32))
                })
                .map(|n| n.id.clone())
                .collect(),
        }
    }
    pub fn light_arrays(&self) -> (Vec<Vector3>, Vec<Vector3>, Vec<f32>) {
        (
            self.data
                .lights
                .iter()
                .map(|l| from_blender(l.position))
                .collect(),
            self.data
                .lights
                .iter()
                .map(|l| Vector3::new(l.color[0], l.color[1], l.color[2]))
                .collect(),
            self.data.lights.iter().map(|l| l.radius).collect(),
        )
    }
    pub fn grounded(&self) -> bool {
        self.player.feet.y == self.world.ground_y
            && self.world.can_occupy(
                self.player.feet,
                self.player.config.radius,
                self.player.config.body_height,
            )
    }
    pub fn bounds_for_block(&self, id: &str) -> Option<Aabb> {
        self.data
            .interactive_blocks
            .iter()
            .find(|b| b.id == id)
            .map(|b| from_blender_bounds(b.min, b.max))
    }
}
