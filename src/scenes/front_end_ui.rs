//! Secondary-screen presentation, sharing the menu's subdued button style.
use super::{chrome, draw_intro_button, slanted, CYAN, INK, MUTED};
use raylib::prelude::*;

const ORANGE: Color = Color::new(255, 178, 88, 255);

/// Pass the codepoint count directly: raylib 5.5.1's string wrapper counts
/// UTF-8 bytes, which is incorrect for accented Spanish characters.
pub(super) fn load_font(path: &std::path::Path) -> Result<Font, super::Error> {
    let path = std::ffi::CString::new(path.to_string_lossy().as_bytes())?;
    let mut glyphs: Vec<i32> = (32..=255).collect();
    let raw = unsafe {
        raylib::ffi::LoadFontEx(path.as_ptr(), 64, glyphs.as_mut_ptr(), glyphs.len() as i32)
    };
    if raw.texture.id == 0 || raw.glyphs.is_null() {
        return Err("No se pudo cargar la tipografía de interfaz".into());
    }
    unsafe {
        raylib::ffi::SetTextureFilter(
            raw.texture,
            raylib::ffi::TextureFilter::TEXTURE_FILTER_BILINEAR as i32,
        );
        Ok(Font::from_raw(raw))
    }
}

pub(super) fn text(
    d: &mut RaylibDrawHandle,
    font: &Font,
    label: &str,
    x: f32,
    y: f32,
    size: f32,
    color: Color,
) {
    d.draw_text_ex(
        font,
        label,
        Vector2::new(x + 1.0, y + 2.0),
        size,
        1.0,
        Color::new(3, 5, 24, 230),
    );
    d.draw_text_ex(font, label, Vector2::new(x, y), size, 1.0, color);
}

pub(super) fn centered(
    d: &mut RaylibDrawHandle,
    font: &Font,
    label: &str,
    w: i32,
    y: f32,
    size: f32,
    color: Color,
) {
    text(
        d,
        font,
        label,
        (w as f32 - font.measure_text(label, size, 1.0).x) * 0.5,
        y,
        size,
        color,
    );
}

fn heading(d: &mut RaylibDrawHandle, font: &Font, label: &str, x: f32, y: f32, size: f32) {
    for (dx, dy, color) in [
        (3.0, 5.0, Color::new(19, 4, 146, 255)),
        (-2.0, 0.0, CYAN),
        (2.0, 0.0, CYAN),
        (0.0, -2.0, CYAN),
        (0.0, 2.0, CYAN),
    ] {
        d.draw_text_ex(font, label, Vector2::new(x + dx, y + dy), size, 1.0, color);
    }
    d.draw_text_ex(font, label, Vector2::new(x, y), size, 1.0, Color::WHITE);
}

pub(super) fn action(
    d: &mut RaylibDrawHandle,
    font: &Font,
    rect: Rectangle,
    label: &str,
    mouse: Vector2,
    enabled: bool,
) {
    draw_intro_button(d, font, rect, label, mouse, enabled);
}

pub(super) fn arrow(
    d: &mut RaylibDrawHandle,
    rect: Rectangle,
    right: bool,
    mouse: Vector2,
    active: bool,
) {
    let lit = active || rect.check_collision_point_rec(mouse);
    let pixel = if lit { 6.0 } else { 5.0 };
    let cx = rect.x + rect.width * 0.5;
    let cy = rect.y + rect.height * 0.5;
    // A seven-step pixel chevron; the original, generous hit rectangle stays intact.
    for row in -3i32..=3 {
        let step = 3 - row.abs();
        let x = cx + (step as f32 - 1.5) * pixel * if right { 1.0 } else { -1.0 };
        let y = cy + row as f32 * pixel;
        d.draw_rectangle(
            (x + 2.0) as i32,
            (y + 3.0) as i32,
            (pixel * 2.0) as i32,
            pixel as i32,
            Color::new(255, 255, 255, 35),
        );
        d.draw_rectangle(
            x as i32,
            y as i32,
            (pixel * 2.0) as i32,
            pixel as i32,
            if lit {
                Color::WHITE
            } else {
                Color::new(210, 220, 230, 215)
            },
        );
    }
}

/// Local contrast fades out instead of covering the scene with solid HUD bars.
pub(super) fn hangar_shade(d: &mut RaylibDrawHandle, w: i32, h: i32) {
    for y in 0..145 {
        let alpha = (205.0 * (1.0 - y as f32 / 145.0).powi(2)) as u8;
        d.draw_rectangle(0, y, w, 1, Color::new(3, 5, 24, alpha));
    }
    for y in 0..125 {
        let alpha = (195.0 * (1.0 - y as f32 / 125.0).powi(2)) as u8;
        d.draw_rectangle(0, h - 1 - y, w, 1, Color::new(3, 5, 24, alpha));
    }
}

pub(super) fn hangar_title(
    d: &mut RaylibDrawHandle,
    font: &Font,
    name: &str,
    logo: Option<&Texture2D>,
    w: i32,
) {
    let name = name.to_uppercase();
    let size = 48.0_f32.min(48.0 * (w as f32 - 265.0) / font.measure_text(&name, 48.0, 1.0).x.max(1.0));
    let x = (w as f32 - font.measure_text(&name, size, 1.0).x) * 0.5;
    d.draw_text_ex(font, &name, Vector2::new(x + 2.0, 58.0), size, 1.0, Color::BLACK);
    d.draw_text_ex(font, &name, Vector2::new(x, 55.0), size, 1.0, Color::WHITE);
    if let Some(logo) = logo {
        let max_height = if logo.width < logo.height { 80.0 } else { 66.0 };
        let scale = (210.0 / logo.width as f32).min(max_height / logo.height as f32);
        let logo_width = logo.width as f32 * scale;
        let dest = Rectangle::new((w as f32 - logo_width) * 0.5, 115.0, logo_width, logo.height as f32 * scale);
        d.draw_texture_pro(
            logo,
            Rectangle::new(0.0, 0.0, logo.width as f32, logo.height as f32),
            dest,
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}

pub(super) fn controls(d: &mut RaylibDrawHandle, title: &Font, body: &Font, w: i32, h: i32) {
    d.draw_rectangle(0, 0, w, h, Color::new(3, 5, 24, 155));
    let left = (w as f32 * 0.065).round();
    text(
        d,
        body,
        "PLANET-HOPPERS / GUÍA DE VUELO",
        left,
        34.0,
        18.0,
        CYAN,
    );
    heading(
        d,
        title,
        "CONTROLES",
        left,
        68.0,
        (w as f32 * 0.047).clamp(40.0, 62.0),
    );
    slanted(d, left, 145.0, 104.0, 4.0, |t| chrome(t, 0.0));
    text(
        d,
        body,
        "Tu próxima aventura empieza al mando.",
        left,
        164.0,
        22.0,
        INK,
    );
    let gap = w as f32 * 0.065;
    let col_width = (w as f32 - left * 2.0 - gap) * 0.5;
    let row_height = ((h as f32 - 295.0) / 5.0).clamp(62.0, 86.0);
    let groups = [
        (
            "01 / PILOTAJE",
            [
                ("W / S", "Acelerar / frenar"),
                ("MOUSE / A / D / R / F", "Orientar la nave"),
                ("SHIFT", "Turbo"),
                ("ESPACIO", "Subir · ascenso vertical"),
                ("TAB / B", "Elegir objetivo / piloto automático"),
            ],
        ),
        (
            "02 / EXPLORACIÓN Y SESIÓN",
            [
                ("E / ENTER", "Explorar planeta / entrar a estación"),
                ("C / O", "Vista general / órbitas"),
                ("ESC", "Soltar / recapturar mouse en vuelo"),
                (
                    "ESC / P · SOUTH PARK",
                    "Pausa, reinicio y regreso al sistema solar",
                ),
                ("CTRL + Q", "Salir del juego"),
            ],
        ),
    ];
    for (column, (section, rows)) in groups.iter().enumerate() {
        let x = left + column as f32 * (col_width + gap);
        text(
            d,
            title,
            section,
            x,
            219.0,
            (col_width / 24.0).min(18.0),
            CYAN,
        );
        for (i, (key, description)) in rows.iter().enumerate() {
            let y = 260.0 + i as f32 * row_height;
            slanted(d, x, y + 3.0, 3.0, 17.0, |_| ORANGE);
            text(d, title, key, x + 16.0, y, 17.0, Color::WHITE);
            let size = (col_width / body.measure_text(description, 100.0, 1.0).x * 100.0).min(21.0);
            text(d, body, description, x + 16.0, y + 25.0, size, INK);
        }
    }
    text(
        d,
        body,
        "ESC / VOLVER AL MENÚ",
        left,
        h as f32 - 35.0,
        16.0,
        MUTED,
    );
}
