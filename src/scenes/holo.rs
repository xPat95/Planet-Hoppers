//! Estilo holográfico de cabina, compartido por el medallero y las dos pantallas de controles (la del menú y la del
//! sistema solar): paneles de vidrio con degradé y líneas de barrido, borde cian fino con esquinas encendidas, una
//! franja de luz que baja despacio, títulos que brillan, teclas en chips y divisores que se desvanecen en las puntas.
use raylib::prelude::*;

pub const CYAN: Color = Color::new(137, 228, 255, 255);
/// El acento que acompaña al cian (en vez del amarillo y el naranja, que Cami no quiere): blanco hielo.
pub const ACCENT: Color = Color::new(235, 245, 255, 255);
pub const INK: Color = Color::new(223, 238, 255, 255);
pub const DEEP: Color = Color::new(4, 14, 32, 255);

pub fn fade(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a as f32 * opacity.clamp(0.0, 1.0)) as u8;
    color
}

/// Los segundos desde que arrancó la ventana (para lo que se mueve solo: el barrido, los brillos).
pub fn now() -> f32 {
    unsafe { raylib::ffi::GetTime() as f32 }
}

/// Un panel de vidrio: fondo azul profundo translúcido con degradé, líneas de barrido finitas, una franja de luz que
/// baja despacio, borde cian fino y esquinas encendidas con resplandor. `opacity` lo desvanece entero.
pub fn panel(d: &mut impl RaylibDraw, rect: Rectangle, accent: Color, opacity: f32) {
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    d.draw_rectangle_gradient_v(x as i32, y as i32, w as i32, h as i32,
        fade(Color::new(12, 34, 66, 235), opacity), fade(Color::new(3, 10, 26, 240), opacity));
    // Brillo arriba, como el reflejo de un vidrio
    d.draw_rectangle_gradient_v(x as i32, y as i32, w as i32, (h * 0.18) as i32,
        fade(Color::new(150, 220, 255, 26), opacity), fade(Color::new(150, 220, 255, 0), opacity));
    // Líneas de barrido
    let mut ly = y + 2.0;
    while ly < y + h {
        d.draw_rectangle(x as i32, ly as i32, w as i32, 1, fade(Color::new(140, 220, 255, 9), opacity));
        ly += 4.0;
    }
    // La franja de luz que baja y vuelve a empezar
    let band = (now() * 0.18).fract();
    let by = y + band * (h + 120.0) - 60.0;
    let (top, bottom) = ((by - 50.0).max(y), (by + 50.0).min(y + h));
    if bottom > top {
        let mid = by.clamp(top, bottom);
        d.draw_rectangle_gradient_v(x as i32, top as i32, w as i32, (mid - top) as i32,
            fade(accent, 0.0), fade(accent, 0.07 * opacity));
        d.draw_rectangle_gradient_v(x as i32, mid as i32, w as i32, (bottom - mid) as i32,
            fade(accent, 0.07 * opacity), fade(accent, 0.0));
    }
    d.draw_rectangle_lines_ex(rect, 1.0, fade(accent, 0.45 * opacity));
    corners(d, rect, accent, opacity);
}

/// Las cuatro esquinas encendidas de un panel (en L, con un resplandor alrededor).
pub fn corners(d: &mut impl RaylibDraw, rect: Rectangle, accent: Color, opacity: f32) {
    let arm = (rect.width.min(rect.height) * 0.08).clamp(14.0, 34.0);
    let pulse = 0.85 + 0.15 * (now() * 2.0).sin();
    for (cx, cy, sx, sy) in [(rect.x, rect.y, 1.0, 1.0), (rect.x + rect.width, rect.y, -1.0, 1.0),
                             (rect.x, rect.y + rect.height, 1.0, -1.0), (rect.x + rect.width, rect.y + rect.height, -1.0, -1.0)] {
        let c = Vector2::new(cx, cy);
        for (thick, a) in [(9.0, 0.08), (5.0, 0.16), (2.5, 1.0)] {
            let color = fade(accent, a * opacity * pulse);
            d.draw_line_ex(c, c + Vector2::new(arm * sx, 0.0), thick, color);
            d.draw_line_ex(c, c + Vector2::new(0.0, arm * sy), thick, color);
        }
    }
}

/// Texto con resplandor (copias corridas y transparentes alrededor, y el texto encima).
pub fn glow_text(d: &mut impl RaylibDraw, font: &Font, label: &str, at: Vector2, size: f32, color: Color, glow: Color) {
    let r = (size * 0.06).max(1.5);
    for (dx, dy) in [(-r, 0.0), (r, 0.0), (0.0, -r), (0.0, r), (-r, -r), (r, r), (-r, r), (r, -r)] {
        d.draw_text_ex(font, label, at + Vector2::new(dx, dy), size, 1.0, fade(glow, 0.16 * color.a as f32 / 255.0));
    }
    d.draw_text_ex(font, label, at, size, 1.0, color);
}

/// Una línea de luz que se desvanece en las dos puntas.
pub fn divider(d: &mut impl RaylibDraw, x0: f32, x1: f32, y: f32, color: Color, opacity: f32) {
    let mid = (x0 + x1) * 0.5;
    d.draw_rectangle_gradient_h(x0 as i32, y as i32, (mid - x0) as i32, 1, fade(color, 0.0), fade(color, 0.85 * opacity));
    d.draw_rectangle_gradient_h(mid as i32, y as i32, (x1 - mid) as i32, 1, fade(color, 0.85 * opacity), fade(color, 0.0));
}

/// Una tecla en un chip: contorno cian redondeado, fondo apenas encendido y el texto adentro. Devuelve el ancho.
pub fn key_chip(d: &mut impl RaylibDraw, font: &Font, label: &str, at: Vector2, size: f32, opacity: f32) -> f32 {
    let text = font.measure_text(label, size, 1.0);
    let (pad_x, pad_y) = (size * 0.45, size * 0.18);
    let rect = Rectangle::new(at.x, at.y - pad_y, text.x + pad_x * 2.0, text.y + pad_y * 2.0);
    d.draw_rectangle_rounded(rect, 0.35, 6, fade(Color::new(137, 228, 255, 24), opacity));
    d.draw_rectangle_rounded_lines(rect, 0.35, 6, fade(CYAN, 0.65 * opacity));
    d.draw_text_ex(font, label, Vector2::new(at.x + pad_x, at.y), size, 1.0, fade(Color::WHITE, opacity));
    rect.width
}

/// Una tarjeta de controles (la ayuda del sistema solar y la pantalla de controles del menú usan la misma): título
/// que brilla, subtítulo, divisor, dos columnas (encabezado del acento y filas de tecla en chip + acción) separadas por
/// una línea vertical, y un pie. Las medidas son de diseño (`width` de ancho; las filas, cada 32) y se escalan con
/// `scale` desde la esquina `at` del panel.
pub struct ControlsCard<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub footer: Option<&'a str>,
    pub columns: [(&'a str, &'a [(&'a str, &'a str)]); 2],
    pub width: f32,
    /// Dónde empieza cada columna y dónde va la línea entre las dos.
    pub column_x: [f32; 2],
    pub divider_x: f32,
}

impl ControlsCard<'_> {
    /// El alto de diseño de la tarjeta: lo que ocupan las filas de la columna más larga, el pie y los márgenes.
    pub fn height(&self) -> f32 {
        Self::FIRST_ROW + self.rows() as f32 * Self::ROW + 61.0
    }

    fn rows(&self) -> usize {
        self.columns[0].1.len().max(self.columns[1].1.len())
    }

    const FIRST_ROW: f32 = 199.0;
    const ROW: f32 = 32.0;
    /// Cuánto se corre la acción desde el principio de su columna (a la derecha del chip de la tecla).
    const ACTION: f32 = 132.0;

    pub fn draw(&self, d: &mut impl RaylibDraw, regular: &Font, bold: &Font, at: Vector2, scale: f32) {
        let tx = |v: f32| (at.x + v * scale) as i32;
        let ty = |v: f32| (at.y + v * scale) as i32;
        let fs = |v: f32| v * scale;
        let blue = Color::new(66, 173, 223, 255);
        glow_text(d, bold, self.title, Vector2::new(tx(72.0) as f32, ty(50.0) as f32), fs(34.0), Color::WHITE, CYAN);
        text(d, regular, self.subtitle, tx(72.0), ty(94.0), fs(19.0), INK);
        divider(d, at.x + 60.0 * scale, at.x + (self.width - 60.0) * scale, at.y + 130.0 * scale, blue, 1.0);
        let last = Self::FIRST_ROW + self.rows() as f32 * Self::ROW - 5.0;
        d.draw_rectangle_gradient_v(tx(self.divider_x), ty(150.0), 1, ((last - 150.0) * scale) as i32,
            Color::new(57, 108, 150, 0), Color::new(57, 108, 150, 220));
        for (col, (heading, rows)) in self.column_x.iter().zip(self.columns.iter()) {
            glow_text(d, bold, heading, Vector2::new(tx(*col) as f32, ty(151.0) as f32), fs(23.0), ACCENT, ACCENT);
            for (i, (key, action)) in rows.iter().enumerate() {
                let yy = ty(Self::FIRST_ROW + i as f32 * Self::ROW);
                key_chip(d, bold, key, Vector2::new(tx(*col) as f32, yy as f32 + fs(1.5)), fs(16.0), 1.0);
                text(d, regular, action, tx(col + Self::ACTION), yy, fs(18.0), INK);
            }
        }
        if let Some(footer) = self.footer {
            let size = fs(18.0);
            let w = regular.measure_text(footer, size, 1.0).x;
            text(d, regular, footer, (at.x + self.width * scale * 0.5 - w * 0.5) as i32,
                 ty(Self::FIRST_ROW + self.rows() as f32 * Self::ROW + 15.0), size, INK);
        }
    }
}

/// Texto en coordenadas enteras (nítido).
fn text(d: &mut impl RaylibDraw, font: &Font, s: &str, x: i32, y: i32, size: f32, color: Color) {
    d.draw_text_ex(font, s, Vector2::new(x as f32, y as f32), size, 1.0, color);
}

/// Un botón del estilo holográfico: panel de vidrio (más encendido con el mouse encima), la tecla en un chip y el
/// texto que brilla al lado, todo centrado. Con `subtitle`, el título va arriba y la tecla con el subtítulo abajo.
#[allow(clippy::too_many_arguments)]
pub fn button(d: &mut impl RaylibDraw, bold: &Font, regular: &Font, rect: Rectangle, key: &str, label: &str,
              subtitle: Option<&str>, hovered: bool) {
    panel(d, rect, if hovered { ACCENT } else { CYAN }, 1.0);
    if hovered {
        d.draw_rectangle_rec(rect, fade(CYAN, 0.08));
    }
    let center = rect.x + rect.width * 0.5;
    let line = |d: &mut _, font: &Font, text: &str, size: f32, y: f32, color: Color| {
        let kw = bold.measure_text(key, size * 0.8, 1.0).x + size * 0.8 * 0.9 + 10.0;
        let w = kw + font.measure_text(text, size, 1.0).x;
        let x = center - w * 0.5;
        key_chip(d, bold, key, Vector2::new(x, y + size * 0.1), size * 0.8, 1.0);
        glow_text(d, font, text, Vector2::new(x + kw, y), size, color, CYAN);
    };
    match subtitle {
        None => line(d, bold, label, 24.0, rect.y + rect.height * 0.5 - 13.0, Color::WHITE),
        Some(sub) => {
            let size = 28.0;
            let w = bold.measure_text(label, size, 1.0).x;
            glow_text(d, bold, label, Vector2::new(center - w * 0.5, rect.y + 10.0), size, Color::WHITE, CYAN);
            line(d, regular, sub, 19.0, rect.y + 46.0, INK);
        }
    }
}
