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
    select_target_with(
        targets,
        world,
        origin,
        direction,
        range,
        |_, _, _, _, hit| Some(hit),
    )
}

/// Optional narrow phase refines a broad AABB hit, e.g. an alpha-tested sprite.
/// Both static props and animated targets share range, nearest-hit and occlusion.
pub fn select_target_with(
    targets: &[InteractionTarget],
    world: &CollisionWorld,
    origin: Vector3,
    direction: Vector3,
    range: f32,
    exact_hit: impl Fn(usize, Vector3, Vector3, f32, f32) -> Option<f32>,
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
                .and_then(|hit| exact_hit(i, origin, direction, range, hit.distance))
                .filter(|distance| distance.is_finite() && *distance >= 0.0 && *distance <= range)
                .map(|distance| (i, distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))?;
    if world
        .ray_distance_ignoring(origin, direction, range, Some(&targets[candidate.0].id))
        .is_some_and(|d| d < candidate.1 - 0.0001)
    {
        return None;
    }
    Some(candidate.0)
}
