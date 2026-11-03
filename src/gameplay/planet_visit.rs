//! Flight pose retained across a planet visit; no scene or resource ownership.
use crate::{
    ships::Ship,
    solar_system::{Planet, SolarSystem},
};
use raylib::prelude::Vector3;

pub struct PlanetReturnPose {
    offset: Vector3,
    yaw: f32,
    pitch: f32,
    bank: f32,
}
impl PlanetReturnPose {
    pub fn capture(ship: &Ship, planet: &Planet) -> Self {
        Self {
            offset: ship.position - planet.position(),
            yaw: ship.yaw,
            pitch: ship.pitch,
            bank: ship.bank,
        }
    }

    /// Rebase the saved approach onto the current orbital center. If obstructed,
    /// search outward from that same planet, checking every body and the station.
    pub fn restore(
        &self,
        ship: &mut Ship,
        system: &SolarSystem,
        index: usize,
        margin: f32,
        extra_clear: impl Fn(Vector3) -> bool,
    ) -> Result<(), String> {
        let planet = &system.planets[index];
        let clear = |point: Vector3| {
            point.length() > system.sun.radius + margin
                && system.planets.iter().all(|p| {
                    p.shape_distance(point, margin) > 1.0
                        && (point - p.position()).length() > p.config.radius + margin
                })
                && extra_clear(point)
        };
        let approach = planet.position() + self.offset;
        let position = if clear(approach) {
            approach
        } else {
            let direction = if self.offset.length() > 0.001 {
                self.offset.normalized()
            } else {
                planet.spin_axis()
            };
            let radius = planet
                .collision_radii
                .x
                .max(planet.collision_radii.y)
                .max(planet.collision_radii.z)
                .max(planet.config.radius);
            (0..16)
                .map(|step| {
                    planet.position()
                        + direction * (radius * 2.2 + margin) * (1.0 + step as f32 * 0.1)
                })
                .find(|p| clear(*p))
                .ok_or("No safe return near the visited planet")?
        };
        ship.position = position;
        ship.yaw = self.yaw;
        ship.pitch = self.pitch;
        ship.bank = self.bank;
        ship.speed = 0.0;
        ship.carried_by = None;
        ship.bumped = false;
        Ok(())
    }
}
