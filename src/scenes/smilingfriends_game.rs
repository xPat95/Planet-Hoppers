//! Pantalla del minijuego de Smiling Friends (después del video de entrada): personajes como recortes de cartón
//! sobre fondos pintados, texto que se escribe solo en cajas de historieta, respuestas en tarjetas y animaciones
//! grandes al reír, al fallar, al ganar y al perder. Cada escena puede tener su efecto: luces de boliche ("disco")
//! o una bomba al fallar ("bomb"); al reírse, un recorte puede darse vuelta y revelar otro personaje, o puede
//! saltar un primer plano a pantalla completa. Las reglas viven en `minigames::smilingfriends`.
use crate::graphics::resources::save_frame;
use crate::minigames::common::host::MinigameResult;
use crate::minigames::smilingfriends::{place_name, quality, scene_art, Dialogue, Encounter, Game, Quality, Step};
use raylib::prelude::*;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::path::Path;

const DIALOGUE: &str = "config/smilingfriends-dialogue.json";
const ART: &str = "assets/textures/minigames/smilingfriends";
const DESMOND_STREET_BACKGROUND: &str = "desmond_end_street.png";
/// Letras que tiene que traer cada fuente (español incluido).
const CHARSET: &str = " !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~áéíóúÁÉÍÓÚñÑüÜ¿¡…“”—";
const TYPE_SPEED: f32 = 48.0;
const MUSIC: &str = "assets/audio/music/Smiling Friends/gameotoon.mp3";
const MUSIC_VOLUME: f32 = 0.3;                            // Letras por segundo
const CUTOUT_BORDER: i32 = 11;                           // Borde blanco del recorte, en píxeles de la imagen
/// Píxeles de los recortes por alto de pantalla: todos se arman a esta escala, así el borde se ve igual en todos.
const CUTOUT_SCALE: f32 = 1000.0;
const VICTORY_POSE: &str = "boss_pointing.png";
const DEFEAT_POSE: &str = "boss_full.png";
/// Remate al reírse: el recorte se da vuelta como una carta entre estos dos momentos y aparece el otro dibujo.
const FLIP: (f32, f32) = (0.6, 1.0);
/// Bomba: cae, se quema la mecha y explota en este momento.
const BOOM_AT: f32 = 1.4;
const BOMB_DONE: f32 = 2.4;

// Paleta del póster de la serie (docs/references/smilingfriends/minigame/poster_palette.png): cielo celeste, amarillo
// de la carita, tinta azul oscura, naranja de Charlie, rosa de Pim y los colores de las letras del título.
const INK: Color = Color::new(43, 42, 62, 255);
const PAPER: Color = Color::new(252, 252, 255, 255);
const SKY: Color = Color::new(172, 222, 255, 255);
const YELLOW: Color = Color::new(253, 226, 80, 255);
const ORANGE: Color = Color::new(244, 104, 44, 255);
const PINK: Color = Color::new(248, 168, 232, 255);
const GREEN: Color = Color::new(120, 222, 120, 255);
const PURPLE: Color = Color::new(150, 104, 214, 255);
const RED: Color = Color::new(240, 92, 86, 255);
const CARDBOARD: Color = Color::new(176, 132, 82, 255);
const GOOD: Color = YELLOW;
const MEH: Color = Color::new(170, 176, 200, 255);
const BAD: Color = RED;

/// Personaje recortado: el dibujo con un borde blanco grueso (`face`) y su silueta lisa (`flat`) para el canto de
/// cartón y la sombra.
struct Cutout {
    face: Texture2D,
    flat: Texture2D,
    /// Centro del cuerpo (promedio de lo opaco), de 0 a 1 a lo ancho: se ubica por ahí y no por el centro de la
    /// imagen (si no, un brazo estirado corre al personaje a un costado).
    anchor: f32,
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, (c.a as f32 * a.clamp(0.0, 1.0)) as u8)
}

/// Color del arcoíris (0..1 da toda la vuelta).
fn hue(t: f32) -> Color {
    let f = |o: f32| (((t + o) * 2.0 * PI).cos() * 0.5 + 0.5) * 255.0;
    Color::new(f(0.0) as u8, f(1.0 / 3.0) as u8, f(2.0 / 3.0) as u8, 255)
}

fn mix(a: Color, b: Color, k: f32) -> Color {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k) as u8;
    Color::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b), m(a.a, b.a))
}

/// Imagen RGBA a partir de píxeles.
fn image_from(w: i32, h: i32, px: &[Color]) -> Image {
    let mut out = Image::gen_image_color(w, h, Color::BLANK);
    out.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let bytes: Vec<u8> = px.iter().flat_map(|c| [c.r, c.g, c.b, c.a]).collect();
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.as_mut().data as *mut u8, bytes.len()) };
    out
}

/// Prepara el dibujo para que todos los recortes tengan el mismo borde blanco: le saca el borde blanco que ya
/// traiga (lo casi blanco pegado al exterior transparente, hasta la línea negra del dibujo) y los márgenes
/// transparentes, así los pies quedan abajo de todo.
fn clean_art(img: &Image) -> (i32, i32, Vec<Color>) {
    let (w, h) = (img.width, img.height);
    let mut px: Vec<Color> = img.get_image_data().to_vec();
    let solid = |c: Color| c.a > 110;
    // Blanco o gris muy claro sin color (algunos originales traen el borde gris claro)
    let whiteish = |c: Color| c.r.min(c.g).min(c.b) > 185 && c.r.max(c.g).max(c.b) - c.r.min(c.g).min(c.b) < 28;
    let limit = (h as f32 * 0.03).max(4.0) as i32;      // Un borde existente no es más grueso que esto
    let mut depth = vec![-1i32; px.len()];
    let mut queue = std::collections::VecDeque::new();
    for i in 0..px.len() {
        if !solid(px[i]) {
            depth[i] = 0;
            queue.push_back(i);
        }
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = ((i as i32) % w, (i as i32) / w);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (u, v) = (x + dx, y + dy);
            if u < 0 || v < 0 || u >= w || v >= h {
                continue;
            }
            let j = (v * w + u) as usize;
            if depth[j] < 0 && whiteish(px[j]) && depth[i] < limit {
                depth[j] = depth[i] + 1;
                px[j].a = 0;                             // Borde blanco del original: afuera
                queue.push_back(j);
            }
        }
    }
    // Recorte de los márgenes transparentes
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if solid(px[(y * w + x) as usize]) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    if x1 <= x0 || y1 <= y0 {
        return (w, h, px);
    }
    let (cw, ch) = (x1 - x0, y1 - y0);
    let mut out = Vec::with_capacity((cw * ch) as usize);
    for y in y0..y1 {
        out.extend_from_slice(&px[(y * w + x0) as usize..(y * w + x1) as usize]);
    }
    (cw, ch, out)
}

/// Arma el recorte a partir de un PNG: limpia el dibujo, lo lleva a `target_h` píxeles de alto (el mismo número
/// de píxeles por alto de pantalla para todos: así el borde, que se mide en píxeles, se ve igual de grueso en
/// todos los personajes) y le agrega el borde blanco (distancia a lo opaco con chaflán: parejo y redondeado).
fn make_cutout(rl: &mut RaylibHandle, thread: &RaylibThread, path: &Path, target_h: i32) -> Result<Cutout, Box<dyn std::error::Error>> {
    let mut raw = Image::load_image(path.to_str().ok_or("ruta inválida")?)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    raw.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
    let (cw, ch, px) = clean_art(&raw);
    let mut img = image_from(cw, ch, &px);
    let inner = (target_h - 2 * (CUTOUT_BORDER + 3)).max(32);
    img.resize((cw as f32 * inner as f32 / ch as f32).round() as i32, inner);
    let (w, h) = (img.width, img.height);
    let src = img.get_image_data();
    let pad = CUTOUT_BORDER + 3;
    let (pw, ph) = (w + 2 * pad, h + 2 * pad);
    let idx = |x: i32, y: i32| (y * pw + x) as usize;
    // Distancia (×3) de cada píxel al píxel opaco más cercano
    let big = i32::MAX / 4;
    let mut dist = vec![big; (pw * ph) as usize];
    for y in 0..h {
        for x in 0..w {
            if src[(y * w + x) as usize].a > 110 {
                dist[idx(x + pad, y + pad)] = 0;
            }
        }
    }
    for y in 0..ph {
        for x in 0..pw {
            let mut d = dist[idx(x, y)];
            for (dx, dy, c) in [(-1, 0, 3), (0, -1, 3), (-1, -1, 4), (1, -1, 4)] {
                let (u, v) = (x + dx, y + dy);
                if u >= 0 && v >= 0 && u < pw {
                    d = d.min(dist[idx(u, v)].saturating_add(c));
                }
            }
            dist[idx(x, y)] = d;
        }
    }
    for y in (0..ph).rev() {
        for x in (0..pw).rev() {
            let mut d = dist[idx(x, y)];
            for (dx, dy, c) in [(1, 0, 3), (0, 1, 3), (1, 1, 4), (-1, 1, 4)] {
                let (u, v) = (x + dx, y + dy);
                if u < pw && v < ph && u >= 0 {
                    d = d.min(dist[idx(u, v)].saturating_add(c));
                }
            }
            dist[idx(x, y)] = d;
        }
    }
    let (mut mass, mut sum) = (0.0f64, 0.0f64);
    for y in 0..h {
        for x in 0..w {
            let a = src[(y * w + x) as usize].a as f64;
            mass += a;
            sum += a * (x + pad) as f64;
        }
    }
    let anchor = if mass > 0.0 { (sum / mass / pw as f64) as f32 } else { 0.5 };
    let mut face = vec![0u8; (pw * ph * 4) as usize];
    let mut flat = vec![0u8; (pw * ph * 4) as usize];
    for y in 0..ph {
        for x in 0..pw {
            let edge = (CUTOUT_BORDER as f32 + 0.5 - dist[idx(x, y)] as f32 / 3.0).clamp(0.0, 1.0);
            let (sx, sy) = (x - pad, y - pad);
            let s = if sx >= 0 && sy >= 0 && sx < w && sy < h { src[(sy * w + sx) as usize] } else { Color::BLANK };
            let a = s.a as f32 / 255.0;
            // El dibujo sobre el cartón blanco
            let over = |c: u8| (c as f32 * a + 255.0 * (1.0 - a)) as u8;
            let alpha = edge.max(a);
            let o = idx(x, y) * 4;
            face[o..o + 4].copy_from_slice(&[over(s.r), over(s.g), over(s.b), (alpha * 255.0) as u8]);
            flat[o..o + 4].copy_from_slice(&[255, 255, 255, (alpha * 255.0) as u8]);
        }
    }
    let texture = |rl: &mut RaylibHandle, bytes: &[u8]| -> Result<Texture2D, Box<dyn std::error::Error>> {
        let mut out = Image::gen_image_color(pw, ph, Color::BLANK);
        out.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8);
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.as_mut().data as *mut u8, bytes.len()) };
        let t = rl.load_texture_from_image(thread, &out)?;
        t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(t)
    };
    Ok(Cutout { face: texture(rl, &face)?, flat: texture(rl, &flat)?, anchor })
}

/// Pose de un recorte en este cuadro.
#[derive(Clone, Copy)]
struct Pose {
    x: f32,
    bottom: f32,
    height: f32,
    angle: f32,
    sx: f32,
    sy: f32,
    tint: Color,
}

fn draw_cutout(d: &mut RaylibDrawHandle, c: &Cutout, p: Pose) {
    let (tw, th) = (c.face.width as f32, c.face.height as f32);
    let h = p.height * p.sy;
    let w = p.height * tw / th * p.sx;
    let src = Rectangle::new(0.0, 0.0, tw, th);
    let origin = Vector2::new(w * c.anchor, h);
    // Sombra en el fondo, canto de cartón y después el dibujo
    for (dx, dy, tint, tex) in [
        (16.0, 10.0, Color::new(0, 0, 0, 70), &c.flat),
        (5.0, 4.0, mix(CARDBOARD, p.tint, 0.25), &c.flat),
        (0.0, 0.0, p.tint, &c.face),
    ] {
        d.draw_texture_pro(tex, src, Rectangle::new(p.x + dx, p.bottom + dy, w, h), origin, p.angle, tint);
    }
}

/// Personaje que todavía no tiene dibujo: una figura de cartón con un signo de pregunta.
fn draw_placeholder(d: &mut RaylibDrawHandle, title: &Font, p: Pose) {
    let h = p.height * p.sy;
    let w = h * 0.42 * p.sx;
    let dx = (p.angle.to_radians()).sin() * h * 0.5;
    let body = Rectangle::new(p.x - w / 2.0 + dx * 0.3, p.bottom - h * 0.62, w, h * 0.62);
    let card = Color::new(214, 196, 170, 255);
    d.draw_rectangle_rounded(Rectangle::new(body.x + 6.0, body.y + 5.0, body.width, body.height), 0.4, 10, CARDBOARD);
    d.draw_rectangle_rounded(body, 0.4, 10, card);
    let head = Vector2::new(p.x + dx, p.bottom - h * 0.78);
    d.draw_circle_v(head + Vector2::new(6.0, 5.0), h * 0.17, CARDBOARD);
    d.draw_circle_v(head, h * 0.17, card);
    let q = title.measure_text("?", h * 0.22, 0.0);
    d.draw_text_ex(title, "?", head - q * 0.5, h * 0.22, 0.0, Color::new(120, 96, 70, 255));
}

fn wrap(font: &Font, text: &str, size: f32, max_w: f32) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split_whitespace() {
        let line = lines.last_mut().unwrap();
        let trial = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if font.measure_text(&trial, size, 1.0).x > max_w && !line.is_empty() {
            lines.push(word.to_string());
        } else {
            *line = trial;
        }
    }
    lines
}

/// Texto con contorno completo (repetido en un círculo alrededor).
#[allow(clippy::too_many_arguments)]
fn outlined(d: &mut RaylibDrawHandle, font: &Font, text: &str, at: Vector2, size: f32, fill: Color, line: Color, r: f32) {
    for k in 0..12 {
        let a = k as f32 / 12.0 * 2.0 * PI;
        d.draw_text_ex(font, text, at + Vector2::new(a.cos() * r, a.sin() * r), size, 1.0, line);
    }
    d.draw_text_ex(font, text, at, size, 1.0, fill);
}

/// Panel de historieta: sombra dura, borde negro grueso y relleno.
fn panel(d: &mut RaylibDrawHandle, r: Rectangle, fill: Color, angle: f32) {
    let o = Vector2::new(r.width / 2.0, r.height / 2.0);
    let c = Rectangle::new(r.x + o.x, r.y + o.y, r.width, r.height);
    d.draw_rectangle_pro(Rectangle::new(c.x + 9.0, c.y + 9.0, c.width, c.height), o, angle, fade(INK, 0.55));
    d.draw_rectangle_pro(Rectangle::new(c.x, c.y, c.width + 10.0, c.height + 10.0), o + Vector2::new(5.0, 5.0), angle, INK);
    d.draw_rectangle_pro(c, o, angle, fill);
}

/// Sello grande que cae sobre la pantalla (de grande a su tamaño, girado).
#[allow(clippy::too_many_arguments)]
fn stamp(d: &mut RaylibDrawHandle, title: &Font, text: &str, center: Vector2, size: f32, color: Color, t: f32, angle: f32) {
    if t <= 0.0 {
        return;
    }
    let k = 1.0 + 2.2 * (1.0 - ease_out_back((t / 0.35).min(1.0)));
    let s = size * k;
    let m = title.measure_text(text, s, 2.0);
    let pad = s * 0.35;
    let r = Rectangle::new(center.x - m.x / 2.0 - pad, center.y - m.y / 2.0 - pad * 0.6, m.x + 2.0 * pad, m.y + 1.2 * pad);
    let o = Vector2::new(r.width / 2.0, r.height / 2.0);
    let a = (t / 0.12).min(1.0);
    d.draw_rectangle_pro(Rectangle::new(center.x, center.y, r.width + 16.0, r.height + 16.0), o + Vector2::new(8.0, 8.0), angle, fade(color, a));
    d.draw_rectangle_pro(Rectangle::new(center.x, center.y, r.width, r.height), o, angle, fade(PAPER, a));
    d.draw_text_pro(title, text, center, m * 0.5, angle, s, 2.0, fade(color, a));
}

/// Keep success labels close to the right edge, including their tilted paper border.
fn success_stamp_center(title: &Font, text: &str, size: f32, w: f32, h: f32) -> Vector2 {
    let m = title.measure_text(text, size, 2.0);
    let pad = size * 0.35;
    let half_w = (m.x + pad * 2.0) * 0.5;
    let half_h = (m.y + pad * 1.2) * 0.5;
    let angle = 6.0_f32.to_radians();
    let tilted_half_w = half_w * angle.cos() + half_h * angle.sin();
    Vector2::new((w - 30.0 - tilted_half_w).max(tilted_half_w + 24.0), (h * 0.235).max(155.0))
}

struct Confetti {
    pieces: Vec<(Vector2, Vector2, f32, Color)>,
}

impl Confetti {
    fn burst(w: f32, h: f32, n: usize, seed: u32) -> Self {
        let mut s = seed;
        let mut rnd = move || {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (s >> 8) as f32 / (1u32 << 24) as f32
        };
        let cols = [YELLOW, ORANGE, SKY, PINK, GREEN, PURPLE, RED];
        let pieces = (0..n)
            .map(|_| {
                let pos = Vector2::new(w * (0.2 + 0.6 * rnd()), h * (0.35 + 0.2 * rnd()));
                let vel = Vector2::new((rnd() - 0.5) * 900.0, -(500.0 + rnd() * 700.0));
                (pos, vel, rnd() * 6.0, cols[(rnd() * cols.len() as f32) as usize % cols.len()])
            })
            .collect();
        Self { pieces }
    }

    fn draw(&self, d: &mut RaylibDrawHandle, t: f32) {
        for (p, v, spin, c) in &self.pieces {
            let pos = *p + *v * t + Vector2::new(0.0, 900.0 * t * t);
            let a = spin + t * 9.0;
            d.draw_rectangle_pro(Rectangle::new(pos.x, pos.y, 14.0, 8.0 * a.cos().abs() + 2.0), Vector2::new(7.0, 4.0), a.to_degrees(), *c);
        }
    }
}

/// Escena del paso actual (None con el jefe).
fn encounter_of(step: Step) -> Option<usize> {
    match step {
        Step::Intro { encounter }
        | Step::Prompt { encounter, .. }
        | Step::Reaction { encounter, .. }
        | Step::Laughing { encounter }
        | Step::Failed { encounter } => Some(encounter),
        _ => None,
    }
}

fn has_effect(e: &Encounter, name: &str) -> bool {
    e.effect.as_deref() == Some(name)
}

/// Quién habla, qué dice y el contexto (si hay) en el paso actual. `revealed`: el remate ya se dio vuelta.
fn speech(game: &Game, revealed: bool) -> (String, String, Option<usize>, Option<String>) {
    let dlg = &game.dialogue;
    let actor = |e: &Encounter, k: usize, laugh: bool| {
        let a = &e.cast[k];
        if laugh && revealed { a.laugh_name.clone().unwrap_or_else(|| a.name.clone()) } else { a.name.clone() }
    };
    match game.step {
        Step::Boss { line } => (dlg.boss.name.clone(), dlg.boss.lines[line].text.clone(), None, None),
        Step::Victory => (dlg.boss.name.clone(), dlg.victory_line.clone(), None, None),
        Step::Defeat => (dlg.boss.name.clone(), dlg.defeat_line.clone(), None, None),
        Step::Intro { encounter } => {
            let e = &dlg.encounters[encounter];
            (actor(e, e.intro.speaker, false), e.intro.text.clone(), Some(e.intro.speaker), e.intro.context.clone())
        }
        Step::Prompt { encounter, round } => {
            let (e, r) = (&dlg.encounters[encounter], &dlg.encounters[encounter].rounds[round]);
            (actor(e, r.speaker, false), r.prompt.clone(), Some(r.speaker), r.context.clone())
        }
        Step::Reaction { encounter, round, choice } => {
            let (e, r) = (&dlg.encounters[encounter], &dlg.encounters[encounter].rounds[round]);
            let c = &r.choices[choice];
            let k = c.speaker.unwrap_or(r.speaker);
            (actor(e, k, false), c.reply.clone(), Some(k), c.context.clone())
        }
        Step::Laughing { encounter } => {
            let e = &dlg.encounters[encounter];
            let l = &e.laugh_line;
            let who = if e.laugh_scene_card.is_some() && revealed {
                "DESMOND'S POSTCARD".to_string()
            } else {
                actor(e, l.speaker, true)
            };
            (who, l.text.clone(), Some(l.speaker), l.context.clone())
        }
        Step::Failed { encounter } => {
            let e = &dlg.encounters[encounter];
            let l = &e.fail_line;
            (actor(e, l.speaker, false), l.text.clone(), Some(l.speaker), l.context.clone())
        }
    }
}

/// Cuándo empieza a escribirse el texto: después de los efectos grandes (remate, primer plano, bomba).
fn text_delay(game: &Game, bomb_defeat: bool) -> f32 {
    match game.step {
        Step::Laughing { encounter } => {
            let e = &game.dialogue.encounters[encounter];
            if e.laugh_scene_card.is_some() {
                CARD_DONE + 0.6
            } else if e.laugh_card.is_some() {
                1.6
            } else if e.cast.iter().any(|a| a.laugh_image.is_some()) {
                FLIP.1 + 0.15
            } else {
                0.25
            }
        }
        Step::Failed { encounter } if has_effect(&game.dialogue.encounters[encounter], "bomb") => BOMB_DONE,
        Step::Defeat if bomb_defeat => BOMB_DONE,
        Step::Prompt { encounter, round } if game.dialogue.encounters[encounter].rounds[round].scene_card.is_some() => CARD_DONE,
        _ => 0.25,
    }
}

/// Cartel de cambio de lugar: entra una tarjeta que tapa todo, se cambia el fondo detrás y se va.
const CARD_SWAP: f32 = 0.55;
const CARD_DONE: f32 = 1.75;

/// Pregunta actual, si el paso tiene una (para saber el lugar vigente).
fn round_of(step: Step) -> Option<usize> {
    match step {
        Step::Prompt { round, .. } | Step::Reaction { round, .. } => Some(round),
        _ => None,
    }
}

fn background_source(bg: &Texture2D, name: &str) -> Rectangle {
    let (w, h) = (bg.width as f32, bg.height as f32);
    if name == DESMOND_STREET_BACKGROUND {
        let zoom = 1.18;
        let (cw, ch) = (w / zoom, h / zoom);
        Rectangle::new((w - cw) * 0.3, (h - ch) * 0.5, cw, ch)
    } else {
        Rectangle::new(0.0, 0.0, w, h)
    }
}

fn draw_scene_card(d: &mut RaylibDrawHandle, title: &Font, text: &str, backdrop: Option<(&Texture2D, &str)>, t: f32, w: f32, h: f32) {
    // Entra desde la izquierda, se queda un momento y sale por la derecha
    let x = if t < CARD_SWAP {
        -w * (1.0 - ease_out_back(t / CARD_SWAP))
    } else if t < CARD_DONE - 0.4 {
        0.0
    } else {
        w * smoothstep((t - (CARD_DONE - 0.4)) / 0.4)
    };
    if t >= CARD_DONE {
        return;
    }
    d.draw_rectangle_rec(Rectangle::new(x, 0.0, w, h), SKY);
    if let Some((bg, name)) = backdrop {
        // La imagen del cartel cubre la tarjeta (recortada al centro)
        let s = (w / bg.width as f32).max(h / bg.height as f32);
        let (bw, bh) = (bg.width as f32 * s, bg.height as f32 * s);
        d.draw_texture_pro(bg, background_source(bg, name),
                           Rectangle::new(x + (w - bw) / 2.0, (h - bh) / 2.0, bw, bh), Vector2::zero(), 0.0, Color::WHITE);
    }
    for k in 0..14 {
        // Rayos de historieta detrás del texto
        let a = k as f32 / 14.0 * 2.0 * PI + t * 0.4;
        let c = Vector2::new(x + w / 2.0, h / 2.0);
        let p1 = c + Vector2::new(a.cos(), a.sin()) * w;
        let p2 = c + Vector2::new((a + 0.16).cos(), (a + 0.16).sin()) * w;
        d.draw_triangle(c, p2, p1, fade(PAPER, 0.35));
    }
    let size = h * 0.07;
    let m = title.measure_text(text, size, 2.0);
    let r = Rectangle::new(x + (w - m.x) / 2.0 - 40.0, h / 2.0 - m.y / 2.0 - 24.0, m.x + 80.0, m.y + 48.0);
    panel(d, r, YELLOW, -3.0);
    d.draw_text_pro(title, text, Vector2::new(r.x + r.width / 2.0, r.y + r.height / 2.0), m * 0.5, -3.0, size, 2.0, INK);
}

/// Juega el minijuego en una ventana ya abierta. `None` si se cerró la ventana.
pub fn play(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path, smoke: bool) -> Result<Option<MinigameResult>, Box<dyn std::error::Error>> {
    play_with_results(rl, thread, root, smoke, None, &mut |_| Ok(()))
}

/// `audio`: el dispositivo de sonido de la entrada (sin él, el juego va en silencio).
pub fn play_with_results(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path, smoke: bool, audio: Option<&RaylibAudio>,
                         observer: &mut crate::minigames::common::host::ResultObserver<'_>) -> Result<Option<MinigameResult>, Box<dyn std::error::Error>> {
    // La música del juego ("Gameotoon"), en loop, desde que termina el video de la intro
    let music = audio.and_then(|d| d.new_music(&root.join(MUSIC).to_string_lossy()).ok()).map(|mut m| {
        m.looping = true;
        m.set_volume(MUSIC_VOLUME);
        m.play_stream();
        m
    });
    let mut reporter = crate::minigames::common::host::AttemptReporter::default();
    let dialogue = Dialogue::load(&root.join(DIALOGUE))?;
    let title = rl.load_font_ex(thread, &root.join("assets/fonts/Bungee-Regular.ttf").to_string_lossy(), 72, Some(CHARSET))?;
    let body = rl.load_font_ex(thread, &root.join("assets/fonts/TitilliumWeb-Bold.ttf").to_string_lossy(), 64, Some(CHARSET))?;
    let effects = rl.load_font_ex(thread, &root.join("assets/fonts/GloriaHallelujah.ttf").to_string_lossy(), 108, Some(CHARSET))?;
    for f in [&title, &body, &effects] {
        unsafe { raylib::ffi::SetTextureFilter(f.texture, TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
    }
    // Fondos, recortes y primeros planos de todo el diálogo, cargados una vez
    let art = root.join(ART);
    let mut backgrounds: HashMap<String, Texture2D> = HashMap::new();
    let mut cutouts: HashMap<String, Cutout> = HashMap::new();
    let mut bgs = vec![dialogue.boss.background.clone()];
    // Cada dibujo con el alto (fracción de la pantalla) con el que se va a ver
    let mut chars: Vec<(String, f32)> = dialogue.boss.lines.iter().map(|l| (l.pose.clone(), 0.86)).collect();
    chars.extend([(VICTORY_POSE.to_string(), 0.86), (DEFEAT_POSE.to_string(), 0.86)]);
    for e in &dialogue.encounters {
        bgs.push(e.background.clone());
        bgs.extend(e.laugh_card.clone());
        bgs.extend(e.laugh_background.clone());
        bgs.extend(e.card_backdrop.clone());
        let defaults: &[f32] = if e.cast.len() == 1 { &[0.86] } else { &[0.78, 0.6] };
        for (k, a) in e.cast.iter().enumerate() {
            let ht = a.size.unwrap_or(defaults[k.min(defaults.len() - 1)]);
            chars.push((a.image.clone(), ht));
            chars.extend(a.laugh_image.clone().map(|i| (i, a.laugh_size.unwrap_or(ht))));
            if k == 0 {
                chars.extend(e.rounds.iter().filter_map(|r| r.image.clone()).map(|i| (i, ht)));
            }
        }
        for r in &e.rounds {
            bgs.extend(r.background.clone());
        }
    }
    for b in bgs {
        if !backgrounds.contains_key(&b) {
            let t = rl.load_texture(thread, &art.join("backgrounds").join(&b).to_string_lossy())?;
            t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            backgrounds.insert(b, t);
        }
    }
    // Si un dibujo se usa con dos altos, se arma para el más grande (el borde puede quedar apenas más fino)
    let mut tallest: HashMap<String, f32> = HashMap::new();
    for (c, ht) in chars.into_iter().filter(|(c, _)| !c.is_empty()) {
        let e = tallest.entry(c).or_insert(0.0);
        *e = e.max(ht);
    }
    for (c, ht) in tallest {
        let target = (ht * CUTOUT_SCALE).round() as i32;
        cutouts.insert(c.clone(), make_cutout(rl, thread, &art.join("characters").join(&c), target)?);
    }

    let mut game = Game::new(dialogue);
    let mut step_time = 0.0f32;
    let mut last_step = game.step;
    let mut meter_shown = 0.0f32;
    let mut strike_time = [f32::MAX; 8];                // Cuándo apareció cada strike (para el sello)
    let mut smile_time = [f32::MAX; 8];                 // Cuándo se completó cada escena (carita del contador)
    let mut shake = 0.0f32;
    let mut confetti: Option<(Confetti, f32)> = None;
    let mut defeat_from: Option<usize> = None;          // Escena en la que se perdió
    let mut time = 0.0f32;
    let mut frame = 0u32;
    let captures = root.join("target/smilingfriends");
    let mut shots: HashSet<&'static str> = HashSet::new();
    // El menú de SALIR (o Esc): con el botón elegido; mientras está abierto, el juego queda quieto
    let mut leaving: Option<usize> = None;

    loop {
        if rl.window_should_close() {
            return Ok(None);
        }
        if let Some(m) = &music {
            // Con el menú de salir abierto, más bajita
            m.set_volume(if leaving.is_some() { MUSIC_VOLUME * 0.4 } else { MUSIC_VOLUME });
            m.update_stream();
        }
        let dt = if smoke { 1.0 / 60.0 } else if leaving.is_some() { 0.0 } else { rl.get_frame_time().min(0.05) };
        time += dt;
        step_time += dt;
        let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
        let bomb_defeat = game.step == Step::Defeat
            && defeat_from.is_some_and(|e| has_effect(&game.dialogue.encounters[e], "bomb"));
        let delay = text_delay(&game, bomb_defeat);
        let (who, text, speaker, context) = speech(&game, step_time >= (FLIP.0 + FLIP.1) / 2.0);
        let n_chars = text.chars().count();
        let shown_chars = (((step_time - delay) * TYPE_SPEED).max(0.0) as usize).min(n_chars);
        let done_typing = shown_chars >= n_chars;
        let typing_end = delay + n_chars as f32 / TYPE_SPEED;   // Cuándo termina de escribirse

        // --- Entrada ---------------------------------------------------------------------------------------
        let mouse = rl.get_mouse_position();
        let mut click = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let esc = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER) || rl.is_key_pressed(KeyboardKey::KEY_SPACE);
        if let Some(sel) = leaving {
            // Menú de salir: seguir jugando, rendirse (cuenta como ganado) o volver al sistema solar
            let rects = leave_buttons(w, h);
            let mut sel = sel;
            if rl.is_key_pressed(KeyboardKey::KEY_UP) {
                sel = (sel + LEAVE.len() - 1) % LEAVE.len();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_DOWN) {
                sel = (sel + 1) % LEAVE.len();
            }
            if let Some(k) = rects.iter().position(|r| r.check_collision_point_rec(mouse)) {
                sel = k;
            }
            let chosen = if click && rects[sel].check_collision_point_rec(mouse) || enter { Some(sel) } else { None };
            leaving = Some(sel);
            match chosen {
                _ if esc => leaving = None,
                Some(0) => leaving = None,
                Some(1) => {
                    observer(MinigameResult::Victory)?;
                    return Ok(Some(MinigameResult::Victory));
                }
                Some(_) => return Ok(Some(MinigameResult::Abandoned)),
                None => {}
            }
            click = false;
        } else if !smoke && (esc || click && exit_button(w, h).check_collision_point_rec(mouse)) {
            leaving = Some(0);
            click = false;
        }
        let next = leaving.is_none() && (click || enter);
        let cards = choice_cards(w, h, &game);
        let mut picked: Option<usize> = None;
        if let Step::Prompt { .. } = game.step {
            if done_typing {
                for (i, key) in [KeyboardKey::KEY_ONE, KeyboardKey::KEY_TWO, KeyboardKey::KEY_THREE].iter().enumerate() {
                    if rl.is_key_pressed(*key) {
                        picked = Some(i);
                    }
                }
                if click {
                    picked = cards.iter().position(|r| r.check_collision_point_rec(mouse)).or(picked);
                }
            } else if next {
                step_time = step_time.max(typing_end);  // Completa el texto
            }
        } else if next && step_time > delay {
            // Durante los efectos grandes (antes de `delay`) no se puede saltar
            if !done_typing {
                step_time = step_time.max(typing_end);
            } else if game.finished() {
                if step_time > typing_end + 0.6 {
                    let result = if game.step == Step::Victory { MinigameResult::Victory } else { MinigameResult::Defeat };
                    return Ok(Some(result));
                }
            } else {
                game.advance();
            }
        }
        // Prueba automática: en la calle falla una vez (para ver la bomba) y después elige lo mejor
        if smoke && done_typing && step_time > typing_end + 1.3 {
            match game.step {
                Step::Prompt { encounter, round } => {
                    let c = &game.dialogue.encounters[encounter].rounds[round].choices;
                    let fail_once = has_effect(&game.dialogue.encounters[encounter], "bomb") && game.strikes == 0;
                    picked = if fail_once {
                        (0..c.len()).find(|&i| quality(&c[i]) == Quality::Meh)
                    } else {
                        (0..c.len()).max_by_key(|&i| c[i].laugh)
                    };
                }
                Step::Victory if step_time > typing_end + 2.0 => return Ok(Some(MinigameResult::Victory)),
                Step::Victory | Step::Defeat => {}
                _ => game.advance(),
            }
        }
        if let Some(i) = picked {
            game.choose(i);
        }
        reporter.observe(&game, observer)?;
        if game.step != last_step {
            // Paso nuevo: animaciones de entrada, strikes y festejos
            let strikes_before = strike_time.iter().filter(|t| **t < f32::MAX).count() as u32;
            for k in strikes_before..game.strikes.min(8) {
                strike_time[k as usize] = time;
                shake = 0.35;
            }
            match game.step {
                Step::Laughing { encounter } => {
                    confetti = Some((Confetti::burst(w, h, 70, 7 + encounter as u32), time));
                    let after = if game.dialogue.encounters[encounter].laugh_scene_card.is_some() { CARD_DONE } else { 0.3 };
                    smile_time[encounter.min(7)] = time + after;
                }
                Step::Victory => confetti = Some((Confetti::burst(w, h, 160, 99), time)),
                Step::Defeat => defeat_from = encounter_of(last_step),
                _ => {}
            }
            last_step = game.step;
            step_time = 0.0;
        }
        let target = game.meter();
        meter_shown += (target - meter_shown) * (1.0 - (-8.0 * dt).exp());
        if matches!(game.step, Step::Intro { .. } | Step::Boss { .. }) {
            meter_shown = target;
        }
        shake = (shake - dt).max(0.0);

        // --- Escena ------------------------------------------------------------------------------------------
        let scene = encounter_of(game.step).or(if bomb_defeat { defeat_from } else { None });
        let enc = scene.map(|e| &game.dialogue.encounters[e]);
        let disco = enc.is_some_and(|e| has_effect(e, "disco"));
        let bombing = (matches!(game.step, Step::Failed { .. }) && enc.is_some_and(|e| has_effect(e, "bomb"))) || bomb_defeat;
        let boom_shake = if bombing && step_time > BOOM_AT { (1.0 - (step_time - BOOM_AT) / 0.7).max(0.0) } else { 0.0 };
        // Lugar vigente: el de la escena o el del último cambio de lugar. Durante el cartel, el fondo cambia
        // recién cuando la tarjeta tapa la pantalla.
        let card = match game.step {
            Step::Prompt { encounter, round } => game.dialogue.encounters[encounter].rounds[round].scene_card.clone(),
            Step::Laughing { encounter } => game.dialogue.encounters[encounter].laugh_scene_card.clone(),
            _ => None,
        };
        // Final en otro lugar: después de que el cartel tapa la pantalla
        let later = matches!(game.step, Step::Laughing { .. })
            && enc.is_some_and(|e| e.laugh_scene_card.is_some())
            && step_time >= CARD_SWAP;
        let card_backdrop = enc.and_then(|e| e.card_backdrop.as_ref()).and_then(|b| backgrounds.get(b).map(|bg| (bg, b.as_str())));
        let start = |e: &Encounter| (e.background.clone(), e.cast[0].image.clone());
        let place = enc.map(|e| match (game.step, round_of(game.step)) {
            (Step::Intro { .. }, _) => start(e),
            // Antes de que la tarjeta tape la pantalla, todavía se ve el lugar anterior
            (Step::Prompt { .. }, Some(r)) if card.is_some() && step_time < CARD_SWAP => {
                r.checked_sub(1).map_or_else(|| start(e), |p| scene_art(e, Some(p)))
            }
            (_, Some(r)) => scene_art(e, Some(r)),
            _ => scene_art(e, None),                    // Final de la escena: el último lugar
        });
        let bg_name = match enc.and_then(|e| e.laugh_background.clone()) {
            Some(b) if later => b,
            _ => place.as_ref().map_or(game.dialogue.boss.background.clone(), |p| p.0.clone()),
        };
        // Personajes en escena: (dibujo, posición x, alto, ¿habla?)
        let mut cast: Vec<(String, f32, f32, bool, Option<usize>, f32)> = Vec::new();
        match (game.step, enc) {
            (Step::Boss { line }, _) => cast.push((game.dialogue.boss.lines[line].pose.clone(), 0.3, 0.86, true, None, 1.02)),
            (Step::Victory, _) => cast.push((VICTORY_POSE.into(), 0.3, 0.86, true, None, 1.02)),
            (Step::Defeat, None) => cast.push((DEFEAT_POSE.into(), 0.3, 0.86, true, None, 1.02)),
            (_, Some(e)) => {
                let layout: &[(f32, f32)] = if e.cast.len() == 1 { &[(0.28, 0.86)] } else { &[(0.2, 0.78), (0.43, 0.6)] };
                for (k, a) in e.cast.iter().enumerate() {
                    let laughing = matches!(game.step, Step::Laughing { .. });
                    let base = if k == 0 { place.as_ref().map_or(a.image.clone(), |p| p.1.clone()) } else { a.image.clone() };
                    // Con cartel de final, el cambio pasa detrás del cartel; si no, se da vuelta como una carta
                    let swap_at = if e.laugh_scene_card.is_some() { CARD_SWAP } else { (FLIP.0 + FLIP.1) / 2.0 };
                    let image = match &a.laugh_image {
                        Some(img) if laughing && step_time >= swap_at => img.clone(),
                        _ => base,
                    };
                    let (x, ht) = layout[k.min(layout.len() - 1)];
                    let (mut x, mut ht) = (a.x.unwrap_or(x), a.size.unwrap_or(ht));   // El JSON puede ubicarlo a mano
                    let flipped = laughing && a.laugh_image.is_some() && step_time >= swap_at;
                    let mut floor = a.floor.unwrap_or(1.02);
                    if flipped {
                        // El remate puede tener otro tamaño: cambia justo cuando la carta está de canto
                        ht = a.laugh_size.unwrap_or(ht);
                        floor = a.laugh_floor.unwrap_or(floor);
                        x = a.laugh_x.unwrap_or(x);
                    }
                    cast.push((image, x, ht, speaker == Some(k) || speaker.is_none() || laughing, Some(k), floor));
                }
            }
            _ => {}
        }

        // --- Dibujo ------------------------------------------------------------------------------------------
        let shake_off = Vector2::new((time * 70.0).sin(), (time * 53.0).cos()) * (shake * 22.0 + boom_shake * 40.0);
        let mut d = rl.begin_drawing(thread);
        d.clear_background(INK);
        if let Some(bg) = backgrounds.get(&bg_name) {
            // El fondo cubre toda la pantalla y se mueve apenas
            let s = (w / bg.width as f32).max(h / bg.height as f32) * 1.04;
            let (bw, bh) = (bg.width as f32 * s, bg.height as f32 * s);
            let drift = (time * 0.15).sin() * 8.0;
            d.draw_texture_pro(bg, background_source(bg, &bg_name),
                               Rectangle::new((w - bw) / 2.0 + drift + shake_off.x, (h - bh) / 2.0 + shake_off.y, bw, bh),
                               Vector2::zero(), 0.0, Color::WHITE);
        }
        let dim = match game.step {
            Step::Defeat if !bomb_defeat => 0.55 * smoothstep(step_time / 0.6),
            Step::Prompt { .. } if done_typing => 0.18,
            _ => 0.0,
        } + if disco { 0.25 } else { 0.0 };
        d.draw_rectangle(0, 0, w as i32, h as i32, fade(INK, dim));
        if disco {
            draw_disco_beams(&mut d, time, w, h);
        }

        for (image, x, ht, talking, actor, floor) in &cast {
            if later {
                continue;
            }
            let mut pose = character_pose(&game, step_time, time, w, h, done_typing && *talking, *x, *ht, *actor);
            pose.bottom += (floor - 1.02) * h;
            if !*talking {
                pose.tint = Color::new(205, 205, 210, 255);  // El que no habla queda un poco atrás
            }
            if disco {
                pose.tint = mix(pose.tint, hue(time * 0.35 + *x), 0.33);
            }
            if bombing {
                launch(&mut pose, step_time, w, h);
            }
            match cutouts.get(image) {
                Some(c) => draw_cutout(&mut d, c, pose),
                None => draw_placeholder(&mut d, &title, pose),
            }
        }
        if later {
            if let Some(e) = enc {
                if let Some(img) = e.cast[0].laugh_image.as_ref().and_then(|name| cutouts.get(name)) {
                    draw_postcard(&mut d, &title, &body, img, w, h);
                }
            }
        }
        if disco {
            draw_disco_spots(&mut d, time, w, h);
        }

        // Reacción que salta junto a la cabeza del que habla
        if let Step::Reaction { encounter, round, choice } = game.step {
            let c = &game.dialogue.encounters[encounter].rounds[round].choices[choice];
            let col = match quality(c) {
                Quality::Good => GOOD,
                Quality::Meh => MEH,
                Quality::Bad => BAD,
            };
            let x = cast.iter().find(|c| c.4 == speaker).map_or(0.3, |c| c.1);
            let k = ease_out_back((step_time / 0.35).min(1.0));
            let size = (h * 0.10) * k.max(0.01);
            let m = effects.measure_text(&c.reaction, size, 1.0);
            let at = Vector2::new(x * w + w * 0.12, h * 0.2 - step_time * 18.0) - m * 0.5;
            outlined(&mut d, &effects, &c.reaction, at, size, col, INK, (size * 0.06).max(2.0));
        }
        if let (Step::Laughing { .. }, Some(e)) = (game.step, enc) {
            if let Some(card) = e.laugh_card.as_ref().and_then(|c| backgrounds.get(c)) {
                draw_laugh_card(&mut d, card, step_time, time, w, h);
            } else if !later {
                let after = if e.laugh_scene_card.is_some() { CARD_DONE } else { 0.0 };
                draw_laughter(&mut d, &effects, step_time - after, w * cast.first().map_or(0.3, |c| c.1), w, h);
            }
            // Remate: el nombre nuevo cae como un sello cuando el recorte se da vuelta
            if let Some(a) = e.cast.iter().find(|a| a.laugh_image.is_some()) {
                if let Some(name) = &a.laugh_name {
                    // Arriba a la derecha, lejos del personaje
                    let label = format!("{name}!");
                    let size = h * 0.052;
                    let center = success_stamp_center(&title, &label, size, w, h);
                    stamp(&mut d, &title, &label, center, size, ORANGE,
                          step_time - if e.laugh_scene_card.is_some() { CARD_DONE + 0.3 } else { FLIP.1 - 0.15 }, -6.0);
                }
            }
        }
        if bombing {
            let target = Vector2::new(w * cast.first().map_or(0.3, |c| c.1), h * 0.74);
            draw_bomb(&mut d, &title, step_time, target, w, h);
        }

        hud(&mut d, &title, &body, &game, meter_shown, &smile_time, time, w);
        if step_time > delay - 0.3 {
            dialogue_box(&mut d, &title, &body, &effects, &who, context.as_deref(), &text, shown_chars, step_time - (delay - 0.25), w, h,
                         done_typing && !matches!(game.step, Step::Prompt { .. }));
        }
        if let Step::Prompt { encounter, round } = game.step {
            if done_typing {
                let choices = &game.dialogue.encounters[encounter].rounds[round].choices;
                for (i, (r, c)) in cards.iter().zip(choices).enumerate() {
                    let t = (step_time - typing_end - i as f32 * 0.09).max(0.0);
                    let slide = (1.0 - ease_out_back((t / 0.35).min(1.0))) * w * 0.5;
                    let hover = r.check_collision_point_rec(mouse);
                    let r2 = Rectangle::new(r.x + slide - if hover { 16.0 } else { 0.0 }, r.y, r.width, r.height);
                    panel(&mut d, r2, if hover { PINK } else { PAPER }, if hover { -1.5 } else { -0.6 });
                    let size = (r.height * 0.3).min(30.0);
                    let lines = wrap(&body, &c.text, size, r.width - 50.0);
                    for (k, line) in lines.iter().enumerate() {
                        let y = r2.y + r.height / 2.0 - lines.len() as f32 * size * 0.5 + k as f32 * size;
                        d.draw_text_ex(&body, line, Vector2::new(r2.x + 26.0, y), size, 1.0, INK);
                    }
                }
            }
        }
        if let Some((c, t0)) = &confetti {
            // Con cartel de final, el confeti sale cuando el cartel ya se fue
            let wait = if matches!(game.step, Step::Laughing { .. }) && card.is_some() { CARD_DONE } else { 0.0 };
            if time - t0 >= wait && !later {
                c.draw(&mut d, time - t0 - wait);
            }
        }
        if let Some(text) = &card {
            draw_scene_card(&mut d, &title, text, card_backdrop, step_time, w, h);
        }
        match game.step {
            Step::Victory => {
                let label = "MISSION COMPLETE!";
                let size = h * 0.052;
                stamp(&mut d, &title, label, success_stamp_center(&title, label, size, w, h), size,
                      Color::new(70, 170, 90, 255), step_time - 0.4, -7.0);
            }
            Step::Defeat => stamp(&mut d, &title, "NOT FUNNY", Vector2::new(w / 2.0, h * 0.36), h * 0.1, BAD,
                                  step_time - if bomb_defeat { BOMB_DONE } else { 0.6 }, -9.0),
            Step::Failed { .. } => stamp(&mut d, &title, "STRIKE!", Vector2::new(w / 2.0, h * 0.34), h * 0.08, BAD,
                                         step_time - if bombing { BOMB_DONE } else { 0.2 }, 6.0),
            _ => {}
        }
        if game.finished() && done_typing {
            // Dentro de la caja, abajo a la derecha
            let hint = "Enter: back to the solar system";
            let m = body.measure_text(hint, 24.0, 1.0);
            d.draw_text_ex(&body, hint, Vector2::new(w * 0.93 - m.x - 40.0, h * 0.96 - 46.0), 24.0, 1.0, ORANGE);
        }
        // Capturas de la prueba automática
        draw_exit_button(&mut d, &title, exit_button(w, h), mouse);
        if let Some(sel) = leaving {
            draw_leave_menu(&mut d, &title, &body, w, h, sel);
        }
        if smoke {
            let want = match (game.step, scene) {
                (Step::Boss { line: 1 }, _) if step_time > 1.0 => Some("boss.png"),
                (Step::Prompt { .. }, Some(0)) if step_time > typing_end + 1.1 => Some("bar-prompt.png"),
                (Step::Laughing { .. }, Some(0)) if step_time > 1.5 => Some("bar-reveal.png"),
                (Step::Prompt { .. }, Some(1)) if step_time > typing_end + 1.1 => Some("forest-prompt.png"),
                (Step::Laughing { .. }, Some(1)) if step_time > 1.5 => Some("forest-free.png"),
                (Step::Failed { .. }, Some(2)) if step_time > 1.0 && step_time < 1.2 => Some("bomb-fuse.png"),
                (Step::Failed { .. }, Some(2)) if step_time > BOOM_AT + 0.15 => Some("bomb-boom.png"),
                (Step::Reaction { .. }, Some(2)) if step_time > 0.6 && game.strikes > 0 => Some("street-reaction.png"),
                (Step::Laughing { .. }, Some(2)) if step_time > 0.8 && step_time < 1.0 => Some("street-later-card.png"),
                (Step::Laughing { .. }, Some(2)) if step_time > CARD_DONE + 1.5 => Some("street-happy.png"),
                (Step::Prompt { round: 1, .. }, Some(2)) if step_time > 0.8 && step_time < 1.0 => Some("park-card.png"),
                (Step::Prompt { round: 1, .. }, Some(2)) if step_time > typing_end + 1.1 => Some("park-prompt.png"),
                (Step::Victory, _) if step_time > typing_end + 1.5 => Some("victory.png"),
                _ => None,
            };
            if let Some(name) = want.filter(|n| !shots.contains(n)) {
                std::fs::create_dir_all(&captures)?;
                save_frame(&captures.join(name), w as i32, h as i32)?;
                shots.insert(name);
            }
            frame += 1;
            if frame > 60 * 300 {
                return Err("La prueba del minijuego no terminó".into());
            }
        }
    }
}

/// Tarjetas de respuesta, a la derecha.
fn exit_button_width(w: f32) -> f32 {
    (w * 0.16).clamp(112.0, 170.0)
}

fn exit_button(w: f32, h: f32) -> Rectangle {
    let width = exit_button_width(w);
    Rectangle::new(w - width - 22.0, (h * 0.027).max(14.0), width, 52.0)
}

/// Los botones del menú de salir.
const LEAVE: [&str; 3] = ["KEEP PLAYING", "I GIVE UP :(", "BACK TO THE SOLAR SYSTEM"];

fn leave_buttons(w: f32, h: f32) -> Vec<Rectangle> {
    let (bw, bh) = ((w * 0.36).clamp(300.0, 460.0), 56.0);
    (0..LEAVE.len()).map(|i| Rectangle::new(w / 2.0 - bw / 2.0, h * 0.5 - 40.0 + i as f32 * (bh + 22.0), bw, bh)).collect()
}

/// El menú de salir: un panel de historieta con el título y tres botones (el elegido, amarillo).
fn draw_leave_menu(d: &mut RaylibDrawHandle, title: &Font, body: &Font, w: f32, h: f32, sel: usize) {
    d.draw_rectangle(0, 0, w as i32, h as i32, fade(INK, 0.55));
    let rects = leave_buttons(w, h);
    let top = rects[0].y - 150.0;
    let bottom = rects[rects.len() - 1].y + rects[0].height + 40.0;
    let pw = rects[0].width + 120.0;
    panel(d, Rectangle::new(w / 2.0 - pw / 2.0, top, pw, bottom - top), PAPER, -1.0);
    let head = "LEAVING ALREADY?";
    let m = title.measure_text(head, 44.0, 1.0);
    d.draw_text_ex(title, head, Vector2::new(w / 2.0 - m.x / 2.0, top + 34.0), 44.0, 1.0, INK);
    let sub = "Giving up counts as a win.";
    let m = body.measure_text(sub, 26.0, 1.0);
    d.draw_text_ex(body, sub, Vector2::new(w / 2.0 - m.x / 2.0, top + 92.0), 26.0, 1.0, INK);
    for (i, (r, label)) in rects.iter().zip(LEAVE).enumerate() {
        panel(d, *r, if i == sel { YELLOW } else { PAPER }, 0.0);
        let m = title.measure_text(label, 24.0, 1.0);
        d.draw_text_ex(title, label, Vector2::new(r.x + r.width / 2.0 - m.x / 2.0, r.y + r.height / 2.0 - m.y / 2.0), 24.0, 1.0, INK);
    }
}

fn draw_exit_button(d: &mut RaylibDrawHandle, title: &Font, r: Rectangle, mouse: Vector2) {
    let hover = r.check_collision_point_rec(mouse);
    panel(d, r, if hover { YELLOW } else { PAPER }, if hover { -2.0 } else { -1.0 });
    let label = "SALIR";
    let size = 27.0;
    let m = title.measure_text(label, size, 1.0);
    d.draw_text_pro(title, label, Vector2::new(r.x + r.width * 0.5, r.y + r.height * 0.5),
                    m * 0.5, if hover { -2.0 } else { -1.0 }, size, 1.0, INK);
}

fn choice_cards(w: f32, h: f32, game: &Game) -> Vec<Rectangle> {
    let n = match game.step {
        Step::Prompt { encounter, round } => game.dialogue.encounters[encounter].rounds[round].choices.len(),
        _ => 0,
    };
    let (cw, ch) = (w * 0.4, h * 0.105);
    (0..n).map(|i| Rectangle::new(w * 0.56, h * 0.16 + i as f32 * (ch + h * 0.035), cw, ch)).collect()
}

/// Pose del recorte: entra rebotando desde abajo, se balancea al hablar y reacciona según la respuesta.
#[allow(clippy::too_many_arguments)]
fn character_pose(game: &Game, t: f32, time: f32, w: f32, h: f32, done_typing: bool, x: f32, height: f32, actor: Option<usize>) -> Pose {
    let mut p = Pose { x: w * x, bottom: h * 1.02, height: h * height, angle: 0.0, sx: 1.0, sy: 1.0, tint: Color::WHITE };
    let seed = actor.unwrap_or(0) as f32 * 1.7;           // Cada personaje se mueve a su ritmo
    let enter = matches!(game.step, Step::Intro { .. } | Step::Boss { line: 0 });
    if enter {
        let k = ease_out_back(((t - seed * 0.1) / 0.55).clamp(0.0, 1.0));
        p.bottom += (1.0 - k) * h * 0.9;
        p.angle = (1.0 - k) * -14.0;
    }
    // Al hablar se balancea un poco, como un recorte que alguien mueve
    let talk = if done_typing { 0.35 } else { 1.0 };
    p.angle += (time * 7.0 + seed).sin() * 2.2 * talk;
    p.bottom += ((time * 14.0 + seed).sin().abs()) * -6.0 * talk;
    match game.step {
        Step::Reaction { encounter, round, choice } => {
            let decay = (-2.5 * t).exp();
            match quality(&game.dialogue.encounters[encounter].rounds[round].choices[choice]) {
                Quality::Good => {
                    p.bottom -= (t * 11.0).sin().abs() * 60.0 * decay;
                    p.sy = 1.0 + 0.06 * (t * 22.0).sin() * decay;
                }
                Quality::Meh => p.angle += (t * 6.0).sin() * 6.0 * decay,
                Quality::Bad => {
                    p.x += (t * 45.0).sin() * 18.0 * decay;
                    p.sy = 1.0 - 0.06 * (1.0 - decay);
                    p.sx = 1.0 + 0.03 * (1.0 - decay);
                }
            }
        }
        Step::Laughing { encounter } => {
            p.bottom -= (t * 12.0 + seed).sin().abs() * 55.0;
            p.angle += (t * 15.0 + seed).sin() * 9.0;
            p.sy = 1.0 + 0.05 * (t * 24.0).sin();
            // El que tiene remate se da vuelta como una carta
            let e = &game.dialogue.encounters[encounter];
            if e.laugh_scene_card.is_some() {
                let k = ease_out_back(((t - CARD_SWAP) / 0.55).clamp(0.0, 1.0));
                p.bottom += (1.0 - k) * h * 0.9;
            }
            let flips = e.laugh_scene_card.is_none() && actor.is_some_and(|k| e.cast[k].laugh_image.is_some());
            if flips && t > FLIP.0 && t < FLIP.1 {
                p.sx *= ((t - FLIP.0) / (FLIP.1 - FLIP.0) * PI).cos().abs().max(0.02);
                p.bottom -= ((t - FLIP.0) / (FLIP.1 - FLIP.0) * PI).sin() * 70.0;
            }
        }
        Step::Failed { .. } => {
            p.angle += -12.0 * smoothstep(t / 0.4);
            p.sy = 1.0 - 0.08 * smoothstep(t / 0.4);
        }
        Step::Victory => {
            p.bottom -= (t * 9.0).sin().abs() * 40.0;
            p.angle += (t * 9.0).sin() * 5.0;
        }
        Step::Defeat => {
            // El recorte se cae de espaldas
            let k = smoothstep((t - 0.15) / 0.55);
            p.angle = -85.0 * k;
            p.bottom += k * h * 0.08;
        }
        _ => {}
    }
    p
}

/// Después de la explosión, el recorte sale volando girando y cae de vuelta, chamuscado.
fn launch(p: &mut Pose, t: f32, w: f32, h: f32) {
    let s = t - BOOM_AT;
    if s <= 0.0 {
        // Antes de explotar: tiembla de miedo
        p.x += (t * 50.0).sin() * 4.0 * smoothstep(t / 0.8);
        return;
    }
    let up = 1800.0 * s - 1500.0 * s * s;
    p.bottom -= up.max(-h * 0.2);
    p.x += s * w * 0.25;
    p.angle += s * 720.0;
    p.tint = mix(p.tint, Color::new(70, 60, 60, 255), (s * 2.0).min(0.6));
}

/// Bomba de caricatura: cae junto al personaje, rebota, se quema la mecha, se infla y explota (¡KABOOM!).
fn draw_bomb(d: &mut RaylibDrawHandle, title: &Font, t: f32, target: Vector2, w: f32, h: f32) {
    let r = h * 0.07;
    if t < BOOM_AT {
        // Cae desde arriba con un rebote
        let fall = (t / 0.45).min(1.0);
        let bounce = if t > 0.45 { ((t - 0.45) * 9.0).sin().abs() * 30.0 * (-(t - 0.45) * 4.0).exp() } else { 0.0 };
        let c = Vector2::new(target.x + w * 0.14, -r + (target.y + r) * fall * fall - bounce);
        let swell = 1.0 + 0.12 * smoothstep((t - 0.8) / 0.6) * (1.0 + (t * 30.0).sin());
        let red = smoothstep((t - 0.9) / 0.5);
        d.draw_circle_v(c + Vector2::new(7.0, 7.0), r * swell, fade(INK, 0.5));
        d.draw_circle_v(c, r * swell + 5.0, INK);
        d.draw_circle_v(c, r * swell, mix(Color::new(40, 40, 52, 255), Color::new(150, 30, 30, 255), red * 0.6));
        d.draw_circle_v(c + Vector2::new(-r * 0.35, -r * 0.35), r * 0.22, Color::new(140, 140, 160, 255));
        // Tapa y mecha que se va acortando
        let cap = c + Vector2::new(r * 0.45, -r * 0.8);
        d.draw_rectangle_pro(Rectangle::new(cap.x, cap.y, r * 0.5, r * 0.32), Vector2::new(r * 0.25, r * 0.16), 35.0, INK);
        let burn = smoothstep((t - 0.5) / (BOOM_AT - 0.5));
        let mut prev = cap;
        let steps = 10;
        let mut tip = cap;
        for k in 1..=steps {
            let u = k as f32 / steps as f32;
            if u > 1.0 - burn * 0.9 {
                break;
            }
            let p = cap + Vector2::new(u * r * 0.9, -u * r * 0.9 + (u * 6.0).sin() * r * 0.15);
            d.draw_line_ex(prev, p, 4.0, Color::new(205, 170, 110, 255));
            prev = p;
            tip = p;
        }
        if t > 0.45 {
            // Chispa que titila en la punta
            for k in 0..8 {
                let a = k as f32 / 8.0 * 2.0 * PI + t * 20.0;
                let len = r * (0.25 + 0.15 * ((t * 40.0 + k as f32).sin()));
                d.draw_line_ex(tip, tip + Vector2::new(a.cos(), a.sin()) * len, 3.0, if k % 2 == 0 { YELLOW } else { ORANGE });
            }
            d.draw_circle_v(tip, 5.0, PAPER);
        }
        return;
    }
    // ¡KABOOM!: destello blanco, bolas de fuego que crecen, humo y el texto
    let s = t - BOOM_AT;
    let at = Vector2::new(target.x + w * 0.14, target.y);
    let flash = (1.0 - s / 0.12).max(0.0);              // Destello corto: no lava los colores del fuego
    for j in 0..14 {
        // Rayos de la explosión
        let a = j as f32 / 14.0 * 2.0 * PI;
        let len = h * 0.6 * ease_out_back((s / 0.3).min(1.0));
        let alpha = (1.0 - s / 0.8).max(0.0);
        d.draw_line_ex(at, at + Vector2::new(a.cos(), a.sin()) * len, 10.0, fade(YELLOW, alpha));
    }
    for (k, col) in [(2.2, Color::new(70, 62, 62, 235)), (1.6, Color::new(232, 70, 30, 255)), (1.1, ORANGE), (0.6, YELLOW)].iter() {
        let grow = ease_out_back((s / 0.4).min(1.0)) * h * 0.32 * k;
        let alpha = if *k > 2.0 { (1.0 - (s - 0.5) / 1.5).clamp(0.0, 1.0) } else { (1.0 - (s - 0.2) / 0.6).clamp(0.0, 1.0) };
        for j in 0..9 {
            let a = j as f32 / 9.0 * 2.0 * PI + k;
            let o = Vector2::new(a.cos(), a.sin()) * grow * 0.45;
            d.draw_circle_v(at + o, grow * 0.55, fade(*col, alpha));
        }
    }
    if s < 1.6 {
        let size = h * 0.16 * ease_out_back((s / 0.3).min(1.0));
        let word = "KABOOM!";
        let m = title.measure_text(word, size, 2.0);
        let c = Vector2::new(w / 2.0, h * 0.42);
        let wob = (s * 30.0).sin() * 3.0 * (1.0 - s).max(0.0);
        d.draw_text_pro(title, word, c + Vector2::new(10.0, 10.0), m * 0.5, -8.0 + wob, size, 2.0, INK);
        d.draw_text_pro(title, word, c, m * 0.5, -8.0 + wob, size, 2.0, ORANGE);
    }
    d.draw_rectangle(0, 0, w as i32, h as i32, fade(PAPER, flash));
}

/// Luces de boliche: haces de colores que barren la escena desde arriba (se suman a lo que hay debajo).
fn draw_disco_beams(d: &mut RaylibDrawHandle, time: f32, w: f32, h: f32) {
    let mut b = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
    for k in 0..5 {
        let from = Vector2::new(w * (0.1 + 0.2 * k as f32), -10.0);
        let a = (time * (0.7 + 0.15 * k as f32) + k as f32 * 1.3).sin() * 0.55 + PI / 2.0;
        let dir = Vector2::new(a.cos(), a.sin());
        let side = Vector2::new(-dir.y, dir.x) * h * 0.16;
        let end = from + dir * h * 1.3;
        let c = fade(hue(time * 0.2 + k as f32 * 0.21), 0.22);
        b.draw_triangle(from, end - side, end + side, c);
        b.draw_triangle(from, end + side, end - side, c);
    }
}

/// Manchas de luz que recorren a los personajes y destellos de la bola de espejos.
fn draw_disco_spots(d: &mut RaylibDrawHandle, time: f32, w: f32, h: f32) {
    let mut b = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
    for k in 0..6 {
        let x = w * (0.5 + 0.42 * (time * (0.5 + 0.13 * k as f32) + k as f32 * 2.1).sin());
        let y = h * (0.5 + 0.3 * (time * (0.37 + 0.11 * k as f32) + k as f32).cos());
        let c = hue(time * 0.25 + k as f32 / 6.0);
        b.draw_circle_gradient(x as i32, y as i32, h * 0.13, fade(c, 0.35), fade(c, 0.0));
    }
    for k in 0..26 {
        // Destellos que aparecen y desaparecen
        let x = w * ((k as f32 * 0.6180339).fract());
        let y = h * ((k as f32 * 0.4142135 + 0.1).fract()) * 0.8;
        let tw = ((time * 3.0 + k as f32 * 1.7).sin() * 0.5 + 0.5).powi(6);
        let s = 3.0 + 9.0 * tw;
        let col = fade(PAPER, tw);
        let c = Vector2::new(x, y);
        let angle = k as f32 * 2.399 + time * 0.35;
        let rotate = |v: Vector2| Vector2::new(v.x * angle.cos() - v.y * angle.sin(), v.x * angle.sin() + v.y * angle.cos());
        let tips = [
            Vector2::new(0.0, -s), Vector2::new(s * 0.17, -s * 0.17),
            Vector2::new(s * 0.75, 0.0), Vector2::new(s * 0.17, s * 0.17),
            Vector2::new(0.0, s), Vector2::new(-s * 0.17, s * 0.17),
            Vector2::new(-s * 0.75, 0.0), Vector2::new(-s * 0.17, -s * 0.17),
        ];
        b.draw_circle_gradient(x as i32, y as i32, s * 1.6, fade(PAPER, tw * 0.16), Color::BLANK);
        for i in 0..tips.len() {
            b.draw_triangle(c, c + rotate(tips[(i + 1) % tips.len()]), c + rotate(tips[i]), col);
        }
    }
}

/// Carcajada: "HA! HAHA! HAHAHA!" saltando alrededor del personaje.
fn draw_laughter(d: &mut RaylibDrawHandle, laughter: &Font, t: f32, x: f32, w: f32, h: f32) {
    for (k, word) in ["HA!", "HAHA!", "HAHAHA!"].iter().enumerate() {
        let t = t - k as f32 * 0.3;
        if t > 0.0 {
            let size = h * (0.084 + 0.028 * k as f32) * ease_out_back((t / 0.3).min(1.0)).max(0.01);
            let px = x + [-0.16, 0.14, -0.02][k] * w;
            let y = h * (0.335 - 0.08 * k as f32) - t * 30.0;
            let m = laughter.measure_text(word, size, 1.0);
            let ang = [-12.0, 9.0, -4.0][k];
            d.draw_text_pro(laughter, word, Vector2::new(px, y) + Vector2::new(5.0, 5.0), m * 0.5, ang, size, 1.0, INK);
            d.draw_text_pro(laughter, word, Vector2::new(px, y), m * 0.5, ang, size, 1.0, YELLOW);
        }
    }
}

/// Primer plano feliz a pantalla completa: salta como una foto con marco blanco, rodeada de destellos.
fn draw_laugh_card(d: &mut RaylibDrawHandle, card: &Texture2D, t: f32, time: f32, w: f32, h: f32) {
    let s = t - 0.3;
    if s <= 0.0 {
        return;
    }
    let k = ease_out_back((s / 0.5).min(1.0));
    let cw = w * 0.62 * k;
    let ch = cw * card.height as f32 / card.width as f32;
    let c = Vector2::new(w * 0.5, h * 0.4);
    let ang = -4.0 * k + (time * 2.0).sin() * 0.8;
    // Rayos de sol detrás
    for j in 0..16 {
        let a = j as f32 / 16.0 * 2.0 * PI + time * 0.4;
        let p1 = c + Vector2::new(a.cos(), a.sin()) * w;
        let p2 = c + Vector2::new((a + 0.12).cos(), (a + 0.12).sin()) * w;
        d.draw_triangle(c, p2, p1, fade(YELLOW, 0.35 * k));
    }
    d.draw_rectangle_pro(Rectangle::new(c.x + 12.0, c.y + 12.0, cw + 30.0, ch + 30.0), Vector2::new(cw / 2.0 + 15.0, ch / 2.0 + 15.0), ang, fade(INK, 0.6));
    d.draw_rectangle_pro(Rectangle::new(c.x, c.y, cw + 30.0, ch + 30.0), Vector2::new(cw / 2.0 + 15.0, ch / 2.0 + 15.0), ang, PAPER);
    d.draw_texture_pro(card, Rectangle::new(0.0, 0.0, card.width as f32, card.height as f32),
                       Rectangle::new(c.x, c.y, cw, ch), Vector2::new(cw / 2.0, ch / 2.0), ang, Color::WHITE);
}

/// The later reveal is a mailed photograph, rather than another animated character on the street.
fn draw_postcard(d: &mut RaylibDrawHandle, title: &Font, body: &Font, desmond: &Cutout, w: f32, h: f32) {
    let card = Rectangle::new(w * 0.12, h * 0.125, w * 0.55, h * 0.54);
    let photo = Rectangle::new(card.x + 18.0, card.y + 18.0, card.width - 36.0, card.height - 98.0);
    d.draw_rectangle_rec(Rectangle::new(card.x + 10.0, card.y + 10.0, card.width, card.height), fade(INK, 0.55));
    d.draw_rectangle_rec(Rectangle::new(card.x - 4.0, card.y - 4.0, card.width + 8.0, card.height + 8.0), INK);
    d.draw_rectangle_rec(card, PAPER);
    d.draw_rectangle_rec(photo, YELLOW);
    let cut_h = (photo.height * 0.94).min(photo.width * 0.75 * desmond.face.height as f32 / desmond.face.width as f32);
    draw_cutout(d, desmond, Pose {
        x: photo.x + photo.width * 0.48,
        bottom: photo.y + photo.height - 4.0,
        height: cut_h,
        angle: 0.0,
        sx: 1.0,
        sy: 1.0,
        tint: Color::WHITE,
    });
    d.draw_rectangle_lines_ex(photo, 3.0, INK);
    d.draw_text_ex(title, "POSTCARD FROM DESMOND", Vector2::new(card.x + 24.0, photo.y + photo.height + 17.0), h * 0.034, 1.0, INK);
    d.draw_text_ex(body, "Doing alright these days.", Vector2::new(card.x + 26.0, photo.y + photo.height + 48.0), h * 0.027, 1.0, ORANGE);
}

#[allow(clippy::too_many_arguments)]
fn hud(d: &mut RaylibDrawHandle, title: &Font, body: &Font, game: &Game, meter: f32, smiles: &[f32; 8], time: f32, w: f32) {
    if let Some(encounter) = encounter_of(game.step) {
        // Medidor de risa: barra amarilla con la etiqueta RISA
        let r = Rectangle::new(28.0, 26.0, w * 0.3, 34.0);
        panel(d, r, PAPER, 0.0);
        d.draw_rectangle_rec(Rectangle::new(r.x + 4.0, r.y + 4.0, (r.width - 8.0) * meter, r.height - 8.0), YELLOW);
        outlined(d, title, "LAUGH", Vector2::new(r.x + 12.0, r.y + 4.0), 26.0, PAPER, INK, 2.0);
        let e = &game.dialogue.encounters[encounter];
        let name = match game.step {
            Step::Intro { .. } => e.name.clone(),
            step => place_name(e, round_of(step)),
        };
        let label = format!("{}  -  {}/{}", name, encounter + 1, game.dialogue.encounters.len());
        outlined(d, body, &label, Vector2::new(r.x + 4.0, r.y + r.height + 14.0), 24.0, PAPER, INK, 2.0);
    }
    // Contador: un círculo por escena; cada escena terminada pone una carita sonriente que cae como un sello
    let n_enc = game.dialogue.encounters.len();
    for k in 0..n_enc {
        let c = Vector2::new(w - exit_button_width(w) - 58.0 - (n_enc - 1 - k) as f32 * 56.0, 46.0);
        d.draw_circle_v(c + Vector2::new(4.0, 4.0), 22.0, fade(INK, 0.6));
        d.draw_circle_v(c, 22.0, PAPER);
        if smiles[k] > time {
            d.draw_circle_lines(c.x as i32, c.y as i32, 22.0, INK);
            continue;
        }
        let t = time - smiles[k];
        let r = 22.0 * (1.0 + 0.8 * (1.0 - ease_out_back((t / 0.35).min(1.0))));
        d.draw_circle_v(c, r, YELLOW);
        d.draw_circle_v(c + Vector2::new(-r * 0.33, -r * 0.2), r * 0.12, INK);
        d.draw_circle_v(c + Vector2::new(r * 0.33, -r * 0.2), r * 0.12, INK);
        for i in 0..8 {
            let a0 = 0.2 * PI + i as f32 / 8.0 * 0.6 * PI;
            let a1 = 0.2 * PI + (i + 1) as f32 / 8.0 * 0.6 * PI;
            d.draw_line_ex(c + Vector2::new(a0.cos(), a0.sin()) * r * 0.55, c + Vector2::new(a1.cos(), a1.sin()) * r * 0.55, 3.0, INK);
        }
        d.draw_circle_lines(c.x as i32, c.y as i32, r, INK);
    }
}

#[allow(clippy::too_many_arguments)]
fn dialogue_box(d: &mut RaylibDrawHandle, title: &Font, body: &Font, effects: &Font, who: &str, context: Option<&str>, text: &str, shown: usize, t: f32, w: f32, h: f32, can_continue: bool) {
    // La caja entra con un pequeño rebote en cada línea nueva
    let k = ease_out_back((t / 0.3).min(1.0));
    let bw = w * 0.86;
    let bh = h * 0.21;
    let x = (w - bw) / 2.0;
    let y = h - bh - h * 0.04 + (1.0 - k) * 40.0;
    panel(d, Rectangle::new(x, y, bw, bh), PAPER, 0.0);
    // Nombre en una etiqueta amarilla inclinada
    let ns = h * 0.04;
    let nm = title.measure_text(who, ns, 1.0);
    let tag = Rectangle::new(x + 24.0, y - ns * 0.9, nm.x + 36.0, ns * 1.45);
    panel(d, tag, YELLOW, -3.0);
    d.draw_text_pro(title, who, Vector2::new(tag.x + tag.width / 2.0, tag.y + tag.height / 2.0), nm * 0.5, -3.0, ns, 1.0, INK);
    // Contexto (lo que hace o quiere decir), en gris y más chico, arriba; después lo que dice, escrito de a poco
    let mut top = y + 30.0;
    if let Some(ctx) = context {
        let cs = (h * 0.034).clamp(16.0, 34.0);
        for line in wrap(effects, ctx, cs, bw - 70.0) {
            d.draw_text_ex(effects, &line, Vector2::new(x + 36.0, top), cs, 1.0, fade(Color::new(110, 98, 120, 255), smoothstep(t / 0.3)));
            top += cs * 1.12;
        }
        top += cs * 0.25;
    }
    let size = if context.is_some() { (h * 0.058).clamp(22.0, 60.0) } else { (h * 0.05).clamp(20.0, 52.0) };
    let all = wrap(body, text, size, bw - 70.0);
    let mut left = shown;
    for (i, line) in all.iter().enumerate() {
        let n = line.chars().count();
        let part: String = line.chars().take(left).collect();
        left = left.saturating_sub(n + 1);
        d.draw_text_ex(body, &part, Vector2::new(x + 36.0, top + i as f32 * size * 1.12), size, 1.0, INK);
    }
    if can_continue {
        // Flechita que late: se puede seguir
        let p = Vector2::new(x + bw - 40.0, y + bh - 30.0 + (t * 6.0).sin() * 4.0);
        d.draw_triangle(p, p + Vector2::new(18.0, 0.0), p + Vector2::new(9.0, 13.0), ORANGE);
    }
}

/// Ruido simple y repetible (0..1) para los trazos hechos a mano.
fn jitter(k: u32) -> f32 {
    let mut s = k.wrapping_mul(2_654_435_761).wrapping_add(0x9e37_79b9);
    s ^= s >> 15;
    s = s.wrapping_mul(2_246_822_519);
    s ^= s >> 13;
    (s & 0xffff) as f32 / 65535.0
}

/// Línea temblorosa de crayón: el trazo cambia un poco ocho veces por segundo, como un dibujo animado a mano.
#[allow(clippy::too_many_arguments)]
fn crayon_line(d: &mut RaylibDrawHandle, a: Vector2, b: Vector2, thick: f32, color: Color, seed: u32, time: f32) {
    let boil = (time * 8.0) as u32;
    let len = (b - a).length();
    let n = ((len / 18.0) as u32).max(2);
    let normal = Vector2::new(-(b - a).y, (b - a).x) / len.max(1.0);
    let mut prev = a;
    for k in 1..=n {
        let t = k as f32 / n as f32;
        let wob = if k == n { 0.0 } else { (jitter(seed * 977 + k * 31 + boil * 7919) - 0.5) * 5.0 };
        let p = a + (b - a) * t + normal * wob;
        d.draw_line_ex(prev, p, thick, color);
        d.draw_circle_v(p, thick * 0.5, color);
        prev = p;
    }
}

/// Rectángulo de crayón: las esquinas se pasan un poquito, como cuando lo dibuja un chico.
fn crayon_rect(d: &mut RaylibDrawHandle, r: Rectangle, thick: f32, color: Color, seed: u32, time: f32) {
    let c = [
        Vector2::new(r.x, r.y),
        Vector2::new(r.x + r.width, r.y),
        Vector2::new(r.x + r.width, r.y + r.height),
        Vector2::new(r.x, r.y + r.height),
    ];
    for k in 0..4 {
        let (a, b) = (c[k], c[(k + 1) % 4]);
        let dir = (b - a) / (b - a).length().max(1.0);
        crayon_line(d, a - dir * 6.0, b + dir * 8.0, thick, color, seed + k as u32, time);
    }
}

/// Filled hand-drawn circle whose silhouette shifts with the intro's crayon animation.
fn crayon_smiley_disc(d: &mut RaylibDrawHandle, c: Vector2, r: f32, seed: u32, time: f32) {
    let boil = (time * 8.0) as u32;
    let edge: Vec<Vector2> = (0..36)
        .map(|i| {
            let a = i as f32 / 36.0 * 2.0 * PI;
            let wobble = (jitter(seed + i * 137 + boil * 7919) - 0.5) * r * 0.17;
            c + Vector2::new(a.cos(), a.sin()) * (r + wobble)
        })
        .collect();
    for i in 0..edge.len() {
        let next = (i + 1) % edge.len();
        let dir = (edge[i] - c) / (edge[i] - c).length().max(1.0);
        let next_dir = (edge[next] - c) / (edge[next] - c).length().max(1.0);
        d.draw_triangle(c, edge[next] + next_dir * 3.0, edge[i] + dir * 3.0, INK);
        d.draw_triangle(c, edge[next], edge[i], YELLOW);
        crayon_line(d, edge[i] + dir * 2.0, edge[next] + next_dir * 2.0, 3.0, INK, seed + i as u32, time);
    }
}

/// Botón SKIP del video de entrada (el mismo rectángulo para dibujarlo y para el clic).
pub fn skip_button(w: f32, h: f32) -> Rectangle {
    Rectangle::new(w - 220.0, h - 110.0, 180.0, 64.0)
}

/// Pantalla del video de entrada: fondo celeste con dibujitos de crayón, el video más chico en el centro con un
/// marco de papel hecho a mano (con cinta en las esquinas) y el botón SKIP.
pub fn draw_intro(d: &mut RaylibDrawHandle, video: &Texture2D, title: &Font, time: f32, mouse: Vector2) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    d.clear_background(SKY);
    // Sol con rayos de crayón
    let sun = Vector2::new(w * 0.09, h * 0.13);
    d.draw_circle_v(sun, h * 0.065, YELLOW);
    for k in 0..10 {
        let a = k as f32 / 10.0 * 2.0 * PI + (time * 0.5).sin() * 0.1;
        let dir = Vector2::new(a.cos(), a.sin());
        crayon_line(d, sun + dir * h * 0.085, sun + dir * h * 0.125, 5.0, ORANGE, 40 + k, time);
    }
    // Nubes como las del póster: bultos redondos con base plana, contorno de tinta, una sombra celeste abajo y
    // un vaivén lento
    // Repartidas sin orden, algunas juntas, y cada una inclinada para su lado (como dibujadas a mano)
    let clouds = [(0.83, 0.11, 1.0, -0.1), (0.69, 0.05, 0.62, 0.13), (0.16, 0.89, 0.85, 0.07), (0.31, 0.95, 0.55, -0.15)];
    for (k, (cx, cy, s, tilt)) in clouds.iter().enumerate() {
        let c = Vector2::new(w * cx + (time * 0.4 + k as f32).sin() * 6.0, h * cy);
        let u = h * 0.06 * s;
        let a = tilt + (time * 0.7 + k as f32).sin() * 0.02;
        let (ca, sa) = (a.cos(), a.sin());
        let rot = |v: Vector2| Vector2::new(v.x * ca - v.y * sa, v.x * sa + v.y * ca);
        let unrot = |v: Vector2| Vector2::new(v.x * ca + v.y * sa, -v.x * sa + v.y * ca);
        let bumps = [(-1.15, 0.05, 0.48), (-0.5, -0.32, 0.62), (0.25, -0.45, 0.72), (0.95, -0.12, 0.55), (1.35, 0.1, 0.4)];
        for (dx, dy, r) in bumps {
            d.draw_circle_v(c + rot(Vector2::new(dx, dy) * u), r * u, PAPER);
        }
        // Base plana (girada con la nube) y la sombra celeste en la panza
        let deg = a.to_degrees();
        d.draw_rectangle_pro(Rectangle::new(c.x, c.y, 3.0 * u, 0.5 * u), Vector2::new(1.5 * u, 0.0), deg, PAPER);
        let shade = c + rot(Vector2::new(0.0, 0.18 * u));
        d.draw_rectangle_pro(Rectangle::new(shade.x, shade.y, 2.6 * u, 0.22 * u), Vector2::new(1.3 * u, 0.0), deg, Color::new(205, 232, 252, 255));
        // Contorno de crayón que "hierve": el borde de la nube, buscado con rayos desde el centro
        let inside = |p: Vector2| {
            let q = unrot(p - c) / u;                    // En las coordenadas de la nube sin girar
            bumps.iter().any(|(dx, dy, r)| (q - Vector2::new(*dx, *dy)).length() <= *r)
                || (q.x >= -1.5 && q.x <= 1.5 && q.y >= 0.0 && q.y <= 0.5)
        };
        let center = c + rot(Vector2::new(0.0, -0.05 * u));
        let edge: Vec<Vector2> = (0..40)
            .map(|i| {
                let a = i as f32 / 40.0 * 2.0 * PI;
                let dir = Vector2::new(a.cos(), a.sin());
                let mut r = 0.0;
                while r < 3.0 * u && inside(center + dir * (r + 2.0)) {
                    r += 2.0;
                }
                center + dir * r
            })
            .collect();
        for i in 0..edge.len() {
            crayon_line(d, edge[i], edge[(i + 1) % edge.len()], 4.0, INK, 200 + k as u32 * 50 + i as u32, time);
        }
    }
    // Estrellas de cinco puntas hechas con líneas
    // Sin orden: un par juntas abajo a la derecha y otro arriba, de distintos tamaños y girada cada una
    let star_cols = [ORANGE, PINK, PURPLE, GREEN, YELLOW];
    let stars = [(0.89, 0.81, 1.0, 0.3), (0.95, 0.74, 0.6, -0.35), (0.05, 0.5, 1.1, 0.45), (0.37, 0.06, 0.9, -0.25), (0.43, 0.1, 0.55, 0.6)];
    for (k, (sx, sy, size, tilt)) in stars.iter().enumerate() {
        let c = Vector2::new(w * sx, h * sy);
        let r = h * 0.035 * size;
        let pts: Vec<Vector2> = (0..5)
            .map(|i| {
                let a = -PI / 2.0 + tilt + i as f32 * 4.0 * PI / 5.0 + (time * 1.5 + k as f32).sin() * 0.08;
                c + Vector2::new(a.cos(), a.sin()) * r
            })
            .collect();
        for i in 0..5 {
            crayon_line(d, pts[i], pts[(i + 1) % 5], 4.0, star_cols[k], 60 + k as u32 * 5 + i as u32, time);
        }
    }
    // Caritas sonrientes
    // Caritas sonrientes inclinadas, cada una para su lado, que se balancean un poco
    for (k, (sx, sy, tilt)) in [(0.08, 0.84, -0.38), (0.93, 0.66, 0.3)].iter().enumerate() {
        let c = Vector2::new(w * sx, h * sy);
        let r = h * 0.05;
        let a = tilt + (time * 1.3 + k as f32 * 2.0).sin() * 0.08;
        let rot = |v: Vector2| Vector2::new(v.x * a.cos() - v.y * a.sin(), v.x * a.sin() + v.y * a.cos());
        crayon_smiley_disc(d, c, r, 300 + k as u32 * 100, time);
        d.draw_circle_v(c + rot(Vector2::new(-r * 0.35, -r * 0.2)), r * 0.1, INK);
        d.draw_circle_v(c + rot(Vector2::new(r * 0.35, -r * 0.2)), r * 0.1, INK);
        for i in 0..6 {
            let a0 = 0.2 * PI + i as f32 / 6.0 * 0.6 * PI;
            let a1 = 0.2 * PI + (i + 1) as f32 / 6.0 * 0.6 * PI;
            let (p0, p1) = (Vector2::new(a0.cos(), a0.sin()), Vector2::new(a1.cos(), a1.sin()));
            crayon_line(d, c + rot(p0 * r * 0.55), c + rot(p1 * r * 0.55), 3.0, INK, 90 + k as u32 * 9 + i, time);
        }
    }
    // Video con su marco de papel: sombra dura, papel, contorno de crayón y cinta en las esquinas
    let mut vw = w * 0.6;
    let mut vh = vw * video.height as f32 / video.width as f32;
    if vh > h * 0.62 {
        vh = h * 0.62;
        vw = vh * video.width as f32 / video.height as f32;
    }
    let v = Rectangle::new((w - vw) / 2.0, (h - vh) / 2.0 - h * 0.03, vw, vh);
    let pad = 22.0;
    let frame = Rectangle::new(v.x - pad, v.y - pad, v.width + 2.0 * pad, v.height + 2.0 * pad);
    d.draw_rectangle_rec(Rectangle::new(frame.x + 12.0, frame.y + 12.0, frame.width, frame.height), fade(INK, 0.55));
    d.draw_rectangle_rec(frame, PAPER);
    d.draw_texture_pro(video, Rectangle::new(0.0, 0.0, video.width as f32, video.height as f32), v, Vector2::zero(), 0.0, Color::WHITE);
    crayon_rect(d, frame, 7.0, INK, 1, time);
    crayon_rect(d, v, 4.0, INK, 7, time);
    let corners = [
        (frame.x, frame.y, -35.0),
        (frame.x + frame.width, frame.y, 35.0),
        (frame.x, frame.y + frame.height, 35.0),
        (frame.x + frame.width, frame.y + frame.height, -35.0),
    ];
    for (k, (cx, cy, ang)) in corners.iter().enumerate() {
        let tape = Rectangle::new(*cx, *cy, 90.0, 30.0);
        d.draw_rectangle_pro(tape, Vector2::new(45.0, 15.0), *ang + k as f32 * 3.0, Color::new(255, 238, 150, 200));
    }
    draw_skip(d, title, w, h, mouse);
}

/// Botón SKIP: tarjeta blanca con brillo, que se levanta al pasar el mouse.
pub fn draw_skip(d: &mut RaylibDrawHandle, title: &Font, w: f32, h: f32, mouse: Vector2) {
    let b = skip_button(w, h);
    let hover = b.check_collision_point_rec(mouse);
    let r = Rectangle::new(b.x, b.y - if hover { 6.0 } else { 0.0 }, b.width, b.height);
    crate::scenes::button_style::surface(d, r, hover, true);
    let label = "SKIP >>";
    let m = title.measure_text(label, 34.0, 1.6);
    d.draw_text_pro(
        title,
        label,
        Vector2::new(r.x + r.width / 2.0, r.y + r.height / 2.0),
        m * 0.5,
        0.0,
        34.0,
        1.6,
        Color::new(8, 24, 55, 255),
    );
}
