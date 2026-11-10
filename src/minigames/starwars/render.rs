use super::simulation::*;
use raylib::{ffi, prelude::*};
use std::f32::consts::TAU;
const ICE: Color = Color::new(135, 218, 237, 255);
const AMBER: Color = Color::new(255, 179, 78, 255);
const WHITE: Color = Color::new(226, 233, 232, 255);
const MUTED: Color = Color::new(128, 153, 163, 255);
const RED: Color = Color::new(255, 83, 54, 255);
const GOLD: Color = Color::new(255, 218, 52, 255);
const MENU_BLUE: Color = Color::new(74, 159, 255, 255);
const MENU_CYAN: Color = Color::new(132, 239, 255, 255);
/// Uses the same logical rectangles for rendering and scaled mouse hit testing.
pub fn victory_buttons(w: f32, h: f32) -> [Rectangle; 2] {
    let scale = (w / 1280.0).min(h / 800.0);
    let ox = (w - 1280.0 * scale) * 0.5;
    let oy = (h - 800.0 * scale) * 0.5;
    [460.0, 540.0].map(|y| {
        Rectangle::new(
            ox + 280.0 * scale,
            oy + y * scale,
            720.0 * scale,
            60.0 * scale,
        )
    })
}

pub fn menu_buttons(phase: Phase, w: f32, h: f32) -> Vec<Rectangle> {
    if matches!(phase, Phase::Briefing | Phase::Won) {
        return victory_buttons(w, h).to_vec();
    }
    if !matches!(phase, Phase::Paused | Phase::Lost) {
        return vec![];
    }
    let scale = (w / 1280.0).min(h / 800.0);
    let ox = (w - 1280.0 * scale) * 0.5;
    let oy = (h - 800.0 * scale) * 0.5;
    [414.0, 486.0, 558.0]
        .map(|y| {
            Rectangle::new(
                ox + 280.0 * scale,
                oy + y * scale,
                720.0 * scale,
                60.0 * scale,
            )
        })
        .to_vec()
}

fn v(x: f32, y: f32, z: f32) -> Vector3 {
    Vector3::new(x, y, z)
}
fn polar(r: f32, a: f32, z: f32) -> Vector3 {
    v(r * a.cos(), r * a.sin(), z)
}
fn shade(c: Color, k: f32) -> Color {
    Color::new(
        (c.r as f32 * k).min(255.0) as u8,
        (c.g as f32 * k).min(255.0) as u8,
        (c.b as f32 * k).min(255.0) as u8,
        c.a,
    )
}
fn fog(c: Color, z: f32) -> Color {
    let k = (-z.max(0.0) * 0.009).exp();
    Color::new(
        (c.r as f32 * k + 5.0 * (1.0 - k)) as u8,
        (c.g as f32 * k + 13.0 * (1.0 - k)) as u8,
        (c.b as f32 * k + 20.0 * (1.0 - k)) as u8,
        c.a,
    )
}
fn quad(d: &mut impl RaylibDraw3D, a: Vector3, b: Vector3, c: Vector3, e: Vector3, color: Color) {
    d.draw_triangle3D(a, b, c, color);
    d.draw_triangle3D(a, c, e, color);
    d.draw_triangle3D(c, b, a, color);
    d.draw_triangle3D(e, c, a, color);
}
fn beam(d: &mut impl RaylibDraw3D, a: Vector3, b: Vector3, r: f32, c: Color) {
    d.draw_cylinder_ex(a, b, r, r, 6, c);
}
fn arc(d: &mut impl RaylibDraw3D, r: f32, z: f32, a: f32, b: f32, c: Color) {
    for i in 0..64 {
        let t = i as f32 / 64.0;
        let u = (i + 1) as f32 / 64.0;
        beam(
            d,
            polar(r, a + (b - a) * t, z),
            polar(r, a + (b - a) * u, z),
            0.055,
            c,
        );
    }
}
fn box3(d: &mut impl RaylibDraw3D, p: Vector3, h: Vector3, c: Color) {
    let a = p - v(h.x, h.y, h.z);
    let b = p + v(h.x, h.y, h.z);
    quad(
        d,
        v(a.x, a.y, a.z),
        v(b.x, a.y, a.z),
        v(b.x, b.y, a.z),
        v(a.x, b.y, a.z),
        c,
    );
    quad(
        d,
        v(a.x, b.y, a.z),
        v(b.x, b.y, a.z),
        v(b.x, b.y, b.z),
        v(a.x, b.y, b.z),
        shade(c, 1.2),
    );
    quad(
        d,
        v(a.x, a.y, a.z),
        v(a.x, b.y, a.z),
        v(a.x, b.y, b.z),
        v(a.x, a.y, b.z),
        shade(c, 0.52),
    );
    quad(
        d,
        v(b.x, a.y, a.z),
        v(b.x, b.y, a.z),
        v(b.x, b.y, b.z),
        v(b.x, a.y, b.z),
        shade(c, 0.72),
    );
    quad(
        d,
        v(a.x, a.y, a.z),
        v(b.x, a.y, a.z),
        v(b.x, a.y, b.z),
        v(a.x, a.y, b.z),
        shade(c, 0.36),
    );
    quad(
        d,
        v(a.x, a.y, b.z),
        v(b.x, a.y, b.z),
        v(b.x, b.y, b.z),
        v(a.x, b.y, b.z),
        shade(c, 0.5),
    );
}
fn wall_point(side: usize, t: f32, z: f32, inset: f32) -> Vector3 {
    let a = (side as f32 + 0.5) * TAU / 8.0;
    v(
        a.cos() * (WALL_APOTHEM - inset) - a.sin() * t,
        a.sin() * (WALL_APOTHEM - inset) + a.cos() * t,
        z,
    )
}
fn wall_panel(
    d: &mut impl RaylibDraw3D,
    side: usize,
    t: [f32; 2],
    z: [f32; 2],
    inset: f32,
    color: Color,
) {
    quad(
        d,
        wall_point(side, t[0], z[0], inset),
        wall_point(side, t[1], z[0], inset),
        wall_point(side, t[1], z[1], inset),
        wall_point(side, t[0], z[1], inset),
        fog(color, (z[0] + z[1]) * 0.5),
    );
}
fn tunnel(d: &mut impl RaylibDraw3D, g: &Game, time: f32) {
    let travel = if g.phase == Phase::Briefing {
        time * 12.0
    } else {
        g.travel
    };
    let offset = travel.rem_euclid(14.0);
    let tile = (travel / 14.0) as usize;
    for ring in (0..19).rev() {
        let z = ring as f32 * 14.0 - offset - 16.0;
        for side in 0..8 {
            let base = match side {
                0 | 3 => Color::new(25, 68, 96, 255),
                1 | 2 => Color::new(43, 56, 100, 255),
                4 | 7 => Color::new(20, 75, 78, 255),
                _ => Color::new(18, 37, 60, 255),
            };
            wall_panel(d, side, [-4.35, 4.35], [z, z + 14.0], 0.0, base);
            // Recessed panels, raised ribs and service plates; details stay anchored to the world.
            for col in 0..3 {
                let t = -3.9 + col as f32 * 2.65;
                wall_panel(
                    d,
                    side,
                    [t, t + 2.45],
                    [z + 0.6, z + 12.8],
                    0.045,
                    Color::new(5, 15, 28, 255),
                );
                wall_panel(
                    d,
                    side,
                    [t + 0.08, t + 2.36],
                    [z + 0.8, z + 7.6],
                    0.07,
                    shade(base, 1.25),
                );
                wall_panel(
                    d,
                    side,
                    [t + 0.08, t + 2.36],
                    [z + 7.8, z + 12.5],
                    0.07,
                    shade(base, 0.91),
                );
                wall_panel(
                    d,
                    side,
                    [t + 0.2, t + 2.2],
                    [z + 1.15, z + 1.27],
                    0.09,
                    Color::new(63, 137, 166, 255),
                );
                for j in 0..5 {
                    wall_panel(
                        d,
                        side,
                        [t + 0.3, t + 1.6],
                        [z + 9.0 + j as f32 * 0.42, z + 9.18 + j as f32 * 0.42],
                        0.1,
                        Color::new(12, 24, 31, 255),
                    );
                }
            }
            let a = side as f32 * TAU / 8.0;
            let b = (side + 1) as f32 * TAU / 8.0;
            beam(
                d,
                polar(10.12, a, z),
                polar(10.12, b, z),
                0.22,
                fog(Color::new(79, 139, 172, 255), z),
            );
            beam(
                d,
                polar(10.12, a, z + 0.45),
                polar(10.12, b, z + 0.45),
                0.1,
                fog(Color::new(21, 34, 42, 255), z),
            );
            for t in [-3.7, 3.7] {
                beam(
                    d,
                    wall_point(side, t, z, 0.25),
                    wall_point(side, t, z + 14.0, 0.25),
                    0.11,
                    fog(Color::new(30, 176, 198, 255), z),
                );
            }
            if side % 2 == 0 {
                for t in [-2.75, 2.75] {
                    wall_panel(
                        d,
                        side,
                        [t - 0.21, t + 0.21],
                        [z + 3.0, z + 9.5],
                        0.14,
                        Color::new(6, 15, 21, 255),
                    );
                    wall_panel(
                        d,
                        side,
                        [t - 0.075, t + 0.075],
                        [z + 3.3, z + 9.2],
                        0.16,
                        if side == 0 || side == 4 {
                            ICE
                        } else {
                            Color::new(183, 200, 255, 255)
                        },
                    );
                }
            } else if (tile + ring + side).is_multiple_of(3) {
                wall_panel(d, side, [-0.3, 0.3], [z + 4.0, z + 5.8], 0.17, RED);
            }
            // Service cassettes and conduits break up the large plates. All remain
            // inside the 0.4 m service layer excluded from flight collision space.
            if (tile + ring + side).is_multiple_of(2) {
                wall_panel(
                    d,
                    side,
                    [-1.05, 1.05],
                    [z + 2.0, z + 6.7],
                    0.08,
                    Color::new(8, 18, 24, 255),
                );
                let p = wall_point(side, 0.0, z + 4.1, 0.15);
                unsafe {
                    ffi::rlPushMatrix();
                    ffi::rlTranslatef(p.x, p.y, p.z);
                    ffi::rlRotatef(
                        ((side as f32 + 0.5) * TAU / 8.0).to_degrees() + 90.0,
                        0.0,
                        0.0,
                        1.0,
                    );
                }
                box3(
                    d,
                    Vector3::zero(),
                    v(0.86, 0.13, 1.8),
                    fog(Color::new(72, 85, 89, 255), z),
                );
                box3(
                    d,
                    v(-0.45, 0.15, 0.0),
                    v(0.29, 0.03, 1.47),
                    fog(Color::new(28, 41, 48, 255), z),
                );
                for j in 0..6 {
                    box3(
                        d,
                        v(-0.45, 0.19, -1.15 + j as f32 * 0.43),
                        v(0.23, 0.02, 0.055),
                        fog(Color::new(113, 123, 120, 255), z),
                    );
                }
                for x in [0.1, 0.35, 0.6] {
                    beam(
                        d,
                        v(x, 0.17, -1.5),
                        v(x, 0.17, 1.5),
                        0.055,
                        fog(Color::new(126, 113, 87, 255), z),
                    );
                }
                unsafe {
                    ffi::rlPopMatrix();
                }
            }
        }
    }
    for i in 0..64 {
        let a = i as f32 * 2.399_963;
        let z = (i as f32 * 7.73 - travel * 1.3).rem_euclid(190.0);
        let r = 7.9 + (i as f32).sin() * 0.7;
        d.draw_line_3D(
            polar(r, a, z),
            polar(r, a, z + 0.3 + g.speed * 0.015),
            fog(Color::new(113, 145, 155, 255), z),
        );
    }
}
fn obstacle(d: &mut impl RaylibDraw3D, o: &Obstacle, g: &Game) {
    let z = o.pos[2] - g.distance;
    if !o.alive() || z + o.half[2] < -g.ship_half[2] || z - o.half[2] >= 240.0 {
        return;
    }
    let h = v(o.half[0], o.half[1], o.half[2]);
    let metal = fog(
        if o.flash > 0.2 {
            WHITE
        } else {
            let variation = (o.seed % 4) as u8 * 5;
            match o.shape {
                Shape::Girder => Color::new(153 + variation, 175 + variation, 193 + variation, 255),
                Shape::Pipe => Color::new(71 + variation, 157 + variation, 142 + variation, 255),
                Shape::Duct => Color::new(105 + variation, 124 + variation, 164 + variation, 255),
                Shape::Panel => Color::new(213 + variation, 132 + variation, 48 + variation, 255),
            }
        },
        z,
    );
    let dark = fog(Color::new(44, 52, 56, 255), z);
    let edge = fog(Color::new(191, 215, 230, 255), z);
    unsafe {
        ffi::rlPushMatrix();
        ffi::rlTranslatef(o.pos[0], o.pos[1], z);
        ffi::rlRotatef(o.angle.to_degrees(), 0.0, 0.0, 1.0);
    }
    match o.shape {
        Shape::Girder => {
            // I section: flanges and web fit inside the shared collision envelope.
            box3(
                d,
                Vector3::zero(),
                v(h.x, h.y * 0.22, h.z),
                shade(metal, 0.72),
            );
            for y in [-1.0, 1.0] {
                box3(
                    d,
                    v(0.0, y * h.y * 0.82, 0.0),
                    v(h.x, h.y * 0.18, h.z),
                    metal,
                );
            }
            // End splice plates and recessed rivets; never extend into an open bay.
            for x in [-1.0, 1.0] {
                box3(
                    d,
                    v(x * (h.x - 0.1).max(0.0), 0.0, -h.z + 0.025),
                    v(h.x.min(0.1), h.y, 0.025),
                    edge,
                );
                box3(
                    d,
                    v(x * (h.x - 0.1).max(0.0), 0.0, -h.z - 0.002),
                    v(0.035, h.y.min(0.04), 0.012),
                    dark,
                );
            }
        }
        Shape::Pipe => {
            // Square conservative collision envelope follows each short pipe, not the whole elbow.
            let radius = h.y.min(h.z);
            beam(d, v(-h.x, 0.0, 0.0), v(h.x, 0.0, 0.0), radius * 0.88, metal);
            for x in [-h.x + 0.12, 0.0, h.x - 0.12] {
                beam(
                    d,
                    v(x - 0.08, 0.0, 0.0),
                    v(x + 0.08, 0.0, 0.0),
                    radius,
                    edge,
                );
            }
        }
        Shape::Duct => {
            box3(d, Vector3::zero(), h, shade(metal, 0.76));
            let front = -h.z - 0.015;
            // Broad seams, inset ventilation banks and small service numbers.
            let nx = (h.x / 1.5).ceil().max(1.0) as usize;
            let ny = (h.y / 1.8).ceil().max(1.0) as usize;
            for ix in 0..nx {
                for iy in 0..ny {
                    let x = -h.x + (ix as f32 + 0.5) * 2.0 * h.x / nx as f32;
                    let y = -h.y + (iy as f32 + 0.5) * 2.0 * h.y / ny as f32;
                    let w = (h.x / nx as f32 - 0.12).max(0.05);
                    let height = (h.y / ny as f32 - 0.12).max(0.05);
                    box3(d, v(x, y, front), v(w, height, 0.015), dark);
                    for j in 0..5 {
                        box3(
                            d,
                            v(x, y + height * (j as f32 - 2.0) * 0.32, front - 0.022),
                            v(w * 0.82, height * 0.055, 0.018),
                            metal,
                        );
                    }
                }
            }
            for y in [-h.y + 0.06, h.y - 0.06] {
                box3(d, v(0.0, y, front), v(h.x, 0.045, 0.035), edge);
            }
        }
        Shape::Panel => {
            let amber = fog(
                if o.flash > 0.2 {
                    WHITE
                } else {
                    Color::new(255, 183, 58, 255)
                },
                z,
            );
            box3(d, Vector3::zero(), h, dark);
            for x in [-h.x + 0.1, h.x - 0.1] {
                box3(d, v(x, 0.0, -h.z), v(0.1, h.y, 0.04), amber);
            }
            for y in [-h.y + 0.1, h.y - 0.1] {
                box3(d, v(0.0, y, -h.z), v(h.x, 0.1, 0.04), amber);
            }
            for j in 0..9 {
                let y = (j as f32 - 4.0) * h.y * 0.19;
                box3(
                    d,
                    v(0.0, y, -h.z - 0.015),
                    v(h.x * 0.87, 0.035, 0.035),
                    metal,
                );
            }
            box3(d, v(0.0, 0.0, -h.z - 0.08), v(0.4, 0.22, 0.04), dark);
            for j in 0..3 {
                box3(
                    d,
                    v((j as f32 - 1.0) * 0.19, 0.0, -h.z - 0.13),
                    v(0.035, 0.12, 0.02),
                    if j < o.hp { fog(AMBER, z) } else { dark },
                );
            }
        }
    }
    unsafe {
        ffi::rlPopMatrix();
    }
}
fn reactor(d: &mut impl RaylibDraw3D, g: &Game, time: f32) {
    let z = REACTOR_Z - g.distance;
    if z > 250.0 {
        return;
    }
    if matches!(g.phase, Phase::Explosion | Phase::Outro | Phase::Won) {
        let t = g.end_time;
        d.draw_sphere_ex(
            v(0.0, 0.0, z),
            2.0 + t * 15.0,
            20,
            24,
            Color::new(255, 169, 70, (255.0 * (1.0 - t / 1.5).max(0.0)) as u8),
        );
        arc(d, 3.0 + t * 35.0, z - t * 15.0, 0.0, TAU, WHITE);
        return;
    }
    for side in 0..8 {
        let a = side as f32 * TAU / 8.0;
        beam(
            d,
            polar(3.7, a, z),
            polar(10.3, a, z + 6.0),
            0.5,
            fog(Color::new(113, 122, 125, 255), z),
        );
        beam(
            d,
            polar(4.0, a, z - 1.0),
            polar(9.0, a, z + 4.0),
            0.08,
            fog(AMBER, z),
        );
    }
    for layer in 0..3 {
        let zz = z + layer as f32 * 1.5;
        arc(
            d,
            3.1 + layer as f32 * 0.7,
            zz,
            0.0,
            TAU,
            fog(Color::new(125, 149, 155, 255), z),
        );
        for i in 0..12 {
            let a = i as f32 * TAU / 12.0 + time * 0.08;
            beam(d, polar(3.0, a, zz), polar(3.5, a, zz), 0.12, fog(ICE, z));
        }
    }
    d.draw_sphere_ex(
        v(0.0, 0.0, z),
        CORE_RADIUS,
        24,
        32,
        Color::new(242, 112, 37, 255),
    );
    d.draw_sphere_wires(v(0.0, 0.0, z), CORE_RADIUS + 0.07, 12, 16, AMBER);
    for i in 0..6 {
        let a = time * 0.5 + i as f32 * TAU / 6.0;
        arc(
            d,
            2.35,
            z - 0.3,
            a,
            a + 0.35,
            if i < g.reactor_hp {
                WHITE
            } else {
                Color::new(56, 49, 42, 255)
            },
        );
    }
    d.draw_sphere_ex(
        v(-0.3, 0.35, z - 1.75),
        0.7,
        12,
        16,
        Color::new(255, 235, 182, 255),
    );
}
pub fn camera(g: &Game) -> Camera3D {
    let shake = g.flash * 0.06;
    Camera3D::perspective(
        v(
            g.camera[0] * 0.58 + (g.time * 70.0).sin() * shake,
            2.25 + g.camera[1] * 0.57,
            -10.7,
        ),
        v(g.camera[0] * 0.68, 0.8 + g.camera[1] * 0.68, 38.0),
        Vector3::up(),
        62.0 + (g.speed - 36.0).max(0.0) * 0.12,
    )
}
fn world(d: &mut impl RaylibDraw, g: &Game, time: f32, ship: &mut super::ship::TunnelShip) {
    d.clear_background(Color::new(5, 13, 20, 255));
    let mut view = d.begin_mode3D(camera(g));
    tunnel(&mut view, g, time);
    for o in g.obstacles.iter().rev() {
        obstacle(&mut view, o, g);
    }
    reactor(&mut view, g, time);
    for s in &g.shots {
        let p = v(s.pos[0], s.pos[1], s.pos[2] - g.distance);
        if s.torpedo {
            beam(
                &mut view,
                p - v(0.0, 0.0, 0.7),
                p + v(0.0, 0.0, 0.4),
                0.14,
                WHITE,
            );
            view.draw_cylinder_ex(
                p + v(0.0, 0.0, 0.4),
                p + v(0.0, 0.0, 0.8),
                0.15,
                0.0,
                8,
                RED,
            );
            beam(
                &mut view,
                p - v(0.0, 0.0, 0.7),
                p - v(0.0, 0.0, 2.5),
                0.09,
                AMBER,
            );
            for i in 0..7 {
                let age = i as f32 / 7.0;
                view.draw_sphere_ex(
                    p - v(0.0, 0.0, 2.5 + age * 6.0),
                    0.13 + age * 0.35,
                    5,
                    6,
                    Color::new(119, 134, 141, (130.0 * (1.0 - age)) as u8),
                );
            }
        } else {
            beam(
                &mut view,
                p,
                p - v(0.0, 0.0, 2.9),
                0.035,
                Color::new(110, 255, 139, 255),
            );
            beam(&mut view, p, p - v(0.0, 0.0, 2.1), 0.016, WHITE);
        }
    }
    if let Some(a) = g.rear_attack {
        if !a.warning() {
            let z = -18.0 + (a.age - REAR_WARNING) / REAR_FLIGHT * 18.0;
            for x in [-0.16, 0.16] {
                let p = v(a.target[0] + x, a.target[1], z);
                beam(
                    &mut view,
                    p - v(0.0, 0.0, 4.0),
                    p,
                    0.075,
                    Color::new(84, 255, 102, 255),
                );
                beam(&mut view, p - v(0.0, 0.0, 3.5), p, 0.028, WHITE);
            }
        }
    }
    ship.draw(&mut view, g, camera(g).position);
    for p in &g.particles {
        let q = v(p.pos[0], p.pos[1], p.pos[2] - g.distance);
        let c = if p.debris && p.age > 0.2 {
            Color::new(112, 125, 129, (255.0 * (1.0 - p.age / p.life)) as u8)
        } else {
            AMBER
        };
        if p.debris {
            unsafe {
                ffi::rlPushMatrix();
                ffi::rlTranslatef(q.x, q.y, q.z);
                ffi::rlRotatef(p.seed + p.age * 180.0, 1.0, 0.7, 0.3);
            }
            box3(
                &mut view,
                Vector3::zero(),
                v(p.size, p.size * 0.45, p.size * 1.5),
                c,
            );
            unsafe {
                ffi::rlPopMatrix();
            }
        } else {
            view.draw_line_3D(
                q,
                q - v(p.velocity[0], p.velocity[1], p.velocity[2]) * 0.035,
                c,
            );
        }
    }
}
fn load_ui_font(_thread: &RaylibThread, root: &std::path::Path) -> Result<Font, String> {
    let path = root.join("fonts/TitilliumWeb-Bold.ttf");
    load_font_file(&path)
}
fn load_font_file(path: &std::path::Path) -> Result<Font, String> {
    if !path.is_file() {
        return Err(format!("No se encontró la fuente: {}", path.display()));
    }
    let filename =
        std::ffi::CString::new(path.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    let mut codepoints: Vec<i32> = (32..255).collect();
    // raylib-rs 5.5.1 load_font_ex(Some(str)) passes the UTF-8 BYTE length
    // as the number of i32 codepoints. Latin-1 needs 223 codepoints, not 350.
    // Pass the actual contiguous array and its element count directly instead.
    let raw = unsafe {
        ffi::LoadFontEx(
            filename.as_ptr(),
            80,
            codepoints.as_mut_ptr(),
            codepoints.len() as i32,
        )
    };
    if raw.texture.id == unsafe { ffi::GetFontDefault().texture.id } {
        // A fallback is borrowed from Raylib and must not become an owned Font.
        return Err("No se pudo cargar la fuente de la interfaz".into());
    }
    if raw.glyphs.is_null() || raw.texture.id == 0 {
        return Err("Fuente de interfaz inválida".into());
    }
    // LoadFontEx has copied the codepoints and owns its returned glyph/atlas data.
    // Transfer that ownership exactly once; Font drops it on this rendering thread.
    Ok(unsafe { Font::from_raw(raw) })
}

pub struct Presentation {
    pub ship: super::ship::TunnelShip,
    pub target: RenderTexture2D,
    pub shader: Shader,
    pub font: Font,
    pub title_font: Font,
    pub menu_selection: usize,
}
impl Presentation {
    pub fn load(
        rl: &mut RaylibHandle,
        t: &RaylibThread,
        root: &std::path::Path,
        project_root: &std::path::Path,
        selected: &crate::ships::ShipModel,
    ) -> Result<Self, String> {
        let target = rl
            .load_render_texture(t, 1920, 1200)
            .map_err(|e| e.to_string())?;
        let shader = rl.load_shader_from_memory(t, None, Some(include_str!("finish.fs")));
        target
            .texture()
            .set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let font = load_ui_font(t, root)?;
        font.texture()
            .set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let title_font = load_font_file(&project_root.join("assets/fonts/Bungee-Regular.ttf"))?;
        title_font
            .texture()
            .set_texture_filter(t, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let ship = super::ship::TunnelShip::load(rl, t, project_root, selected)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            ship,
            target,
            shader,
            font,
            title_font,
            menu_selection: 0,
        })
    }
    pub fn draw(
        &mut self,
        d: &mut RaylibDrawHandle,
        t: &RaylibThread,
        g: &Game,
        time: f32,
        w: f32,
        h: f32,
    ) {
        {
            let mut canvas = d.begin_texture_mode(t, &mut self.target);
            world(&mut canvas, g, time, &mut self.ship);
        }
        d.clear_background(Color::BLACK);
        {
            let mut post = d.begin_shader_mode(&mut self.shader);
            post.draw_texture_pro(
                self.target.texture(),
                Rectangle::new(0.0, 0.0, 1920.0, -1200.0),
                {
                    let scale = (w / 1280.0).min(h / 800.0);
                    Rectangle::new(
                        (w - 1280.0 * scale) * 0.5,
                        (h - 800.0 * scale) * 0.5,
                        1280.0 * scale,
                        800.0 * scale,
                    )
                },
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        let scale = (w / 1280.0).min(h / 800.0);
        let cam = Camera2D {
            offset: Vector2::new((w - 1280.0 * scale) * 0.5, (h - 800.0 * scale) * 0.5),
            target: Vector2::zero(),
            rotation: 0.0,
            zoom: scale,
        };
        let mut ui = d.begin_mode2D(cam);
        self.ui(&mut ui, g, time);
    }
    fn text(&self, d: &mut impl RaylibDraw, s: &str, x: f32, y: f32, size: f32, c: Color) {
        d.draw_text_ex(&self.font, s, Vector2::new(x, y), size, 1.0, c);
    }
    fn centered(&self, d: &mut impl RaylibDraw, s: &str, y: f32, size: f32, color: Color) {
        let x = (1280.0 - self.font.measure_text(s, size, 1.0).x) * 0.5;
        for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 3.0)] {
            self.text(d, s, x + dx, y + dy, size, Color::BLACK);
        }
        self.text(d, s, x, y, size, color);
    }
    fn stud(d: &mut impl RaylibDraw, x: i32, y: i32, r: f32, color: Color) {
        d.draw_circle(x + 2, y + 4, r + 2.0, Color::new(0, 0, 0, 160));
        d.draw_circle(x, y, r, shade(color, 0.55));
        d.draw_circle(x, y - 3, r * 0.84, color);
        d.draw_circle_lines(x, y - 3, r * 0.57, shade(color, 1.25));
    }
    fn wordmark(&self, d: &mut impl RaylibDraw, word: &str, y: f32) {
        let size = 102.0;
        let x = (1280.0 - self.title_font.measure_text(word, size, 2.0).x) * 0.5;
        for (dx, dy) in [(-3.0, 0.0), (3.0, 0.0), (0.0, -3.0), (0.0, 5.0)] {
            d.draw_text_ex(
                &self.title_font,
                word,
                Vector2::new(x + dx, y + dy),
                size,
                2.0,
                Color::BLACK,
            );
        }
        d.draw_text_ex(&self.title_font, word, Vector2::new(x, y), size, 2.0, GOLD);
    }
    fn menu_backdrop(&self, d: &mut impl RaylibDraw, time: f32, title: &str) {
        d.draw_rectangle(0, 0, 1280, 800, Color::new(2, 6, 20, 242));
        d.draw_rectangle_gradient_v(
            0,
            0,
            1280,
            800,
            Color::new(13, 23, 58, 160),
            Color::new(1, 3, 12, 240),
        );
        for i in 0..145 {
            let x = ((i * 197 + 31) % 1280) as i32;
            let y = ((i * 113 + 17) % 800) as i32;
            let alpha = (125.0 + (time * 0.7 + i as f32).sin() * 65.0) as u8;
            d.draw_circle(
                x,
                y,
                if i % 9 == 0 { 1.7 } else { 0.8 },
                Color::new(183, 213, 255, alpha),
            );
        }
        // A solid station silhouette replaces the old tactical grid.
        d.draw_circle(1100, 302, 162.0, Color::new(24, 37, 62, 255));
        d.draw_circle_lines(1100, 302, 162.0, Color::new(51, 76, 112, 255));
        for y in [-115.0_f32, -65.0, 0.0, 65.0, 115.0] {
            let half = (162.0_f32.powi(2) - y.powi(2)).sqrt() as i32;
            d.draw_line(
                1100 - half,
                302 + y as i32,
                1100 + half,
                302 + y as i32,
                Color::new(44, 63, 90, 255),
            );
        }
        d.draw_ellipse_lines(1100, 302, 65.0, 162.0, Color::new(44, 63, 90, 255));
        d.draw_rectangle(940, 302, 320, 7, Color::new(6, 12, 28, 255));
        d.draw_circle(1050, 235, 36.0, Color::new(8, 17, 33, 255));
        d.draw_circle_lines(1050, 235, 36.0, Color::new(66, 93, 120, 255));
        self.centered(d, "PLANET HOPPERS", 40.0, 20.0, WHITE);
        self.wordmark(d, "STAR", 64.0);
        self.wordmark(d, "WARS", 153.0);
        d.draw_line(436, 270, 844, 270, GOLD);
        self.centered(d, title, 283.0, 24.0, WHITE);
        for (i, x) in [92, 134, 176, 1104, 1146, 1188].iter().enumerate() {
            Self::stud(d, *x, 735, 12.0, if i % 3 == 1 { WHITE } else { GOLD });
        }
        self.centered(
            d,
            "ASALTO AL NÚCLEO  /  ESTRELLA DE LA MUERTE",
            748.0,
            16.0,
            MUTED,
        );
    }
    fn menu_option(&self, d: &mut impl RaylibDraw, label: &str, index: usize, y: f32) {
        let active = self.menu_selection == index;
        if active {
            d.draw_rectangle_gradient_h(
                280,
                y as i32,
                360,
                60,
                Color::new(12, 39, 71, 0),
                Color::new(28, 94, 145, 125),
            );
            d.draw_rectangle_gradient_h(
                640,
                y as i32,
                360,
                60,
                Color::new(28, 94, 145, 125),
                Color::new(12, 39, 71, 0),
            );
            self.text(d, ">", 300.0, y + 6.0, 36.0, GOLD);
            self.text(d, "<", 958.0, y + 6.0, 36.0, GOLD);
        }
        self.centered(
            d,
            label,
            y + 8.0,
            32.0,
            if active { MENU_CYAN } else { MENU_BLUE },
        );
    }
    fn briefing(&self, d: &mut impl RaylibDraw, time: f32) {
        self.menu_backdrop(d, time, "UNA NUEVA MISIÓN");
        self.centered(
            d,
            "Pilota por el túnel. Abre las compuertas.",
            354.0,
            23.0,
            WHITE,
        );
        self.centered(
            d,
            "Destruye el reactor con seis torpedos.",
            385.0,
            23.0,
            WHITE,
        );
        self.menu_option(d, "INICIAR MISIÓN", 0, 460.0);
        self.menu_option(d, "VOLVER AL SISTEMA SOLAR", 1, 540.0);
        self.centered(
            d,
            "WASD / FLECHAS  Volar     ESPACIO  Disparar     SHIFT  Turbo",
            641.0,
            19.0,
            WHITE,
        );
        self.centered(
            d,
            "ENTER  Seleccionar     ARRIBA / ABAJO  Elegir     ESC  Volver",
            679.0,
            18.0,
            GOLD,
        );
        self.centered(
            d,
            &format!(
                "OBJETIVO: {:.0} M     /     3 BARRAS DE INTEGRIDAD",
                REACTOR_Z
            ),
            712.0,
            16.0,
            MUTED,
        );
    }
    fn flight_menu(&self, d: &mut impl RaylibDraw, g: &Game, time: f32) {
        let lost = g.phase == Phase::Lost;
        self.menu_backdrop(d, time, if lost { "MISIÓN FALLIDA" } else { "PAUSA" });
        self.centered(
            d,
            if lost {
                "¡La galaxia necesita otro intento!"
            } else {
                "Que la Fuerza te acompañe."
            },
            354.0,
            28.0,
            if lost { RED } else { GOLD },
        );
        self.menu_option(
            d,
            if lost {
                "REINTENTAR MISIÓN"
            } else {
                "CONTINUAR"
            },
            0,
            414.0,
        );
        self.menu_option(d, "REINICIAR DESDE LA INTRO", 1, 486.0);
        self.menu_option(d, "VOLVER AL SISTEMA SOLAR", 2, 558.0);
        self.centered(
            d,
            &format!(
                "{:06} PUNTOS     /     {:.0} M RECORRIDOS",
                g.score, g.distance
            ),
            648.0,
            20.0,
            WHITE,
        );
        self.centered(
            d,
            "ENTER  Seleccionar     ARRIBA / ABAJO  Elegir     R  Reiniciar     ESC  Volver",
            690.0,
            17.0,
            GOLD,
        );
    }
    fn victory_menu(&self, d: &mut impl RaylibDraw, g: &Game, time: f32) {
        self.menu_backdrop(d, time, "MISIÓN CUMPLIDA");
        self.centered(d, "¡REACTOR DESTRUIDO!", 344.0, 34.0, GOLD);
        Self::stud(d, 475, 416, 13.0, GOLD);
        self.centered(d, &format!("{:06} PUNTOS", g.score), 398.0, 28.0, WHITE);
        self.menu_option(d, "VOLVER A JUGAR", 0, 460.0);
        self.menu_option(d, "REGRESAR AL SISTEMA SOLAR", 1, 540.0);
        self.centered(
            d,
            &format!(
                "{} CARGAS DESTRUIDAS     /     INTEGRIDAD: {} DE 3",
                g.destroyed, g.hull
            ),
            639.0,
            20.0,
            WHITE,
        );
        self.centered(
            d,
            "ENTER  Seleccionar     ARRIBA / ABAJO  Elegir     ESC  Volver",
            679.0,
            18.0,
            GOLD,
        );
        self.centered(
            d,
            "Una nueva aventura te espera con tu misma nave.",
            714.0,
            17.0,
            MUTED,
        );
    }
    fn ui(&self, d: &mut impl RaylibDraw, g: &Game, time: f32) {
        if g.phase == Phase::Won {
            self.victory_menu(d, g, time);
            return;
        }
        if g.phase == Phase::Briefing {
            self.briefing(d, time);
            return;
        }
        if matches!(g.phase, Phase::Intro | Phase::Outro) {
            return;
        }
        d.draw_rectangle_gradient_v(
            0,
            0,
            1280,
            145,
            Color::new(3, 9, 15, 175),
            Color::new(3, 9, 15, 0),
        );
        d.draw_rectangle_gradient_v(
            0,
            650,
            1280,
            150,
            Color::new(3, 9, 15, 0),
            Color::new(3, 9, 15, 210),
        );
        if let Some(a) = g
            .rear_attack
            .filter(|_| matches!(g.phase, Phase::Running | Phase::Paused))
        {
            let warning = a.warning();
            let color = if warning { AMBER } else { RED };
            d.draw_rectangle(423, 82, 434, 66, Color::new(8, 15, 22, 225));
            d.draw_rectangle(423, 82, 4, 66, color);
            self.text(
                d,
                if warning {
                    "CAZAS A POPA · CAMBIA DE POSICIÓN"
                } else {
                    "FUEGO ENEMIGO POR POPA"
                },
                442.0,
                91.0,
                20.0,
                color,
            );
            self.text(
                d,
                &if warning {
                    format!("DISPARO EN {:.1} S", (REAR_WARNING - a.age).max(0.0))
                } else {
                    "SALVA EN CAMINO".to_owned()
                },
                442.0,
                121.0,
                15.0,
                MUTED,
            );
            if warning {
                let project = |x, y| unsafe {
                    let p =
                        ffi::GetWorldToScreenEx(v(x, y, 0.0).into(), camera(g).into(), 1280, 800);
                    Vector2::new(p.x, p.y)
                };
                let p = project(
                    a.target[0] + g.hull_half()[0] + REAR_RADIUS,
                    a.target[1] + g.hull_half()[1] + REAR_RADIUS,
                );
                let q = project(
                    a.target[0] - g.hull_half()[0] - REAR_RADIUS,
                    a.target[1] - g.hull_half()[1] - REAR_RADIUS,
                );
                let left = p.x.min(q.x);
                let top = p.y.min(q.y);
                let w = (q.x - p.x).abs();
                let h = (q.y - p.y).abs();
                let alpha = (18.0 + 12.0 * (a.age * 14.0).sin().abs()) as u8;
                d.draw_rectangle_rec(
                    Rectangle::new(left, top, w, h),
                    Color::new(255, 142, 35, alpha),
                );
                for (x, sx) in [(left, 1.0), (left + w, -1.0)] {
                    for (y, sy) in [(top, 1.0), (top + h, -1.0)] {
                        d.draw_line_ex(
                            Vector2::new(x, y),
                            Vector2::new(x + sx * 18.0, y),
                            2.0,
                            color,
                        );
                        d.draw_line_ex(
                            Vector2::new(x, y),
                            Vector2::new(x, y + sy * 18.0),
                            2.0,
                            color,
                        );
                    }
                }
            }
        }
        self.text(d, "ASALTO AL NÚCLEO", 40.0, 27.0, 21.0, WHITE);
        let sector = if g.attacking() {
            "03 / REACTOR PRINCIPAL"
        } else if g.distance > BULKHEADS[2].0 - 100.0 {
            "02 / ACCESO AL REACTOR"
        } else {
            "01 / DUCTO DE SERVICIO"
        };
        self.text(d, sector, 40.0, 55.0, 14.0, MUTED);
        self.text(d, &format!("{:06}", g.score), 1125.0, 28.0, 28.0, WHITE);
        self.text(d, "INTEGRIDAD", 40.0, 709.0, 14.0, MUTED);
        for i in 0..3 {
            d.draw_rectangle(
                40 + i * 42,
                738,
                34,
                7,
                if i < (g.hull as i32) {
                    ICE
                } else {
                    Color::new(58, 51, 51, 255)
                },
            );
        }
        let progress = (g.distance / ATTACK_Z).min(1.0);
        d.draw_rectangle(490, 744, 300, 2, Color::new(45, 64, 74, 255));
        d.draw_rectangle(490, 744, (progress * 300.0) as i32, 2, AMBER);
        self.text(
            d,
            &format!("{:02.0}%  /  {:04.0} M", progress * 100.0, g.distance),
            560.0,
            714.0,
            15.0,
            MUTED,
        );
        self.text(
            d,
            &format!("{:03.0}", g.speed * 12.0),
            1125.0,
            701.0,
            32.0,
            WHITE,
        );
        self.text(d, "KM/H", 1180.0, 742.0, 13.0, MUTED);
        if g.flash > 0.0 {
            d.draw_rectangle(
                0,
                0,
                1280,
                800,
                Color::new(235, 58, 30, (g.flash * 65.0) as u8),
            );
        }
        if matches!(g.phase, Phase::Paused | Phase::Lost) {
            self.flight_menu(d, g, time);
        }
        if g.phase == Phase::Explosion {
            d.draw_rectangle(
                0,
                0,
                1280,
                800,
                Color::new(
                    255,
                    221,
                    164,
                    ((time * 0.0 + g.end_time - 0.65).max(0.0) * 230.0).min(255.0) as u8,
                ),
            );
        }
    }
}
