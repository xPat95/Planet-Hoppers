//! La historieta del minijuego de Halloween: la intro (es la pantalla de inicio) y el final feliz. Los cuadros son
//! escenas 3D en vivo, con los mismos modelos, la misma luz y la misma lente que el juego, a pantalla completa (sin
//! marco), uno por vez, que aparecen desde el negro. El texto le habla a quien juega, que es la bruja: grande y rojo,
//! sin globos. Se avanza con un clic (o Espacio) y se puede saltar.
//!
//! Intro: (1) el título, en el espacio, con la bruja en su escoba; (2) el planeta calavera de frente, rojo y
//! encendido: las brujas de la fiesta la invitaron, pero algo no está bien; (3) el planeta desde arriba, a oscuras: la fiesta se apagó; (4) la bruja, parada en la luz
//! de su farol. Final: la bruja baila feliz (ojos cerrados de felicidad) con luces de disco y una bola de espejos.
use super::render::{
    arm_bend, fade, noise, place, render_view, smooth, text_center, text_size, Art, Backdrop, Extra, Item, Light, Shape, Who, BEAT,
    FISH_HALF, FORWARD, UP, WITCH_DANCE, WITCH_H,
};
use raylib::prelude::*;

/// Lo que dice un cuadro: el título (el logo), o lo que se le cuenta a quien juega.
#[derive(Clone, Copy)]
enum Line {
    Title,
    Tell(&'static str),
}

/// Los cuadros de la intro.
const FRAMES: [Line; 4] = [
    Line::Title,
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
            Line::Title => self.t > FADE_IN,
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
    // En el título, atrás de todo, las estrellas y el logo enorme; la bruja y el planeta van encima
    let title = matches!(FRAMES[story.frame], Line::Title);
    if title {
        draw_stars(d, w, h, time);
        draw_title(d, art, w, h);
    }
    render_view(d, thread, art, &shot.items, &[], shot.camera, shot.light, FRAME_HALF, screen, time, 0.0,
                Backdrop { stars: shot.stars, halo: false, see_through: title });
    // Aparece desde el negro (el texto, encima, ya se ve)
    let dark = 1.0 - smooth(story.t / FADE_IN);
    if dark > 0.0 {
        d.draw_rectangle(0, 0, w as i32, h as i32, fade(Color::BLACK, dark));
    }
    let red = Color::new(205, 14, 24, 255);
    match FRAMES[story.frame] {
        Line::Title => {}
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

/// Estrellas finitas en la pantalla, quietas y algunas titilando (atrás del logo del título).
fn draw_stars(d: &mut RaylibDrawHandle, w: f32, h: f32, t: f32) {
    for i in 0..260 {
        let k = i as f32;
        let p = Vector2::new(noise(k * 1.31 + 0.2) * w, noise(k * 2.17 + 5.1) * h);
        let a = 0.25 + 0.6 * noise(k * 0.71 + 9.0) * (0.7 + 0.3 * (t * (1.0 + noise(k) * 3.0) + k).sin());
        d.draw_circle_v(p, 0.8 + 1.4 * noise(k * 3.3), fade(Color::new(220, 225, 255, 255), a));
    }
}

/// El logo de Hexbound, enorme y centrado (aparece desde el negro con el resto del cuadro).
fn draw_title(d: &mut RaylibDrawHandle, art: &Art, w: f32, h: f32) {
    let logo = &art.title;
    let lh = h * 0.9;
    let lw = lh * logo.width as f32 / logo.height as f32;
    d.draw_texture_pro(logo, Rectangle::new(0.0, 0.0, logo.width as f32, logo.height as f32),
                       Rectangle::new((w - lw) / 2.0, (h - lh) / 2.0, lw, lh), Vector2::zero(), 0.0, Color::WHITE);
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
            let planet = Matrix::scale(1.5, 1.5, 1.5) * Matrix::rotate_y(time * 0.05) * Matrix::translate(-4.8, -2.3, -8.0);
            Shot {
                items: vec![Item::plain(Who::Broom, Shape::Still, m), Item::plain(Who::Planet, Shape::Still, planet)],
                camera: Camera3D::perspective(Vector3::new(0.12, 0.08, 1.7 - push * 0.15), Vector3::new(-0.24, 0.0, 0.0), UP, fov),
                light: Light { lamp: Vector3::new(0.4, 1.2, 1.4), power: 3.2, tint: Vector3::zero(), range: 3.0, fog: 0.0, fill: 0.0, rim: 1.0 },
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
                light: Light { lamp: Vector3::new(-0.9, 1.4, 2.0), power: 8.0, tint: Vector3::new(1.0, 0.85, 0.8), range: 2.5, fog: 0.0, fill: 0.0, rim: 1.0 },
                stars: true,
            }
        }
        // El planeta desde arriba, a oscuras: la fiesta no se oye ni se ve
        2 => Shot {
            items: vec![Item::plain(Who::Planet, Shape::Still, Matrix::rotate_y(0.6 + time * 0.02))],
            camera: Camera3D::perspective(Vector3::new(0.0, 2.7 - push * 0.35, 2.0), Vector3::zero(), UP, fov),
            // Casi sin luz: el planeta es una sombra con el borde frío
            light: Light { lamp: Vector3::new(0.0, 3.0, 2.2), power: 0.12, tint: Vector3::zero(), range: 3.0, fog: 0.0, fill: 0.0, rim: 1.0 },
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
                light: Light { lamp: Vector3::new(0.0, 1.95, -0.45), power: 1.8 * flicker, tint: Vector3::zero(), range: 1.6, fog: 0.0, fill: 0.0, rim: 1.0 },
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
    render_view(d, thread, art, &items, &extras, camera, Light { lamp, power: 30.0, tint: Vector3::new(r, g, b), range: 2.0, fog: 0.0, fill: 0.0, rim: 1.0 },
                FISH_HALF, Rectangle::new(0.0, 0.0, w, h), t, 0.0, Backdrop { stars: false, halo: false, see_through: false });
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

/// Cuánto dura la invitación (la abducción de Hexbound en el hangar y el sistema solar).
pub const INVITATION: f32 = 3.1;
/// Cuándo pasa cada cosa (segundos): la carta, y un círculo que se abre en el medio y la deja ver a ella en su escoba
/// (la transición de iris, de `SCENE_START` a `IRIS_END`); la cámara la rodea desde adelante hasta quedar detrás; el
/// papelito se le vuela y ella sale disparada hacia el fondo.
const SCENE_START: f32 = 1.4;
const IRIS_END: f32 = 1.85;
const ORBIT_END: f32 = 2.2;
const RELEASE: f32 = 2.05;
const STORM: f32 = 2.15;
/// La nave (la bruja en la escoba, en el espacio del archivo: el asiento en el origen, mirando hacia +z), agrandada a
/// la escala de la escena, y la mano que saluda (la levantada), donde tiene el papelito.
const RIDER_SCALE: f32 = 1.45;
const WAVE_HAND: Vector3 = Vector3::new(-0.34, 0.43, 0.105);
/// Hacia dónde mira la cámara: el pecho de la bruja.
const CHEST: Vector3 = Vector3::new(-0.06, 0.26, 0.0);

/// Cuánto avanzó (en el mundo) `s` segundos después de salir disparada: arranca suave (cúbica, `EASE` segundos) y
/// después sigue acelerando parejo, sin saltos de posición ni de velocidad.
fn launch_distance(s: f32) -> f32 {
    const EASE: f32 = 0.4;
    const A: f32 = 7.5;
    const ACCEL: f32 = 13.0;
    let s = s.max(0.0);
    if s < EASE {
        return A * s * s * s;
    }
    let (d0, v0, r) = (A * EASE * EASE * EASE, 3.0 * A * EASE * EASE, s - EASE);
    d0 + v0 * r + 0.5 * ACCEL * r * r
}

/// La luz de la invitación: el farol (dónde, cuánto, a qué distancia cae a la mitad y de qué color: cero, el cálido
/// del juego), el relleno parejo, cuánto da la luz fría de atrás, si atrás hay estrellas y un cielo (degradé de arriba
/// abajo; sin él, negro).
pub struct InvitationLight {
    pub lamp: Vector3,
    pub power: f32,
    pub range: f32,
    pub tint: Vector3,
    pub fill: f32,
    pub rim: f32,
    pub stars: bool,
    pub sky: Option<(Color, Color)>,
}

/// Luz cálida y suave (la que eligió Cami: más suave que la del juego, con su negro): la luz cálida del farol, un
/// relleno suave para que nada quede negro y apenas el borde frío.
pub const INVITATION_LIGHT: InvitationLight = InvitationLight {
    lamp: Vector3::new(0.0, 0.0, 0.0), power: 3.6, range: 3.2, tint: Vector3::new(0.0, 0.0, 0.0), fill: 0.22, rim: 0.3,
    stars: false, sky: None,
};

/// El cielo de la invitación (si tiene; si no, negro), con `alpha`: el degradé y, si hay, las estrellas que titilan.
fn draw_sky(d: &mut impl RaylibDraw, look: &InvitationLight, w: f32, h: f32, t: f32, alpha: f32) {
    let Some((top, bottom)) = look.sky else {
        d.draw_rectangle(0, 0, w as i32, h as i32, fade(Color::BLACK, alpha));
        return;
    };
    d.draw_rectangle_gradient_v(0, 0, w as i32, h as i32, fade(top, alpha), fade(bottom, alpha));
    if look.stars {
        for i in 0..260 {
            let k = i as f32;
            let p = Vector2::new(noise(k * 1.31 + 0.2) * w, noise(k * 2.17 + 5.1) * h * 0.8);
            let a = 0.3 + 0.5 * noise(k * 0.71 + 9.0) * (0.7 + 0.3 * (t * (1.0 + noise(k) * 3.0) + k).sin());
            d.draw_circle_v(p, 0.8 + 1.4 * noise(k * 3.3), fade(Color::new(255, 245, 230, 255), a * alpha));
        }
    }
}

/// La invitación a la fiesta de las brujas (la abducción de Hexbound): primero la carta de cerca, sola en lo negro,
/// con la luz cálida del farol que parpadea encima; después ella, ya en la escoba, de cerca, quieta con el papelito
/// en la mano. Toma impulso (se echa apenas para atrás) y despega hacia adelante, acelerando e inclinándose, con su
/// estela, y se va de la luz del farol hacia lo oscuro. Las mismas luz, sombras, niebla negra y lente que el juego.
/// `letter`: el dibujo de la carta (sin él, un papel escrito).
pub fn draw_invitation(d: &mut RaylibDrawHandle, thread: &RaylibThread, art: &mut Art, letter: Option<&Texture2D>, t: f32) {
    draw_invitation_lit(d, thread, art, letter, t, &INVITATION_LIGHT);
}

/// `draw_invitation` con otra luz (para comparar).
pub fn draw_invitation_lit(d: &mut RaylibDrawHandle, thread: &RaylibThread, art: &mut Art, letter: Option<&Texture2D>, t: f32,
                           look: &InvitationLight) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    d.clear_background(Color::BLACK);
    let flicker = 0.85 + 0.1 * (t * 9.0).sin() * (t * 3.1).sin() + 0.05 * (noise((t * 18.0).floor()) - 0.5);
    if t >= SCENE_START {
        // Ella, en su escoba, flota meciéndose; al salir disparada se va hacia adelante (+z, al fondo), casi derecha
        let go = launch_distance(t - STORM);
        let launch = smooth((t - STORM) / 0.35);
        // Al salir disparada va culebreando de un lado al otro (y se ladea hacia cada curva), con sacudones de viento
        let fly = (t - STORM).max(0.0);
        let weave = (fly * 4.2).sin() * 0.28 * launch;
        let bump = |f: f32, o: f32| (t * f + o).sin() * (t * f * 0.61 + o * 1.7).sin();
        // Flotando nunca está quieta: sube y baja y se mece con varios ritmos encimados (no uno solo, parejo)
        let bob = 0.04 * (t * 2.4).sin() + 0.018 * (t * 5.3 + 1.1).sin();
        let at = Vector3::new(weave, 0.55 + bob + 0.04 * launch * bump(9.0, 0.3), 0.0) + Vector3::new(0.0, go * 0.03, go);
        // Se mece de costado, cabecea y gira (alrededor del asiento); al salir disparada se inclina hacia adelante y se
        // ladea hacia donde dobla
        let roll = 0.09 * (t * 1.6).sin() + 0.04 * (t * 3.7 + 0.4).sin() + 0.12 * launch
            - (fly * 4.2).cos() * 0.22 * launch + 0.05 * launch * bump(11.0, 2.0);
        let pitch = 0.06 * (t * 2.1 + 0.7).sin() + 0.025 * (t * 4.9).sin() + 0.1 * launch + 0.05 * launch * bump(8.0, 4.1);
        let yaw = 0.06 * (t * 1.3 + 1.9).sin() + (fly * 4.2).cos() * 0.12 * launch;
        let m = Matrix::scale(RIDER_SCALE, RIDER_SCALE, RIDER_SCALE) * Matrix::rotate_y(yaw) * Matrix::rotate_x(pitch)
            * Matrix::rotate_z(roll) * Matrix::translate(at.x, at.y, at.z);
        // El brazo levantado saluda; cuando se le vuela el papelito deja de saludar y lo baja. La falda flamea
        let down = smooth((t - RELEASE - 0.08) / 0.45);
        let (lift, wave) = (0.12 * (t * 3.1).sin() * (1.0 - down), 0.45 * (t * 7.5).sin() * (1.0 - down));
        art.animate_rider(lift, wave, down, t);
        let mut items = vec![Item::plain(Who::Rider, Shape::Still, m)];
        // El papelito: en la mano levantada; al soltarlo, el viento lo lleva para atrás (hacia la cámara), arriba y a
        // un costado, dando vueltas y flameando
        let bend = arm_bend(WAVE_HAND, lift, wave, 0.0).unwrap_or(Matrix::identity());
        let held = Matrix::rotate_x(-0.2) * Matrix::translate(WAVE_HAND.x, WAVE_HAND.y, WAVE_HAND.z) * bend;
        let note = if t < RELEASE {
            held * m
        } else {
            let s = t - RELEASE;
            let (lift0, wave0) = (0.12 * (RELEASE * 3.1).sin(), 0.45 * (RELEASE * 7.5).sin());
            let from = WAVE_HAND.transform_with(arm_bend(WAVE_HAND, lift0, wave0, 0.0).unwrap_or(Matrix::identity())).transform_with(m);
            let drift = Vector3::new(-0.3 * s + 0.12 * (s * 7.0).sin(), 0.22 * s - 0.12 * s * s, -1.5 * s - 0.5 * s * s);
            let p = from + drift;
            Matrix::rotate_x(-0.2 + s * 5.5) * Matrix::rotate_z(s * 3.2 + 0.4 * (s * 9.0).sin()) * Matrix::rotate_y(s * 2.0)
                * Matrix::scale(RIDER_SCALE, RIDER_SCALE, RIDER_SCALE) * Matrix::translate(p.x, p.y, p.z)
        };
        items.push(Item::plain(Who::Note, Shape::Still, note));
        // La cámara la rodea desde abajo (mirándola hacia arriba, como en el cine): de adelante, de tres cuartos del lado
        // de la mano levantada, hasta detrás y casi a su altura; cuando sale disparada la sigue apenas
        let k = smooth((t - SCENE_START) / (ORBIT_END - SCENE_START));
        let angle = (-30.0 + (-170.0) * k).to_radians();
        let elevation = (-18.0 + 12.0 * k).to_radians();
        let dist = 2.45 + 0.3 * k;
        let rest = Vector3::new(0.0, 0.55, 0.0) + CHEST * RIDER_SCALE;
        let target = rest + Vector3::new(0.0, go * 0.03, go) * 0.08;
        let dir = Vector3::new(angle.sin() * elevation.cos(), elevation.sin(), angle.cos() * elevation.cos());
        let camera = Camera3D::perspective(target + dir * dist, target, UP, FRAME_HALF * 2.0);
        // La luz de luna, de arriba y del lado de la cámara (así de cualquier lado se le ve la cara y la silueta)
        let side = Vector3::new(dir.x, 0.0, dir.z).normalized();
        let lamp = rest + UP * 1.5 + side * 1.0;
        let light = Light { lamp, power: look.power * flicker, tint: look.tint, range: look.range, fog: 0.0,
                            fill: look.fill, rim: look.rim };
        // El cielo (si hay) va atrás: la escena se dibuja encima, transparente donde no hay nada
        draw_sky(d, look, w, h, t, 1.0);
        render_view(d, thread, art, &items, &[], camera, light, FRAME_HALF, Rectangle::new(0.0, 0.0, w, h), t, 0.0,
                    Backdrop { stars: look.stars && look.sky.is_none(), halo: false, see_through: look.sky.is_some() });
        // Se va a negro al final (al principio no: la carta se funde encima)
        let dark = smooth((t - (INVITATION - 0.4)) / 0.4);
        d.draw_rectangle(0, 0, w as i32, h as i32, fade(Color::BLACK, dark));
    }
    if t < IRIS_END {
        // La carta; al final se le abre un círculo en el medio que crece y deja ver la escena (como en los dibujos
        // animados viejos), sin borde
        let iris = smooth((t - SCENE_START) / (IRIS_END - SCENE_START));
        if iris <= 0.0 {
            draw_letter(d, &art.font_big, letter, look, w, h, t);
        } else {
            let mut layer = art.overlay.take().expect("Art::prepare antes de dibujar");
            {
                let mut l = d.begin_texture_mode(thread, &mut layer);
                l.clear_background(Color::BLANK);
                draw_letter(&mut l, &art.font_big, letter, look, w, h, t);
            }
            let center = Vector2::new(w * 0.5, h * 0.5);
            let radius = iris * (w * w + h * h).sqrt() * 0.53;
            art.iris.set_shader_value(art.iris_center, center);
            art.iris.set_shader_value(art.iris_radius, radius);
            {
                let (lw, lh) = (layer.texture.width as f32, layer.texture.height as f32);
                let mut s = d.begin_shader_mode(&mut art.iris);
                s.draw_texture_pro(layer.texture(), Rectangle::new(0.0, 0.0, lw, -lh), Rectangle::new(0.0, 0.0, w, h),
                                   Vector2::zero(), 0.0, Color::WHITE);
            }
            art.overlay = Some(layer);
        }
        // Aparece desde el negro
        d.draw_rectangle(0, 0, w as i32, h as i32, fade(Color::BLACK, 1.0 - smooth(t / 0.3)));
    }
}

/// La carta de cerca, sola en el cielo de la invitación (o en lo negro), con la luz cálida encima: se acerca apenas y
/// se mece; y el grano de película.
fn draw_letter(d: &mut impl RaylibDraw, font: &Font, letter: Option<&Texture2D>, look: &InvitationLight, w: f32, h: f32, t: f32) {
    draw_sky(d, look, w, h, t, 1.0);
    let lh = h * (0.8 + 0.06 * smooth(t / IRIS_END));
    let lw = letter.map_or(lh * 0.8, |l| lh * l.width as f32 / l.height as f32);
    let center = Vector2::new(w * 0.5, h * 0.5 + (t * 1.7).sin() * h * 0.006);
    let angle = -2.5 + (t * 1.3).sin() * 1.2;
    let rect = Rectangle::new(center.x, center.y, lw, lh);
    let origin = Vector2::new(lw * 0.5, lh * 0.5);
    match letter {
        Some(l) => d.draw_texture_pro(l, Rectangle::new(0.0, 0.0, l.width as f32, l.height as f32), rect, origin, angle,
                                      Color::new(255, 240, 222, 255)),
        None => {
            // Sin el dibujo: un papel viejo con la invitación escrita a mano
            d.draw_rectangle_pro(rect, origin, angle, Color::new(214, 196, 160, 255));
            let size = lh * 0.15;
            let (sa, ca) = angle.to_radians().sin_cos();
            for (i, line) in ["Witch party", "tonight", "xoxo"].iter().enumerate() {
                let m = font.measure_text(line, size, 2.0);
                let dy = (i as f32 - 1.0) * size * 1.1;
                let at = center + Vector2::new(-dy * sa, dy * ca);
                let ink = if i == 2 { Color::new(150, 25, 30, 255) } else { Color::new(30, 14, 20, 255) };
                d.draw_text_pro(font, line, at, m * 0.5, angle, size, 2.0, ink);
            }
        }
    }
    // Grano de película
    for i in 0..260 {
        let k = i as f32 + (t * 24.0).floor() * 260.0;
        let p = Vector2::new(noise(k * 1.31) * w, noise(k * 2.17 + 3.0) * h);
        let g = if noise(k * 0.7) > 0.5 { Color::WHITE } else { Color::BLACK };
        d.draw_pixel_v(p, fade(g, 0.18));
    }
}

/// Cuánto dura la carta de agradecimiento (la escena de Hexbound al ganar el minijuego).
pub const THANKS: f32 = 3.0;

/// Un corazón relleno de ancho `size`, centrado en `c`: la curva clásica del corazón, en abanico desde el centro (una
/// sola forma, sin piezas encimadas, así se ve pareja aunque sea transparente).
fn heart(d: &mut impl RaylibDraw, c: Vector2, size: f32, color: Color) {
    const N: usize = 28;
    let k = size / 34.0;
    let point = |i: usize| {
        let a = i as f32 / N as f32 * std::f32::consts::TAU;
        let (s, co) = a.sin_cos();
        c + Vector2::new(16.0 * s * s * s, -(13.0 * co - 5.0 * (2.0 * a).cos() - 2.0 * (3.0 * a).cos() - (4.0 * a).cos())) * k
    };
    let center = c + Vector2::new(0.0, -2.0 * k);
    for i in 0..N {
        d.draw_triangle(center, point(i + 1), point(i), color);
    }
}

/// Los corazones que suben flotando, meciéndose y latiendo: los de atrás (`front` falso, más chicos y oscuros) o los
/// de adelante de la carta. `alpha`: cuánto se ven.
fn draw_hearts(d: &mut impl RaylibDraw, w: f32, h: f32, t: f32, front: bool, alpha: f32) {
    const COLORS: [Color; 4] = [Color::new(235, 40, 70, 255), Color::new(255, 105, 150, 255), Color::new(200, 20, 50, 255),
                                Color::new(255, 160, 185, 255)];
    for i in 0..26 {
        let k = i as f32;
        if (noise(k * 4.7 + 1.0) > 0.62) != front {
            continue;
        }
        // Cada uno sube a su velocidad desde abajo (empiezan repartidos, así ya hay corazones desde el principio)
        let speed = 0.16 + 0.14 * noise(k * 2.3);
        let rise = (noise(k * 1.9 + 3.0) + t * speed).fract();
        let mut x = w * (0.06 + 0.88 * noise(k * 1.31 + 0.5)) + (t * (1.2 + noise(k) * 1.5) + k).sin() * w * 0.02;
        // Los de adelante van a los costados de la carta, así nunca tapan lo que dice
        if front && (x - w * 0.5).abs() < h * 0.36 {
            x = w * 0.5 + (x - w * 0.5).signum() * (h * 0.36 + (x - w * 0.5).abs() * 0.5);
        }
        let y = h * (1.1 - 1.25 * rise);
        let beat = 1.0 + 0.12 * (t * 7.0 + k * 1.7).sin().max(0.0);
        let size = h * (0.035 + 0.05 * noise(k * 3.7 + 2.0)) * beat * if front { 1.25 } else { 0.8 };
        // Aparecen abajo y se desvanecen arriba
        let a = alpha * smooth(rise / 0.15) * (1.0 - smooth((rise - 0.75) / 0.25)) * if front { 0.95 } else { 0.6 };
        heart(d, Vector2::new(x, y), size, fade(COLORS[i % 4], a));
    }
}

/// La carta de agradecimiento de las brujas (la escena de Hexbound al ganar el minijuego, sin la bruja): la carta sola
/// en lo negro, con la luz cálida encima, que aparece con un saltito y flota, entre corazones que suben flotando (unos
/// atrás y otros adelante). Aparece y se va por negro.
pub fn draw_thanks(d: &mut RaylibDrawHandle, font: &Font, letter: Option<&Texture2D>, t: f32) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    d.clear_background(Color::BLACK);
    let show = smooth(t / 0.4);
    draw_hearts(d, w, h, t, false, show);
    // La carta: un saltito al aparecer (se agranda un poco de más y vuelve) y después flota meciéndose
    let pop = {
        let k = (t / 0.55).clamp(0.0, 1.0) - 1.0;
        1.0 + k * k * (2.2 * k + 1.2)
    };
    let lh = h * 0.78 * (0.85 + 0.15 * pop);
    let lw = letter.map_or(lh * 0.8, |l| lh * l.width as f32 / l.height as f32);
    let center = Vector2::new(w * 0.5, h * 0.5 + (t * 1.6).sin() * h * 0.01);
    let angle = -2.0 + (t * 1.2).sin() * 1.8;
    let rect = Rectangle::new(center.x, center.y, lw, lh);
    let origin = Vector2::new(lw * 0.5, lh * 0.5);
    let tint = Color::new(255, 240, 222, (255.0 * show) as u8);
    match letter {
        Some(l) => d.draw_texture_pro(l, Rectangle::new(0.0, 0.0, l.width as f32, l.height as f32), rect, origin, angle, tint),
        None => {
            // Sin el dibujo: un papel viejo con el agradecimiento escrito a mano
            d.draw_rectangle_pro(rect, origin, angle, fade(Color::new(214, 196, 160, 255), show));
            let size = lh * 0.12;
            let m = font.measure_text("Thanks for saving us!", size, 2.0);
            d.draw_text_pro(font, "Thanks for saving us!", center, m * 0.5, angle, size, 2.0, fade(Color::new(30, 14, 20, 255), show));
        }
    }
    draw_hearts(d, w, h, t, true, show);
    // Grano de película
    for i in 0..260 {
        let k = i as f32 + (t * 24.0).floor() * 260.0;
        let p = Vector2::new(noise(k * 1.31) * w, noise(k * 2.17 + 3.0) * h);
        let g = if noise(k * 0.7) > 0.5 { Color::WHITE } else { Color::BLACK };
        d.draw_pixel_v(p, fade(g, 0.18));
    }
    // Se va a negro al final
    d.draw_rectangle(0, 0, w as i32, h as i32, fade(Color::BLACK, smooth((t - (THANKS - 0.45)) / 0.45)));
}
