//! La historieta del minijuego de Halloween: la intro (es la pantalla de inicio) y el final feliz. Los cuadros son
//! escenas 3D en vivo, con los mismos modelos, la misma luz y la misma lente que el juego, a pantalla completa (sin
//! marco), uno por vez, que aparecen desde el negro. El texto le habla a quien juega, que es la bruja: grande y rojo,
//! sin globos. Se avanza con un clic (o Espacio) y se puede saltar.
//!
//! Intro: (1) el título, en el espacio, con la bruja en su escoba; (2) el planeta calavera de frente, rojo y
//! encendido: las brujas de la fiesta la invitaron, pero algo no está bien; (3) el planeta desde arriba, a oscuras: la fiesta se apagó; (4) la bruja, parada en la luz
//! de su farol. Final: la bruja baila feliz (ojos cerrados de felicidad) con luces de disco y una bola de espejos.
use super::render::{
    fade, noise, place, render_view, smooth, text_center, text_size, Art, Backdrop, Extra, Item, Light, Shape, Who, BEAT,
    FISH_HALF, FORWARD, UP, WITCH_DANCE, WITCH_H,
};
use raylib::prelude::*;

/// Lo que dice un cuadro: el título, o lo que se le cuenta a quien juega.
#[derive(Clone, Copy)]
enum Line {
    Title(&'static str),
    Tell(&'static str),
}

/// Los cuadros de la intro.
const FRAMES: [Line; 4] = [
    Line::Title("Hexbound"),
    Line::Tell("The witches of the skull planet invited you to their party... but something feels wrong."),
    Line::Tell("The party went silent an hour ago. Something is out there in the dark."),
    Line::Tell("Light your lantern. Draw their glyphs. Curse them all back into the dark."),
];

/// Letras por segundo con que se escribe el texto.
const TYPE_SPEED: f32 = 38.0;
/// Lo que tarda un cuadro nuevo en aparecer desde el negro.
const FADE_IN: f32 = 0.5;
/// Lente de los cuadros: más suave que la del juego.
const FRAME_HALF: f32 = 46.0;
/// La pose de la bruja parada (la de `witch_stand`).
const WITCH_STAND: usize = 8;

/// Por dónde va la intro.
#[derive(Default)]
pub struct Story {
    frame: usize,
    /// Tiempo en el cuadro.
    t: f32,
}

impl Story {
    pub fn update(&mut self, dt: f32) {
        self.t += dt;
    }

    fn typed(&self) -> bool {
        match FRAMES[self.frame] {
            Line::Title(_) => self.t > 1.5,
            Line::Tell(s) => self.t * TYPE_SPEED >= s.len() as f32,
        }
    }

    /// Un clic: termina de escribir el texto, o pasa al cuadro siguiente. `true` cuando se terminó la historieta.
    pub fn click(&mut self) -> bool {
        if !self.typed() {
            self.t = 10.0;
            return false;
        }
        if self.frame + 1 < FRAMES.len() {
            self.frame += 1;
            self.t = 0.0;
            return false;
        }
        true
    }

    /// Para la prueba automática: cuadro y si el texto ya está escrito.
    pub fn progress(&self) -> (usize, bool) {
        (self.frame, self.typed())
    }
}

/// El botón para saltar la historieta, abajo a la derecha.
pub fn skip_button(w: f32, h: f32) -> Rectangle {
    Rectangle::new(w - 150.0, h - 62.0, 124.0, 42.0)
}

/// Dibuja la intro: el cuadro a pantalla completa (apareciendo desde el negro), su texto y el botón de saltar.
pub fn draw_story(d: &mut RaylibDrawHandle, thread: &RaylibThread, art: &mut Art, story: &Story, time: f32) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    d.clear_background(Color::BLACK);
    let screen = Rectangle::new(0.0, 0.0, w, h);
    let shot = shot(art, story, time);
    render_view(d, thread, art, &shot.items, &[], shot.camera, shot.light, FRAME_HALF, screen, time, 0.0,
                Backdrop { stars: shot.stars, halo: false });
    // Aparece desde el negro (el texto, encima, ya se ve)
    let dark = 1.0 - smooth(story.t / FADE_IN);
    if dark > 0.0 {
        d.draw_rectangle(0, 0, w as i32, h as i32, fade(Color::BLACK, dark));
    }
    let red = Color::new(205, 14, 24, 255);
    match FRAMES[story.frame] {
        Line::Title(s) => {
            // Como película rota: al aparecer salta, se parte en copias roja y azul corridas y le faltan cuadros;
            // después se calma, pero cada tanto vuelve a fallar un instante
            let size = h * 0.2;
            let p = Vector2::new(w * 0.07, h * 0.08);
            let frame = (time * 14.0).floor();
            let r = |i: f32| noise(frame * 3.71 + i * 1.618);
            let settling = 1.0 - smooth((story.t - 0.3) / 1.2);
            let glitch = settling.max(if r(9.0) > 0.93 { 0.6 } else { 0.0 });
            let missing = story.t < 1.2 && r(1.0) < 0.35 * settling;
            if story.t > 0.05 && !missing {
                let jump = Vector2::new((r(2.0) - 0.5) * size * 0.3, (r(3.0) - 0.5) * size * 0.12) * glitch;
                let split = Vector2::new(size * 0.06 * glitch * (0.5 + r(4.0)), 0.0);
                let at = p + jump;
                let big = &art.font_big;
                d.draw_text_ex(big, s, at + Vector2::new(0.0, size * 0.04), size, 2.0, Color::new(20, 18, 24, 230));
                if glitch > 0.02 {
                    d.draw_text_ex(big, s, at - split, size, 2.0, fade(Color::new(255, 30, 50, 255), 0.75 * glitch));
                    d.draw_text_ex(big, s, at + split, size, 2.0, fade(Color::new(40, 140, 255, 255), 0.75 * glitch));
                }
                d.draw_text_ex(big, s, at, size, 2.0, Color::new(245, 242, 236, 255));
                // Una franja corrida, como una línea de la cinta que se salió de lugar
                if glitch > 0.3 && r(5.0) > 0.4 {
                    let m = text_size(big, s, size);
                    let y = at.y + m.y * (0.2 + 0.6 * r(6.0));
                    let band = m.y * 0.12;
                    d.draw_rectangle_v(Vector2::new(at.x - 10.0, y), Vector2::new(m.x + 20.0, band), Color::BLACK);
                    d.draw_rectangle_v(Vector2::new(at.x + (r(7.0) - 0.5) * size * 0.4, y), Vector2::new(m.x, band * 0.5),
                                       fade(Color::new(245, 242, 236, 255), 0.8));
                }
            }
        }
        Line::Tell(s) => {
            // Abajo, centrado, en una o dos líneas, escribiéndose de a poco
            let size = (h * 0.06).max(30.0);
            let shown = (story.t * TYPE_SPEED) as usize;
            for (k, (part, line)) in wrap(&art.font, shown, s, size, w * 0.8).iter().enumerate() {
                let m = text_size(&art.font, line, size);
                let p = Vector2::new(w / 2.0 - m.x / 2.0, h * 0.78 + k as f32 * size * 1.1);
                // Con borde oscuro, así se lee sobre lo que haya atrás
                for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0), (-2.0, -2.0), (2.0, 2.0), (-2.0, 2.0), (2.0, -2.0), (2.0, 4.0)] {
                    d.draw_text_ex(&art.font, part, p + Vector2::new(dx, dy), size, 1.0, Color::new(10, 0, 2, 255));
                }
                d.draw_text_ex(&art.font, part, p, size, 1.0, red);
            }
        }
    }
    // Saltar, y abajo cómo seguir
    let skip = skip_button(w, h);
    let on = skip.check_collision_point_rec(d.get_mouse_position());
    d.draw_rectangle_rounded(skip, 0.4, 8, if on { Color::new(235, 230, 220, 255) } else { Color::new(30, 26, 34, 200) });
    text_center(d, &art.font, "Skip >>", Vector2::new(skip.x + skip.width / 2.0, skip.y + skip.height / 2.0), 28.0,
                if on { Color::new(12, 10, 16, 255) } else { Color::new(230, 225, 235, 255) });
    if story.typed() {
        let a = 0.45 + 0.35 * (time * 3.0).sin();
        text_center(d, &art.font, "Click to continue", Vector2::new(w / 2.0, h - 36.0), 24.0, fade(Color::new(210, 205, 215, 255), a));
    }
}

/// Parte el texto en líneas que entren en `width` (según el texto completo, así las palabras no saltan de línea
/// mientras se escribe): para cada línea, lo ya escrito (de las primeras `shown` letras) y la línea completa (para
/// centrarla).
fn wrap(font: &Font, shown: usize, full: &str, size: f32, width: f32) -> Vec<(String, String)> {
    let mut lines: Vec<String> = vec![String::new()];
    for word in full.split(' ') {
        let last = lines.last_mut().unwrap();
        let next = if last.is_empty() { word.to_string() } else { format!("{last} {word}") };
        if !last.is_empty() && text_size(font, &next, size).x > width {
            lines.push(word.to_string());
        } else {
            *last = next;
        }
    }
    let mut left = shown;
    lines.into_iter().map(|line| {
        let n = line.chars().count();
        let part = line.chars().take(left).collect();
        left = left.saturating_sub(n + 1);                 // Más el espacio que quedó entre las dos líneas
        (part, line)
    }).collect()
}

/// Lo que se ve en un cuadro: modelos, cámara, luz y si hay estrellas.
struct Shot {
    items: Vec<Item>,
    camera: Camera3D,
    light: Light,
    stars: bool,
}

fn witch_scale(art: &Art) -> (f32, f32) {
    (WITCH_H / art.witch.height, art.witch.base)
}

fn shot(art: &Art, story: &Story, time: f32) -> Shot {
    let fov = FRAME_HALF * 2.0;
    let push = smooth(story.t / 9.0);
    match story.frame {
        // El título: en el espacio, la bruja vuela en su escoba hacia el planeta calavera, que está lejos
        0 => {
            let drift = (time * 1.3).sin() * 0.02;
            let m = Matrix::rotate_y(-std::f32::consts::FRAC_PI_2 * 0.85) * Matrix::translate(0.0, drift, 0.0);
            let planet = Matrix::scale(1.5, 1.5, 1.5) * Matrix::rotate_y(time * 0.05) * Matrix::translate(-11.5, -5.5, -14.0);
            Shot {
                items: vec![Item::plain(Who::Broom, Shape::Still, m), Item::plain(Who::Planet, Shape::Still, planet)],
                camera: Camera3D::perspective(Vector3::new(0.3, 0.2, 2.0 - push * 0.25), Vector3::new(-0.35, 0.05, 0.0), UP, fov),
                light: Light { lamp: Vector3::new(0.4, 1.2, 1.4), power: 3.2, tint: Vector3::zero(), range: 3.0, fog: 0.0 },
                stars: true,
            }
        }
        // El planeta calavera de frente, rojo y encendido, con la fiesta brillando en las cuencas
        1 => {
            // De frente: la cara de la calavera mira a la cámara (como viene en el modelo), apenas meciéndose
            let face = Matrix::rotate_y((time * 0.3).sin() * 0.04);
            let mut planet = Item::plain(Who::Planet, Shape::Still, face);
            planet.boost = 1.0;
            Shot {
                items: vec![planet],
                camera: Camera3D::perspective(Vector3::new(0.0, -0.1, 2.75 - push * 0.2), Vector3::new(0.0, -0.48, 0.0), UP, fov),
                light: Light { lamp: Vector3::new(-0.9, 1.4, 2.0), power: 8.0, tint: Vector3::new(1.0, 0.85, 0.8), range: 2.5, fog: 0.0 },
                stars: true,
            }
        }
        // El planeta desde arriba, a oscuras: la fiesta no se oye ni se ve
        2 => Shot {
            items: vec![Item::plain(Who::Planet, Shape::Still, Matrix::rotate_y(0.6 + time * 0.02))],
            camera: Camera3D::perspective(Vector3::new(0.0, 2.7 - push * 0.35, 2.0), Vector3::zero(), UP, fov),
            // Casi sin luz: el planeta es una sombra con el borde frío
            light: Light { lamp: Vector3::new(0.0, 3.0, 2.2), power: 0.12, tint: Vector3::zero(), range: 3.0, fog: 0.0 },
            stars: true,
        },
        // La bruja, parada en el centro, en la luz de su farol que parpadea
        _ => {
            let (scale, base) = witch_scale(art);
            let m = place(Vector3::zero(), FORWARD, scale, base);
            let flicker = 0.85 + 0.15 * (time * 9.0).sin() * (time * 3.1).sin();
            Shot {
                items: vec![Item::plain(Who::Witch, Shape::Blend(WITCH_STAND, WITCH_STAND, 0.0), m)],
                // De la cintura para arriba
                camera: Camera3D::perspective(Vector3::new(0.0, 1.12, -0.78 + push * 0.05), UP * 1.08, UP, fov),
                light: Light { lamp: Vector3::new(0.0, 1.95, -0.45), power: 1.8 * flicker, tint: Vector3::zero(), range: 1.6, fog: 0.0 },
                stars: false,
            }
        }
    }
}

/// Cuánto dura el baile del final antes de que aparezca la pantalla de victoria (sigue bailando atrás).
pub const ENDING: f32 = 5.5;

/// El final feliz: la bruja baila con los ojos cerrados de felicidad, la luz cambia de color con cada paso y da
/// vueltas, hay focos de colores barriendo y una bola de espejos tira puntitos de luz. La cámara se aleja despacio.
pub fn draw_ending(d: &mut RaylibDrawHandle, thread: &RaylibThread, art: &mut Art, t: f32) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    d.clear_background(Color::BLACK);
    let (scale, base) = witch_scale(art);
    let beat = t / BEAT;
    let i = beat.floor() as usize;
    let dance = Shape::Blend(WITCH_DANCE[i % 6], WITCH_DANCE[(i + 1) % 6], smooth(beat.fract() * 1.6 - 0.3));
    let items = vec![Item::plain(Who::HappyWitch, dance, place(Vector3::zero(), FORWARD, scale, base))];
    let back = smooth(t / ENDING);
    let camera = Camera3D::perspective(FORWARD * (1.2 + 0.6 * back) + UP * (0.95 + 0.15 * back), UP * 0.8, UP, FISH_HALF * 2.0);
    const DISCO: [(f32, f32, f32); 4] = [(1.0, 0.2, 0.7), (0.2, 0.9, 1.0), (0.6, 0.3, 1.0), (1.0, 0.85, 0.2)];
    let (r, g, b) = DISCO[i % 4];
    let spin = t * 2.2;
    // La luz de color da vueltas cerca de ella, adelante, y le da de lleno
    let lamp = Vector3::new(spin.cos() * 0.8, 1.7, -0.7 + spin.sin() * 0.5);
    let ball = UP * 1.95 + FORWARD * 0.1;
    let extras = [Extra::Glow(ball, 0.11, Color::new(205, 210, 225, 255))];
    render_view(d, thread, art, &items, &extras, camera, Light { lamp, power: 30.0, tint: Vector3::new(r, g, b), range: 2.0, fog: 0.0 },
                FISH_HALF, Rectangle::new(0.0, 0.0, w, h), t, 0.0, Backdrop { stars: false, halo: false });
    // Focos de colores que barren desde arriba
    {
        let mut add = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
        for k in 0..4 {
            let (r, g, b) = DISCO[k];
            let color = Color::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 38);
            let from = Vector2::new(w * (0.15 + 0.23 * k as f32), -20.0);
            let a = (t * (0.9 + 0.25 * k as f32) + k as f32 * 1.7).sin() * 0.55;
            let dir = Vector2::new(a.sin(), a.cos());
            let side = Vector2::new(dir.y, -dir.x) * (w * 0.09);
            let to = from + dir * (h * 1.25);
            add.draw_triangle(from, to + side, to - side, color);
            add.draw_triangle(from, to - side, to + side, color);
        }
        // Puntitos de la bola de espejos, girando por toda la pantalla
        for k in 0..90 {
            let f = k as f32;
            let a = t * 0.5 + noise(f * 1.3) * std::f32::consts::TAU;
            let rr = (0.15 + 0.85 * noise(f * 2.9 + 1.0)) * w * 0.6;
            let p = Vector2::new(w / 2.0 + a.cos() * rr, h * 0.35 + a.sin() * rr * 0.55);
            let tw = 0.4 + 0.6 * (t * (3.0 + noise(f) * 5.0) + f).sin().max(0.0);
            let (r, g, b) = DISCO[k % 4];
            let c = Color::new((150.0 + r * 105.0) as u8, (150.0 + g * 105.0) as u8, (150.0 + b * 105.0) as u8, (200.0 * tw) as u8);
            add.draw_circle_v(p, 2.5 + 2.0 * noise(f * 4.1), c);
        }
    }
}
