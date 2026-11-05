use raylib::prelude::*;

const TRACKING: f32 = 0.6;
const ROUNDNESS: f32 = 0.4;

pub(super) fn surface(d: &mut impl RaylibDraw, rect: Rectangle, hovered: bool, enabled: bool) {
    d.draw_rectangle_rounded(
        Rectangle::new(rect.x - 1.0, rect.y + 2.0, rect.width + 2.0, rect.height),
        ROUNDNESS,
        12,
        Color::new(255, 255, 255, if enabled { 100 } else { 42 }),
    );
    d.draw_rectangle_rounded(
        Rectangle::new(rect.x + 1.0, rect.y + 5.0, rect.width, rect.height),
        ROUNDNESS,
        12,
        Color::new(5, 18, 46, if enabled { 92 } else { 42 }),
    );
    d.draw_rectangle_rounded(
        rect,
        ROUNDNESS,
        12,
        if !enabled {
            Color::new(205, 213, 226, 255)
        } else if hovered {
            Color::WHITE
        } else {
            Color::new(242, 247, 255, 255)
        },
    );
    d.draw_rectangle_rounded(
        Rectangle::new(
            rect.x + 3.0,
            rect.y + 2.0,
            (rect.width - 6.0).max(1.0),
            (rect.height * 0.18).max(5.0),
        ),
        0.4,
        8,
        Color::new(255, 255, 255, if hovered { 255 } else { 210 }),
    );
    d.draw_rectangle_rounded_lines(
        rect,
        ROUNDNESS,
        12,
        Color::new(255, 255, 255, if hovered { 255 } else { 190 }),
    );
}

fn text_width(d: &RaylibDrawHandle, text: &str, size: i32) -> f32 {
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .map(|ch| d.measure_text(&ch.to_string(), size) as f32)
        .sum::<f32>()
        + TRACKING * chars.len().saturating_sub(1) as f32
}

pub(super) fn centered_label(
    d: &mut RaylibDrawHandle,
    rect: Rectangle,
    label: &str,
    size: i32,
    color: Color,
) {
    let width = text_width(d, label, size);
    let mut x = rect.x + (rect.width - width) * 0.5;
    let y = rect.y + (rect.height - size as f32) * 0.5;
    let chars: Vec<char> = label.chars().collect();
    for (index, ch) in chars.iter().enumerate() {
        let glyph = ch.to_string();
        d.draw_text(&glyph, x as i32, y as i32, size, color);
        x += d.measure_text(&glyph, size) as f32;
        if index + 1 < chars.len() {
            x += TRACKING;
        }
    }
}

pub(super) fn draw(
    d: &mut RaylibDrawHandle,
    rect: Rectangle,
    label: &str,
    size: i32,
    mouse: Vector2,
    enabled: bool,
) {
    let hovered = enabled && rect.check_collision_point_rec(mouse);
    surface(d, rect, hovered, enabled);
    centered_label(
        d,
        rect,
        label,
        size,
        if enabled {
            Color::new(8, 24, 55, 255)
        } else {
            Color::new(72, 86, 108, 255)
        },
    );
}
