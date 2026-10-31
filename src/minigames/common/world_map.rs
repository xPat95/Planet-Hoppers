//! Shared horizontal map math. North is -Z; map Y increases down the screen.
use raylib::prelude::{Vector2, Vector3};
#[derive(Clone, Copy, Debug)]
pub struct MapBounds {
    pub min: Vector2,
    pub max: Vector2,
}
impl MapBounds {
    pub fn project(&self, p: Vector3) -> Vector2 {
        Vector2::new(
            (p.x - self.min.x) / (self.max.x - self.min.x),
            (p.z - self.min.y) / (self.max.y - self.min.y),
        )
    }
    pub fn unproject(&self, uv: Vector2) -> Vector3 {
        Vector3::new(
            self.min.x + uv.x * (self.max.x - self.min.x),
            0.0,
            self.min.y + uv.y * (self.max.y - self.min.y),
        )
    }
    pub fn centre(&self) -> Vector2 {
        (self.min + self.max) * 0.5
    }
    pub fn size(&self) -> Vector2 {
        self.max - self.min
    }
}
#[derive(Clone, Copy, Debug)]
pub struct MapView {
    pub centre: Vector2,
    pub zoom: f32,
    pub bounds: MapBounds,
}
impl MapView {
    pub fn new(bounds: MapBounds) -> Self {
        Self {
            centre: bounds.centre(),
            zoom: 1.0,
            bounds,
        }
    }
    pub fn fit(&mut self) {
        self.centre = self.bounds.centre();
        self.zoom = 1.0;
    }
    pub fn scale(&self, size: Vector2) -> f32 {
        (size.x / self.bounds.size().x).min(size.y / self.bounds.size().y) * self.zoom
    }
    pub fn screen(&self, p: Vector3, origin: Vector2, size: Vector2) -> Vector2 {
        origin + size * 0.5 + (Vector2::new(p.x, p.z) - self.centre) * self.scale(size)
    }
    pub fn pan(&mut self, delta: Vector2, size: Vector2) {
        self.centre -= delta / self.scale(size);
        self.clamp(size);
    }
    pub fn zoom_by(&mut self, wheel: f32, size: Vector2) {
        if wheel.is_finite() {
            self.zoom = (self.zoom * (wheel * 0.18).exp()).clamp(1.0, 6.0);
            self.clamp(size);
        }
    }
    pub fn recenter(&mut self, player: Vector3, size: Vector2) {
        self.centre = Vector2::new(player.x, player.z);
        self.clamp(size);
    }
    pub fn clamp(&mut self, size: Vector2) {
        let half = size / (2.0 * self.scale(size));
        let mid = self.bounds.centre();
        let range = self.bounds.size() * 0.5;
        self.centre.x = if half.x >= range.x {
            mid.x
        } else {
            self.centre
                .x
                .clamp(self.bounds.min.x + half.x, self.bounds.max.x - half.x)
        };
        self.centre.y = if half.y >= range.y {
            mid.y
        } else {
            self.centre
                .y
                .clamp(self.bounds.min.y + half.y, self.bounds.max.y - half.y)
        };
    }
}
pub fn circle_marker(
    player: Vector3,
    target: Vector3,
    centre: Vector2,
    radius: f32,
    metres: f32,
) -> (Vector2, bool) {
    let d = Vector2::new(target.x - player.x, target.z - player.z) * (radius / metres);
    let outside = d.length() > radius;
    (
        centre + if outside { d.normalized() * radius } else { d },
        outside,
    )
}
