//! Reuse the exact resolved solar ship; only its local uniform scale changes.
use super::simulation::{Game, Phase};
use crate::{
    graphics::space::{Fit, SpaceModel, SpaceShader},
    ships::ShipModel,
};
use raylib::prelude::*;
use std::{f32::consts::PI, path::Path};

pub struct TunnelShip {
    // Drop mesh/material references before their shader and before the window closes.
    model: SpaceModel,
    shader: SpaceShader,
    half: [f32; 3],
}

pub fn tunnel_scale(half: [f32; 3]) -> Result<f32, Box<dyn std::error::Error>> {
    if half.iter().any(|h| !h.is_finite() || *h <= 0.0) {
        return Err("El modelo de nave tiene límites inválidos".into());
    }
    // A single scale keeps proportions, including tall ships and wide wings.
    Ok((0.95 / half[0]).min(0.65 / half[1]).min(1.4 / half[2]))
}
impl TunnelShip {
    pub fn load(
        rl: &mut RaylibHandle,
        t: &RaylibThread,
        root: &Path,
        ship: &ShipModel,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if ship.model.is_empty() {
            return Err(format!("La nave {} no tiene modelo", ship.id).into());
        }
        let shader = SpaceShader::load(rl, t)?;
        let mut model = SpaceModel::load(
            rl,
            t,
            &root.join(&ship.model),
            &shader,
            ship.flight_rotation_degrees(),
            Fit::Length(1.0),
        )?;
        let scale = tunnel_scale([
            model.half_extents.x,
            model.half_extents.y,
            model.half_extents.z,
        ])?;
        model.fit *= Matrix::scale(scale, scale, scale);
        model.half_extents *= scale;
        let h = model.half_extents;
        let half = [h.x + 0.035, h.y + 0.035, h.z + 0.035];
        println!(
            "TUNNEL_SHIP id={} model={} half={half:?} scale={scale}",
            ship.id, ship.model
        );
        Ok(Self {
            model,
            shader,
            half,
        })
    }
    pub fn half(&self) -> [f32; 3] {
        self.half
    }
    pub fn draw(&mut self, d: &mut impl RaylibDraw3D, g: &Game, camera: Vector3) {
        if g.phase == Phase::Lost {
            return;
        }
        self.shader.begin_frame(
            Vector3::new(g.pos[0] - 3.0, g.pos[1] + 6.0, -8.0),
            camera,
            0.8,
        );
        self.shader.set_studio(true);
        // Solar flight normalizes the nose to -Z; the tunnel advances toward +Z.
        let world = Matrix::rotate_y(PI)
            * Matrix::rotate_x(g.pitch)
            * Matrix::rotate_z(g.bank)
            * Matrix::translate(g.pos[0], g.pos[1], 0.0);
        self.model.draw(d, &mut self.shader, world);
        let h = g.hull_half();
        for (offset, radius, alpha) in crate::graphics::ship_presentation::exhaust_spheres(
            self.half.iter().copied().fold(0.0, f32::max) * 2.0,
            h[2],
            (g.speed / 62.0).clamp(0.0, 1.0),
        ) {
            d.draw_sphere_ex(
                Vector3::new(g.pos[0], g.pos[1], -offset),
                radius,
                6,
                6,
                Color::new(125, 205, 255, alpha),
            );
        }
        if g.muzzle > 0.0 {
            d.draw_sphere_ex(
                Vector3::new(g.pos[0], g.pos[1], h[2] + 0.25),
                0.12 * g.muzzle,
                6,
                6,
                Color::new(177, 255, 177, 255),
            );
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uniform_fit_handles_very_large_small_and_tall_ships() {
        for h in [
            [800.0, 12.0, 900.0],
            [0.0002, 0.0001, 0.0003],
            [1.0, 100.0, 2.0],
            [9.0, 1.0, 1.0],
        ] {
            let scale = tunnel_scale(h).unwrap();
            let fitted = h.map(|a| a * scale);
            assert!(fitted[0] <= 0.951 && fitted[1] <= 0.651 && fitted[2] <= 1.401);
            assert!((fitted[0] / fitted[2] - h[0] / h[2]).abs() < 0.001);
        }
        assert!(tunnel_scale([0.0, 1.0, 1.0]).is_err());
        assert!(tunnel_scale([f32::NAN, 1.0, 1.0]).is_err());
    }
}
