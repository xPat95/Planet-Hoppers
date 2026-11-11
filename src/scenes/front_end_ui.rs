//! Secondary-screen presentation, sharing the menu's subdued button style.
use super::draw_intro_button;
use raylib::prelude::*;


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

/// La pantalla de controles del menú: la misma tarjeta de vidrio que la ayuda del sistema solar (`holo::ControlsCard`,
/// con la misma letra), centrada sobre el fondo de hiperespacio y escalada a la ventana.
pub(super) fn controls(d: &mut RaylibDrawHandle, regular: &Font, bold: &Font, w: i32, h: i32) {
    use crate::scenes::holo;
    d.draw_rectangle(0, 0, w, h, Color::new(3, 5, 24, 110));
    let card = holo::ControlsCard {
        title: "CONTROLES",
        subtitle: "Tu próxima aventura empieza al mando.",
        footer: Some("Esc para volver al menú"),
        columns: [
            ("PILOTAR LA NAVE", &[
                ("W / S", "Acelerar / frenar"), ("Mouse o A / D", "Girar"),
                ("R / F", "Inclinar arriba / abajo"), ("Espacio", "Subir"),
                ("Shift", "Turbo"), ("Tab", "Cambiar objetivo"),
                ("B", "Piloto automático"),
            ]),
            ("EXPLORACIÓN Y SESIÓN", &[
                ("E / Enter", "Explorar / entrar al hangar"), ("C", "Vista general"),
                ("O", "Mostrar / ocultar órbitas"), ("T", "Medallero de tokens"),
                ("Esc", "Soltar el mouse"), ("Esc / P", "Pausa en los minijuegos"),
                ("Ctrl + X", "Salir del juego"),
            ]),
        ],
        width: 820.0,
        column_x: [72.0, 440.0],
        divider_x: 410.0,
    };
    let (cw, ch) = (card.width, card.height());
    let scale = ((w as f32 - 120.0) / cw).min((h as f32 - 150.0) / ch).clamp(0.4, 1.7);
    let at = Vector2::new((w as f32 - cw * scale) * 0.5, (h as f32 - ch * scale) * 0.5 + 18.0);
    holo::panel(d, Rectangle::new(at.x, at.y, cw * scale, ch * scale), holo::CYAN, 1.0);
    card.draw(d, regular, bold, at, scale);
}
