//! Selective mining session. Detection, timing, removal and reward use stable manifest IDs.
use super::mine::MineScene;
use crate::minigames::common::{
    collision::{Aabb, CollisionWorld},
    interaction::{select_target, InteractionTarget},
    manifest::{from_blender, from_blender_bounds},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Stone,
    Coal,
    Iron,
    Lapis,
    Redstone,
    Gold,
    Diamond,
}
impl BlockKind {
    pub fn from_manifest(kind: &str) -> Option<Self> {
        Some(match kind {
            "breakable" => Self::Stone,
            "coal" => Self::Coal,
            "iron" => Self::Iron,
            "lapis" => Self::Lapis,
            "redstone" => Self::Redstone,
            "gold" => Self::Gold,
            "diamond" => Self::Diamond,
            _ => return None,
        })
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Stone => "Piedra",
            Self::Coal => "Carbón",
            Self::Iron => "Hierro",
            Self::Lapis => "Lapislázuli",
            Self::Redstone => "Redstone",
            Self::Gold => "Oro",
            Self::Diamond => "Diamante",
        }
    }
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Durations {
    pub stone: f32,
    pub coal: f32,
    pub iron: f32,
    pub lapis: f32,
    pub redstone: f32,
    pub gold: f32,
    pub diamond: f32,
}
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiningConfig {
    pub reach: f32,
    pub target_diamonds: u32,
    pub swing_period: f32,
    pub impact_phase: f32,
    pub durations: Durations,
}
impl MiningConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        let d = &c.durations;
        if ![
            d.stone,
            d.coal,
            d.iron,
            d.lapis,
            d.redstone,
            d.gold,
            d.diamond,
            c.swing_period,
            c.reach,
        ]
        .iter()
        .all(|n| n.is_finite() && *n > 0.0 && *n <= 60.0)
            || c.reach > 5.0
            || !(0.15..=1.0).contains(&c.swing_period)
            || c.target_diamonds == 0
            || !c.impact_phase.is_finite()
            || !(0.0..1.0).contains(&c.impact_phase)
        {
            return Err("Invalid Minecraft mining configuration".into());
        }
        Ok(c)
    }
    pub fn duration(&self, kind: BlockKind) -> f32 {
        let d = &self.durations;
        match kind {
            BlockKind::Stone => d.stone,
            BlockKind::Coal => d.coal,
            BlockKind::Iron => d.iron,
            BlockKind::Lapis => d.lapis,
            BlockKind::Redstone => d.redstone,
            BlockKind::Gold => d.gold,
            BlockKind::Diamond => d.diamond,
        }
    }
}
pub struct MiningBlock {
    pub id: String,
    pub kind: BlockKind,
    pub bounds: Aabb,
    pub alive: bool,
    pub gate: Option<String>,
}
pub struct Progress {
    pub block: usize,
    pub elapsed: f32,
}
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MiningEvent {
    SwingStarted { block: String },
    Impact { block: String },
    BlockDestroyed { block: String, material: BlockKind },
    DiamondCollected { block: String, total: u32 },
    CommonOreCollected { block: String, material: BlockKind },
    PassageUnlocked { gate: String },
    SessionReset,
}
pub struct MiningSession {
    pub config: MiningConfig,
    pub blocks: Vec<MiningBlock>,
    pub active: Option<Progress>,
    pub hovered: Option<usize>,
    pub diamonds: u32,
    pub feedback: Option<(String, f32)>,
    pub finish_kick: f32,
    events: Vec<MiningEvent>,
    initial_world: CollisionWorld,
    initial_targets: Vec<InteractionTarget>,
    await_release: bool,
    unlocked: BTreeSet<String>,
}
impl MiningSession {
    pub fn new(scene: &MineScene, config: MiningConfig) -> Self {
        let blocks = scene
            .data
            .interactive_blocks
            .iter()
            .map(|b| MiningBlock {
                id: b.id.clone(),
                kind: BlockKind::from_manifest(&b.kind).expect("Validated mine kind"),
                bounds: from_blender_bounds(b.min, b.max),
                alive: true,
                gate: b.gate.clone(),
            })
            .collect();
        Self {
            config,
            blocks,
            active: None,
            hovered: None,
            diamonds: 0,
            feedback: None,
            finish_kick: 0.0,
            events: vec![],
            initial_world: scene.world.clone(),
            initial_targets: scene.targets.clone(),
            await_release: true,
            unlocked: BTreeSet::new(),
        }
    }
    pub fn cancel(&mut self) {
        self.active = None;
        self.finish_kick = 0.0;
    }
    pub fn progress(&self) -> f32 {
        self.active.as_ref().map_or(0.0, |p| {
            p.elapsed / self.config.duration(self.blocks[p.block].kind)
        })
    }
    pub fn crack_stage(&self) -> usize {
        if self.active.is_none() {
            0
        } else {
            ((self.progress() * 8.0) as usize + 1).min(8)
        }
    }
    pub fn swing_phase(&self) -> Option<f32> {
        self.active
            .as_ref()
            .map(|p| (p.elapsed / self.config.swing_period).fract())
    }
    pub fn drain_events(&mut self) -> Vec<MiningEvent> {
        std::mem::take(&mut self.events)
    }
    pub fn update(&mut self, scene: &mut MineScene, held: bool, enabled: bool, dt: f32) {
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        self.finish_kick = (self.finish_kick - dt).max(0.0);
        if let Some((_, time)) = &mut self.feedback {
            *time -= dt;
            if *time <= 0.0 {
                self.feedback = None;
            }
        }
        self.hovered = None;
        if !enabled {
            self.cancel();
            self.await_release = true;
            return;
        }
        let selected = select_target(
            &scene.targets,
            &scene.world,
            scene.player.eye(),
            scene.player.direction(),
            self.config.reach.min(scene.interaction_distance),
        );
        self.hovered = selected.and_then(|i| {
            self.blocks
                .iter()
                .position(|b| b.alive && b.id == scene.targets[i].id)
        });
        if !held {
            self.cancel();
            self.await_release = false;
            return;
        }
        if self.await_release {
            self.cancel();
            return;
        }
        let Some(index) = self.hovered else {
            self.cancel();
            return;
        };
        if self.active.as_ref().is_none_or(|p| p.block != index) {
            self.active = Some(Progress {
                block: index,
                elapsed: 0.0,
            });
            self.events.push(MiningEvent::SwingStarted {
                block: self.blocks[index].id.clone(),
            });
        }
        let duration = self.config.duration(self.blocks[index].kind);
        let active = self.active.as_mut().unwrap();
        let before = active.elapsed;
        active.elapsed = (active.elapsed + dt).min(duration);
        let after = active.elapsed;
        let period = self.config.swing_period;
        for cycle in 0..=(after / period).ceil() as usize {
            let start = cycle as f32 * period;
            let impact = start + period * self.config.impact_phase;
            if cycle > 0 && before < start && start <= after {
                self.events.push(MiningEvent::SwingStarted {
                    block: self.blocks[index].id.clone(),
                });
            }
            if before < impact && impact <= after {
                self.events.push(MiningEvent::Impact {
                    block: self.blocks[index].id.clone(),
                });
            }
        }
        if after + 1e-6 >= duration {
            self.destroy(scene, index);
        }
    }
    fn destroy(&mut self, scene: &mut MineScene, index: usize) {
        let block = &mut self.blocks[index];
        if !block.alive {
            return;
        }
        block.alive = false;
        let id = block.id.clone();
        let kind = block.kind;
        let gate = block.gate.clone();
        scene.world.obstacles.retain(|c| c.id != id);
        scene.targets.retain(|t| t.id != id);
        scene.selected = None;
        self.events.push(MiningEvent::BlockDestroyed {
            block: id.clone(),
            material: kind,
        });
        if kind == BlockKind::Diamond {
            self.diamonds += 1;
            self.events.push(MiningEvent::DiamondCollected {
                block: id.clone(),
                total: self.diamonds,
            });
            self.feedback = Some(("+1 diamante".into(), 1.5));
        } else {
            if kind != BlockKind::Stone {
                self.events.push(MiningEvent::CommonOreCollected {
                    block: id.clone(),
                    material: kind,
                });
            }
            self.feedback = Some((format!("{} extraído", kind.label()), 1.2));
        }
        if let Some(gate) = gate {
            let open = self
                .blocks
                .iter()
                .filter(|b| b.gate.as_ref() == Some(&gate))
                .any(|b| {
                    let center = b.bounds.center();
                    let feet =
                        raylib::prelude::Vector3::new(center.x, scene.world.ground_y, center.z);
                    scene.world.can_occupy(
                        feet,
                        scene.player.config.radius,
                        scene.player.config.body_height,
                    )
                });
            if open && self.unlocked.insert(gate.clone()) {
                self.events.push(MiningEvent::PassageUnlocked { gate });
            }
        }
        self.active = None;
        self.hovered = None;
        self.finish_kick = 0.12;
    }
    pub fn reset_mining_session(&mut self, scene: &mut MineScene) {
        scene.world = self.initial_world.clone();
        scene.targets = self.initial_targets.clone();
        scene.selected = None;
        let eye = from_blender(scene.manifest.spawn.position);
        let forward = from_blender(scene.manifest.spawn.forward);
        scene.player.feet = eye - raylib::prelude::Vector3::up() * scene.player.eye_height;
        scene.player.yaw = forward.z.atan2(forward.x);
        scene.player.pitch = 0.0;
        for b in &mut self.blocks {
            b.alive = true;
        }
        self.diamonds = 0;
        self.active = None;
        self.hovered = None;
        self.feedback = None;
        self.finish_kick = 0.0;
        self.await_release = true;
        self.unlocked.clear();
        self.events.clear();
        self.events.push(MiningEvent::SessionReset);
    }
}
