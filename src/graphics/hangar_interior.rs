//! Interior interpretation of the Citadel's segmented ring and five radial couplings.
//! Independent of selection, ship ownership and exhibition framing.
use raylib::prelude::*;
use std::f32::consts::TAU;

pub struct InteriorStyle {
    pub radius: f32,
    pub height: f32,
    pub radial_bays: usize,
    pub panels: usize,
    pub floor: Color,
    pub graphite: Color,
    pub titanium: Color,
    pub silver: Color,
    pub cyan: Color,
    pub ice: Color,
    pub window: Color,
}
impl Default for InteriorStyle {
    fn default() -> Self {
        Self {
            radius: 12.0,
            height: 6.8,
            radial_bays: 5,
            panels: 60,
            floor: Color::new(17, 25, 36, 255),
            graphite: Color::new(20, 27, 38, 255),
            titanium: Color::new(45, 57, 76, 255),
            silver: Color::new(78, 94, 116, 255),
            cyan: Color::new(49, 139, 180, 255),
            ice: Color::new(155, 202, 222, 255),
            window: Color::new(190, 153, 98, 255),
        }
    }
}
fn polar(radius: f32, y: f32, angle: f32) -> Vector3 {
    Vector3::new(radius * angle.cos(), y, radius * angle.sin())
}
fn quad(d: &mut impl RaylibDraw3D, v: [Vector3; 4], color: Color) {
    d.draw_triangle3D(v[0], v[1], v[2], color);
    d.draw_triangle3D(v[0], v[2], v[3], color);
}
fn ring(d: &mut impl RaylibDraw3D, inner: f32, outer: f32, y: f32, a: f32, b: f32, color: Color) {
    quad(
        d,
        [
            polar(inner, y, a),
            polar(inner, y, b),
            polar(outer, y, b),
            polar(outer, y, a),
        ],
        color,
    );
}
// Radially oriented boxes: shaded faces without extra lights or GPU resources.
fn panel(d: &mut impl RaylibDraw3D, angle: f32, center: Vector3, size: Vector3, color: Color) {
    let transform = |p: Vector3| {
        let p = p + center;
        Vector3::new(
            p.z * angle.cos() + p.x * angle.sin(),
            p.y,
            p.z * angle.sin() - p.x * angle.cos(),
        )
    };
    let h = size * 0.5;
    let v = [
        Vector3::new(-h.x, -h.y, -h.z),
        Vector3::new(h.x, -h.y, -h.z),
        Vector3::new(h.x, h.y, -h.z),
        Vector3::new(-h.x, h.y, -h.z),
        Vector3::new(-h.x, -h.y, h.z),
        Vector3::new(h.x, -h.y, h.z),
        Vector3::new(h.x, h.y, h.z),
        Vector3::new(-h.x, h.y, h.z),
    ]
    .map(transform);
    for (face, shade) in [
        ([0, 3, 2, 1], 1.0),
        ([4, 5, 6, 7], 0.7),
        ([0, 4, 7, 3], 0.75),
        ([1, 2, 6, 5], 0.85),
        ([3, 7, 6, 2], 1.15),
        ([0, 1, 5, 4], 0.6),
    ] {
        let tint = Color::new(
            (color.r as f32 * shade).min(255.0) as u8,
            (color.g as f32 * shade).min(255.0) as u8,
            (color.b as f32 * shade).min(255.0) as u8,
            255,
        );
        quad(d, face.map(|i| v[i]), tint);
    }
}
impl InteriorStyle {
    pub fn draw(&self, d: &mut impl RaylibDraw3D, camera: Camera3D) {
        let r = self.radius;
        let count = self.panels.max(12);
        let step = TAU / count as f32;
        let bays = self.radial_bays.max(1);
        let view = Vector3::new(camera.position.x, 0.0, camera.position.z).normalized();
        for i in 0..count {
            let a = i as f32 * step;
            let b = a + step;
            let mid = (a + b) * 0.5;
            ring(d, 0.0, r, -0.03, a, b, self.floor);
            ring(
                d,
                r * 0.65,
                r * 0.98,
                0.0,
                a + 0.004,
                b - 0.004,
                if i % 3 == 0 {
                    self.graphite
                } else {
                    Color::new(27, 37, 51, 255)
                },
            );
            for radius in [r * 0.65, r * 0.94] {
                ring(
                    d,
                    radius,
                    radius + 0.035,
                    0.012,
                    a + 0.012,
                    b - 0.012,
                    self.cyan,
                );
            }
            d.draw_line_3D(
                polar(5.0, 0.015, a),
                polar(r * 0.65, 0.015, a),
                self.titanium,
            );
            // Architectural cutaway: the near wall/ceiling must never hide the ship.
            if polar(1.0, 0.0, mid).dot(view) > 0.35 {
                continue;
            }
            let bay_distance = (0..bays)
                .map(|j| {
                    let delta = mid - (j as f32 * TAU / bays as f32 + 0.3);
                    delta.sin().atan2(delta.cos()).abs()
                })
                .fold(f32::MAX, f32::min);
            if bay_distance > 0.17 {
                quad(
                    d,
                    [
                        polar(r, 0.0, a),
                        polar(r, 0.0, b),
                        polar(r, self.height, b),
                        polar(r, self.height, a),
                    ],
                    self.graphite,
                );
                panel(
                    d,
                    mid,
                    Vector3::new(0.0, self.height * 0.52, r - 0.08),
                    Vector3::new(r * step * 0.88, self.height * 0.64, 0.14),
                    self.titanium,
                );
                for y in [1.25, 1.55, 5.7] {
                    panel(
                        d,
                        mid,
                        Vector3::new(0.0, y, r - 0.18),
                        Vector3::new(r * step * 0.55, 0.055, 0.03),
                        if i % 5 == 0 { self.ice } else { self.cyan },
                    );
                }
                // Small habitation windows echo the dense light ribbons on the
                // approved exterior, with sparse warm accents inside the cold metal.
                for x in [-0.22, 0.0, 0.22] {
                    panel(
                        d,
                        mid,
                        Vector3::new(x, 4.85, r - 0.18),
                        Vector3::new(0.08, 0.12, 0.03),
                        if i % 4 == 0 { self.window } else { self.ice },
                    );
                }
            }
            // Paired concentric bands and repeated silver ribs recall the Presidium.
            ring(
                d,
                r * 0.82,
                r * 1.06,
                self.height,
                a + 0.004,
                b - 0.004,
                self.titanium,
            );
            for (radius, color) in [(r * 0.82, self.ice), (r * 0.99, self.silver)] {
                d.draw_cylinder_ex(
                    polar(radius, self.height - 0.06, a),
                    polar(radius, self.height - 0.06, b),
                    0.045,
                    0.045,
                    4,
                    color,
                );
            }
            if i % 3 == 0 {
                panel(
                    d,
                    mid,
                    Vector3::new(0.0, self.height - 0.12, r * 0.94),
                    Vector3::new(0.14, 0.36, r * 0.28),
                    self.silver,
                );
            }
        }
        // Five access modules, with nested frames suggesting depth into the arms.
        for j in 0..bays {
            let a = j as f32 * TAU / bays as f32 + 0.3;
            if polar(1.0, 0.0, a).dot(view) > 0.35 {
                continue;
            }
            panel(
                d,
                a,
                Vector3::new(0.0, -0.04, r + 2.0),
                Vector3::new(3.8, 0.12, 5.0),
                self.graphite,
            );
            panel(
                d,
                a,
                Vector3::new(0.0, 2.3, r + 4.5),
                Vector3::new(3.8, 4.6, 0.2),
                Color::new(6, 11, 19, 255),
            );
            for depth in [0.0, 1.8, 3.6] {
                for x in [-1.9, 1.9] {
                    panel(
                        d,
                        a,
                        Vector3::new(x, 2.4, r + depth),
                        Vector3::new(0.3, 4.8, 0.4),
                        self.titanium,
                    );
                    panel(
                        d,
                        a,
                        Vector3::new(x * 0.9, 2.4, r + depth - 0.23),
                        Vector3::new(0.055, 3.9, 0.04),
                        self.cyan,
                    );
                }
                panel(
                    d,
                    a,
                    Vector3::new(0.0, 4.8, r + depth),
                    Vector3::new(4.1, 0.45, 0.5),
                    self.silver,
                );
            }
            for x in [-1.4, 1.4] {
                panel(
                    d,
                    a,
                    Vector3::new(x, 0.035, r + 1.8),
                    Vector3::new(0.045, 0.025, 4.5),
                    self.cyan,
                );
            }
        }
    }
}
