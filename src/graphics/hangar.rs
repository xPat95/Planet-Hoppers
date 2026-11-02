//! Exhibition only: no ownership or selection rules. Replace the backdrop independently.
use super::space::{Fit, SpaceModel, SpaceShader};
use crate::ships::ShipModel;
use raylib::prelude::*;
use std::path::Path;

pub struct HangarStyle {
    pub display_length: f32,
    pub platform_radius: f32,
    pub platform_top: f32,
    pub rotation_degrees_per_second: f32,
    /// Future station GLB. None keeps the inexpensive temporary interior.
    pub station_model: Option<String>,
    pub station_length: f32,
    pub station_position: Vector3,
    pub station_rotation: [f32; 3],
}
impl Default for HangarStyle {
    fn default() -> Self {
        Self {
            display_length: 6.0,
            platform_radius: 4.5,
            platform_top: 0.8,
            rotation_degrees_per_second: 10.0,
            station_model: None,
            station_length: 22.0,
            station_position: Vector3::new(0.0, 4.0, 0.0),
            station_rotation: [0.0; 3],
        }
    }
}

pub struct HangarCamera {
    pub fov: f32,
    pub framing_margin: f32,
}
impl Default for HangarCamera {
    fn default() -> Self {
        Self {
            fov: 42.0,
            framing_margin: 0.80,
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
        let distance = (half_extents.length() / half_angle.sin() * self.framing_margin).max(8.0);
        Camera3D::perspective(
            center + Vector3::new(0.9, 0.6, 1.4).normalized() * distance,
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
    ship: Option<SpaceModel>,
    station: Option<SpaceModel>,
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
        style: HangarStyle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let shader = SpaceShader::load(rl, thread)?;
        let station = if let Some(path) = &style.station_model {
            Some(SpaceModel::load(
                rl,
                thread,
                &root.join(path),
                &shader,
                style.station_rotation,
                Fit::Length(style.station_length),
            )?)
        } else {
            None
        };
        Ok(Self {
            style,
            camera: HangarCamera::default(),
            shader,
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
        self.angle = 25.0;
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
    pub fn draw(&mut self, d: &mut impl RaylibDraw3D, camera: Camera3D) {
        self.shader
            .begin_frame(Vector3::new(-4.0, 9.0, 5.0), camera.position, 0.55);
        self.draw_backdrop(d);
        let base = Color::new(24, 38, 53, 255);
        d.draw_cylinder(
            Vector3::new(0.0, 0.0, 0.0),
            self.style.platform_radius,
            self.style.platform_radius + 0.2,
            self.style.platform_top,
            64,
            base,
        );
        d.draw_cylinder_wires(
            Vector3::new(0.0, 0.0, 0.0),
            self.style.platform_radius,
            self.style.platform_radius + 0.2,
            self.style.platform_top,
            64,
            Color::new(50, 122, 148, 255),
        );
        for i in 0..12 {
            let angle = self.angle.to_radians() + i as f32 * std::f32::consts::TAU / 12.0;
            let ray = Vector3::new(angle.cos(), 0.0, angle.sin());
            let top = Vector3::up() * (self.style.platform_top + 0.01);
            d.draw_line_3D(
                top + ray * self.style.platform_radius * 0.85,
                top + ray * self.style.platform_radius * 0.98,
                Color::new(100, 205, 225, 255),
            );
        }
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
    fn draw_backdrop(&mut self, d: &mut impl RaylibDraw3D) {
        if let Some(model) = &self.station {
            let p = self.style.station_position;
            model.draw(d, &mut self.shader, Matrix::translate(p.x, p.y, p.z));
            return;
        }
        // Temporary room; the station adapter above leaves selection/camera unchanged.
        d.draw_cube(
            Vector3::new(0.0, -0.2, 0.0),
            24.0,
            0.4,
            24.0,
            Color::new(10, 17, 26, 255),
        );
        d.draw_cube(
            Vector3::new(0.0, 5.0, -10.0),
            24.0,
            10.0,
            0.3,
            Color::new(13, 23, 35, 255),
        );
        d.draw_cube(
            Vector3::new(-12.0, 5.0, 0.0),
            0.3,
            10.0,
            24.0,
            Color::new(10, 20, 31, 255),
        );
        for x in [-9.0, -6.0, 6.0, 9.0] {
            d.draw_cube(
                Vector3::new(x, 4.0, -9.7),
                0.18,
                7.0,
                0.12,
                Color::new(64, 137, 165, 255),
            );
        }
        for x in [-8.0, 8.0] {
            d.draw_line_3D(
                Vector3::new(x, 0.01, -9.0),
                Vector3::new(x, 0.01, 9.0),
                Color::new(35, 65, 83, 255),
            );
        }
    }
}
