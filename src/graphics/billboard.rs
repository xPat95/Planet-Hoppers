//! Cylindrical, alpha-tested sprite geometry. Rendering and ray refinement use
//! the same rolled quad, so transparent corners never become interaction boxes.
use crate::minigames::common::collision::Aabb;
use raylib::{ffi, prelude::*};
use std::path::Path;

pub fn alpha_visible(pixels: &[u8], width: usize, height: usize, uv: (f32, f32)) -> bool {
    let (u, v) = uv;
    if width == 0 || height == 0 || !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
        return false;
    }
    let x = ((u * width as f32) as usize).min(width - 1);
    let y = ((v * height as f32) as usize).min(height - 1);
    pixels.get(y * width + x).is_some_and(|a| *a >= 32)
}

pub struct BillboardPose {
    pub origin: Vector3,
    pub right: Vector3,
    pub up: Vector3,
    pub normal: Vector3,
    pub width: f32,
    pub height: f32,
    pub roll: f32,
}
impl BillboardPose {
    pub fn new(
        feet: Vector3,
        camera: Vector3,
        width: f32,
        height: f32,
        roll: f32,
        bounce: f32,
    ) -> Self {
        let delta = Vector3::new(camera.x - feet.x, 0.0, camera.z - feet.z);
        let normal = if delta.length() > 0.0001 {
            delta.normalized()
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };
        let right = Vector3::new(normal.z, 0.0, -normal.x);
        let up = Vector3::up();
        Self {
            origin: feet + up * (roll.sin().abs() * width * 0.5 + bounce),
            right: right * roll.cos() + up * roll.sin(),
            up: up * roll.cos() - right * roll.sin(),
            normal,
            width,
            height,
            roll,
        }
    }
    pub fn corners(&self) -> [Vector3; 4] {
        let left = self.origin - self.right * (self.width * 0.5);
        [
            left,
            left + self.right * self.width,
            left + self.right * self.width + self.up * self.height,
            left + self.up * self.height,
        ]
    }
    pub fn bounds(&self) -> Aabb {
        let points = self.corners();
        Aabb::new(
            Vector3::new(
                points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min),
                points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
                points.iter().map(|p| p.z).fold(f32::INFINITY, f32::min),
            ),
            Vector3::new(
                points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max),
                points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max),
                points.iter().map(|p| p.z).fold(f32::NEG_INFINITY, f32::max),
            ),
        )
    }
    pub fn ray_uv(
        &self,
        origin: Vector3,
        direction: Vector3,
        range: f32,
    ) -> Option<(f32, f32, f32)> {
        let denominator = direction.dot(self.normal);
        if denominator.abs() < 1e-7 {
            return None;
        }
        let distance = (self.origin - origin).dot(self.normal) / denominator;
        if !(0.0..=range).contains(&distance) {
            return None;
        }
        let relative = origin + direction * distance - self.origin;
        let u = relative.dot(self.right) / self.width + 0.5;
        let v = 1.0 - relative.dot(self.up) / self.height;
        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }
        Some((distance, u, v))
    }
}

pub struct SpriteBillboard {
    texture: Texture2D,
    pub pixel_width: usize,
    pub pixel_height: usize,
    pub alpha: Vec<u8>,
    pub height: f32,
}
impl SpriteBillboard {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &Path,
        height: f32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_image(rl, thread, path, height, false)
    }
    /// Remove source canvas margins in RAM, keeping the original PNG untouched.
    pub fn load_trimmed(rl: &mut RaylibHandle, thread: &RaylibThread, path: &Path, height: f32)
        -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_image(rl, thread, path, height, true)
    }
    fn load_image(rl: &mut RaylibHandle, thread: &RaylibThread, path: &Path, height: f32, trim: bool)
        -> Result<Self, Box<dyn std::error::Error>> {
        let image = Image::load_image(path.to_str().ok_or("Invalid sprite PNG path")?)?;
        Self::from_image(rl, thread, image, height, trim)
    }
    pub fn from_image(rl: &mut RaylibHandle, thread: &RaylibThread, mut image: Image, height: f32, trim: bool)
        -> Result<Self, Box<dyn std::error::Error>> {
        if trim { image.alpha_crop(0.125); }
        let (pixel_width, pixel_height) = (image.width() as usize, image.height() as usize);
        let alpha: Vec<_> = image.get_image_data().iter().map(|p| p.a).collect();
        if pixel_width == 0 || pixel_height == 0 || !alpha.contains(&0) || !alpha.contains(&255) {
            return Err("Kenny PNG must have visible pixels and transparent background".into());
        }
        let texture = rl.load_texture_from_image(thread, &image)?;
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(Self {
            texture,
            pixel_width,
            pixel_height,
            alpha,
            height,
        })
    }
    pub fn width(&self) -> f32 {
        self.height * self.pixel_width as f32 / self.pixel_height as f32
    }
    pub fn pose(&self, feet: Vector3, camera: Vector3, roll: f32, bounce: f32) -> BillboardPose {
        BillboardPose::new(feet, camera, self.width(), self.height, roll, bounce)
    }
    pub fn hit(
        &self,
        pose: &BillboardPose,
        origin: Vector3,
        direction: Vector3,
        range: f32,
    ) -> Option<f32> {
        let (distance, u, v) = pose.ray_uv(origin, direction, range)?;
        alpha_visible(&self.alpha, self.pixel_width, self.pixel_height, (u, v)).then_some(distance)
    }
    pub fn draw(&self, view: &mut RaylibMode3D<RaylibDrawHandle>, pose: &BillboardPose) {
        let level_camera = Camera3D::perspective(
            pose.origin + pose.normal * 5.0,
            pose.origin,
            Vector3::up(),
            65.0,
        );
        // Opaque town is already drawn. Keep depth testing, but transparent texels
        // must not write depth and hide other geometry. Flush before restoring.
        unsafe {
            ffi::rlDrawRenderBatchActive();
            ffi::rlDisableDepthMask();
        }
        view.draw_billboard_pro(
            level_camera,
            *self.texture.as_ref(),
            Rectangle::new(0.0, 0.0, self.pixel_width as f32, self.pixel_height as f32),
            pose.origin,
            Vector3::up(),
            Vector2::new(self.width(), self.height),
            Vector2::new(self.width() * 0.5, 0.0),
            pose.roll.to_degrees(),
            Color::WHITE,
        );
        unsafe {
            ffi::rlDrawRenderBatchActive();
            ffi::rlEnableDepthMask();
        }
    }
}
