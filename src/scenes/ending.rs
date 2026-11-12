//! Final temporizado; transforma copias visuales, nunca las órbitas ni el vuelo.
use super::{holo, token_collection::TokenPresentation};
use crate::{
    gameplay::{progress::Franchise, selection::GameState},
    graphics::{
        bloom::Bloom,
        space::{
            draw_sun_glow, NebulaSky, Sky, SpaceModel, SpaceShader, SunShader, SKY_BACKGROUND,
        },
    },
    ships::Ship,
    solar_system::SolarSystem,
};
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const HOME: f32 = 27.0;
pub const FIN: f32 = 36.0;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EndingState {
    pub elapsed: f32,
    pub finished: bool,
}

pub fn ready(game: &GameState, presentation: &TokenPresentation) -> bool {
    game.progress().is_complete()
        && presentation.phase().is_none()
        && presentation.pending_count() == 0
}

pub fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Contracción acelerada y espiral creciente. En el extremo final desaparece.
pub fn absorb(start: Vector3, center: Vector3, progress: f32) -> (Vector3, f32) {
    let p = progress.clamp(0.0, 1.0);
    let v = start - center;
    let angle = p * p * 11.0;
    let radius = 1.0 - p * p;
    let pos = Vector3::new(
        v.x * angle.cos() - v.z * angle.sin(),
        v.y,
        v.x * angle.sin() + v.z * angle.cos(),
    );
    (center + pos * radius, 1.0 - smooth((p - 0.55) / 0.45))
}

pub fn protagonist(game: &GameState) -> Result<Franchise, String> {
    Franchise::from_id(game.first_ship())
}

/// true: botón de menú; false: cierre de ventana. Guarda el tiempo para reanudar sin repetir el evento.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    root: &Path,
    game: &GameState,
    presentation: &mut TokenPresentation,
    system: &SolarSystem,
    ship: &Ship,
    sun: &Model,
    sun_shader: &mut SunShader,
    planets: &[(Option<SpaceModel>, Option<SpaceModel>)],
    ship_model: Option<&SpaceModel>,
    shader: &mut SpaceShader,
    sky: &Sky,
    nebula: &mut NebulaSky,
    bloom: &mut Bloom,
    font: &Font,
    initial_camera: Camera3D,
    smoke: bool,
) -> Result<bool, Box<dyn std::error::Error>> {
    let identity = protagonist(game)?;
    let postcards = Postcards::load(rl, thread, root)?;
    let mut disk = rl.load_shader_from_memory(
        thread,
        None,
        Some(include_str!("../graphics/shaders/black_hole.fs")),
    );
    if !disk.is_shader_valid() {
        return Err("No se pudo compilar el disco de acreción".into());
    }
    let loc = ["center", "radius", "time", "strength"].map(|name| disk.get_shader_location(name));
    if loc.contains(&-1) {
        return Err("Uniformes del agujero negro ausentes".into());
    }
    rl.enable_cursor();
    if smoke {
        rl.set_target_fps(0);
    }
    let center = system.center();
    let outer = system
        .planets
        .iter()
        .map(|p| p.config.orbit_radius + p.config.radius)
        .fold(1.0_f32, f32::max);
    let mut checkpoint = -1;
    let mut final_frame = false;
    loop {
        if crate::graphics::window::should_close(rl) {
            if !smoke {
                crate::gameplay::save::store(
                    &crate::gameplay::save::path(root),
                    game,
                    presentation,
                )?;
            }
            return Ok(false);
        }
        presentation.ending.elapsed = if presentation.ending.finished {
            FIN
        } else {
            (presentation.ending.elapsed
                + if smoke {
                    0.1
                } else {
                    rl.get_frame_time().clamp(0.0, 0.1)
                })
            .min(FIN)
        };
        let t = presentation.ending.elapsed;
        if t >= FIN {
            presentation.ending.finished = true;
        }
        if !smoke && t as i32 != checkpoint {
            crate::gameplay::save::store(&crate::gameplay::save::path(root), game, presentation)?;
            checkpoint = t as i32;
        }
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        let button = Rectangle::new(w as f32 * 0.5 - 170.0, h as f32 * 0.84, 340.0, 52.0);
        if t >= FIN
            && (rl.is_key_pressed(KeyboardKey::KEY_ENTER)
                || (rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                    && button.check_collision_point_rec(rl.get_mouse_position())))
        {
            return Ok(true);
        }
        let wide = center
            + Vector3::new(outer * 1.1, outer * 0.8, outer * 1.5)
                * (1.0 - smooth((t - 7.0) / 11.0) * 0.45);
        let blend = smooth(t / 4.0);
        let mut camera = Camera3D::perspective(
            initial_camera.position + (wide - initial_camera.position) * blend,
            initial_camera.target + (center - initial_camera.target) * blend,
            Vector3::up(),
            48.0,
        );
        let (ship_pos, ship_scale) = absorb(ship.position, center, (t - 19.0) / 6.0);
        // Raccord hacia la nave antes de su caída, después retrocede con ella hacia el horizonte.
        let chase = smooth((t - 18.5) / 2.0) * (1.0 - smooth((t - 23.0) / 2.5));
        let length = ship_model.map_or(system.sun.radius * 0.08, |m| m.half_extents.length() * 2.0);
        let offset =
            (ship.position - center).normalized() * length * 2.5 + Vector3::up() * length * 0.8;
        camera.position += (ship_pos + offset - camera.position) * chase;
        camera.target += (ship_pos - camera.target) * chase;
        unsafe {
            raylib::ffi::rlSetClipPlanes(0.5, (outer * 10.0 + camera.position.length()) as f64);
        }
        let sky_frame = sky.project(rl, camera);
        let at = rl.get_world_to_screen(center, camera);
        let up = (camera.target - camera.position)
            .normalized()
            .cross(Vector3::up())
            .normalized();
        let radius =
            (rl.get_world_to_screen(center + up * system.sun.radius, camera) - at).length();
        let collapse = smooth((t - 4.0) / 3.0);
        shader.begin_frame(center, camera.position, 0.35);
        sun_shader.begin_frame(center, camera.position, t);
        bloom.prepare(rl, thread, w, h)?;
        let mut d = crate::graphics::window::begin_drawing(rl, thread);
        if t < HOME {
            {
                let mut scene = d.begin_texture_mode(thread, &mut bloom.scene);
                scene.clear_background(SKY_BACKGROUND);
                nebula.draw(&mut scene, camera, w, h);
                sky.draw(&mut scene, &sky_frame, t);
                if t < 7.0 {
                    draw_sun_glow(
                        &mut scene,
                        at,
                        radius * (1.0 + smooth(t / 4.0) * 1.8) * (1.0 - collapse * 0.9),
                        Vector2::new(w as f32, h as f32),
                    );
                }
                if t > 4.0 {
                    disk.set_shader_value(loc[0], Vector2::new(at.x, h as f32 - at.y));
                    disk.set_shader_value(loc[1], radius.max(1.0));
                    disk.set_shader_value(loc[2], t);
                    disk.set_shader_value(loc[3], collapse);
                    let mut layer = scene.begin_shader_mode(&mut disk);
                    layer.draw_rectangle(0, 0, w, h, Color::WHITE);
                }
                let mut d3 = scene.begin_mode3D(camera);
                if t < 7.0 {
                    d3.draw_model_ex(
                        sun,
                        center,
                        Vector3::up(),
                        t * 15.0,
                        Vector3::one() * system.sun.radius * (1.0 - collapse * 0.92),
                        Color::WHITE,
                    );
                }
                if t > 5.0 {
                    d3.draw_sphere(center, system.sun.radius * 0.65 * collapse, Color::BLACK);
                }
                for (i, (p, (near, far))) in system.planets.iter().zip(planets).enumerate() {
                    let progress = (t - 7.0 - i as f32 * 1.25) / 5.0;
                    let (pos, scale) = absorb(p.position(), center, progress);
                    if scale < 0.001 {
                        continue;
                    }
                    if progress > 0.0 {
                        for k in 0..12 {
                            let a = absorb(p.position(), center, progress - k as f32 * 0.006).0;
                            let b =
                                absorb(p.position(), center, progress - (k + 1) as f32 * 0.006).0;
                            d3.draw_line_3D(
                                a,
                                b,
                                Color::new(255, 162, 73, ((1.0 - k as f32 / 12.0) * 180.0) as u8),
                            );
                        }
                    }
                    let world = p.orientation()
                        * Matrix::scale(scale, scale, scale)
                        * Matrix::translate(pos.x, pos.y, pos.z);
                    shader.set_planet_fill(super::solar_system::planet_fill(&p.config.id));
                    shader.set_planet_toon(p.config.id == "smilingfriends");
                    shader.bloom_only =
                        (p.config.id == "smilingfriends").then_some(&["Satan", "FireRed"][..]);
                    if let Some(model) = far.as_ref().or(near.as_ref()) {
                        model.draw(&mut d3, shader, world);
                    } else {
                        d3.draw_sphere(
                            pos,
                            p.config.radius * scale,
                            super::solar_system::color(&p.config.color),
                        );
                    }
                }
                shader.set_planet_fill(Vector3::zero());
                shader.set_planet_toon(false);
                shader.bloom_only = None;
                if ship_scale > 0.001 {
                    let spin = smooth((t - 19.0) / 6.0) * t * 2.5;
                    let world = Matrix::scale(ship_scale, ship_scale, ship_scale)
                        * Matrix::rotate_z(ship.bank + spin)
                        * Matrix::rotate_x(ship.pitch + spin * 0.6)
                        * Matrix::rotate_y(ship.yaw + spin)
                        * Matrix::translate(ship_pos.x, ship_pos.y, ship_pos.z);
                    if let Some(model) = ship_model {
                        model.draw(&mut d3, shader, world);
                    } else {
                        d3.draw_sphere(
                            ship_pos,
                            system.sun.radius * 0.04 * ship_scale,
                            Color::WHITE,
                        );
                    }
                }
            }
            bloom.composite(&mut d, thread);
            let flash = smooth((t - 3.5) / 0.5) * (1.0 - smooth((t - 4.0) / 0.8));
            d.draw_rectangle(0, 0, w, h, Color::new(255, 242, 215, (flash * 220.0) as u8));
            d.draw_rectangle(
                0,
                0,
                w,
                h,
                Color::new(0, 0, 0, (smooth((t - 25.0) / 2.0) * 255.0) as u8),
            );
        } else {
            d.clear_background(Color::BLACK);
            postcards.draw(&mut d, font, t - HOME, w, h);
            let fade = 1.0 - smooth((t - HOME) / 1.2);
            d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, (fade * 255.0) as u8));
            // FIN: en la misma pantalla del collage (no otra encima): los créditos debajo y el botón para volver
            if t >= FIN {
                let credits = "PLANET HOPPERS   ·   Gracias por jugar   ·   Pablo  /  Cami  /  Norman";
                let size = (h as f32 * 0.026).clamp(16.0, 24.0);
                let cw = font.measure_text(credits, size, 1.0).x;
                d.draw_text_ex(font, credits, Vector2::new((w as f32 - cw) * 0.5, h as f32 * 0.795), size, 1.0, holo::CYAN);
                let hovered = button.check_collision_point_rec(d_mouse());
                holo::button(&mut d, font, font, button, "Enter", "Volver al menú principal", None, hovered);
            }
        }
        d.draw_rectangle(0, 0, w, (h as f32 * 0.065) as i32, Color::BLACK);
        d.draw_rectangle(
            0,
            (h as f32 * 0.935) as i32,
            w,
            (h as f32 * 0.065 + 1.0) as i32,
            Color::BLACK,
        );
        drop(d);
        if smoke && (t as i32 != checkpoint || (t >= FIN && final_frame)) {
            checkpoint = t as i32;
            if [3, 6, 12, 21, 26, 31].contains(&checkpoint) || (t >= FIN && final_frame) {
                let dir = root.join("target/ending");
                std::fs::create_dir_all(&dir)?;
                crate::graphics::resources::save_frame(
                    &dir.join(format!("{}-{checkpoint}.png", identity.as_str())),
                    w,
                    h,
                )?;
            }
            if t >= FIN && final_frame {
                println!(
                    "ENDING_SMOKE_PASS initial={} active={}",
                    game.first_ship(),
                    game.active_ship()
                );
                return Ok(true);
            }
        }
        final_frame = t >= FIN;
    }
}

/// Las seis postales del final: el agujero negro se llevó los planetas, pero ganaste y estás a salvo. Es un collage
/// tipo bento (tarjetas de distintos tamaños que encajan en una grilla de 4×3) con una foto de verdad de cada minijuego
/// (`assets/images/ending/<franquicia>.png`: el monstruo de Hexbound, la explosión de Smiling Friends, el muñeco de
/// nieve asesino...), en el estilo de la interfaz: vidrio oscuro, borde y esquinas encendidas del color de la
/// franquicia y su logo abajo. Caen de a una, asentándose; después aparece el mensaje de que ganaste.
struct Postcards {
    cards: Vec<(Texture2D, Texture2D, Color, [f32; 4], f32)>,
}

/// Dónde va cada franquicia en la grilla de 4 columnas × 3 filas: columna, fila, ancho y alto (en celdas), según
/// la forma de su foto: Smiling Friends, la grande; Hexbound (el render de la calavera, vertical), la alta; South
/// Park y Star Wars, las anchas; Regular Show y Minecraft, las chicas.
fn bento_cell(franchise: Franchise) -> [f32; 4] {
    match franchise {
        Franchise::SmilingFriends => [0.0, 0.0, 2.0, 2.0],
        Franchise::Halloween => [2.0, 0.0, 1.0, 2.0],
        Franchise::RegularShow => [3.0, 0.0, 1.0, 1.0],
        Franchise::Minecraft => [3.0, 1.0, 1.0, 1.0],
        Franchise::SouthPark => [0.0, 2.0, 2.0, 1.0],
        Franchise::StarWars => [2.0, 2.0, 2.0, 1.0],
    }
}

/// Qué altura de la foto queda en el centro al recortarla (0 arriba, 0,5 al medio): a Steve no se le corta la cabeza.
fn photo_focus(franchise: Franchise) -> f32 {
    match franchise {
        Franchise::Minecraft => 0.3,
        _ => 0.5,
    }
}

/// Cuándo cae cada tarjeta (segundos desde que empieza esta parte) y cuánto tarda en asentarse.
const CARD_FIRST: f32 = 1.0;
const CARD_STEP: f32 = 0.45;
const CARD_FALL: f32 = 0.6;

impl Postcards {
    fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cards = Vec::new();
        // Primero la grande y después las demás, en el orden en que caen
        for franchise in [Franchise::SmilingFriends, Franchise::Halloween, Franchise::RegularShow, Franchise::Minecraft,
                          Franchise::SouthPark, Franchise::StarWars] {
            let mut tex = |path: String| -> Result<Texture2D, Box<dyn std::error::Error>> {
                let t = rl.load_texture(thread, &root.join(path).to_string_lossy())?;
                t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
                Ok(t)
            };
            let photo = tex(format!("assets/images/ending/{}.png", franchise.as_str()))?;
            let logo = tex(format!("assets/ui/logos/{}.png", franchise.as_str()))?;
            cards.push((photo, logo, super::token_collection_ui::franchise_color(franchise), bento_cell(franchise),
                        photo_focus(franchise)));
        }
        Ok(Self { cards })
    }

    fn draw(&self, d: &mut RaylibDrawHandle, font: &Font, t: f32, width: i32, height: i32) {
        let (w, h) = (width as f32, height as f32);
        // Fondo: la oscuridad azul del sistema solar, con estrellas que titilan
        d.draw_rectangle_gradient_v(0, 0, width, height, Color::new(6, 14, 34, 255), Color::new(1, 4, 14, 255));
        let hash = |s: f32| ((s.sin() * 43_758.547).fract() + 1.0).fract();
        for i in 0..160 {
            let k = i as f32;
            let p = Vector2::new(hash(k * 12.9 + 1.0) * w, hash(k * 78.2 + 2.0) * h);
            let a = 0.25 + 0.55 * hash(k * 3.7) * (0.6 + 0.4 * (t * (1.0 + hash(k) * 2.0) + k).sin());
            d.draw_circle_v(p, 0.8 + 1.2 * hash(k * 5.1), holo::fade(Color::new(200, 225, 255, 255), a));
        }
        // La grilla: 4×3 celdas apaisadas, centrada debajo del mensaje
        let gap = (h * 0.014).max(8.0);
        let cell_h = (h * 0.5 - gap * 2.0) / 3.0;
        let cell_w = (cell_h * 1.55).min((w * 0.86 - gap * 3.0) / 4.0);
        let grid_w = cell_w * 4.0 + gap * 3.0;
        let origin = Vector2::new((w - grid_w) * 0.5, h * 0.26);
        for (i, (photo, logo, color, [c, r, cw, ch], focus)) in self.cards.iter().enumerate() {
            let k = (t - (CARD_FIRST + i as f32 * CARD_STEP)) / CARD_FALL;
            if k <= 0.0 {
                continue;
            }
            let rect = Rectangle::new(origin.x + c * (cell_w + gap), origin.y + r * (cell_h + gap),
                                      cw * cell_w + (cw - 1.0) * gap, ch * cell_h + (ch - 1.0) * gap);
            // Se asienta: empieza un poco más grande y más arriba, y baja a su lugar con un rebote
            let land = ease_out_back(k);
            let grow = 1.0 + (1.0 - land) * 0.12;
            let (rw, rh) = (rect.width * grow, rect.height * grow);
            let at = Rectangle::new(rect.x + rect.width * 0.5 - rw * 0.5, rect.y + rect.height * 0.5 - rh * 0.5 - (1.0 - land) * gap * 3.0, rw, rh);
            let tilt = [-1.2, 1.5, -1.8, 1.4, -1.0, 1.7][i] * (0.6 + 0.4 * land);
            draw_bento(d, photo, logo, *color, at, smooth(k * 2.5), t + i as f32, tilt, *focus);
        }
        // El mensaje, cuando ya cayeron todas: ganaste y estás bien
        let all = CARD_FIRST + 5.0 * CARD_STEP + CARD_FALL;
        let words = smooth((t - all) / 0.8);
        if words > 0.0 {
            let title = "¡Ganaste! Estás a salvo";
            let size = (h * 0.065).clamp(32.0, 62.0);
            let tw = font.measure_text(title, size, 1.0).x;
            holo::glow_text(d, font, title, Vector2::new((w - tw) * 0.5, h * 0.09), size,
                holo::fade(Color::WHITE, words), holo::CYAN);
            let sub = "Sobreviviste al agujero negro y salvaste los seis mundos";
            let ss = (h * 0.028).clamp(16.0, 26.0);
            let sw = font.measure_text(sub, ss, 1.0).x;
            d.draw_text_ex(font, sub, Vector2::new((w - sw) * 0.5, h * 0.09 + size * 1.15), ss, 1.0, holo::fade(holo::INK, words));
        }
    }
}

/// Un rebote al final (para que la tarjeta "se asiente").
fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    1.0 + t * t * (2.4 * t + 1.4)
}

/// Una tarjeta del bento, como una postal: papel crema con sombra, apenas torcida, la foto adentro con margen, su
/// logo sobre una franja oscura al pie de la foto, y una estampilla del color de la franquicia (borde dentado y una
/// estrellita) en la esquina de arriba.
#[allow(clippy::too_many_arguments)]
fn draw_bento(d: &mut RaylibDrawHandle, photo: &Texture2D, logo: &Texture2D, color: Color, rect: Rectangle,
              alpha: f32, t: f32, angle: f32, focus: f32) {
    let paper = Color::new(244, 238, 226, 255);
    let center = Vector2::new(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
    let (sin, cos) = angle.to_radians().sin_cos();
    let local = |x: f32, y: f32| center + Vector2::new(x * cos - y * sin, x * sin + y * cos);
    let card = |d: &mut RaylibDrawHandle, c: Vector2, w: f32, h: f32, col: Color| {
        d.draw_rectangle_pro(Rectangle::new(c.x, c.y, w, h), Vector2::new(w * 0.5, h * 0.5), angle, col);
    };
    let (w, h) = (rect.width, rect.height);
    // Brillo del color de la franquicia, sombra y papel
    let pulse = 0.85 + 0.15 * (t * 1.7).sin();
    d.draw_circle_gradient(center.x as i32, center.y as i32, w.max(h) * 0.7, holo::fade(color, 0.12 * alpha * pulse),
        holo::fade(color, 0.0));
    card(d, center + Vector2::new(5.0, 8.0), w, h, holo::fade(Color::BLACK, 0.5 * alpha));
    card(d, center, w, h, holo::fade(paper, alpha));
    // La foto, con margen de papel parejo
    let m = (w.min(h) * 0.055).clamp(6.0, 16.0);
    let (pw, ph) = (w - m * 2.0, h - m * 2.0);
    let mut src = super::abduction::cover_source([photo.width as f32, photo.height as f32], [pw, ph]);
    src.y = (photo.height as f32 - src.height) * focus;
    d.draw_texture_pro(photo, src, Rectangle::new(center.x, center.y, pw, ph), Vector2::new(pw * 0.5, ph * 0.5), angle,
        holo::fade(Color::WHITE, alpha));
    // La franja oscura al pie de la foto, con el logo
    let band = (ph * 0.24).clamp(30.0, 64.0);
    let bc = local(0.0, ph * 0.5 - band * 0.5);
    card(d, bc, pw, band, holo::fade(Color::new(4, 10, 24, 190), alpha));
    let scale = ((pw * 0.45) / logo.width as f32).min((band * 0.74) / logo.height as f32);
    let (lw, lh) = (logo.width as f32 * scale, logo.height as f32 * scale);
    let lc = local(-pw * 0.5 + band * 0.3 + lw * 0.5, ph * 0.5 - band * 0.5);
    d.draw_texture_pro(logo, Rectangle::new(0.0, 0.0, logo.width as f32, logo.height as f32),
        Rectangle::new(lc.x, lc.y, lw, lh), Vector2::new(lw * 0.5, lh * 0.5), angle, holo::fade(Color::WHITE, alpha));
    // La estampilla: papel dentado, el color de la franquicia adentro y una estrellita de cuatro puntas
    let st = (w.min(h) * 0.2).clamp(26.0, 54.0);
    let sc = local(w * 0.5 - m - st * 0.62, -h * 0.5 + m + st * 0.72);
    card(d, sc + Vector2::new(2.0, 3.0), st, st * 1.18, holo::fade(Color::BLACK, 0.35 * alpha));
    card(d, sc, st, st * 1.18, holo::fade(paper, alpha));
    let teeth = 6;
    for i in 0..=teeth {
        let f = i as f32 / teeth as f32 - 0.5;
        for (x, y) in [(f * st, -st * 0.59), (f * st, st * 0.59), (-st * 0.5, f * st * 1.18), (st * 0.5, f * st * 1.18)] {
            let p = sc + Vector2::new(x * cos - y * sin, x * sin + y * cos);
            d.draw_circle_v(p, st * 0.045, holo::fade(Color::new(4, 10, 24, 255), alpha));
        }
    }
    card(d, sc, st * 0.78, st * 0.96, holo::fade(color, alpha));
    let star = st * 0.22;
    for (dx, dy) in [(1.0, 0.0), (0.0, 1.0)] {
        let v = Vector2::new(dx * cos - dy * sin, dx * sin + dy * cos) * star;
        d.draw_line_ex(sc - v, sc + v, 2.5, holo::fade(Color::WHITE, alpha));
    }
    d.draw_circle_v(sc, st * 0.06, holo::fade(Color::WHITE, alpha));
}

fn d_mouse() -> Vector2 {
    unsafe { raylib::ffi::GetMousePosition().into() }
}
