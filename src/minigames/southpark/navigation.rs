//! A small deterministic road graph. Splits crossings and validates every edge
//! with the existing swept volume, including temporary development obstacles.
use crate::minigames::common::{
    collision::CollisionWorld,
    manifest::{from_blender, Road},
};
use raylib::prelude::Vector3;

pub const NPC_RADIUS: f32 = 0.28;
pub const NPC_HEIGHT: f32 = 1.35;

/// Event routing runs once at activation. Reuses the road graph, testing its
/// edges with the larger pursuer volume rather than assuming NPC clearance.
pub fn event_route(
    graph: &RoadNetwork,
    world: &CollisionWorld,
    a: Vector3,
    b: Vector3,
    radius: f32,
    height: f32,
) -> Result<Vec<Vector3>, String> {
    let clear = |a: Vector3, b: Vector3| {
        world.can_occupy(a, radius, height)
            && world.can_occupy(b, radius, height)
            && (world.move_body(a, b - a, radius, height) - b).length() < 0.002
    };
    if clear(a, b) {
        return Ok(vec![a, b]);
    }
    let nearest = |p: Vector3| {
        graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| clear(p, **n))
            .min_by(|(_, x), (_, y)| (**x - p).length().total_cmp(&(**y - p).length()))
            .map(|(i, _)| i)
    };
    let start = nearest(a).ok_or("No road access for pursuer")?;
    let end = nearest(b).ok_or("No road access to destination")?;
    let mut parent = vec![usize::MAX; graph.nodes.len()];
    let mut queue = std::collections::VecDeque::from([start]);
    parent[start] = start;
    while let Some(i) = queue.pop_front() {
        if i == end {
            break;
        }
        for &j in &graph.neighbors[i] {
            if parent[j] == usize::MAX && clear(graph.nodes[i], graph.nodes[j]) {
                parent[j] = i;
                queue.push_back(j);
            }
        }
    }
    if parent[end] == usize::MAX {
        return Err("No safe road route for pursuer".into());
    }
    let mut path = vec![b];
    let mut i = end;
    loop {
        path.push(graph.nodes[i]);
        if i == start {
            break;
        }
        i = parent[i];
    }
    path.push(a);
    path.reverse();
    Ok(path)
}

pub fn route_length(route: &[Vector3]) -> f32 {
    route.windows(2).map(|p| (p[1] - p[0]).length()).sum()
}
pub fn route_point(route: &[Vector3], mut distance: f32) -> Vector3 {
    for p in route.windows(2) {
        let length = (p[1] - p[0]).length();
        if distance <= length {
            return p[0] + (p[1] - p[0]) * (distance / length.max(0.0001));
        }
        distance -= length;
    }
    *route.last().expect("Validated nonempty route")
}

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
