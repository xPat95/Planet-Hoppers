//! Fixed point of interest, deliberately separate from planets and orbital simulation.
use crate::ships::{FlightConfig, FlightInput, Ship};
use raylib::prelude::{Matrix, Vector3};
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
    pub dock: Dock,
    pub lod_distance: f32,
}
/// Position is a fraction of the fitted model length, in centered GLB axes.
/// Dimensions are world units; the complementary room occupies the lower ring wall.
#[derive(Debug, Deserialize)]
pub struct Dock {
    pub local_position: [f32; 3],
    pub local_forward: [f32; 3],
    pub local_up: [f32; 3],
    pub width: f32,
    pub height: f32,
    pub room_width: f32,
    pub room_height: f32,
    pub room_depth: f32,
    pub departure_clearance: f32,
    pub approach_distance: f32,
    pub interaction_radius: f32,
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
                s.dock.width,
                s.dock.height,
                s.dock.room_width,
                s.dock.room_height,
                s.dock.room_depth,
                s.dock.departure_clearance,
                s.dock.approach_distance,
                s.dock.interaction_radius,
                s.lod_distance,
            ]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
            || !s
                .dock
                .local_position
                .iter()
                .chain(s.dock.local_forward.iter())
                .chain(s.dock.local_up.iter())
                .all(|v| v.is_finite())
            || s.exit_forward().cross(s.exit_up()).length() < 0.99
            || s.dock.departure_clearance >= s.dock.interaction_radius
            || s.dock.width >= s.dock.room_width
            || s.dock.height >= s.dock.room_height
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
    pub fn rotation(&self) -> Matrix {
        Matrix::rotate_x(self.rotation_degrees[0].to_radians())
            * Matrix::rotate_y(self.rotation_degrees[1].to_radians())
            * Matrix::rotate_z(self.rotation_degrees[2].to_radians())
    }
    fn rotated(&self, v: [f32; 3]) -> Vector3 {
        Vector3::new(v[0], v[1], v[2]).transform_with(self.rotation())
    }
    pub fn exit_position(&self) -> Vector3 {
        self.center() + self.rotated(self.dock.local_position) * self.length
    }
    pub fn exit_forward(&self) -> Vector3 {
        self.rotated(self.dock.local_forward).normalized()
    }
    pub fn exit_up(&self) -> Vector3 {
        self.rotated(self.dock.local_up).normalized()
    }
    pub fn exit_right(&self) -> Vector3 {
        self.exit_forward().cross(self.exit_up()).normalized()
    }
    pub fn departure(&self) -> Vector3 {
        self.exit_position() + self.exit_forward() * self.dock.departure_clearance
    }
    /// Aim farther outside the mouth before the final approach.
    pub fn approach(&self) -> Vector3 {
        self.exit_position() + self.exit_forward() * self.dock.approach_distance
    }
    /// Align outside the hull before entering the narrow access lane.
    pub fn navigation_target(&self, point: Vector3) -> Vector3 {
        let rel = point - self.exit_position();
        if rel.dot(self.exit_forward()) > 0.0
            && rel.dot(self.exit_forward()) < self.dock.approach_distance + self.dock.width
            && rel.dot(self.exit_right()).abs() < self.dock.width * 0.3
            && rel.dot(self.exit_up()).abs() < self.dock.height * 0.3
        {
            self.departure()
        } else {
            self.approach()
        }
    }
    /// B keeps steering; near the narrow mouth it also limits requested forward
    /// speed while aligning, so a turbo approach cannot overshoot into the ring.
    pub fn assist_approach(&self, ship: &Ship, input: &mut FlightInput, flight: &FlightConfig) {
        if input.throttle <= 0.0
            || (ship.position - self.approach()).length() > self.dock.approach_distance * 4.0
        {
            return;
        }
        let route = self.navigation_target(ship.position) - ship.position;
        let alignment = ship.forward().dot(route.normalized()).max(0.0);
        let speed = (route.length() * 0.6).clamp(0.5, flight.cruise_speed) * alignment.powi(8);
        let top = if input.boost {
            flight.boost_speed
        } else {
            flight.cruise_speed
        };
        input.throttle = input.throttle.min((speed / top).max(0.00001));
    }
    pub fn in_reach(&self, point: Vector3) -> bool {
        let rel = point - self.exit_position();
        rel.dot(self.exit_forward()) >= 0.0
            && rel.length() <= self.dock.interaction_radius
            && rel.dot(self.exit_right()).abs() < self.dock.width * 0.5
            && rel.dot(self.exit_up()).abs() < self.dock.height * 0.5
    }
    /// Bounded access lane through the conservative envelope, aligned to the real hatch.
    pub fn in_dock_lane(&self, point: Vector3, margin: f32) -> bool {
        let rel = point - self.exit_position();
        rel.dot(self.exit_forward()) >= 0.0
            && rel.dot(self.exit_right()).abs() <= (self.dock.width * 0.5 - margin).max(0.0)
            && rel.dot(self.exit_up()).abs() <= (self.dock.height * 0.5 - margin).max(0.0)
    }
    /// Swept exterior collision, including turbo and vertical flight; also usable by cameras.
    pub fn push_out(&self, start: Vector3, end: Vector3, margin: f32) -> Option<Vector3> {
        if self.in_dock_lane(end, margin) {
            // A segment is safe if both ends lie in the convex lane, or if it
            // enters that lane through its outer mouth rather than a side wall.
            let movement = end - start;
            let rel = start - self.center();
            let a = movement.dot(movement);
            let b = rel.dot(movement);
            let c = rel.dot(rel) - (self.collision_radius + margin).powi(2);
            let disc = b * b - a * c;
            let entry = if a > 1e-8 && disc >= 0.0 {
                Some(start + movement * ((-b - disc.sqrt()) / a))
            } else {
                None
            };
            if self.in_dock_lane(start, margin)
                || c >= 0.0 && entry.is_some_and(|p| self.in_dock_lane(p, margin))
            {
                return None;
            }
        }
        if self.in_dock_lane(start, margin) {
            // Leaving the lane sideways inside the envelope is blocked at its edge.
            if (start - self.center()).length() < self.collision_radius + margin {
                return Some(start);
            }
        }
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
