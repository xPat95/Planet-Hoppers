use super::navigation::{clear_segment, RoadNetwork, NPC_HEIGHT, NPC_RADIUS};
use crate::minigames::common::collision::{Aabb, CollisionWorld};
use raylib::prelude::Vector3;
use std::collections::VecDeque;

pub const KENNY_ID: &str = "southpark-kenny";

#[derive(Default)]
pub struct WalkAnimation {
    pub phase: f32,
    pub weight: f32,
}
impl WalkAnimation {
    pub fn update(&mut self, dt: f32, distance: f32) {
        let walking = distance > 0.00001;
        self.phase =
            (self.phase + distance * std::f32::consts::TAU / 0.9).rem_euclid(std::f32::consts::TAU);
        let target = if walking { 1.0 } else { 0.0 };
        self.weight += (target - self.weight) * (1.0 - (-12.0 * dt).exp());
    }
    pub fn roll(&self) -> f32 {
        10.0_f32.to_radians() * self.phase.sin() * self.weight
    }
    pub fn bounce(&self) -> f32 {
        0.025 * (self.phase * 2.0).sin().abs() * self.weight
    }
}

pub struct Kenny {
    feet: Vector3,
    pub animation: WalkAnimation,
    destination: Option<usize>,
    previous: Option<usize>,
    visits: usize,
    wait: f32,
    paused: bool,
    walking: bool,
    blocked_for: f32,
    avoid_node: Option<usize>,
    override_route: VecDeque<Vector3>,
    pub danger_animation_requested: bool,
    contact: Option<String>,
}

impl Kenny {
    pub fn new(
        feet: Vector3,
        network: &RoadNetwork,
        world: &CollisionWorld,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let destination = network
            .nearest_accessible(feet, world, None)
            .ok_or("Kenny cannot reach a road from his spawn")?;
        if !world.can_occupy(feet, NPC_RADIUS, NPC_HEIGHT) {
            return Err("Kenny spawn is obstructed".into());
        }
        Ok(Self {
            feet,
            animation: WalkAnimation::default(),
            destination: Some(destination),
            previous: None,
            visits: 0,
            wait: 0.4,
            paused: false,
            walking: false,
            blocked_for: 0.0,
            avoid_node: None,
            override_route: VecDeque::new(),
            danger_animation_requested: false,
            contact: None,
        })
    }
    pub fn position(&self) -> Vector3 {
        self.feet
    }
    pub fn is_walking(&self) -> bool {
        self.walking
    }
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    pub fn pause(&mut self) {
        self.paused = true;
        self.walking = false;
    }
    pub fn resume(&mut self) {
        self.paused = false;
    }
    pub fn body(&self, _world: &CollisionWorld) -> Aabb {
        Aabb::new(
            self.feet - Vector3::new(NPC_RADIUS, 0.0, NPC_RADIUS),
            self.feet + Vector3::new(NPC_RADIUS, NPC_HEIGHT, NPC_RADIUS),
        )
    }
    /// Threat director owns validation of its staged corridor and vertical pose.
    /// This does not implement damage, lives or automatic respawn.
    pub(crate) fn event_pose(&mut self, position: Vector3, dt: f32) {
        let movement =
            Vector3::new(position.x - self.feet.x, 0.0, position.z - self.feet.z).length();
        self.feet = position;
        self.walking = movement > 0.00001;
        self.animation.update(dt, movement);
    }
    pub fn request_danger_animation(&mut self, enabled: bool) {
        self.danger_animation_requested = enabled;
    }
    pub fn notify_threat_contact(&mut self, source: &str) {
        self.contact = Some(source.into());
    }
    pub fn take_threat_contact(&mut self) -> Option<String> {
        self.contact.take()
    }
    pub fn set_temporary_route(&mut self, points: &[Vector3], world: &CollisionWorld) -> bool {
        let mut previous = self.feet;
        for &p in points {
            if ![p.x, p.y, p.z].iter().all(|n| n.is_finite())
                || (p.y - world.ground_y).abs() > 0.01
                || !clear_segment(world, previous, p, Some(KENNY_ID))
            {
                return false;
            }
            previous = p;
        }
        self.override_route = points.iter().copied().collect();
        self.destination = None;
        self.wait = 0.0;
        true
    }
    pub fn reposition_safe(&mut self, position: Vector3, world: &CollisionWorld) -> bool {
        if position.x < world.terrain.min.x + NPC_RADIUS
            || position.x > world.terrain.max.x - NPC_RADIUS
            || position.z < world.terrain.min.z + NPC_RADIUS
            || position.z > world.terrain.max.z - NPC_RADIUS
        {
            return false;
        }
        if ![position.x, position.y, position.z]
            .iter()
            .all(|n| n.is_finite())
            || (position.y - world.ground_y).abs() > 0.01
            || world
                .obstacles
                .iter()
                .filter(|o| o.id != KENNY_ID)
                .any(|o| {
                    world
                        .body(position, NPC_RADIUS, NPC_HEIGHT)
                        .overlaps(o.bounds)
                })
        {
            return false;
        }
        self.feet = Vector3::new(position.x, world.ground_y, position.z);
        self.destination = None;
        self.previous = None;
        self.override_route.clear();
        self.animation = WalkAnimation::default();
        self.walking = false;
        self.blocked_for = 0.0;
        self.avoid_node = None;
        true
    }
    pub fn update(&mut self, dt: f32, network: &RoadNetwork, world: &CollisionWorld) {
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.1)
        } else {
            0.0
        };
        let mut traveled = 0.0;
        let mut time = if self.paused { 0.0 } else { dt };
        for _ in 0..16 {
            if time < 0.000001 {
                break;
            }
            if self.wait > 0.0 {
                let consumed = self.wait.min(time);
                self.wait -= consumed;
                time -= consumed;
                continue;
            }
            if self.override_route.is_empty() && self.destination.is_none() {
                self.destination = network.nearest_accessible(self.feet, world, Some(KENNY_ID));
            }
            let target = if let Some(p) = self.override_route.front() {
                *p
            } else if let Some(index) = self.destination {
                network.nodes[index]
            } else {
                break;
            };
            let delta = target - self.feet;
            let distance = delta.length();
            if distance < 0.002 {
                if self.override_route.pop_front().is_none() {
                    if let Some(index) = self.destination {
                        let mut options: Vec<_> = network.neighbors[index]
                            .iter()
                            .copied()
                            .filter(|p| {
                                (Some(*p) != self.previous || network.neighbors[index].len() == 1)
                                    && Some(*p) != self.avoid_node
                            })
                            .collect();
                        if options.is_empty() {
                            options = network.neighbors[index]
                                .iter()
                                .copied()
                                .filter(|p| Some(*p) != self.avoid_node)
                                .collect();
                        }
                        if options.is_empty() {
                            options = network.neighbors[index].clone();
                        }
                        self.destination = options.get(self.visits % options.len().max(1)).copied();
                        self.previous = Some(index);
                        self.visits += 1;
                        if self.visits.is_multiple_of(5) || network.neighbors[index].len() > 2 {
                            self.wait = 0.8;
                        }
                    }
                }
                continue;
            }
            let travel = (1.3 * time).min(distance);
            let proposed = world.move_body_ignoring(
                self.feet,
                delta * (travel / distance),
                NPC_RADIUS,
                NPC_HEIGHT,
                Some(KENNY_ID),
            );
            let actual = (proposed - self.feet).length();
            traveled += actual;
            self.feet = proposed;
            time -= travel / 1.3;
            if actual < travel * 0.25 {
                self.blocked_for += dt;
                if self.blocked_for > 1.5 && self.override_route.is_empty() {
                    let blocked = self.destination;
                    self.destination = network.nearest_accessible(self.feet, world, Some(KENNY_ID));
                    self.avoid_node = blocked;
                    self.previous = None;
                    self.blocked_for = 0.0;
                    self.wait = 0.3;
                }
                break;
            } else {
                self.blocked_for = 0.0;
            }
        }
        self.walking = traveled > 0.00001;
        self.animation.update(dt, traveled);
    }
}
