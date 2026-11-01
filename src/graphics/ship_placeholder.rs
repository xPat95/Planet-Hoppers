//! Identifiable temporary ship shared by the hangar and space. Length is in flight units.
use raylib::{ffi, prelude::*};

pub fn draw(d: &mut impl RaylibDraw3D, position: Vector3, rotation: Vector3, length: f32) {
    unsafe {
        ffi::rlPushMatrix();
        ffi::rlTranslatef(position.x, position.y, position.z);
        ffi::rlRotatef(rotation.y, 0.0, 1.0, 0.0);
        ffi::rlRotatef(rotation.x, 1.0, 0.0, 0.0);
        ffi::rlRotatef(rotation.z, 0.0, 0.0, 1.0);
        ffi::rlScalef(length / 4.0, length / 4.0, length / 4.0);
    }
    let hull = Color::new(126, 143, 164, 255);
    let accent = Color::new(255, 158, 72, 255);
    d.draw_cube(Vector3::zero(), 0.9, 0.45, 3.8, hull);
    d.draw_cube(Vector3::new(0.0, -0.05, 0.5), 3.0, 0.16, 1.1, hull);
    d.draw_cube(
        Vector3::new(0.0, 0.35, -0.5),
        0.6,
        0.35,
        0.8,
        Color::new(24, 45, 65, 255),
    );
    for x in [-0.85, 0.85] {
        d.draw_cube(Vector3::new(x, 0.0, 0.9), 0.4, 0.35, 1.0, hull);
        d.draw_cube(Vector3::new(x, 0.0, 1.42), 0.25, 0.2, 0.03, accent);
    }
    d.draw_cube_wires(Vector3::zero(), 0.92, 0.47, 3.82, accent);
    unsafe {
        ffi::rlPopMatrix();
    }
}
