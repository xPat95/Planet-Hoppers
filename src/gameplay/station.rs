//! Fixed point of interest, deliberately separate from planets and orbital simulation.
use raylib::prelude::Vector3;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Station {
    pub name: String,
    pub model: String,
    pub model_far: String,
    pub position: [f32; 3],
    pub rotation_degrees: [f32; 3],
    pub length: f32,
    /// Conservative exterior envelope; traversing the detailed arms is not implemented.
    pub collision_radius: f32,
    pub approach_margin: f32,
    pub interaction_margin: f32,
    pub lod_distance: f32,
}
impl Station {
    pub fn load(root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let s: Self =
            serde_json::from_str(&std::fs::read_to_string(root.join("config/station.json"))?)?;
        if !s
            .position
            .iter()
            .chain(s.rotation_degrees.iter())
            .all(|v| v.is_finite())
            || ![
                s.length,
                s.collision_radius,
                s.approach_margin,
                s.interaction_margin,
                s.lod_distance,
            ]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
            || s.interaction_margin <= s.approach_margin
            || !root.join(&s.model).is_file()
            || !root.join(&s.model_far).is_file()
        {
            return Err("Configuración o recursos de la estación inválidos".into());
        }
        Ok(s)
    }
    pub fn center(&self) -> Vector3 {
        Vector3::new(self.position[0], self.position[1], self.position[2])
    }
    /// Future hangar entry anchor, outside the exterior collision envelope.
    pub fn approach(&self) -> Vector3 {
        self.center() + Vector3::new(0.0, 0.0, self.collision_radius + self.approach_margin)
    }
    pub fn in_reach(&self, point: Vector3) -> bool {
        (point - self.approach()).length() <= self.interaction_margin - self.approach_margin
    }
    /// Swept exterior collision, including turbo and vertical flight; also usable by cameras.
    pub fn push_out(&self, start: Vector3, end: Vector3, margin: f32) -> Option<Vector3> {
        let radius = self.collision_radius + margin;
        let rel = start - self.center();
        if rel.length() < radius {
            let n = if rel.length() > 0.001 {
                rel.normalized()
            } else {
                Vector3::up()
            };
            return Some(self.center() + n * (radius + 0.01));
        }
        let movement = end - start;
        let a = movement.dot(movement);
        let b = rel.dot(movement);
        let c = rel.dot(rel) - radius * radius;
        let disc = b * b - a * c;
        if a <= 1e-8 || b >= 0.0 || disc < 0.0 {
            return None;
        }
        let t = (-b - disc.sqrt()) / a;
        if !(0.0..=1.0).contains(&t) {
            return None;
        }
        let hit = start + movement * t;
        Some(hit + (hit - self.center()).normalized() * 0.01)
    }
}
