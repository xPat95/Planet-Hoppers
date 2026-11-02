//! Procedural pixel cracks in world space and a screen-space iron pickaxe. No PNG changes.
use crate::minigames::minecraft::mining::MiningSession;
use raylib::prelude::*;
use std::collections::BTreeSet;

pub struct MiningVisuals {
    pixels: Vec<(i32, i32)>,
}
impl Default for MiningVisuals {
    fn default() -> Self {
        let paths: &[&[(i32, i32)]] = &[
            &[
                (8, 0),
                (8, 2),
                (7, 3),
                (8, 5),
                (7, 7),
                (8, 8),
                (7, 10),
                (8, 12),
                (7, 14),
                (8, 15),
            ],
            &[(7, 7), (5, 6), (4, 7), (2, 6), (0, 6)],
            &[(8, 5), (10, 4), (11, 5), (13, 3), (15, 3)],
            &[(8, 12), (10, 11), (11, 12), (13, 10), (15, 11)],
            &[(7, 10), (5, 11), (5, 13), (3, 14), (2, 15)],
            &[(5, 6), (4, 4), (5, 2), (3, 1)],
            &[(11, 5), (12, 7), (14, 8), (15, 7)],
            &[(5, 11), (3, 10), (1, 11), (0, 10)],
        ];
        let mut used = BTreeSet::new();
        let mut pixels = vec![];
        for path in paths {
            for pair in path.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                let n = (b.0 - a.0).abs().max((b.1 - a.1).abs());
                for t in 0..=n {
                    let p = (
                        a.0 + ((b.0 - a.0) as f32 * t as f32 / n as f32).round() as i32,
                        a.1 + ((b.1 - a.1) as f32 * t as f32 / n as f32).round() as i32,
                    );
                    if used.insert(p) {
                        pixels.push(p);
                    }
                }
            }
        }
        Self { pixels }
    }
}
impl MiningVisuals {
    pub fn draw_cracks(&self, d: &mut RaylibMode3D<RaylibDrawHandle>, session: &MiningSession) {
        let Some(active) = &session.active else {
            return;
        };
        let bounds = session.blocks[active.block].bounds;
        let count = (self.pixels.len() * session.crack_stage()).div_ceil(8);
        let e = 0.003;
        for &(u, v) in self.pixels.iter().take(count) {
            let u = u as f32 / 16.0;
            let v = v as f32 / 16.0;
            let s = 1.0 / 16.0;
            for face in 0..6 {
                let point = |a: f32, b: f32| -> Vector3 {
                    let lo = bounds.min;
                    let hi = bounds.max;
                    match face {
                        0 => Vector3::new(lo.x - e, lo.y + b, lo.z + a),
                        1 => Vector3::new(hi.x + e, lo.y + b, lo.z + a),
                        2 => Vector3::new(lo.x + a, lo.y - e, lo.z + b),
                        3 => Vector3::new(lo.x + a, hi.y + e, lo.z + b),
                        4 => Vector3::new(lo.x + a, lo.y + b, lo.z - e),
                        _ => Vector3::new(lo.x + a, lo.y + b, hi.z + e),
                    }
                };
                let (a, b, c, f) = (
                    point(u, v),
                    point(u + s, v),
                    point(u + s, v + s),
                    point(u, v + s),
                );
                for (x, y, z) in [(a, b, c), (a, c, f), (c, b, a), (f, c, a)] {
                    d.draw_triangle3D(x, y, z, Color::new(19, 15, 19, 255));
                }
            }
        }
    }
    pub fn draw_pickaxe(&self, d: &mut RaylibDrawHandle, w: i32, h: i32, session: &MiningSession) {
        // Dark outline, iron highlights and diagonal oak handle, drawn as rotated square pixels.
        const ART: [&str; 16] = [
            "....kkkkkkkk....",
            "...ksssssssskk..",
            "..kssIIIIIIsssk.",
            "...kkkkkkk..kssk",
            ".......kwk...ksk",
            "......kwwk...ksk",
            ".....kwwk.....kk",
            "....kwwk........",
            "...kwwk.........",
            "..kwwk..........",
            ".kwwk...........",
            "kwwk............",
            "kwk.............",
            "kk..............",
            "................",
            "................",
        ];
        let pixel = (w as f32 / 160.0).clamp(5.0, 9.0);
        let swing = session
            .swing_phase()
            .map_or(0.0, |p| (p * std::f32::consts::PI).sin());
        let kick = session.finish_kick / 0.12;
        let angle = -8.0 - 37.0 * swing - 8.0 * kick;
        let rad = angle.to_radians();
        let anchor = Vector2::new(
            w as f32 - 18.0 * pixel - 14.0 * swing,
            h as f32 - 7.0 * pixel - 8.0 * swing,
        );
        for (y, row) in ART.iter().enumerate() {
            for (x, c) in row.bytes().enumerate() {
                let color = match c {
                    b'k' => Color::new(38, 29, 26, 255),
                    b's' => Color::new(137, 153, 158, 255),
                    b'I' => Color::new(224, 235, 234, 255),
                    b'w' => Color::new(139, 94, 45, 255),
                    _ => continue,
                };
                let p = Vector2::new((x as f32 - 3.0) * pixel, (y as f32 - 13.0) * pixel);
                let pos = anchor
                    + Vector2::new(
                        p.x * rad.cos() - p.y * rad.sin(),
                        p.x * rad.sin() + p.y * rad.cos(),
                    );
                d.draw_rectangle_pro(
                    Rectangle::new(pos.x, pos.y, pixel + 0.1, pixel + 0.1),
                    Vector2::zero(),
                    angle,
                    color,
                );
            }
        }
    }
    pub fn draw_hud(&self, d: &mut RaylibDrawHandle, w: i32, h: i32, s: &MiningSession) {
        let text = format!("Diamantes: {} / {}", s.diamonds, s.config.target_diamonds);
        let width = d.measure_text(&text, 23);
        let x = w - width - 26;
        let color = if s
            .feedback
            .as_ref()
            .is_some_and(|(text, _)| text.starts_with("+1"))
        {
            Color::new(121, 255, 229, 255)
        } else {
            Color::WHITE
        };
        d.draw_rectangle(x - 12, 77, width + 24, 41, Color::new(8, 13, 18, 235));
        d.draw_text(&text, x, 87, 23, color);
        if let Some(index) = s.hovered {
            let label = s.blocks[index].kind.label();
            let width = d.measure_text(label, 22);
            d.draw_rectangle(
                (w - width) / 2 - 10,
                h / 2 + 22,
                width + 20,
                31,
                Color::new(8, 13, 18, 235),
            );
            d.draw_text(label, (w - width) / 2, h / 2 + 28, 22, Color::YELLOW);
            let help = "Mantener clic izquierdo para picar";
            let x = (w - d.measure_text(help, 18)) / 2;
            d.draw_text(help, x, h / 2 + 61, 18, Color::WHITE);
        }
        if s.active.is_some() {
            d.draw_rectangle(w / 2 - 60, h / 2 + 88, 120, 6, Color::new(28, 32, 37, 255));
            d.draw_rectangle(
                w / 2 - 60,
                h / 2 + 88,
                (120.0 * s.progress()) as i32,
                6,
                Color::new(103, 220, 211, 255),
            );
        }
        if let Some((text, _)) = &s.feedback {
            d.draw_text(text, 20, 82, 22, Color::new(119, 243, 224, 255));
        }
    }
}
