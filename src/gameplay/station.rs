//! Fixed point of interest, deliberately separate from planets and orbital simulation.
use super::station_collision::CollisionHull;
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
    /// Conservative launch envelope; actual movement uses the generated surface hull.
    pub collision_radius: f32,
    pub dock: Dock,
    pub lod_distance: f32,
    #[serde(skip)]
    pub hull: CollisionHull,
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
    /// Minimum free margin beyond the hull/camera; also the close docking waypoint distance.
    pub departure_clearance: f32,
    pub approach_distance: f32,
    pub interaction_radius: f32,
}

/// Bounds already fitted by the same loader used for rendering, in game units.
#[derive(Debug, Clone, Copy)]
pub struct DepartureBounds {
    pub station_half_extents: Vector3,
    pub ship_half_extents: Vector3,
    pub ship_collision_radius: f32,
    pub ship_length: f32,
    pub camera_length: f32,
    pub camera_radius: f32,
}

#[derive(Debug, Clone)]
pub struct Departure {
    pub ship: Ship,
    pub camera_position: Vector3,
    pub ship_radius: f32,
    pub hull_support: f32,
    pub safety_margin: f32,
    pub distance_from_dock: f32,
}

/// Shared with the flight view so launch clearance cannot drift from its camera.
pub fn chase_position(position: Vector3, forward: Vector3, ship_length: f32) -> Vector3 {
    position - forward * ship_length * 2.76 + Vector3::up() * ship_length * 0.9
}

impl Station {
    pub fn load(root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut s: Self =
            serde_json::from_str(&std::fs::read_to_string(root.join("config/station.json"))?)?;
        s.hull = serde_json::from_str(&std::fs::read_to_string(
            root.join(&s.model).with_extension("collision.json"),
        )?)?;
        if s.hull.source != s.model
            || s.hull.rows.is_empty()
            || s.hull.cell <= 0.0
            || s.hull.reference_length <= 0.0
        {
            return Err("Colisión derivada de estación inválida".into());
        }
        s.hull.bounds = s.hull.boxes().fold(
            [
                Vector3::one() * f32::INFINITY,
                Vector3::one() * f32::NEG_INFINITY,
            ],
            |[lo, hi], (a, b)| {
                [
                    Vector3::new(lo.x.min(a.x), lo.y.min(a.y), lo.z.min(a.z)),
                    Vector3::new(hi.x.max(b.x), hi.y.max(b.y), hi.z.max(b.z)),
                ]
            },
        );
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
    /// Close docking waypoint, inside the access corridor. Never a flight spawn.
    pub fn dock_target(&self) -> Vector3 {
        self.exit_position() + self.exit_forward() * self.dock.departure_clearance
    }

    /// Launch beyond both the actual fitted model and the entire collision hull,
    /// rather than relying on the narrow corridor's collision exemption.
    pub fn departure(&self, bounds: DepartureBounds) -> Result<Departure, String> {
        let halves = [bounds.station_half_extents, bounds.ship_half_extents];
        if !halves
            .iter()
            .all(|h| [h.x, h.y, h.z].iter().all(|v| v.is_finite() && *v >= 0.0))
            || ![
                bounds.ship_collision_radius,
                bounds.ship_length,
                bounds.camera_length,
                bounds.camera_radius,
            ]
            .iter()
            .all(|v| v.is_finite() && *v > 0.0)
        {
            return Err("Límites de despegue inválidos".into());
        }
        // Use the controller's representable heading, including its pitch limit.
        // No GLB-specific yaw corrections belong to gameplay orientation.
        let mut ship = Ship::new(Vector3::zero(), 0.0);
        ship.look_at(self.exit_forward());
        let forward = ship.forward();
        let half = bounds.station_half_extents;
        let visual_support = self.rotated([1.0, 0.0, 0.0]).dot(forward).abs() * half.x
            + self.rotated([0.0, 1.0, 0.0]).dot(forward).abs() * half.y
            + self.rotated([0.0, 0.0, 1.0]).dot(forward).abs() * half.z;
        let door_projection = (self.exit_position() - self.center()).dot(forward);
        // Inspected exterior hatch geometry: depth 9, lateral frame up to w/2+1.9,
        // upper frame up to h/2+0.85 (graphics/station_dock.rs).
        let hatch_support = door_projection
            + self.exit_forward().dot(forward).max(0.0) * 9.0
            + self.exit_right().dot(forward).abs() * (self.dock.width * 0.5 + 1.9)
            + self.exit_up().dot(forward).abs() * (self.dock.height * 0.5 + 0.85);
        let hull_support = self.collision_radius.max(visual_support).max(hatch_support);
        let ship_radius = bounds
            .ship_half_extents
            .length()
            .max(bounds.ship_collision_radius);
        let camera_offset = chase_position(Vector3::zero(), forward, bounds.camera_length);
        let rear = ship_radius.max(-camera_offset.dot(forward) + bounds.camera_radius);
        let safety_margin = self
            .dock
            .departure_clearance
            .max(ship_radius * 2.0)
            .max(bounds.camera_radius * 2.0);
        let distance = (hull_support + rear + safety_margin - door_projection)
            .max(self.dock.departure_clearance);
        let step = ship_radius.max(bounds.camera_radius).max(1.0);
        // Bounded outward search, with independent collision and support-plane
        // validation of ship, camera and a forward movement segment.
        for attempt in 0..=16 {
            let travel = distance + attempt as f32 * step;
            let position = self.exit_position() + forward * travel;
            let camera_position = position + camera_offset;
            let outside = |p: Vector3, radius: f32| {
                let rel = p - self.center();
                rel.length() > self.collision_radius + radius
                    && rel.dot(forward) - radius > hull_support
                    && self.push_out(p, p, radius).is_none()
            };
            let ahead = position + forward * (safety_margin + bounds.ship_length);
            if outside(position, ship_radius)
                && outside(camera_position, bounds.camera_radius)
                && outside(ahead, ship_radius)
                && self.push_out(position, ahead, ship_radius).is_none()
            {
                ship.position = position;
                return Ok(Departure {
                    ship,
                    camera_position,
                    ship_radius,
                    hull_support,
                    safety_margin,
                    distance_from_dock: travel,
                });
            }
        }
        Err("No se encontró un despegue seguro fuera de la estación".into())
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
            self.dock_target()
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
    /// Front access point beyond the fitted surface bounds and exterior hatch.
    /// Includes the complete arriving ship rather than just its centre.
    pub fn entry_anchor(&self, ship_radius: f32) -> Vector3 {
        let door = self.exit_position();
        let forward = self.exit_forward();
        let projection = (door - self.center()).dot(forward);
        let [lo, hi] = self.hull.bounds;
        let surface = (0..8)
            .map(|k| {
                let p = Vector3::new(
                    if k & 1 == 0 { lo.x } else { hi.x },
                    if k & 2 == 0 { lo.y } else { hi.y },
                    if k & 4 == 0 { lo.z } else { hi.z },
                );
                (self.collision_point(p) - self.center()).dot(forward)
            })
            .fold(f32::NEG_INFINITY, f32::max)
            .max(projection + 9.0);
        let distance =
            (surface + ship_radius.max(0.0) * 3.0_f32.sqrt() + 0.02 - projection).max(0.0);
        door + forward * distance
    }
    pub fn in_reach(&self, point: Vector3, ship_radius: f32) -> bool {
        let rel = point - self.exit_position();
        // Preserve access at the real door for players already in its open lane.
        let at_door =
            self.in_dock_lane(point, ship_radius) && rel.length() <= self.dock.interaction_radius;
        let outside = rel.dot(self.exit_forward()) >= 0.0
            && (point - self.entry_anchor(ship_radius)).length()
                <= self.dock.interaction_radius.min(self.dock.width * 2.0);
        (at_door || outside) && self.push_out(point, point, ship_radius).is_none()
    }
    /// Bounded access lane through the conservative envelope, aligned to the real hatch.
    pub fn in_dock_lane(&self, point: Vector3, margin: f32) -> bool {
        let rel = point - self.exit_position();
        rel.dot(self.exit_forward()) >= 0.0
            && rel.dot(self.exit_right()).abs() <= (self.dock.width * 0.5 - margin).max(0.0)
            && rel.dot(self.exit_up()).abs() <= (self.dock.height * 0.5 - margin).max(0.0)
    }
    /// Match the exact fit/centering used by the loaded visual model.
    pub fn align_collision(&mut self, fit: Matrix) {
        let c = self.hull.raw_center;
        self.hull.offset_fraction =
            Vector3::new(c[0], c[1], c[2]).transform_with(fit) / self.length;
    }
    pub fn collision_point(&self, local: Vector3) -> Vector3 {
        let p = local * (self.length / self.hull.reference_length)
            + self.hull.offset_fraction * self.length;
        self.center() + p.transform_with(self.rotation())
    }
    pub fn collision_local(&self, world: Vector3) -> Vector3 {
        let d = world - self.center();
        let local = Vector3::new(
            d.dot(self.rotated([1.0, 0.0, 0.0])),
            d.dot(self.rotated([0.0, 1.0, 0.0])),
            d.dot(self.rotated([0.0, 0.0, 1.0])),
        );
        (local - self.hull.offset_fraction * self.length)
            / (self.length / self.hull.reference_length)
    }
    /// Swept collision against occupied GLB surface cells, never the old sphere.
    pub fn push_out(&self, start: Vector3, end: Vector3, margin: f32) -> Option<Vector3> {
        let scale = self.length / self.hull.reference_length;
        let body = self
            .hull
            .push_out(
                self.collision_local(start),
                self.collision_local(end),
                margin / scale,
            )
            .map(|out| self.collision_point(out));
        // Physical frame extension drawn by graphics/station_dock.rs, in door axes.
        let (w, h) = (self.dock.width * 0.5, self.dock.height * 0.5);
        let frame = CollisionHull {
            extra: vec![
                (
                    Vector3::new(-w - 1.0, -h - 1.0, 0.0),
                    Vector3::new(-w, h + 1.0, 9.0),
                ),
                (
                    Vector3::new(w, -h - 1.0, 0.0),
                    Vector3::new(w + 1.0, h + 1.0, 9.0),
                ),
                (Vector3::new(-w, h, 0.0), Vector3::new(w, h + 1.0, 9.0)),
                (Vector3::new(-w, -h - 1.0, 0.0), Vector3::new(w, -h, 9.0)),
            ],
            ..Default::default()
        };
        let local = |p: Vector3| {
            let d = p - self.exit_position();
            Vector3::new(
                d.dot(self.exit_right()),
                d.dot(self.exit_up()),
                d.dot(self.exit_forward()),
            )
        };
        let world = |p: Vector3| {
            self.exit_position()
                + self.exit_right() * p.x
                + self.exit_up() * p.y
                + self.exit_forward() * p.z
        };
        let hatch = frame.push_out(local(start), local(end), margin).map(world);
        let contact = match (body, hatch) {
            (Some(a), Some(b)) => Some(if (a - start).length() < (b - start).length() {
                a
            } else {
                b
            }),
            (a, b) => a.or(b),
        };
        contact.map(|mut p| {
            for _ in 0..8 {
                if let Some(out) = self.hull.push_out(
                    self.collision_local(p),
                    self.collision_local(p),
                    margin / scale,
                ) {
                    p = self.collision_point(out);
                } else if let Some(out) = frame.push_out(local(p), local(p), margin) {
                    p = world(out);
                } else {
                    return p;
                }
            }
            start
        })
    }
}
