//! Exhibition only: no ownership or selection rules. Replace the backdrop independently.
pub use super::hangar_interior::InteriorStyle;
use super::space::{Fit, SpaceModel, SpaceShader};
use crate::ships::ShipModel;
use raylib::prelude::*;
use std::path::Path;

pub struct HangarStyle {
    pub display_length: f32,
    pub platform_radius: f32,
    pub platform_top: f32,
    pub rotation_degrees_per_second: f32,
    pub interior: InteriorStyle,
    pub light_position: Vector3,
    pub ambient: f32,
    /// Blender module authored in game units, with the platform center at the origin.
    pub station_model: String,
}
impl Default for HangarStyle {
    fn default() -> Self {
        Self {
            display_length: 6.0,
            platform_radius: 4.5,
            platform_top: 0.8,
            rotation_degrees_per_second: 10.0,
            interior: InteriorStyle::default(),
            light_position: Vector3::new(-4.0, 9.0, 5.0),
            ambient: 0.55,
            station_model:
                "assets/models/environment/spaceStation/exports/citadel_hangar_phase1.glb".into(),
        }
    }
}

pub struct HangarCamera {
    pub fov: f32,
    pub framing_margin: f32,
    pub direction: Vector3,
    pub minimum_distance: f32,
}
impl Default for HangarCamera {
    fn default() -> Self {
        Self {
            fov: 42.0,
            framing_margin: 1.08,
            direction: Vector3::new(0.9, 0.6, 1.4),
            minimum_distance: 11.0,
        }
    }
}
impl HangarCamera {
    pub fn camera(
        &self,
        center: Vector3,
        half_extents: Vector3,
        viewport: Vector2,
        insets: Vector2,
    ) -> Camera3D {
        // Frame the bounding sphere inside the actual space left by the HUD and arrows.
        let aspect = viewport.x / viewport.y.max(1.0);
        let usable_height =
            ((viewport.y - insets.x - insets.y) / viewport.y.max(1.0)).clamp(0.2, 1.0);
        let usable_width = ((viewport.x - 180.0) / viewport.x.max(1.0)).clamp(0.2, 1.0);
        let tangent = (self.fov.to_radians() * 0.5).tan();
        let half_angle = (tangent * usable_height)
            .atan()
            .min((tangent * aspect * usable_width).atan());
        let distance = (half_extents.length() / half_angle.sin() * self.framing_margin)
            .max(self.minimum_distance);
        Camera3D::perspective(
            center + self.direction.normalized() * distance,
            center,
            Vector3::up(),
            self.fov,
        )
    }
}

pub struct Hangar {
    pub style: HangarStyle,
    pub camera: HangarCamera,
    shader: SpaceShader,
    environment_shader: SpaceShader,
    ship: Option<SpaceModel>,
    station: SpaceModel,
    pub placeholder: bool,
    center: Vector3,
    half_extents: Vector3,
    angle: f32,
}
impl Hangar {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
        mut style: HangarStyle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dock = crate::gameplay::station::Station::load(root)?;
        style.interior.configure(&dock);
        let shader = SpaceShader::load(rl, thread)?;
        let environment_shader = SpaceShader::load_hangar(rl, thread)?;
        // Preserve the architectural origin and metres; automatic model fitting would
        // recenter the deck and separate it from the historical platform.
        let station = SpaceModel::load(
            rl,
            thread,
            &root.join(&style.station_model),
            &environment_shader,
            [0.0; 3],
            Fit::Same(Matrix::identity(), Vector3::new(18.7, 16.7, 32.0)),
        )?;
        Ok(Self {
            style,
            camera: HangarCamera::default(),
            shader,
            environment_shader,
            ship: None,
            station,
            placeholder: true,
            center: Vector3::zero(),
            half_extents: Vector3::one() * 3.0,
            angle: 0.0,
        })
    }
    pub fn exhibit(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
        ship: &ShipModel,
    ) {
        // Drop the previous exhibit before loading the next: only one ship is retained.
        self.ship = None;
        if !ship.model.is_empty() {
            self.ship = SpaceModel::load(
                rl,
                thread,
                &root.join(&ship.model),
                &self.shader,
                ship.model_rotation_degrees,
                Fit::Length(self.style.display_length),
            )
            .map_err(|e| eprintln!("HANGAR_MODEL_WARNING {}: {e}", ship.id))
            .ok();
        }
        self.placeholder = self.ship.is_none();
        self.half_extents = self
            .ship
            .as_ref()
            .map_or(Vector3::new(2.3, 0.8, 3.0), |s| s.half_extents);
        self.center = Vector3::new(0.0, self.style.platform_top + self.half_extents.y, 0.0);
        if self.placeholder {
            // The shared primitive hull is not centered on its asymmetric bounds.
            self.center.y = self.style.platform_top + 0.225 * self.style.display_length / 4.0;
        }
        self.angle = 0.0;
    }
    pub fn update(&mut self, dt: f32) {
        self.angle = (self.angle + dt * self.style.rotation_degrees_per_second).rem_euclid(360.0);
    }
    /// Fixed viewpoints for the automated cutout preview.
    pub fn set_preview_angle(&mut self, angle: f32) {
        self.angle = angle;
    }
    pub fn camera(&self, viewport: Vector2, insets: Vector2) -> Camera3D {
        self.camera
            .camera(self.center, self.half_extents, viewport, insets)
    }
    /// Inspection viewpoints stay inside the configured walls and below the roof.
    pub fn inspection_camera(&self, exit: bool) -> Camera3D {
        let room = &self.style.interior;
        if exit {
            Camera3D::perspective(
                Vector3::new(0.0, room.exit_height * 0.5, room.depth * 0.14),
                Vector3::new(0.0, room.exit_height * 0.5, -room.depth * 0.5),
                Vector3::up(),
                60.0,
            )
        } else {
            Camera3D::perspective(
                Vector3::new(room.width * 0.33, room.height * 0.625, room.depth * 0.41),
                Vector3::new(0.0, room.height * 0.31, -room.depth * 0.18),
                Vector3::up(),
                65.0,
            )
        }
    }
    pub fn draw(&mut self, d: &mut impl RaylibDraw3D, camera: Camera3D) {
        self.shader.begin_frame(
            self.style.light_position,
            camera.position,
            self.style.ambient,
        );
        self.environment_shader.begin_frame(self.style.light_position, camera.position, 0.24);
        self.station.draw(d, &mut self.environment_shader, Matrix::identity());
        self.draw_platform(d);
        let world = Matrix::rotate_y(self.angle.to_radians())
            * Matrix::translate(self.center.x, self.center.y, self.center.z);
        if let Some(ship) = &self.ship {
            ship.draw(d, &mut self.shader, world);
        } else {
            super::ship_placeholder::draw(
                d,
                self.center,
                Vector3::new(0.0, self.angle, 0.0),
                self.style.display_length,
            );
        }
    }
    /// Original circular exhibition base, independent from the station architecture.
    fn draw_platform(&self, d: &mut impl RaylibDraw3D) {
        let radius = self.style.platform_radius;
        let top = self.style.platform_top;
        d.draw_cylinder(
            Vector3::zero(),
            radius,
            radius + 0.2,
            top - 0.08,
            64,
            self.style.interior.titanium,
        );
        d.draw_cylinder(
            Vector3::new(0.0, top - 0.08, 0.0),
            radius + 0.04,
            radius + 0.04,
            0.04,
            64,
            self.style.interior.cyan,
        );
        // The top ends at platform_top: the ship's minimum Y rests exactly here.
        d.draw_cylinder(
            Vector3::new(0.0, top - 0.04, 0.0),
            radius - 0.15,
            radius - 0.15,
            0.04,
            64,
            self.style.interior.graphite,
        );
        for i in 0..12 {
            let angle = self.angle.to_radians() + i as f32 * std::f32::consts::TAU / 12.0;
            let ray = Vector3::new(angle.cos(), 0.0, angle.sin());
            let at = Vector3::up() * (top + 0.01);
            d.draw_line_3D(
                at + ray * radius * 0.85,
                at + ray * radius * 0.98,
                Color::new(100, 205, 225, 255),
            );
        }
    }
}
