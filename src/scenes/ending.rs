//! Final temporizado; transforma copias visuales, nunca las órbitas ni el vuelo.
use super::{abduction::AbductionRenderer, token_collection::TokenPresentation};
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
    let home = AbductionRenderer::load_home(rl, thread, root, identity)?;
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
        let button = Rectangle::new(w as f32 * 0.5 - 170.0, h as f32 * 0.79, 340.0, 52.0);
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
            home.draw_home(&mut d, identity, t - HOME, w, h);
            let fade = 1.0 - smooth((t - HOME) / 1.5);
            d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, (fade * 255.0) as u8));
            if t > HOME + 4.0 && t < FIN {
                d.draw_rectangle(0, (h as f32 * 0.1) as i32, w, 54, Color::new(0, 0, 0, 130));
                centered(
                    &mut d,
                    font,
                    "De vuelta a casa",
                    w,
                    (h as f32 * 0.12) as i32,
                    34.0,
                    Color::WHITE,
                );
            }
            if t >= FIN {
                d.draw_rectangle(0, 0, w, h, Color::new(0, 0, 0, 170));
                centered(&mut d, font, "FIN", w, h / 3, 90.0, Color::WHITE);
                centered(
                    &mut d,
                    font,
                    "PLANET HOPPERS",
                    w,
                    h / 3 + 104,
                    30.0,
                    Color::SKYBLUE,
                );
                centered(
                    &mut d,
                    font,
                    "Gracias por jugar",
                    w,
                    h / 3 + 148,
                    25.0,
                    Color::WHITE,
                );
                centered(
                    &mut d,
                    font,
                    "Pablo  /  Cami  /  Norman",
                    w,
                    (h as f32 * 0.70) as i32,
                    23.0,
                    Color::WHITE,
                );
                super::button_style::surface(
                    &mut d,
                    button,
                    button.check_collision_point_rec(d_mouse()),
                    true,
                );
                centered(
                    &mut d,
                    font,
                    "Volver al menu principal  [Enter]",
                    w,
                    button.y as i32 + 14,
                    23.0,
                    Color::new(8, 24, 55, 255),
                );
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

fn d_mouse() -> Vector2 {
    unsafe { raylib::ffi::GetMousePosition().into() }
}
fn centered(
    d: &mut impl RaylibDraw,
    font: &Font,
    text: &str,
    w: i32,
    y: i32,
    size: f32,
    color: Color,
) {
    let width = font.measure_text(text, size, 1.0).x;
    d.draw_text_ex(
        font,
        text,
        Vector2::new((w as f32 - width) * 0.5, y as f32),
        size,
        1.0,
        color,
    );
}
