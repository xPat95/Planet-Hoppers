//! Exhibition only: no ownership or selection rules. Replace the backdrop independently.
pub use super::hangar_interior::InteriorStyle;
use super::space::{Fit, SpaceModel, SpaceShader};
use super::hangar_backdrop::HangarBackdrop;
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
                "assets/models/environment/spaceStation/exports/citadel_hangar_reconstructed.glb".into(),
        }
    }
}

pub struct HangarCamera {
    pub fov: f32,
    pub framing_margin: f32,
    pub position: Vector3,
    pub target: Vector3,
}
impl Default for HangarCamera {
    fn default() -> Self {
        Self {
            fov: 70.0,
            framing_margin: 1.08,
            // Slight dolly toward the platform; keep the surrounding architecture.
            position: Vector3::new(10.8, 7.2, 12.6),
            // Keep the exhibit prominent and the composition stable between ships.
            target: Vector3::new(0.0, 4.8, -5.5),
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
        // Keep the approved architectural viewpoint fixed; fit each rotating ship
        // inside the actual HUD region by changing only the vertical field of view.
        let aspect = viewport.x / viewport.y.max(1.0);
        let top = (1.0 - 2.0 * insets.x / viewport.y.max(1.0)).max(0.1);
        let bottom = (1.0 - 2.0 * insets.y / viewport.y.max(1.0)).max(0.1);
        let side = (1.0 - 180.0 / viewport.x.max(1.0)).max(0.1);
        let target = self.target;
        let forward = (target - self.position).normalized();
        let right = forward.cross(Vector3::up()).normalized();
        let up = right.cross(forward).normalized();
        // A cylinder bounds every yaw angle without treating a thin ship as a
        // tall sphere. Include the historical platform's 4.7-unit base as well.
        let radius = half_extents.x.hypot(half_extents.z).max(4.7) * self.framing_margin;
        let mut tangent = (self.fov.to_radians() * 0.5).tan();
        for step in 0..16 {
            let a = step as f32 * std::f32::consts::TAU / 16.0;
            for y in [-half_extents.y, half_extents.y] {
                let point = center + Vector3::new(radius * a.cos(), y * self.framing_margin, radius * a.sin());
                let delta = point - self.position;
                let depth = delta.dot(forward).max(0.1);
                let vertical = delta.dot(up) / depth;
                let horizontal = delta.dot(right) / depth;
                tangent = tangent.max(if vertical >= 0.0 { vertical / top } else { -vertical / bottom })
                    .max(horizontal.abs() / (side * aspect));
            }
        }
        let fov = (2.0 * tangent.atan()).to_degrees();
        Camera3D::perspective(
            self.position,
            target,
            Vector3::up(),
            fov,
        )
    }
}

pub struct Hangar {
    pub style: HangarStyle,
    pub camera: HangarCamera,
    shader: SpaceShader,
    environment_shader: SpaceShader,
    backdrop: HangarBackdrop,
    ship: Option<SpaceModel>,
    station: SpaceModel,
    pub placeholder: bool,
    center: Vector3,
    half_extents: Vector3,
    angle: f32,
    exhibit_yaw_offset: f32,
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
        let backdrop = HangarBackdrop::load(rl, thread, root)?;
        // Preserve the architectural origin and metres; automatic model fitting would
        // recenter the deck and separate it from the historical platform.
        let mut station = SpaceModel::load(
            rl,
            thread,
            &root.join(&style.station_model),
            &environment_shader,
            [0.0; 3],
            Fit::Same(Matrix::identity(), Vector3::new(26.626122, 25.748001, 29.763777)),
        )?;
        station.filter_albedo(TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(Self {
            style,
            camera: HangarCamera::default(),
            shader,
            environment_shader,
            backdrop,
            ship: None,
            station,
            placeholder: true,
            center: Vector3::zero(),
            half_extents: Vector3::one() * 3.0,
            angle: 0.0,
            exhibit_yaw_offset: 0.0,
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
        self.exhibit_yaw_offset = super::ship_presentation::hangar_start_yaw(ship, self.camera.position);
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
    pub fn draw_backdrop(&mut self, d: &mut impl RaylibDraw, camera: Camera3D, viewport: Vector2) {
        self.backdrop.draw(d, camera, viewport);
    }
    /// Approved Blender review views, converted from Z-up to game Y-up.
    /// These auxiliary views do not change the ship selection camera.
    pub fn inspection_camera(&self, exit: bool) -> Camera3D {
        if exit {
            Camera3D::perspective(
                Vector3::new(0.0, 5.0, 2.0),
                Vector3::new(0.0, 9.0, -20.0),
                Vector3::up(),
                67.0,
            )
        } else {
            Camera3D::perspective(
                Vector3::new(12.0, 8.0, 14.0),
                Vector3::new(0.0, 6.0, -7.0),
                Vector3::up(),
                74.0,
            )
        }
    }
    pub fn draw(&mut self, d: &mut impl RaylibDraw3D, camera: Camera3D) {
        self.shader.begin_frame(
            self.style.light_position,
            camera.position,
            self.style.ambient,
        );
        self.environment_shader.begin_frame(self.style.light_position, camera.position, 0.50);
        self.station.draw(d, &mut self.environment_shader, Matrix::identity());
        self.draw_platform(d);
        let exhibit_angle = self.angle + self.exhibit_yaw_offset;
        let world = Matrix::rotate_y(exhibit_angle.to_radians())
            * Matrix::translate(self.center.x, self.center.y, self.center.z);
        if let Some(ship) = &self.ship {
            ship.draw(d, &mut self.shader, world);
        } else {
            super::ship_placeholder::draw(
                d,
                self.center,
                Vector3::new(0.0, exhibit_angle, 0.0),
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
