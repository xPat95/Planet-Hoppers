//! Camera-oriented panorama rendered from the approved Blender world.
use raylib::prelude::*;
use std::path::Path;

pub(super) struct HangarBackdrop {
    texture: Texture2D,
    shader: Shader,
    forward: i32,
    right: i32,
    up: i32,
    projection: i32,
}

impl HangarBackdrop {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let texture = rl.load_texture(
            thread,
            &root
                .join("assets/textures/environment/hangar/approved_world.png")
                .to_string_lossy(),
        )?;
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let shader = rl.load_shader_from_memory(
            thread,
            None,
            Some(include_str!("shaders/hangar_backdrop.fs")),
        );
        let forward = shader.get_shader_location("viewForward");
        let right = shader.get_shader_location("viewRight");
        let up = shader.get_shader_location("viewUp");
        let projection = shader.get_shader_location("projectionScale");
        if !shader.is_shader_valid() || [forward, right, up, projection].contains(&-1) {
            return Err("El shader del fondo aprobado del hangar no compiló".into());
        }
        Ok(Self {
            texture,
            shader,
            forward,
            right,
            up,
            projection,
        })
    }

    pub fn draw(&mut self, d: &mut impl RaylibDraw, camera: Camera3D, viewport: Vector2) {
        let forward = (camera.target - camera.position).normalized();
        let right = forward.cross(camera.up).normalized();
        let up = right.cross(forward).normalized();
        self.shader.set_shader_value(self.forward, forward);
        self.shader.set_shader_value(self.right, right);
        self.shader.set_shader_value(self.up, up);
        self.shader.set_shader_value(
            self.projection,
            Vector2::new(
                viewport.x / viewport.y.max(1.0),
                (camera.fovy.to_radians() * 0.5).tan(),
            ),
        );
        let mut layer = d.begin_shader_mode(&mut self.shader);
        layer.draw_texture_pro(
            &self.texture,
            Rectangle::new(
                0.0,
                0.0,
                self.texture.width as f32,
                self.texture.height as f32,
            ),
            Rectangle::new(0.0, 0.0, viewport.x, viewport.y),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}
