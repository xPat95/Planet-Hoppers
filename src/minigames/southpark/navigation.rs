//! A small deterministic road graph. Splits crossings and validates every edge
//! with the existing swept volume, including temporary development obstacles.
use crate::minigames::common::{
    collision::CollisionWorld,
    manifest::{from_blender, Road},
};
use raylib::prelude::Vector3;

pub const NPC_RADIUS: f32 = 0.28;
pub const NPC_HEIGHT: f32 = 1.35;

pub struct RoadNetwork {
    pub nodes: Vec<Vector3>,
    pub neighbors: Vec<Vec<usize>>,
}

fn cross(a: Vector3, b: Vector3) -> f32 {
    a.x * b.z - a.z * b.x
}

pub fn clear_segment(world: &CollisionWorld, a: Vector3, b: Vector3, ignore: Option<&str>) -> bool {
    let body = world.body(b, NPC_RADIUS, NPC_HEIGHT);
    !world
        .obstacles
        .iter()
        .filter(|o| Some(o.id.as_str()) != ignore)
        .any(|o| body.overlaps(o.bounds))
        && (world.move_body_ignoring(a, b - a, NPC_RADIUS, NPC_HEIGHT, ignore) - b).length() < 0.002
}

impl RoadNetwork {
    pub fn build(
        roads: &[Road],
        world: &CollisionWorld,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let segments: Vec<_> = roads
            .iter()
            .flat_map(|road| {
                road.points
                    .windows(2)
                    .map(|p| (from_blender(p[0]), from_blender(p[1])))
            })
            .collect();
        let mut graph = Self {
            nodes: Vec::new(),
            neighbors: Vec::new(),
        };
        for &(a, b) in &segments {
            let d = b - a;
            if d.length() < 0.01 {
                continue;
            }
            let steps = (d.length() / 4.0).ceil() as usize;
            let mut cuts: Vec<f32> = (0..=steps).map(|i| i as f32 / steps as f32).collect();
            for &(c, e) in &segments {
                let other = e - c;
                let denominator = cross(d, other);
                if denominator.abs() < 0.0001 {
                    continue;
                }
                let t = cross(c - a, other) / denominator;
                let u = cross(c - a, d) / denominator;
                if (-0.00001..=1.00001).contains(&t) && (-0.00001..=1.00001).contains(&u) {
                    cuts.push(t.clamp(0.0, 1.0));
                }
            }
            cuts.sort_by(f32::total_cmp);
            cuts.dedup_by(|a, b| (*a - *b).abs() < 0.00001);
            let mut previous: Option<usize> = None;
            for t in cuts {
                let point = a + d * t;
                if !world.can_occupy(point, NPC_RADIUS, NPC_HEIGHT) {
                    previous = None;
                    continue;
                }
                let index = if let Some(index) = graph
                    .nodes
                    .iter()
                    .position(|p| (*p - point).length() < 0.02)
                {
                    index
                } else {
                    graph.nodes.push(point);
                    graph.neighbors.push(Vec::new());
                    graph.nodes.len() - 1
                };
                if let Some(last) = previous {
                    if clear_segment(world, graph.nodes[last], point, None) {
                        if !graph.neighbors[last].contains(&index) {
                            graph.neighbors[last].push(index);
                        }
                        if !graph.neighbors[index].contains(&last) {
                            graph.neighbors[index].push(last);
                        }
                    }
                }
                previous = Some(index);
            }
        }
        for neighbors in &mut graph.neighbors {
            neighbors.sort_unstable();
        }
        if graph.nodes.len() < 2 {
            return Err("No navigable South Park road network".into());
        }
        Ok(graph)
    }
    pub fn nearest_accessible(
        &self,
        feet: Vector3,
        world: &CollisionWorld,
        ignore: Option<&str>,
    ) -> Option<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                !self.neighbors[*i].is_empty() && clear_segment(world, feet, **p, ignore)
            })
            .min_by(|a, b| (*a.1 - feet).length().total_cmp(&(*b.1 - feet).length()))
            .map(|(i, _)| i)
    }
}
