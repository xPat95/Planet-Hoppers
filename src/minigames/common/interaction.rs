use super::collision::{Aabb, CollisionWorld};
use raylib::prelude::Vector3;

#[derive(Clone, Debug)]
pub struct InteractionTarget {
    pub id: String,
    pub label: String,
    pub bounds: Aabb,
    /// Scene interprets the action. The ray system knows nothing about Kenny.
    pub action: String,
}

pub fn select_target(
    targets: &[InteractionTarget],
    world: &CollisionWorld,
    origin: Vector3,
    direction: Vector3,
    range: f32,
) -> Option<usize> {
    if direction.dot(direction) < 1e-10 || !range.is_finite() || range <= 0.0 {
        return None;
    }
    let direction = direction.normalized();
    let candidate = targets
        .iter()
        .enumerate()
        .filter_map(|(i, target)| {
            target
                .bounds
                .ray(origin, direction, range)
                .map(|hit| (i, hit.distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))?;
    if world
        .ray_distance(origin, direction, range)
        .is_some_and(|d| d < candidate.1 - 0.0001)
    {
        return None;
    }
    Some(candidate.0)
}
