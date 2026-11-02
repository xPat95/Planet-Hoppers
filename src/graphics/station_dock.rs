//! Exterior hatch extension of the lower ring. Its frame is the same configured
//! aperture used by the complementary interior, navigation, collision and launch.
use crate::gameplay::station::Station;
use raylib::prelude::*;

pub fn draw(d: &mut impl RaylibDraw3D, station: &Station) {
    let (r, u, f) = (
        station.exit_right(),
        station.exit_up(),
        station.exit_forward(),
    );
    let point = |x: f32, y: f32, z: f32| station.exit_position() + r * x + u * y + f * z;
    let (w, h) = (station.dock.width * 0.5, station.dock.height * 0.5);
    let mut quad = |corners: [Vector3; 4], color: Color| {
        for p in [corners, [corners[3], corners[2], corners[1], corners[0]]] {
            d.draw_triangle3D(p[0], p[1], p[2], color);
            d.draw_triangle3D(p[0], p[2], p[3], color);
        }
    };
    // Dark recess overlays a small section of existing armor, without editing the GLB.
    quad(
        [
            point(-w, -h, 0.1),
            point(w, -h, 0.1),
            point(w, h, 0.1),
            point(-w, h, 0.1),
        ],
        Color::new(3, 8, 15, 255),
    );
    quad(
        [
            point(-w - 1.0, -h - 1.0, 0.0),
            point(w + 1.0, -h - 1.0, 0.0),
            point(w + 1.0, -h - 1.0, 9.0),
            point(-w - 1.0, -h - 1.0, 9.0),
        ],
        Color::new(45, 57, 76, 255),
    );
    drop(quad);
    for side in [-1.0, 1.0] {
        d.draw_cylinder_ex(
            point(side * (w + 0.4), -h, 0.8),
            point(side * (w + 0.4), h, 0.8),
            0.45,
            0.45,
            6,
            Color::new(95, 113, 136, 255),
        );
        d.draw_cylinder_ex(
            point(side * (w - 0.2), -h + 0.4, 1.0),
            point(side * (w - 0.2), h - 0.4, 1.0),
            0.08,
            0.08,
            4,
            Color::new(109, 222, 239, 255),
        );
        d.draw_sphere(
            point(side * (w + 1.6), 0.0, 1.0),
            0.3,
            Color::new(215, 163, 94, 255),
        );
    }
    d.draw_cylinder_ex(
        point(-w, h + 0.4, 0.8),
        point(w, h + 0.4, 0.8),
        0.45,
        0.45,
        6,
        Color::new(95, 113, 136, 255),
    );
}
