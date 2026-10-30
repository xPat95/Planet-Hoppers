//! Metres, Y up. Static AABBs and a swept upright player box; no physics engine.
use raylib::prelude::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub min: Vector3,
    pub max: Vector3,
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    /// Distance for a normalized ray, fraction for a displacement vector.
    pub distance: f32,
    pub normal: Vector3,
}

impl Aabb {
    pub fn new(min: Vector3, max: Vector3) -> Self {
        Self { min, max }
    }
    pub fn expanded(self, amount: Vector3) -> Self {
        Self::new(self.min - amount, self.max + amount)
    }
    pub fn center(self) -> Vector3 {
        (self.min + self.max) * 0.5
    }
    pub fn size(self) -> Vector3 {
        self.max - self.min
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }
    /// Slab intersection, including parallel rays and origins inside the volume.
    pub fn ray(self, origin: Vector3, direction: Vector3, maximum: f32) -> Option<RayHit> {
        if !maximum.is_finite() || maximum < 0.0 {
            return None;
        }
        let mut near: f32 = 0.0;
        let mut far = maximum;
        let mut normal = Vector3::zero();
        for (p, d, lo, hi, axis) in [
            (
                origin.x,
                direction.x,
                self.min.x,
                self.max.x,
                Vector3::right(),
            ),
            (origin.y, direction.y, self.min.y, self.max.y, Vector3::up()),
            (
                origin.z,
                direction.z,
                self.min.z,
                self.max.z,
                Vector3::new(0.0, 0.0, 1.0),
            ),
        ] {
            if d.abs() < 1e-8 {
                if p < lo || p > hi {
                    return None;
                }
            } else {
                let (a, b) = ((lo - p) / d, (hi - p) / d);
                let (entry, exit) = (a.min(b), a.max(b));
                if entry >= near {
                    near = entry;
                    normal = axis * (-d.signum());
                }
                far = far.min(exit);
                if near > far {
                    return None;
                }
            }
        }
        Some(RayHit {
            distance: near,
            normal,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Obstacle {
    pub id: String,
    pub bounds: Aabb,
}

#[derive(Clone, Debug)]
pub struct CollisionWorld {
    pub obstacles: Vec<Obstacle>,
    pub ground_y: f32,
    pub terrain: Aabb,
}

impl CollisionWorld {
    pub fn new(terrain: Aabb, ground_y: f32, mut obstacles: Vec<Obstacle>) -> Self {
        // Solid slabs protect the edge continuously, even under a very large step.
        let (a, b) = (terrain.min, terrain.max);
        for (id, min, max) in [
            (
                "boundary-west",
                Vector3::new(a.x - 1000.0, -1000.0, a.z - 1000.0),
                Vector3::new(a.x, 1000.0, b.z + 1000.0),
            ),
            (
                "boundary-east",
                Vector3::new(b.x, -1000.0, a.z - 1000.0),
                Vector3::new(b.x + 1000.0, 1000.0, b.z + 1000.0),
            ),
            (
                "boundary-north",
                Vector3::new(a.x - 1000.0, -1000.0, a.z - 1000.0),
                Vector3::new(b.x + 1000.0, 1000.0, a.z),
            ),
            (
                "boundary-south",
                Vector3::new(a.x - 1000.0, -1000.0, b.z),
                Vector3::new(b.x + 1000.0, 1000.0, b.z + 1000.0),
            ),
        ] {
            obstacles.push(Obstacle {
                id: id.into(),
                bounds: Aabb::new(min, max),
            });
        }
        Self {
            obstacles,
            ground_y,
            terrain,
        }
    }
    pub fn body(&self, feet: Vector3, radius: f32, height: f32) -> Aabb {
        Aabb::new(
            Vector3::new(feet.x - radius, self.ground_y, feet.z - radius),
            Vector3::new(feet.x + radius, self.ground_y + height, feet.z + radius),
        )
    }
    pub fn can_occupy(&self, feet: Vector3, radius: f32, height: f32) -> bool {
        let body = self.body(feet, radius, height);
        !self.obstacles.iter().any(|o| body.overlaps(o.bounds))
    }
    pub fn ray_distance(&self, origin: Vector3, direction: Vector3, max: f32) -> Option<f32> {
        self.obstacles
            .iter()
            .filter_map(|o| o.bounds.ray(origin, direction, max).map(|h| h.distance))
            .min_by(f32::total_cmp)
    }
    /// Continuous collision detection prevents tunnelling, then projects remaining
    /// motion onto the contact tangent. Four contacts suffice for static box corners.
    pub fn move_body(
        &self,
        feet: Vector3,
        displacement: Vector3,
        radius: f32,
        height: f32,
    ) -> Vector3 {
        let half = Vector3::new(radius, height * 0.5, radius);
        let mut center = Vector3::new(feet.x, self.ground_y + half.y, feet.z);
        let mut remaining = Vector3::new(displacement.x, 0.0, displacement.z);
        for _ in 0..4 {
            if remaining.dot(remaining) < 1e-12 {
                break;
            }
            let hit = self
                .obstacles
                .iter()
                .filter_map(|o| o.bounds.expanded(half).ray(center, remaining, 1.0))
                .filter(|h| remaining.dot(h.normal) < -1e-8)
                .min_by(|a, b| a.distance.total_cmp(&b.distance));
            match hit {
                None => {
                    center += remaining;
                    break;
                }
                Some(hit) => {
                    center += remaining * hit.distance + hit.normal * 0.0001;
                    remaining *= 1.0 - hit.distance;
                    remaining -= hit.normal * remaining.dot(hit.normal).min(0.0);
                }
            }
        }
        Vector3::new(center.x, self.ground_y, center.z)
    }
}
