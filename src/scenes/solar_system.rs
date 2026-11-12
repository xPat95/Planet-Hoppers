//! Sistema solar jugable (fases 1 a 6). Une la lógica de `solar_system` (órbitas, rotación, proximidad) y `ships`
//! (vuelo y choques) con la representación de `graphics::space`, y coordina las visitas: "Explorar planeta" cierra
//! esta ventana, abre el minijuego y al terminar vuelve al mismo sistema solar, con la nave junto al planeta.
use super::minigame_host::{self, MinigameVisits};
use crate::gameplay::planet_visit::PlanetReturnPose;
use crate::gameplay::station::{chase_position, DepartureBounds, Station};
use crate::minigames::common::host::{MinigameCompletion, MinigameExit, MinigameResult};
use crate::graphics::resources::save_frame;
use crate::graphics::space::{draw_sun_glow, Fit, NebulaSky, Sky, SpaceModel, SpaceShader, SunShader, SKY_BACKGROUND};
use crate::ships::{FlightInput, Ship, ShipsConfig};
use crate::gameplay::selection::{GameState, ShipCatalog, SelectionTransition};
use crate::solar_system::{hex_color, Planet, SolarSystem, SolarSystemConfig};
use raylib::{ffi, prelude::*};
use std::collections::BTreeMap;
use std::ffi::CString;
use std::path::{Path, PathBuf};

const HELP_SHIP: &str = "W/S: acelerar / frenar | Mouse o A/D: girar | Espacio: subir | Shift: turbo | Tab: objetivo | B: piloto automático\n\
                         E: explorar / entrar a estación | C: vista general | N: otra nave | Esc: soltar el mouse | Ctrl+X: salir";
const HELP_OVERVIEW: &str = "Arrastrar: girar cámara | Rueda: acercar | Tab: seguir objetivo | P: pausa | Arriba/Abajo: velocidad del tiempo\n\
                             C: volver a la nave | F12: captura | Ctrl+X: salir";

/// El plano cercano conserva el detalle de las naves; el lejano se calcula con el sistema.
const NEAR_PLANE: f64 = 0.5;
/// Distancia de la vista general y alejamiento máximo, en múltiplos de la órbita más lejana.
const OVERVIEW: f32 = 2.7;
const MAX_ZOOM_OUT: f32 = 3.5;
/// Más lejos que esto (en radios del planeta) se usa la versión liviana del modelo.
const FAR_LOD: f32 = 9.0;
/// Duración del fundido a negro al entrar o salir (segundos).
const FADE: f32 = 0.6;
const ENTRY_STATION_HOLD: f32 = 0.15;
const ENTRY_CAMERA_MOVE: f32 = 1.35;
const ORBIT_SEGMENTS: usize = 144;

fn help_button(width: i32, height: i32) -> Rectangle {
    Rectangle::new((width - 178) as f32, (height - 106) as f32, 160.0, 86.0)
}

fn draw_help_overlay(d: &mut RaylibDrawHandle, regular: &Font, bold: &Font, width: i32, height: i32, progress: f32, open: bool) {
    let button = help_button(width, height);
    let cyan = Color::new(137, 228, 255, 255);
    // El acento del estilo holográfico (antes naranja)
    let accent = super::holo::ACCENT;
    if progress < 0.01 {
        let hovered = button.check_collision_point_rec(d.get_mouse_position());
        super::holo::button(d, bold, regular, button, "H", "AYUDA", None, hovered);
        return;
    }
    let eased = progress * progress * (3.0 - 2.0 * progress);
    d.draw_rectangle(0, 0, width, height, Color::new(1, 6, 20, (eased * 202.0) as u8));
    let panel_w = (width - 36).min(820) as f32;
    let panel_h = (height - 38).min(580) as f32;
    let w = button.width + (panel_w - button.width) * eased;
    let h = button.height + (panel_h - button.height) * eased;
    let cx = button.x + button.width * 0.5 + (width as f32 * 0.5 - button.x - button.width * 0.5) * eased;
    let cy = button.y + button.height * 0.5 + (height as f32 * 0.5 - button.y - button.height * 0.5) * eased;
    let x = cx - w * 0.5;
    let y = cy - h * 0.5;
    // Panel de vidrio holográfico (el mismo estilo que el medallero y los controles del menú)
    let panel = Rectangle::new(x, y, w, h);
    super::holo::panel(d, panel, cyan, 1.0);
    if eased < 0.8 { return; }
    let scale = (panel_w / 820.0).min(panel_h / 580.0);
    let tx = |v: f32| (x + v * scale) as i32;
    let ty = |v: f32| (y + v * scale) as i32;
    let fs = |v: f32| v * scale;
    // La misma tarjeta que la pantalla de controles del menú
    super::holo::ControlsCard {
        title: "CONTROLES",
        subtitle: "Sistema solar / guía de vuelo",
        footer: open.then_some("H o Esc para cerrar"),
        columns: [
            ("PILOTAR LA NAVE", &[
                ("W / S", "Acelerar / frenar"), ("Mouse o A / D", "Girar"),
                ("Espacio", "Subir"), ("Shift", "Turbo"),
                ("Tab", "Cambiar objetivo"), ("B", "Piloto automático"),
                ("E", "Explorar / entrar al hangar"), ("C", "Vista general"),
                ("N", "Otra nave"), ("Esc", "Soltar el mouse"),
            ]),
            ("VISTA GENERAL", &[
                ("Arrastrar", "Girar cámara"), ("Rueda", "Acercar / alejar"),
                ("Tab", "Seguir objetivo"), ("P", "Pausa"),
                ("Arriba / Abajo", "Velocidad del tiempo"), ("C", "Volver a la nave"),
                ("O", "Mostrar / ocultar órbitas"), ("T", "Medallero de tokens"),
                ("F12", "Guardar captura"), ("Ctrl + X", "Salir del juego"),
            ]),
        ],
        width: 820.0,
        column_x: [72.0, 440.0],
        divider_x: 410.0,
    }.draw(d, regular, bold, Vector2::new(x, y), scale);
    if open {
        super::holo::glow_text(d, bold, "X", Vector2::new(tx(737.0) as f32, ty(34.0) as f32), fs(24.0), accent, accent);
    }
}

fn load_solar_font(path: &Path) -> Result<Font, Box<dyn std::error::Error>> {
    let path = CString::new(path.to_string_lossy().as_bytes())?;
    let mut glyphs: Vec<i32> = (32..=126).collect();
    glyphs.extend("áéíóúÁÉÍÓÚñÑüÜ¿¡·".chars().map(|c| c as i32));
    let raw = unsafe { ffi::LoadFontEx(path.as_ptr(), 64, glyphs.as_mut_ptr(), glyphs.len() as i32) };
    if raw.glyphs.is_null() || raw.texture.id == 0 {
        return Err("No se pudo cargar Advent Pro".into());
    }
    unsafe { ffi::SetTextureFilter(raw.texture, TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
    Ok(unsafe { Font::from_raw(raw) })
}

fn text_width(font: &Font, text: &str, size: f32) -> i32 {
    font.measure_text(text, size, 1.0).x.ceil() as i32
}

fn draw_ui(d: &mut impl RaylibDraw, font: &Font, text: &str, x: i32, y: i32, size: f32, color: Color) {
    d.draw_text_ex(font, text, Vector2::new(x as f32, y as f32), size, 1.0, color);
}

fn draw_planet_logo(d: &mut impl RaylibDraw, logo: &Texture2D, center_x: f32, top: f32) -> f32 {
    let max_height = if logo.width < logo.height { 34.0 } else { 29.0 };
    let scale = (88.0 / logo.width as f32).min(max_height / logo.height as f32);
    let width = logo.width as f32 * scale;
    let height = logo.height as f32 * scale;
    d.draw_texture_pro(
        logo,
        Rectangle::new(0.0, 0.0, logo.width as f32, logo.height as f32),
        Rectangle::new(center_x - width * 0.5, top, width, height),
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
    height
}

#[derive(Clone, Copy)]
struct DiskBounds { left: f32, right: f32, top: f32, bottom: f32 }

fn projected_disk_bounds(
    rl: &RaylibHandle,
    camera: Camera3D,
    planet: &Planet,
    model: &SpaceModel,
) -> Option<DiskBounds> {
    let half = model.half_extents;
    let position = planet.position();
    let world = planet.orientation() * Matrix::translate(position.x, position.y, position.z);
    let forward = (camera.target - camera.position).normalized();
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    for sample in 0..32 {
        let angle = sample as f32 * std::f32::consts::TAU / 32.0;
        let corner = Vector3::new(
            half.x * angle.cos(),
            0.0,
            half.z * angle.sin(),
        ).transform_with(world);
        if (corner - camera.position).dot(forward) <= 0.1 { continue; }
        let screen = rl.get_world_to_screen(corner, camera);
        min_x = min_x.min(screen.x);
        max_x = max_x.max(screen.x);
        min_y = min_y.min(screen.y);
        max_y = max_y.max(screen.y);
    }
    min_y.is_finite().then_some(DiskBounds { left: min_x, right: max_x, top: min_y, bottom: max_y })
}

#[derive(Clone, Copy, PartialEq)]
enum View {
    Ship,
    Overview,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Destination {
    Planet(usize),
    Station,
}
/// Qué hacer ahora (la línea de la misión, arriba a la izquierda) y, si se habla de un planeta, el color de su logo:
/// volar al objetivo (con la distancia), explorarlo si ya se llegó, o volver a la estación con todo ganado.
fn mission_step(s: &Session, in_reach: Option<usize>) -> (String, Option<Color>) {
    let all_won = s.game.as_ref().is_some_and(|g| g.progress().is_complete());
    match s.target {
        Some(Destination::Planet(i)) => {
            let p = &s.system.planets[i];
            let color = logo_color(&p.config.id);
            if in_reach == Some(i) {
                (format!("Presiona E para explorar {}", p.config.name), color)
            } else {
                let distance = ((p.position() - s.ship.position).length() - p.config.radius).max(0.0);
                (format!("Vuela a {}  ·  {distance:.0}", p.config.name), color)
            }
        }
        Some(Destination::Station) => {
            let distance = (s.station.exit_position() - s.ship.position).length();
            if all_won {
                (format!("¡Galaxia salvada! Vuelve a la estación  ·  {distance:.0}"), None)
            } else {
                (format!("Vuela a la estación  ·  {distance:.0}"), None)
            }
        }
        None => ("Elige un destino con Tab".to_owned(), None),
    }
}

/// La letra y el color de la distancia de un planeta con logo: el color del logo (la del objetivo, entera y en
/// negrita; las demás, un poco transparentes). Sin color propio, `fallback`.
fn logo_tint<'f>(id: &str, target: bool, fallback: Color, regular: &'f Font, bold: &'f Font) -> (&'f Font, Color) {
    let c = logo_color(id).unwrap_or(fallback);
    if target { (bold, c) } else { (regular, Color::new(c.r, c.g, c.b, 185)) }
}

/// El color de la distancia de cada planeta, el de su logo, para que la etiqueta y el logo vayan juntos.
fn logo_color(id: &str) -> Option<Color> {
    crate::gameplay::progress::Franchise::from_id(id).ok().map(super::token_collection_ui::franchise_color)
}

/// El planeta del video de entrega (`--demo-video`): Smiling Friends.
const DEMO_PLANET: usize = 2;

/// El piloto del video de entrega: vuela por el espacio hasta el planeta (con turbo), se acerca frenando, le da una
/// vuelta completa alrededor y se aleja. Arma la misma entrada que el teclado (`FlightInput`), así la nave vuela con
/// la física de siempre.
#[derive(Default)]
struct DemoPilot {
    phase: u8,
    time: f32,
    phase_time: f32,
    /// En la vuelta: cuánto giró alrededor del planeta y el ángulo anterior.
    turned: f32,
    last_angle: Option<f32>,
}

impl DemoPilot {
    fn input(&mut self, ship: &Ship, system: &SolarSystem, flight: &crate::ships::FlightConfig, dt: f32) -> FlightInput {
        self.time += dt;
        self.phase_time += dt;
        let p = &system.planets[DEMO_PLANET];
        let (c, r) = (p.position(), p.config.radius);
        let rel = ship.position - c;
        let mut input = FlightInput { ascend: false, throttle: 1.0, boost: false, turn: (0.0, 0.0), mouse: (0.0, 0.0), aim_at: None };
        let next = |me: &mut Self| { me.phase += 1; me.phase_time = 0.0; };
        match self.phase {
            // Viaje: con turbo hasta acercarse, y después frenando hasta quedar cerca
            0 => {
                input.aim_at = Some(ship.autopilot_aim(DEMO_PLANET, flight, system));
                let dist = rel.length() - r;
                input.boost = dist > r * 5.0;
                let wanted = (flight.cruise_speed * (dist / (r * 3.0)).clamp(0.35, 1.0)).max(flight.cruise_speed * 0.35);
                input.throttle = if ship.speed > wanted * 1.05 { -1.0 } else { 1.0 };
                if dist < r * 0.8 || self.phase_time > 25.0 { next(self); }
            }
            // La vuelta: apunta un poco adelante sobre un círculo a 2,2 radios del centro, a velocidad tranquila
            1 => {
                let flat = Vector3::new(rel.x, 0.0, rel.z).normalized();
                let angle = flat.z.atan2(flat.x);
                if let Some(last) = self.last_angle {
                    let mut d = angle - last;
                    if d > std::f32::consts::PI { d -= std::f32::consts::TAU; }
                    if d < -std::f32::consts::PI { d += std::f32::consts::TAU; }
                    self.turned += d.abs();
                }
                self.last_angle = Some(angle);
                let ahead = angle + 0.6;
                let height = rel.y.clamp(-r * 0.2, r * 0.2);
                input.aim_at = Some(c + Vector3::new(ahead.cos(), 0.0, ahead.sin()) * (r * 1.65) + Vector3::up() * height);
                let wanted = flight.cruise_speed * 1.1;
                input.throttle = if ship.speed > wanted { -0.4 } else { 0.6 };
                if self.turned >= std::f32::consts::TAU || self.phase_time > 30.0 { next(self); }
            }
            // Se aleja, con turbo, hacia afuera del planeta
            2 => {
                input.aim_at = Some(ship.position + rel.normalized() * (r * 50.0) + Vector3::up() * r * 4.0);
                input.boost = self.phase_time > 1.0;
                if self.phase_time > 6.0 { next(self); }
            }
            _ => input.throttle = 0.0,
        }
        input
    }

    fn done(&self) -> bool {
        self.phase >= 3
    }

    /// En la vuelta, cuánto mira la cámara hacia el planeta (entra y sale suave).
    fn look(&self) -> f32 {
        let smooth = |t: f32| { let t = t.clamp(0.0, 1.0); t * t * (3.0 - 2.0 * t) };
        match self.phase {
            1 => smooth(self.phase_time / 1.5) * (1.0 - smooth((self.turned / std::f32::consts::TAU - 0.88) / 0.12)),
            _ => 0.0,
        }
    }

    /// El contador de F4 se ve en el acercamiento y la vuelta (así se ve cambiar el nivel de detalle).
    fn show_stats(&self) -> bool {
        self.phase == 1
    }
}

/// Lo que usa el renderizador por software (F7): el renderizador y la geometría de la escena en la CPU, con dos
/// niveles de detalle por planeta generados al cargar (`SoftMesh::decimate`) y el sol generado a mano.
struct SoftScene {
    renderer: crate::graphics::software::SoftRenderer,
    planets: Vec<(crate::graphics::software::SoftMesh, crate::graphics::software::SoftMesh)>,
    ship: crate::graphics::software::SoftMesh,
    station: crate::graphics::software::SoftMesh,
    sun: crate::graphics::software::SoftMesh,
}

/// Alto de la imagen del renderizador por software (el ancho sale de la ventana) y cuántos triángulos puede tener
/// cada versión de un planeta.
const SOFT_HEIGHT: i32 = 420;
const SOFT_NEAR_TRIS: usize = 40_000;
const SOFT_FAR_TRIS: usize = 5_000;

impl SoftScene {
    fn load(rl: &mut RaylibHandle, thread: &RaylibThread, planets: &[(Option<SpaceModel>, Option<SpaceModel>)],
            ship: Option<&SpaceModel>, station: &SpaceModel) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::graphics::software::{SoftMesh, SoftRenderer};
        let planets = planets.iter().map(|(near, _)| {
            let full = near.as_ref().map(SpaceModel::cpu_mesh).unwrap_or_else(|| SoftMesh::sphere(3, Vector3::new(0.6, 0.6, 0.65)));
            let near = full.decimate(SOFT_NEAR_TRIS);
            let far = near.decimate(SOFT_FAR_TRIS);
            (near, far)
        }).collect();
        let (w, h) = (rl.get_screen_width().max(1), rl.get_screen_height().max(1));
        Ok(Self {
            renderer: SoftRenderer::new(rl, thread, SOFT_HEIGHT * w / h, SOFT_HEIGHT)?,
            planets,
            ship: ship.map(|m| m.cpu_mesh().decimate(15_000)).unwrap_or_default(),
            station: station.cpu_mesh().decimate(6_000),
            sun: SoftMesh::sphere(4, Vector3::one()),
        })
    }

    /// Arma la lista de lo que se dibuja (con descarte y nivel de detalle, como con la GPU) y renderiza.
    #[allow(clippy::too_many_arguments)]
    fn render(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, s: &Session, camera: Camera3D, center: Vector3,
              time: f32, culling: bool, lod_on: bool, stats: &mut RenderStats) -> Result<(), Box<dyn std::error::Error>> {
        use crate::graphics::software::{Draw, Shade, SoftRenderer};
        let (w, h) = (rl.get_screen_width().max(1), rl.get_screen_height().max(1));
        if self.renderer.width != SOFT_HEIGHT * w / h {
            self.renderer = SoftRenderer::new(rl, thread, SOFT_HEIGHT * w / h, SOFT_HEIGHT)?;
        }
        let aspect = w as f32 / h as f32;
        let mut draws = Vec::new();
        let r = s.system.sun.radius;
        draws.push(Draw { mesh: &self.sun, world: Matrix::scale(r, r, r) * Matrix::rotate_y(s.system.sun_spin_angle)
            * Matrix::translate(center.x, center.y, center.z), shade: Shade::Sun { center } });
        for (i, (p, (near, far))) in s.system.planets.iter().zip(&self.planets).enumerate() {
            let pos = p.position();
            if culling && !in_view(camera, aspect, pos, p.config.radius * 1.6) {
                stats.planets[i] = PlanetRender::Culled(near.tris.len());
                continue;
            }
            let use_far = lod_on && (pos - camera.position).length() > p.config.radius * FAR_LOD;
            let mesh = if use_far { far } else { near };
            stats.planets[i] = if use_far { PlanetRender::Far(far.tris.len(), near.tris.len()) } else { PlanetRender::Near(near.tris.len()) };
            draws.push(Draw { mesh, world: p.orientation() * Matrix::translate(pos.x, pos.y, pos.z), shade: Shade::Lit });
        }
        let station = s.station.center();
        if !culling || in_view(camera, aspect, station, s.station.collision_radius) {
            draws.push(Draw { mesh: &self.station, world: s.station.rotation() * Matrix::translate(station.x, station.y, station.z), shade: Shade::Lit });
        }
        let sp = &s.ship;
        draws.push(Draw { mesh: &self.ship, world: Matrix::rotate_z(sp.bank) * Matrix::rotate_x(sp.pitch) * Matrix::rotate_y(sp.yaw)
            * Matrix::translate(sp.position.x, sp.position.y, sp.position.z), shade: Shade::Lit });
        self.renderer.render(camera, center, &draws, time);
        Ok(())
    }
}

/// Qué pasó con cada planeta este cuadro: descartado (con los triángulos que se ahorró), dibujado entero, o dibujado
/// con su versión liviana (los que dibujó y los que tendría la completa).
#[derive(Clone, Copy)]
enum PlanetRender {
    Culled(usize),
    Near(usize),
    Far(usize, usize),
}

/// Lo que hizo el renderizador este cuadro con los planetas (para el contador de F4).
struct RenderStats {
    planets: Vec<PlanetRender>,
}

/// Si una esfera (centro y radio) entra en la pirámide de visión de la cámara: se compara su posición, en los ejes de
/// la cámara, con los bordes de la pantalla (con el radio de margen).
fn in_view(camera: Camera3D, aspect: f32, center: Vector3, radius: f32) -> bool {
    let forward = (camera.target - camera.position).normalized();
    let right = forward.cross(camera.up).normalized();
    let up = right.cross(forward);
    let to = center - camera.position;
    let z = to.dot(forward);
    if z < -radius {
        return false;
    }
    let half_y = (camera.fovy.to_radians() * 0.5).tan();
    let half_x = half_y * aspect;
    // El radio, medido de costado a la pirámide (un poco más grande cuanto más abierta)
    let (ry, rx) = (radius * (1.0 + half_y * half_y).sqrt(), radius * (1.0 + half_x * half_x).sqrt());
    to.dot(right).abs() <= z * half_x + rx && to.dot(up).abs() <= z * half_y + ry
}

/// El contador de F4: triángulos dibujados y ahorrados por el descarte y el nivel de detalle, y qué versión de cada
/// planeta se dibujó. F5 y F6 apagan el descarte y el nivel de detalle para comparar.
#[allow(clippy::too_many_arguments)]
fn draw_render_stats(d: &mut RaylibDrawHandle, regular: &Font, bold: &Font, stats: &RenderStats, s: &Session,
                     culling: bool, lod_on: bool, width: i32) {
    let (mut drawn, mut culled, mut saved) = (0usize, 0usize, 0usize);
    for p in &stats.planets {
        match *p {
            PlanetRender::Culled(t) => culled += t,
            PlanetRender::Near(t) => drawn += t,
            PlanetRender::Far(t, full) => { drawn += t; saved += full - t; }
        }
    }
    let rows = stats.planets.len() as f32;
    let rect = Rectangle::new(width as f32 - 400.0, 44.0, 380.0, 170.0 + rows * 24.0);
    super::holo::panel(d, rect, super::holo::CYAN, 1.0);
    let thousands = |n: usize| {
        let s = n.to_string();
        let mut out = String::new();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 { out.push('.'); }
            out.push(c);
        }
        out
    };
    let (x, mut y) = (rect.x + 18.0, rect.y + 12.0);
    let line = |d: &mut RaylibDrawHandle, font: &Font, text: &str, y: f32, color: Color| draw_ui(d, font, text, x as i32, y as i32, 19.0, color);
    line(d, bold, "RENDIMIENTO (F4)", y, super::holo::CYAN);
    y += 28.0;
    line(d, regular, &format!("Triángulos dibujados: {}", thousands(drawn)), y, Color::WHITE);
    y += 24.0;
    line(d, regular, &format!("Ahorrados por descarte: {}", thousands(culled)), y, super::holo::INK);
    y += 24.0;
    line(d, regular, &format!("Ahorrados por nivel de detalle: {}", thousands(saved)), y, super::holo::INK);
    y += 30.0;
    for (p, r) in s.system.planets.iter().zip(&stats.planets) {
        let (state, n) = match *r {
            PlanetRender::Culled(t) => ("fuera de vista", t),
            PlanetRender::Near(t) => ("completo", t),
            PlanetRender::Far(t, _) => ("liviano", t),
        };
        let color = logo_color(&p.config.id).unwrap_or(Color::WHITE);
        line(d, regular, &format!("{}: {state} ({})", p.config.name, thousands(n)), y, color);
        y += 24.0;
    }
    y += 6.0;
    let flag = |on: bool| if on { "sí" } else { "NO" };
    line(d, regular, &format!("F5 descarte: {}   F6 detalle: {}", flag(culling), flag(lod_on)), y, super::holo::INK);
}

/// El objetivo de la navegación: el primer planeta, desde `from` (incluido) en el orden del sistema, cuyo minijuego
/// todavía no se ganó; con todos ganados, la estación. Así, al ganar un planeta el objetivo pasa al siguiente que falta,
/// y si se perdió o se abandonó, se queda en el mismo para reintentar.
fn next_objective(system: &SolarSystem, game: Option<&GameState>, from: usize) -> Destination {
    let won = |i: usize| {
        let id = system.planets[i].config.minigame.as_deref();
        match (game, id.map(crate::gameplay::progress::Franchise::from_id)) {
            (Some(g), Some(Ok(f))) => g.progress().is_completed(f),
            _ => false,
        }
    };
    let n = system.planets.len();
    (0..n).map(|k| (from + k) % n).find(|&i| !won(i)).map_or(Destination::Station, Destination::Planet)
}

impl Destination {
    fn next(current: Option<Self>, count: usize) -> Option<Self> {
        match current {
            None => Some(Self::Planet(0)),
            Some(Self::Planet(i)) if i + 1 < count => Some(Self::Planet(i + 1)),
            Some(Self::Planet(_)) => Some(Self::Station),
            Some(Self::Station) => None,
        }
    }
    fn position(self, s: &Session) -> Vector3 {
        match self { Self::Planet(i) => s.system.planets[i].position(), Self::Station => s.station.center() }
    }
    fn radius(self, s: &Session) -> f32 {
        match self { Self::Planet(i) => s.system.planets[i].config.radius, Self::Station => s.station.collision_radius }
    }
    fn name(self, s: &Session) -> &str {
        match self { Self::Planet(i) => &s.system.planets[i].config.name, Self::Station => &s.station.name }
    }
}

/// Cámara de la vista general: gira alrededor del Sol o del planeta que se sigue.
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl OrbitCamera {
    fn camera(&self, target: Vector3, aspect: f32) -> Camera3D {
        let offset = Vector3::new(
            self.pitch.cos() * self.yaw.cos(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.sin(),
        ) * (self.distance / aspect.clamp(0.1, 1.0));
        Camera3D::perspective(target + offset, target, Vector3::up(), 42.0)
    }
}

/// Lo que se conserva entre ventanas: al volver de un minijuego todo sigue donde estaba.
struct Session {
    token_presentation: super::token_collection::TokenPresentation,
    system: SolarSystem,
    station: Station,
    ships: ShipsConfig,
    ship_index: usize,
    ship: Ship,
    view: View,
    orbit: OrbitCamera,
    focus: Option<Destination>,
    paused: bool,
    time_scale: f32,
    target: Option<Destination>,
    minigames: MinigameVisits,
    last_completion: Option<MinigameCompletion>,
    return_smoke: bool,
    launch_smoke: bool,
    intro_smoke: bool,
    help_smoke: bool,
    /// `--demo-video`: vuelo automático para el video de entrega (ver `DemoPilot`), guardando cada cuadro.
    demo_video: bool,
    /// `--software`: empieza con el renderizador por software (F7) encendido.
    software: bool,
    station_smoke: bool,
    ending_smoke: bool,
    collision_debug: bool,
    docking_smoke: Option<u32>,
    departing: bool,
    intro_pending: bool,
    game: Option<GameState>,
    returning_from: Option<usize>,
    return_pose: Option<PlanetReturnPose>,
    minecraft_smoke: bool,
    starwars_smoke: bool,
    regularshow_smoke: bool,
    /// Prueba de ida y vuelta a un minijuego (--smoke-visit): cuántas visitas van.
    visit_smoke: Option<u32>,
}

/// Planeta visto desde la cámara en este cuadro.
struct Marker {
    at: Vector2,
    radius: Option<f32>,
    angle: f32,
}

pub(super) fn planet_fill(id: &str) -> Vector3 {
    match id {
        "regularshow" => Vector3::new(0.30, 0.34, 0.34),
        "southpark" => Vector3::new(0.22, 0.24, 0.26),
        "minecraft" => Vector3::new(0.18, 0.20, 0.21),
        "smilingfriends" => Vector3::new(0.16, 0.18, 0.20),
        "starwars" => Vector3::new(0.09, 0.10, 0.12),
        "halloween" => Vector3::new(0.08, 0.035, 0.025),
        _ => Vector3::zero(),
    }
}

fn planet_glow_color(id: &str) -> Color {
    match id {
        "starwars" => Color::new(150, 170, 255, 30),
        "smilingfriends" => Color::new(105, 205, 255, 13),
        "minecraft" => Color::new(110, 225, 180, 27),
        "regularshow" => Color::new(148, 130, 245, 29),
        "southpark" => Color::new(155, 205, 255, 27),
        _ => Color::new(255, 100, 130, 28),
    }
}

fn draw_planet_glow(
    d: &mut impl RaylibDraw,
    id: &str,
    marker: &Marker,
    disk: Option<DiskBounds>,
    viewport: Vector2,
) {
    let Some(radius) = marker.radius else { return };
    let tint = planet_glow_color(id);
    let outer = Color::new(tint.r, tint.g, tint.b, 0);
    let mut glow = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
    if let Some(disk) = disk {
        let center = Vector2::new((disk.left + disk.right) * 0.5, (disk.top + disk.bottom) * 0.5);
        let rx = (disk.right - disk.left) * 0.5;
        let ry = (disk.bottom - disk.top) * 0.5;
        if rx < 2.0 || ry < 2.0 || rx > viewport.x * 1.5 || ry > viewport.y * 1.5 { return; }
        let blur = (rx.max(ry) * 0.15).clamp(11.0, 68.0);
        for i in 0..32 {
            let angle = i as f32 * std::f32::consts::TAU / 32.0;
            let x = center.x + rx * angle.cos();
            let y = center.y + ry * angle.sin();
            if x + blur < 0.0 || y + blur < 0.0 || x - blur > viewport.x || y - blur > viewport.y { continue; }
            glow.draw_circle_gradient(x as i32, y as i32, blur, tint, outer);
        }
    } else {
        if radius > viewport.x.max(viewport.y) * 1.2 { return; }
        let extent = (radius * 1.75 + 10.0).max(14.0);
        if marker.at.x + extent < 0.0 || marker.at.y + extent < 0.0
            || marker.at.x - extent > viewport.x || marker.at.y - extent > viewport.y { return; }
        glow.draw_circle_gradient(marker.at.x as i32, marker.at.y as i32, extent, tint, outer);
    }
}

/// Project a fixed POI, falling back to a direction marker inside the HUD margins.
fn rl_screen_marker(point: Vector3, camera: Camera3D, width: i32, height: i32) -> (Vector2, bool) {
    let forward = (camera.target - camera.position).normalized();
    let right = forward.cross(camera.up).normalized();
    let up = right.cross(forward);
    let rel = point - camera.position;
    let depth = rel.dot(forward);
    let scale = height as f32 * 0.5 / (camera.fovy.to_radians() * 0.5).tan();
    let center = Vector2::new(width as f32 * 0.5, height as f32 * 0.5);
    let direction = Vector2::new(rel.dot(right), -rel.dot(up));
    let projected = center + direction * (scale / depth.max(0.001));
    if depth > 0.0 && projected.x > 30.0 && projected.x < width as f32 - 30.0
        && projected.y > 100.0 && projected.y < height as f32 - 100.0 {
        (projected, true)
    } else {
        let dir = if direction.length() > 0.001 { direction.normalized() } else { Vector2::new(0.0, 1.0) };
        (center + dir * (width.min(height) as f32 * 0.35), false)
    }
}

enum Exit {
    Quit,
    Menu,
    Visit(usize),
    Dock,
}

pub(super) fn color(hex: &str) -> Color {
    let [r, g, b] = hex_color(hex).unwrap_or([255, 0, 255]);
    Color::new(r, g, b, 255)
}

fn outer_orbit(system: &SolarSystem) -> f32 {
    system.planets.iter().map(|p| p.config.orbit_radius + p.config.radius).fold(0.0, f32::max)
}

/// Deja la nave junto al planeta, del lado del Sol (el lado iluminado), un poco por encima de la órbita.
/// `facing_planet`: mirando al planeta (para llegar) o hacia afuera (al volver de un minijuego).
fn park_ship(ship: &mut Ship, planet: &Planet, radii: f32, facing_planet: bool) {
    let center = planet.position();
    let to_sun = (-center).normalized();
    let side = to_sun.cross(Vector3::up()).normalized();
    let dir = (to_sun + Vector3::up() * 0.35 + side * 0.3).normalized();
    ship.position = center + dir * planet.config.radius * radii;
    ship.speed = 0.0;
    ship.bank = 0.0;
    if facing_planet {
        ship.look_at(center);
    } else {
        ship.look_at(ship.position + dir);
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    run_session(std::env::args().skip(1), None).map(|_| ())
}

pub fn run_with_game(game: GameState, args: impl Iterator<Item = String>) -> Result<GameState, Box<dyn std::error::Error>> {
    run_with_progress(game, super::token_collection::TokenPresentation::default(), args)
}

pub fn run_with_progress(game: GameState, presentation: super::token_collection::TokenPresentation,
    args: impl Iterator<Item = String>) -> Result<GameState, Box<dyn std::error::Error>> {
    run_with_progress_and_menu(game, presentation, args).map(|(game, _)| game)
}

pub(super) fn run_with_progress_and_menu(game: GameState, presentation: super::token_collection::TokenPresentation,
    args: impl Iterator<Item = String>) -> Result<(GameState, bool), Box<dyn std::error::Error>> {
    let mut menu = false;
    let game = run_session_with_progress(args, Some(game), presentation, &mut menu)?
        .ok_or("Se perdió el estado de la aventura")?;
    Ok((game, menu))
}

fn run_session(args: impl Iterator<Item = String>, game: Option<GameState>) -> Result<Option<GameState>, Box<dyn std::error::Error>> {
    run_session_with_progress(args, game, super::token_collection::TokenPresentation::default(), &mut false)
}

fn run_session_with_progress(args: impl Iterator<Item = String>, game: Option<GameState>,
    mut presentation: super::token_collection::TokenPresentation, menu: &mut bool) -> Result<Option<GameState>, Box<dyn std::error::Error>> {
    let mut game = game;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = root.join("config/solar-system.json");
    let mut ships_path = root.join("config/ships.json");
    let mut smoke = false;
    let mut visit_smoke = false;
    let mut minecraft_smoke = false;
    let mut starwars_smoke = false;
    let mut regularshow_smoke = false;
    let mut return_smoke = false;
    let mut launch_smoke = false;
    let mut intro_smoke = false;
    let mut help_smoke = false;
    let mut demo_video = false;
    let mut software = false;
    let mut station_smoke = false;
    let mut ending_origin = None;
    let mut collision_debug = false;
    let mut docking_smoke = false;
    let mut overview = false;
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--smoke-ending" => { smoke = true; ending_origin = Some(args.next().ok_or("--smoke-ending necesita la franquicia inicial")?); }
            "--smoke-launch" => { smoke = true; launch_smoke = true; }
            "--smoke-intro" => { smoke = true; intro_smoke = true; }
            "--smoke-help" => { smoke = true; help_smoke = true; overview = true; }
            "--demo-video" => { smoke = true; demo_video = true; }
            "--software" => software = true,
            "--smoke-station" => { smoke = true; station_smoke = true; }
            "--debug-collisions" => collision_debug = true,
            "--smoke-docking" => { smoke = true; docking_smoke = true; }
            "--smoke-returns" => { smoke = true; visit_smoke = true; return_smoke = true; }
            "--smoke-regularshow" => { smoke=true; visit_smoke=true; regularshow_smoke=true; }
            "--smoke-starwars" => { smoke=true;visit_smoke=true;starwars_smoke=true; }
            "--smoke-minecraft" => { smoke = true; visit_smoke = true; minecraft_smoke = true; }
            "--smoke-visit" => {
                smoke = true;
                visit_smoke = true;
            }
            "--overview" => overview = true,
            "--config" => config_path = PathBuf::from(args.next().ok_or("--config necesita la ruta de un JSON")?),
            "--ships" => ships_path = PathBuf::from(args.next().ok_or("--ships necesita la ruta de un JSON")?),
            "--help" | "-h" => {
                println!("--debug-collisions: F3 toggles collision surfaces and diagnostics during flight");
                println!("cargo run --bin solarsystem -- [--overview] [--config PATH] [--ships PATH] [--smoke]");
                println!("Nave:\n{HELP_SHIP}\nVista general:\n{HELP_OVERVIEW}");
                println!("--smoke: recorrido automático sin ventana visible; guarda capturas en target/solarsystem/");
                println!("--smoke-ending FRANQUICIA: final completo sin modificar la partida; capturas en target/ending/");
                println!("--smoke-visit: ida y vuelta a South Park (con su prueba de entrada) y captura al volver");
                println!("--smoke-regularshow: interacción E, tres niveles, videos, derrota, abandono y regreso seguro");
                println!("--smoke-starwars: visita al túnel con la nave activa y regreso al sistema solar");
                println!("--smoke-minecraft: dos visitas a la aldea y Diamond Rush, regreso confirmado y misma nave");
                println!("--smoke-returns: victoria, derrota, pausa/reinicio/abandono y cuatro visitas a South Park");
                println!("--smoke-station: estación fija, aproximación y controles de ascenso/piloto");
                println!("--smoke-intro: capturas de la entrada y transición de cámara");
                println!("--smoke-docking: tres entradas al hangar, selección, cambio, cancelación y despegue");
                return Ok(game);
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let config = SolarSystemConfig::load(&config_path)?;
    let mut ships = ShipsConfig::load(&ships_path)?;
    if let Some(origin) = &ending_origin {
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships)?;
        let mut fixture = GameState::new_game(origin, &catalog)?;
        crate::gameplay::rewards::complete_for_presentation(&mut fixture, &catalog)?;
        fixture.activate(if origin == "minecraft" { "southpark" } else { "minecraft" })?;
        presentation.finish_for_presentation();
        game = Some(fixture);
    }
    if docking_smoke && game.is_none() {
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships)?;
        game = Some(GameState::new_game("minecraft", &catalog)?);
    }
    if docking_smoke {
        // Test fixture: two owned ships for selection/cancellation checks. No fake win/token.
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships)?;
        let game = game.as_mut().unwrap();
        let owned_id = if game.active_ship() == "starwars" { "southpark" } else { "starwars" };
        game.unlock(owned_id, &catalog)?;
    }
    if let Some(game) = &game {
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships)?;
        catalog.configure_active(game, &mut ships)?;
    }
    let system = SolarSystem::new(&config);
    if !smoke {
        if let Some(game) = &game {
            presentation.enqueue_initial(crate::gameplay::progress::Franchise::from_id(game.first_ship())?);
            crate::gameplay::save::store(&crate::gameplay::save::path(&root), game, &presentation)?;
        }
    }
    let outer = outer_orbit(&system);
    let station = Station::load(&root)?;
    // The actual departure is resolved after loading the fitted visual bounds.
    let mut ship = Ship::new(station.dock_target(), 0.0);
    ship.look_at(ship.position + station.exit_forward());
    let objective = next_objective(&system, game.as_ref(), 0);
    let mut session = Session {
        station,
        ship_index: ships.ships.iter().position(|s| s.id == ships.active_ship).unwrap_or(0),
        ships,
        system,
        ship,
        view: if overview { View::Overview } else { View::Ship },
        orbit: OrbitCamera { yaw: 0.6, pitch: 0.75, distance: OVERVIEW * outer },
        focus: None,
        paused: false,
        time_scale: 1.0,
        target: Some(objective),
        minigames: MinigameVisits::default(),
        token_presentation: presentation,
        last_completion: None,
        return_smoke,
        launch_smoke,
        intro_smoke,
        help_smoke,
        demo_video,
        software,
        station_smoke,
        ending_smoke: ending_origin.is_some(),
        collision_debug,
        docking_smoke: docking_smoke.then_some(0),
        departing: true,
        intro_pending: true,
        game,
        returning_from: None,
        return_pose: None,
        minecraft_smoke,
        starwars_smoke,
        regularshow_smoke,
        visit_smoke: visit_smoke.then_some(0),
    };
    loop {
        match fly(&root, &mut session, smoke)? {
            Exit::Quit => return Ok(session.game),
            Exit::Menu => { *menu = true; return Ok(session.game); }
            Exit::Dock => {
                let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &session.ships)?;
                if session.game.is_none() {
                    let active=&session.ships.ships[session.ship_index];
                    let choice=catalog.choices.iter().find(|c| c.id==active.id || c.ship_id.as_deref()==Some(active.id.as_str())).ok_or("Nave de desarrollo sin identidad de selección")?;
                    session.game=Some(GameState::new_game(&choice.id,&catalog)?);
                }
                let before=session.game.as_ref().unwrap().clone();
                let angles: Vec<_>=session.system.planets.iter().map(|p|(p.orbit_angle,p.spin_angle)).collect();
                let completion=session.last_completion.clone();
                let goal=session.docking_smoke.map(|n| match n {0=>before.active_ship().to_string(),1=>if before.active_ship()=="starwars" {"southpark".into()} else {"starwars".into()},_=>"@cancel".into()});
                let outcome=super::front_end::dock(before.clone(),goal)?;
                if matches!(outcome,Some(SelectionTransition::Quit)) {return Ok(session.game);}
                if let Some(SelectionTransition::ChangedShip(next))=outcome {
                    catalog.configure_active(&next,&mut session.ships)?;
                    session.ship_index=session.ships.ships.iter().position(|s| s.id==next.active_ship()).ok_or("Modelo activo no resuelto")?;
                    session.game=Some(next);
                    if !smoke { crate::gameplay::save::store(&crate::gameplay::save::path(&root), session.game.as_ref().unwrap(), &session.token_presentation)?; }
                }
                if session.system.planets.iter().map(|p|(p.orbit_angle,p.spin_angle)).collect::<Vec<_>>()!=angles || session.last_completion!=completion {
                    return Err("El hangar alteró el estado de la sesión solar".into());
                }
                if let Some(n)=&mut session.docking_smoke {
                    let next=session.game.as_ref().unwrap();
                    if next.first_ship()!=before.first_ship() || (*n!=1 && next!=&before) || (*n==1 && next.active_ship()==before.active_ship()) {
                        return Err("El ciclo de hangar perdió o no aplicó la selección".into());
                    }
                    *n+=1;
                }
                session.ship=Ship::new(session.station.dock_target(),0.0);
                session.ship.look_at(session.ship.position+session.station.exit_forward());
                session.view=View::Ship;
                session.paused=false;
                session.departing=true;
                session.target=None;
                session.returning_from=None;
                session.return_pose=None;
            }
            Exit::Visit(i) => {
                if session.system.planets[i].config.id == "minecraft" {
                    session.return_pose = Some(PlanetReturnPose::capture(&session.ship, &session.system.planets[i]));
                }
                // La ventana del sistema solar ya se cerró: el minijuego abre la suya
                let planet = &session.system.planets[i].config;
                let id = planet.minigame.clone().ok_or("El planeta no tiene minijuego")?;
                let args = if session.regularshow_smoke {
                    match session.visit_smoke.unwrap() {
                        0 => vec!["--smoke".into()],
                        1 => vec!["--smoke-return".into(), "defeat".into()],
                        _ => vec!["--smoke-return".into(), "abandoned".into()],
                    }
                } else if session.starwars_smoke {
                    vec!["--smoke-hidden".into()]
                } else if session.minecraft_smoke {
                    vec!["--smoke-host".into(), "--capture-dir".into(), root.join(format!("target/minecraft-host-{}", session.visit_smoke.unwrap())).to_string_lossy().into_owned()]
                } else if session.return_smoke {
                    let goal = ["victory", "defeat", "abandoned", "abandoned"][session.visit_smoke.unwrap() as usize];
                    vec!["--smoke-return".into(), goal.into(), "--mute".into()]
                } else {
                    session.visit_smoke.map(|_| "--smoke-entry".to_string()).into_iter().collect()
                };
                let ship_index = session.ship_index;
                let selected_ship=session.ships.ships[ship_index].clone();
                let orbit_angles: Vec<_> = session.system.planets.iter().map(|p| p.orbit_angle).collect();
                let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &session.ships)?;
                let reward_ship_name = catalog.choice(&id).map(|choice| catalog.model(choice, &session.ships).name)
                    .unwrap_or_else(|| id.clone());
                if id == "southpark" {
                    if let Some(game) = &session.game {
                        session.minigames.southpark.chef_intro_seen = game.southpark_chef_intro_seen;
                    }
                }
                let mut on_result = |result| {
                    if let Some(game) = &mut session.game {
                        let outcome = crate::gameplay::rewards::record_result(game, &id, result, &catalog)?;
                        if outcome.token_obtained {
                            let franchise = crate::gameplay::progress::Franchise::from_id(&id)?;
                            session.token_presentation.enqueue(franchise, outcome, &reward_ship_name);
                            if !smoke { crate::gameplay::save::store(&crate::gameplay::save::path(&root), game, &session.token_presentation).map_err(|e| e.to_string())?; }
                        }
                    }
                    Ok(())
                };
                let minigame_exit = minigame_host::run_with_ship_and_results(&id, &mut session.minigames, &selected_ship, &root, args.into_iter(), &mut on_result)?;
                if id == "southpark" {
                    if let Some(game) = &mut session.game {
                        game.southpark_chef_intro_seen = session.minigames.southpark.chef_intro_seen;
                        if !smoke { crate::gameplay::save::store(&crate::gameplay::save::path(&root), game, &session.token_presentation)?; }
                    }
                }
                match minigame_exit {
                    MinigameExit::WindowClosed if session.visit_smoke.is_none() => return Ok(session.game),
                    MinigameExit::Returned(result) => {
                        if session.regularshow_smoke {
                            let expected = [MinigameResult::Victory, MinigameResult::Defeat, MinigameResult::Abandoned][session.visit_smoke.unwrap() as usize];
                            if result != expected { return Err(format!("Regular Show esperaba {expected:?}, recibió {result:?}").into()); }
                        }
                        let completion = MinigameCompletion { planet_id: planet.id.clone(), minigame_id: id, result };
                        println!("MINIGAME_RETURN {}", serde_json::to_string(&completion)?);
                        session.last_completion = Some(completion);
                    }
                    _ => {}
                }
                if (session.return_smoke || session.minecraft_smoke || session.starwars_smoke || session.regularshow_smoke) && (session.ship_index != ship_index
                    || session.system.planets.iter().map(|p| p.orbit_angle).collect::<Vec<_>>() != orbit_angles) {
                    return Err("La visita alteró la nave o las órbitas del anfitrión".into());
                }
                session.view = View::Ship;
                session.target = Some(next_objective(&session.system, session.game.as_ref(), i));
                session.ship.carried_by = None;
                if let Some(n) = &mut session.visit_smoke {
                    *n += 1;
                }
                session.returning_from = Some(i);
            }
        }
    }
}

fn load_planet_models(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    root: &Path,
    shader: &SpaceShader,
    system: &mut SolarSystem,
) -> Vec<(Option<SpaceModel>, Option<SpaceModel>)> {
    let mut models = Vec::new();
    for planet in &mut system.planets {
        let c = &planet.config;
        // Si un modelo falta o falla, ese planeta sigue como esfera temporal (hay planetas en construcción)
        let near = c.model.as_ref().and_then(|m| {
            SpaceModel::load(rl, thread, &root.join(m), shader, c.model_rotation_degrees, Fit::Radius(c.radius))
                .map_err(|e| eprintln!("Aviso: {}: {e}", c.id))
                .ok()
        });
        let far = near.as_ref().and_then(|n| {
            c.model_far.as_ref().and_then(|m| {
                SpaceModel::load(rl, thread, &root.join(m), shader, c.model_rotation_degrees, Fit::Same(n.fit, n.half_extents))
                    .map_err(|e| eprintln!("Aviso: {} (lejos): {e}", c.id))
                    .ok()
            })
        });
        if let Some(n) = &near {
            // La forma de choque sigue la caja del modelo (un poco adentro para no chocar con el aire)
            planet.collision_radii = n.half_extents * 0.95;
        }
        models.push((near, far));
    }
    models
}

fn load_ship_model(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    root: &Path,
    shader: &SpaceShader,
    session: &Session,
) -> Option<SpaceModel> {
    let s = &session.ships.ships[session.ship_index];
    if s.model.is_empty() { return None; }
    SpaceModel::load(rl, thread, &root.join(&s.model), shader, s.flight_rotation_degrees(), Fit::Length(s.length))
        .map_err(|e| eprintln!("Aviso: nave {}: {e}", s.id))
        .ok()
}

fn station_ship_radius(s: &Session, model: &Option<SpaceModel>) -> f32 {
    model.as_ref().map_or(s.ships.ships[s.ship_index].length * 0.5 * 3.0_f32.sqrt(),
        |m| m.half_extents.length()).max(s.ships.flight.radius)
}

/// Una ventana del sistema solar, hasta salir del juego o ir a un minijuego.
fn fly(root: &Path, s: &mut Session, smoke: bool) -> Result<Exit, Box<dyn std::error::Error>> {
    let (mut rl, thread) = crate::graphics::window::build(
        raylib::init()
            .size(1280, 800)
            .resizable()
            .msaa_4x()
            .title("Planet-Hoppers | Sistema solar"),
        smoke,
    );
    rl.set_exit_key(None);
    rl.set_window_min_size(640, 480);
    rl.set_target_fps(60);
    // El audio del sistema solar: se cierra al salir de esta ventana (los minijuegos abren el suyo). El crescendo suena
    // mientras se carga todo
    let audio = if smoke { None } else { RaylibAudio::init_audio_device().ok() };
    let riser = super::menu_music::riser(audio.as_ref(), root);
    if let Some(r) = &riser {
        r.play();
    }
    let far_plane = outer_orbit(&s.system) * (2.0 * MAX_ZOOM_OUT + 2.0);
    {
        let mut d = crate::graphics::window::begin_drawing(&mut rl, &thread);
        d.clear_background(SKY_BACKGROUND);
        d.draw_text("Cargando el sistema solar...", 40, 40, 24, Color::RAYWHITE);
    }

    let mut shader = SpaceShader::load(&mut rl, &thread)?;
    let mut sun_shader = SunShader::load(&mut rl, &thread)?;
    let mut sun = rl.load_model_from_mesh(&thread, unsafe { Mesh::gen_mesh_sphere(&thread, 1.0, 48, 64).make_weak() })?;
    *sun.materials_mut()[0].shader_mut() = unsafe { WeakShader::from_raw(*sun_shader.shader.as_ref()) };
    let placeholder = rl.load_model_from_mesh(&thread, unsafe { Mesh::gen_mesh_sphere(&thread, 1.0, 24, 32).make_weak() })?;
    let planets = load_planet_models(&mut rl, &thread, root, &shader, &mut s.system);
    let station_near = SpaceModel::load(&mut rl, &thread, &root.join(&s.station.model), &shader,
        [0.0;3], Fit::Length(s.station.length))?;
    let station_far = SpaceModel::load(&mut rl, &thread, &root.join(&s.station.model_far), &shader,
        [0.0;3], Fit::Same(station_near.fit, station_near.half_extents))?;
    let mut ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
    s.station.align_collision(station_near.fit);
    if smoke {
        println!("COLLISION_HULL boxes={} cell={} offset_fraction={:?} world_limit=none",s.station.hull.rows.len(),s.station.hull.cell,s.station.hull.offset_fraction);
    }
    let departing = std::mem::take(&mut s.departing);
    let mut intro_time = (s.intro_pending && (!smoke || s.intro_smoke)).then_some(0.0_f32);
    let mut launch_coast = intro_time.is_some();
    let mut entry_handoff_speed = None;
    s.intro_pending = false;
    let departure = if departing {
        let length = s.ships.ships[s.ship_index].length;
        let half = ship_model.as_ref().map_or(Vector3::one() * length * 0.5, |m| m.half_extents);
        let aspect = rl.get_screen_width() as f32 / rl.get_screen_height().max(1) as f32;
        let near = NEAR_PLANE as f32;
        let camera_radius = (near * (1.0 + (30.0_f32.to_radians().tan()).powi(2)
            * (1.0 + aspect * aspect)).sqrt()).max(near * 2.0);
        let plan = s.station.departure(DepartureBounds {
            station_half_extents: station_near.half_extents,
            ship_half_extents: half,
            ship_collision_radius: s.ships.flight.radius,
            ship_length: length,
            camera_length: s.ships.ships[s.ship_index].camera_length(),
            camera_radius,
        })?;
        s.ship = plan.ship.clone();
        if intro_time.is_some() {
            // Start on the station's central axis so its full ring frames the ship.
            // This point is beyond the hull and is checked before the first frame.
            s.ship.position.y = s.station.center().y;
            let ship_radius = plan.ship_radius;
            if (s.ship.position - s.station.center()).length() <= s.station.collision_radius + ship_radius
                || s.station.push_out(s.ship.position, s.ship.position, ship_radius).is_some() {
                return Err("La posición inicial de la nave toca la estación".into());
            }
        }
        if smoke {
            println!("DEPARTURE_SAFE {}", serde_json::json!({
                "ship":s.ships.ships[s.ship_index].id,
                "position":[s.ship.position.x,s.ship.position.y,s.ship.position.z],
                "forward":[s.ship.forward().x,s.ship.forward().y,s.ship.forward().z],
                "camera":[plan.camera_position.x,plan.camera_position.y,plan.camera_position.z],
                "visual_half":[half.x,half.y,half.z],"ship_radius":plan.ship_radius,
                "hull_support":plan.hull_support,"safety_margin":plan.safety_margin,
                "distance_from_dock":plan.distance_from_dock,
            }));
        }
        Some(plan)
    } else { None };
    let entry_start_yaw = s.ship.yaw;
    let entry_start_pitch = s.ship.pitch;
    let mut entry_goal = s.ship.clone();
    entry_goal.look_at(s.system.planets[0].position());
    let sky = Sky::new();
    let mut nebula = NebulaSky::load(&mut rl, &thread)?;
    let (bw, bh) = (rl.get_screen_width(), rl.get_screen_height());
    let mut bloom = crate::graphics::bloom::Bloom::load(&mut rl, &thread, bw, bh)?;
    let ui_regular = load_solar_font(&root.join("assets/fonts/AdventPro-Regular.ttf"))?;
    let ui_bold = load_solar_font(&root.join("assets/fonts/AdventPro-Bold.ttf"))?;
    let token_textures = super::token_collection_ui::TokenTextures::load(&mut rl, &thread, root)?;
    let mut abductions = super::abduction::AbductionRenderer::load(&mut rl, &thread, root,
        &s.token_presentation.required_abductions())?;
    // La de Smiling Friends es un video, que solo se carga si va a hacer falta
    let mut abduction_video = if s.token_presentation.required_abductions().contains(&crate::gameplay::progress::Franchise::SmilingFriends) {
        Some(super::abduction::AbductionVideo::load(&mut rl, &thread, root, audio.as_ref())?)
    } else {
        None
    };
    let mut video_playing = false;
    // La de Hexbound trae su música (la de la intro del minijuego)
    let hexbound_music = super::abduction::abduction_music(audio.as_ref(), root, crate::gameplay::progress::Franchise::Halloween)
        .filter(|_| s.token_presentation.required_abductions().contains(&crate::gameplay::progress::Franchise::Halloween));
    let mut hexbound_playing = false;
    let empty_progress = crate::gameplay::progress::ProgressState::default();
    let mut planet_logos = BTreeMap::new();
    for id in ["halloween", "starwars", "smilingfriends", "minecraft", "regularshow", "southpark"] {
        let path = root.join(format!("assets/ui/logos/{id}.png"));
        planet_logos.insert(id, rl.load_texture(&thread, &path.to_string_lossy())?);
    }
    let outer = outer_orbit(&s.system);
    // Las pruebas siguen a South Park por identidad, aunque cambie de órbita.
    let southpark = s.system.planets.iter().position(|p| p.config.id == "southpark").expect("South Park validado");
    let captures = root.join("target/solarsystem");
    if smoke {
        std::fs::create_dir_all(&captures)?;
    }

    if let Some(i) = s.returning_from.take() {
        if let Some(pose) = s.return_pose.take() {
            let margin = station_ship_radius(s, &ship_model);
            pose.restore(&mut s.ship, &s.system, i, margin,
                |point| s.station.push_out(point, point, margin).is_none())?;
            println!("PLANET_RETURN_POSE planet={} ship={} position={:?}", s.system.planets[i].config.id, s.ships.ships[s.ship_index].id, s.ship.position);
        } else {
            park_ship(&mut s.ship, &s.system.planets[i], 2.2, false);
        }
    }
    if s.view == View::Ship && !smoke {
        rl.disable_cursor();
    }
    let mut chase = chase_position(s.ship.position, s.ship.forward(), s.ships.ships[s.ship_index].camera_length());
    let mut previous_ship_position = s.ship.position;
    let mut debug_last = "ninguna".to_owned();
    let mut debug_camera;
    let mut snap = true;
    let mut fade = if smoke { 0.0 } else { 1.0_f32 }; // 1 = negro; baja al entrar
    let mut leaving: Option<usize> = None;
    let mut entering_station = false;
    let mut station_reentry_delay=if departing {1.2_f32} else {0.0};
    let mut interaction_armed = false;
    let mut discard_mouse_frames = 2u8;
    let mut notice: Option<(String, f32)> = None;
    let mut frame = 0u32;
    let mut shake = 0.0_f32;
    let mut side_camera: Option<Camera3D> = None; // Solo para las capturas de prueba
    let mut help_open = s.help_smoke;
    let mut help_progress = if s.help_smoke { 1.0_f32 } else { 0.0_f32 };
    let mut help_restore_cursor = false;
    let mut show_orbits = true;
    // Contador de rendimiento (F4) y, para compararlos, el descarte (F5) y el nivel de detalle (F6) encendidos o no
    let mut show_stats = false;
    let mut demo = DemoPilot::default();
    let mut culling = true;
    let mut lod_on = true;
    // Renderizador por software (F7): se prepara la primera vez que se enciende
    let mut software_on = s.software;
    let mut soft: Option<SoftScene> = None;
    if software_on {
        soft = Some(SoftScene::load(&mut rl, &thread, &planets, ship_model.as_ref(), &station_far)?);
    }
    let mut collection_open = false;
    let mut presentation_shortcut = crate::minigames::common::shortcuts::PresentationShortcut::default();
    let mut collection_progress = 0.0_f32;
    let mut token_was_blocking = false;
    let mut token_restore_cursor = false;
    let music = super::menu_music::MenuMusic::start(audio.as_ref(), root);

    loop {
        music.update();
        if crate::graphics::window::should_close(&rl) {
            return Ok(Exit::Quit);
        }
        let ctrl = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL);
        if presentation_shortcut.poll(&rl) {
            let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &s.ships)?;
            if s.game.is_none() {
                let active = &s.ships.ships[s.ship_index];
                let choice = catalog.choices.iter().find(|c| c.id == active.id || c.ship_id.as_deref() == Some(active.id.as_str()))
                    .ok_or("Nave de desarrollo sin identidad de selección")?;
                s.game = Some(GameState::new_game(&choice.id, &catalog)?);
            }
            let game = s.game.as_mut().unwrap();
            let granted = crate::gameplay::rewards::complete_for_presentation(game, &catalog)?;
            s.token_presentation.finish_for_presentation();
            if !smoke {
                crate::gameplay::save::store(&crate::gameplay::save::path(root), game, &s.token_presentation)?;
            }
            collection_open = true;
            token_restore_cursor |= rl.is_cursor_hidden() || (help_open && help_restore_cursor);
            token_was_blocking = true;
            rl.enable_cursor();
            help_open = false;
            notice = Some((format!("CNP: {granted} tokens obtenidos | Coleccion completa: 6/6"), 5.0));
        }
        if ctrl && rl.is_key_pressed(KeyboardKey::KEY_Q) {
            return Ok(Exit::Quit);
        }
        let dt = if smoke { 1.0 / 60.0 } else { rl.get_frame_time().min(0.1) };
        let help_click = !smoke && !rl.is_cursor_hidden() && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        let help_icon_hit = help_button(rl.get_screen_width(), rl.get_screen_height()).check_collision_point_rec(rl.get_mouse_position());
        let panel_w = (rl.get_screen_width() - 36).min(820) as f32;
        let panel_h = (rl.get_screen_height() - 38).min(580) as f32;
        let panel_x = (rl.get_screen_width() as f32 - panel_w) * 0.5;
        let panel_y = (rl.get_screen_height() as f32 - panel_h) * 0.5;
        let panel_hit = Rectangle::new(panel_x, panel_y, panel_w, panel_h).check_collision_point_rec(rl.get_mouse_position());
        let close_hit = Rectangle::new(panel_x + panel_w - 86.0, panel_y + 22.0, 55.0, 55.0).check_collision_point_rec(rl.get_mouse_position());
        let toggle_help = !smoke && !token_was_blocking && !s.token_presentation.is_busy() && s.token_presentation.pending_count() == 0 && (rl.is_key_pressed(KeyboardKey::KEY_H) || (help_click && help_icon_hit));
        let close_help = help_open && !smoke && (rl.is_key_pressed(KeyboardKey::KEY_ESCAPE)
            || (help_click && (close_hit || !panel_hit)));
        if toggle_help || close_help {
            help_open = !help_open;
            if help_open {
                help_restore_cursor = rl.is_cursor_hidden();
                rl.enable_cursor();
            } else if help_restore_cursor && s.view == View::Ship {
                rl.disable_cursor();
                discard_mouse_frames = 2;
            }
        }
        help_progress = (help_progress + if help_open { dt * 4.0 } else { -dt * 4.0 }).clamp(0.0, 1.0);
        let token_hud_hit = super::token_collection_ui::TokenTextures::hud_button(rl.get_screen_width(), rl.get_screen_height())
            .check_collision_point_rec(rl.get_mouse_position());
        let token_toggle = !smoke && !help_open && !s.token_presentation.is_busy() && s.token_presentation.pending_count() == 0
            && (rl.is_key_pressed(KeyboardKey::KEY_T) || (help_click && token_hud_hit));
        let token_close = collection_open && rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
        if token_toggle || token_close { collection_open = !collection_open; }
        collection_progress = (collection_progress + if collection_open { dt * 5.0 } else { -dt * 5.0 }).clamp(0.0, 1.0);
        let cinematic_was_blocking = s.token_presentation.is_busy();
        let reward_revision = s.token_presentation.revision();
        let video_abduction = abduction_video.as_mut()
            .filter(|_| s.token_presentation.abduction() == Some(crate::gameplay::progress::Franchise::SmilingFriends));
        if video_abduction.is_some_and(|v| v.skip_requested(&rl)) {
            s.token_presentation.skip_abduction();
        }
        s.token_presentation.advance(dt, !smoke && fade <= 0.0 && intro_time.is_none()
            && !help_open && help_progress <= 0.0 && collection_progress <= 0.0
            && leaving.is_none() && !entering_station);
        // El video sigue al reloj de la abducción; mientras suena, la música del menú se calla
        let video_now = abduction_video.is_some()
            && s.token_presentation.abduction() == Some(crate::gameplay::progress::Franchise::SmilingFriends);
        if let Some(video) = abduction_video.as_mut() {
            if video_now {
                video.update(s.token_presentation.elapsed())?;
            } else if video_playing {
                video.stop();
            }
        }
        let hexbound_now = s.token_presentation.abduction() == Some(crate::gameplay::progress::Franchise::Halloween);
        if let Some(m) = &hexbound_music {
            if hexbound_now && !hexbound_playing {
                m.seek_stream(0.0);
                m.play_stream();
            } else if !hexbound_now && hexbound_playing {
                m.stop_stream();
            }
            if hexbound_now {
                m.set_volume(super::abduction::abduction_music_volume(s.token_presentation.elapsed(),
                    super::token_collection::abduction_length(crate::gameplay::progress::Franchise::Halloween,
                                                              s.token_presentation.abduction_is_initial())));
                m.update_stream();
            }
        }
        if hexbound_music.is_some() && hexbound_now != hexbound_playing {
            music.set_paused(hexbound_now);
            hexbound_playing = hexbound_now;
        }
        if video_now != video_playing {
            music.set_paused(video_now);
            video_playing = video_now;
        }
        if reward_revision != s.token_presentation.revision() {
            if let Some(game) = &s.game {
                crate::gameplay::save::store(&crate::gameplay::save::path(root), game, &s.token_presentation)?;
            }
        }
        let ending_ready = (!smoke || s.ending_smoke) && fade <= 0.0 && intro_time.is_none() && leaving.is_none() && !entering_station
            && !help_open && help_progress <= 0.0 && !collection_open && collection_progress <= 0.0
            && s.game.as_ref().is_some_and(|game| super::ending::ready(game, &s.token_presentation));
        let tokens_blocking = ending_ready || collection_open || collection_progress > 0.0 || s.token_presentation.is_busy()
            || (!smoke && s.token_presentation.phase().is_none() && s.token_presentation.pending_count() > 0);
        if tokens_blocking && !token_was_blocking {
            token_restore_cursor = rl.is_cursor_hidden();
            rl.enable_cursor();
        } else if !tokens_blocking && token_was_blocking && token_restore_cursor && s.view == View::Ship {
            rl.disable_cursor();
            discard_mouse_frames = 2;
        }
        token_was_blocking = tokens_blocking;
        // Sin teclado durante las pruebas ni mientras se sale hacia un minijuego
        let intro_controls_ready = intro_time.is_none_or(|elapsed| elapsed >= ENTRY_STATION_HOLD + ENTRY_CAMERA_MOVE - 0.15);
        let accepts_input = (!smoke || s.launch_smoke || s.station_smoke || s.minecraft_smoke || s.regularshow_smoke || s.docking_smoke.is_some()) && !tokens_blocking && !cinematic_was_blocking && !token_toggle && !token_close && !help_open && !toggle_help && !close_help && intro_controls_ready && leaving.is_none() && !entering_station;
        let pressed = |rl: &RaylibHandle, k: KeyboardKey| accepts_input
            && !(ctrl && matches!(k, KeyboardKey::KEY_C | KeyboardKey::KEY_N | KeyboardKey::KEY_P))
            && rl.is_key_pressed(k);
        let down = |rl: &RaylibHandle, k: KeyboardKey| accepts_input && rl.is_key_down(k);
        if pressed(&rl,KeyboardKey::KEY_F3) {s.collision_debug=!s.collision_debug;}
        if pressed(&rl, KeyboardKey::KEY_F4) { show_stats = !show_stats; }
        if pressed(&rl, KeyboardKey::KEY_F5) { culling = !culling; }
        if pressed(&rl, KeyboardKey::KEY_F6) { lod_on = !lod_on; }
        if pressed(&rl, KeyboardKey::KEY_F7) {
            software_on = !software_on;
            if software_on && soft.is_none() {
                {
                    let mut d = crate::graphics::window::begin_drawing(&mut rl, &thread);
                    d.clear_background(SKY_BACKGROUND);
                    d.draw_text("Preparando el renderizador por software...", 40, 40, 24, Color::RAYWHITE);
                }
                soft = Some(SoftScene::load(&mut rl, &thread, &planets, ship_model.as_ref(), &station_far)?);
            }
        }
        if pressed(&rl,KeyboardKey::KEY_O) { show_orbits = !show_orbits; }
        debug_camera="ninguna";

        // --- Recorrido automático de prueba (--smoke) -------------------------------------------------------
        let mut capture: Option<String> = None;
        if let Some(cycles)=s.docking_smoke {
            for key in [KeyboardKey::KEY_E,KeyboardKey::KEY_W,KeyboardKey::KEY_B] {
                unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {frame:0,type_:1,params:[key as i32,0,0,0]}); }
            }
            let mut keys=Vec::new();
            if cycles>0 && frame<10 {keys.push(KeyboardKey::KEY_E);}
            if frame==0 {
                if (s.ship.position-departure.as_ref().ok_or("Falta plan de despegue")?.ship.position).length()>0.01 || s.ship.speed!=0.0
                    || s.station.push_out(s.ship.position,s.ship.position,s.ships.flight.radius).is_some()
                    || s.ship.forward().dot(departure.as_ref().unwrap().ship.forward())<0.999 {
                    return Err("Despegue fuera de la salida o con una transformación insegura".into());
                }
                if s.game.as_ref().unwrap().active_ship()!=s.ships.ships[s.ship_index].id {return Err("Modelo de despegue incorrecto".into());}
                if !s.ships.ships[s.ship_index].model.is_empty() && ship_model.is_none() {return Err("Fallo al cargar el modelo de despegue".into());}
            }
            if (2..=12).contains(&frame) { keys.push(KeyboardKey::KEY_W); }
            if frame==20 {capture=Some(format!("dock-launch-{cycles}.png"));}
            if cycles==3 && frame==25 {
                println!("DOCKING_SMOKE_PASS cycles=3 same_ship=true changed_ship=true cancel=true safe_launch=true active={}",s.game.as_ref().unwrap().active_ship());
                return Ok(Exit::Quit);
            }
            if frame==30 {
                s.ship.position=s.station.approach()+s.station.exit_forward()*100.0;
                s.ship.look_at(s.station.exit_position());
                s.ship.speed=0.0;
                s.target=Some(Destination::Station);
                snap=true;
            }
            if frame>30 {
                if s.station.in_reach(s.ship.position, station_ship_radius(s, &ship_model)) || entering_station {
                    if !entering_station {
                        let radius=station_ship_radius(s,&ship_model);
                        if s.station.push_out(s.ship.position,s.ship.position,radius).is_some() || s.ship.bumped {
                            return Err("La interacción no es accesible fuera del colisionador".into());
                        }
                        println!("ENTRY_OUTSIDE ship={} radius={} centre_distance={}",s.ships.ships[s.ship_index].id,radius,(s.ship.position-s.station.center()).length());
                        capture=Some(format!("dock-approach-{cycles}.png"));
                    }
                    keys.push(KeyboardKey::KEY_E);
                } else {
                    keys.push(KeyboardKey::KEY_W); keys.push(KeyboardKey::KEY_B);
                }
            }
            for key in keys {unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {frame:0,type_:2,params:[key as i32,0,0,0]}); }}
            if frame>600 {return Err("El recorrido de aproximación no entró al hangar".into());}
        } else if s.station_smoke {
            // Native key events exercise the same input path as ordinary flight.
            for key in [KeyboardKey::KEY_SPACE, KeyboardKey::KEY_B, KeyboardKey::KEY_E, KeyboardKey::KEY_TAB] {
                unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent { frame: 0, type_: 1, params: [key as i32,0,0,0] }); }
            }
            let held = match frame {
                36 => Some(KeyboardKey::KEY_TAB),
                40..=49 => Some(KeyboardKey::KEY_SPACE),
                51..=69 => Some(KeyboardKey::KEY_B),
                _ => None,
            };
            if let Some(key) = held {
                unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent { frame: 0, type_: 2, params: [key as i32,0,0,0] }); }
            }
            match frame {
                0 => { s.view = View::Overview; s.focus = None; }
                10 => capture = Some("station-overview.png".into()),
                20 => {
                    s.view=View::Ship;
                    s.ship=Ship::new(s.station.center()+Vector3::up()*500.0,0.0);
                    s.ship.look_at(s.station.center());
                    snap=true;
                }
                22 => {
                    if s.ship.bumped || (s.ship.position-s.station.center()-Vector3::up()*500.0).length()>0.01 {
                        return Err("Barrera invisible en espacio vacio sobre estacion".into());
                    }
                    capture=Some("station-debug-above.png".into());
                }
                25 => {
                    let (lo,hi)=s.station.hull.boxes().next().ok_or("Colisionador vacio")?;
                    s.ship=Ship::new(s.station.collision_point((lo+hi)*0.5),0.0);
                    snap=true;
                }
                23 => {
                    s.ship=Ship::new(s.station.center()+Vector3::new(500.0,0.0,0.0),0.0);
                    s.ship.look_at(s.station.center());
                    snap=true;
                }
                24 => {
                    if s.ship.bumped || (s.ship.position-s.station.center()-Vector3::new(500.0,0.0,0.0)).length()>0.01 {
                        return Err("Barrera invisible al costado de estacion".into());
                    }
                    capture=Some("station-debug-side.png".into());
                }
                27 => {
                    if s.ship.bumped || s.station.push_out(s.ship.position,s.ship.position,station_ship_radius(s,&ship_model)).is_some() {
                        return Err("Recuperacion de colision inestable".into());
                    }
                    capture=Some("station-debug-recovery.png".into());
                }
                15 => {
                    s.focus = Some(Destination::Station);
                    s.orbit.distance = s.station.length * 2.0;
                    s.orbit.pitch = 0.3;
                    s.orbit.yaw = 1.1;
                }
                18 => capture = Some("station-exterior.png".into()),
                35 => {
                    s.view = View::Ship;
                    s.ship.position = s.station.approach() + Vector3::new(0.0, 0.0, 80.0);
                    s.ship.speed = 0.0;
                    s.ship.carried_by = None;
                    s.ship.yaw = 0.5;
                    s.ship.pitch = 0.0;
                    s.target = Some(Destination::Planet(s.system.planets.len()-1));
                    snap = true;
                }
                50 => {
                    if s.ship.position.y <= s.station.exit_position().y + 15.0 || s.ship.pitch != 0.0
                        || s.ship.yaw != 0.5 || s.target != Some(Destination::Station) {
                        return Err("Espacio no produjo ascenso vertical independiente".into());
                    }
                }
                70 => {
                    if s.ship.forward().dot((s.station.approach()-s.ship.position).normalized()) < 0.98 {
                        return Err("B no activó el piloto automático".into());
                    }
                    capture = Some("station-approach.png".into());
                }
                75 => {s.ship.position=s.station.entry_anchor(station_ship_radius(s, &ship_model));s.ship.speed=0.0;}
                85 => {
                    if !s.station.in_reach(s.ship.position, station_ship_radius(s, &ship_model)) { return Err("Falta proximidad con la estación".into()); }
                    capture = Some("station-contact.png".into());
                }
                90 => {
                    println!("STATION_SMOKE_PASS fixed={:?} length={} ascend=Space autopilot=B proximity=true", s.station.position, s.station.length);
                    return Ok(Exit::Quit);
                }
                _ => {}
            }
        } else if let Some(visits) = s.visit_smoke {
            // Ida: estacionar junto a South Park y explorar. Vuelta: capturar la nave junto al planeta y salir
            let destination = if s.regularshow_smoke { s.system.planets.iter().position(|p| p.config.id=="regularshow").unwrap() } else if s.starwars_smoke { s.system.planets.iter().position(|p|p.config.id=="starwars").unwrap() } else if s.minecraft_smoke { s.system.planets.iter().position(|p| p.config.id == "minecraft").unwrap() } else { southpark };
            let total = if s.regularshow_smoke { 3 } else if s.minecraft_smoke { 2 } else if s.return_smoke { 4 } else { 1 };
            if s.minecraft_smoke || s.regularshow_smoke {
                let event = if visits > 0 && frame == 0 || visits < total && frame == 30 { Some(2) }
                    else if frame == 10 || frame == 31 { Some(1) } else { None };
                if let Some(kind) = event { unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {
                    frame: 0, type_: kind, params: [KeyboardKey::KEY_E as i32, 0, 0, 0],
                }); } }
                if visits > 0 && frame == 12 && leaving.is_some() {
                    return Err("Held E re-entered a minigame before a new press".into());
                }
            }
            match (visits, frame) {
                (_, 0) => {
                    if visits > 0 {
                        if (!s.minecraft_smoke && s.system.planet_in_reach(s.ship.position).is_some()) || s.ship.speed != 0.0
                            || s.system.planets[destination].push_out(s.ship.position, station_ship_radius(s, &ship_model)).is_some() {
                            return Err("Retorno inseguro o dentro del alcance de exploración".into());
                        }
                        capture = Some(format!("after-return-{visits}.png"));
                    }
                    if visits == 0 { park_ship(&mut s.ship, &s.system.planets[destination], 1.7, true); }
                    snap = true;
                }
                (_, 1) if visits > 0 && visits < total => {
                    if s.game.is_none() && !s.regularshow_smoke { s.ship_index = visits as usize % s.ships.ships.len(); }
                    ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
                    park_ship(&mut s.ship, &s.system.planets[destination], 1.7, true);
                    snap = true;
                }
                (_, 30) if visits < total => {
                    capture = Some("before-visit.png".into());
                    if s.system.planet_in_reach(s.ship.position) != Some(destination) {
                        return Err("La nave no quedó al alcance de South Park".into());
                    }
                    if !s.minecraft_smoke && !s.regularshow_smoke { leaving = Some(destination); }
                }
                (_, 60) if visits == total => capture = Some("after-visit.png".into()),
                (_, 61) if visits == total => {
                    if s.return_smoke && s.last_completion.as_ref().map(|c| c.result) != Some(MinigameResult::Abandoned) {
                        return Err("Falta resultado del último regreso".into());
                    }
                    if s.regularshow_smoke {
                        if s.last_completion.as_ref().map(|c| (&*c.minigame_id, c.result)) != Some(("regularshow", MinigameResult::Abandoned)) {
                            return Err("Regular Show no devolvió el abandono confirmado".into());
                        }
                        if let Some(game) = &s.game {
                            if game.progress().token_count() != 1 || !game.is_unlocked("regularshow") {
                                return Err("La victoria de El Martillo no conservó su recompensa".into());
                            }
                        }
                        println!("REGULARSHOW_HOST_PASS visits=3 victory=true defeat=true abandon=true reward_preserved=true");
                    }
                    if s.starwars_smoke {
                        if s.last_completion.as_ref().map(|c|(&*c.minigame_id,c.result)) != Some(("starwars",MinigameResult::Victory)) {return Err("No regresó victoria de Star Wars".into());}
                        println!("STARWARS_HOST_PASS ship={} model={} result=Victory",s.ships.ships[s.ship_index].id,s.ships.ships[s.ship_index].model);
                    }
                    println!("VISIT_SMOKE_PASS visitas={visits}");
                    return Ok(Exit::Quit);
                }
                _ => {}
            }
        } else if s.help_smoke {
            if frame == 10 { capture = Some("help-frame.png".into()); }
            if frame == 12 { println!("HELP_SMOKE_PASS"); return Ok(Exit::Quit); }
        } else if s.intro_smoke {
            if frame == 7 { capture = Some("entry-station.png".into()); }
            if frame == 50 { capture = Some("entry-transition.png".into()); }
            if frame == 100 {
                if s.ship.speed <= 1.0 || s.ship.bumped {
                    return Err("La nave se detuvo al terminar la entrada".into());
                }
                entry_handoff_speed = Some(s.ship.speed);
                capture = Some("entry-flight.png".into());
            }
            if frame == 180 {
                if s.ship.bumped || entry_handoff_speed.is_none_or(|speed| s.ship.speed < speed - 2.0) {
                    return Err("La nave perdiÃ³ el impulso despuÃ©s de la entrada".into());
                }
                println!("ENTRY_SMOKE_PASS speed={:.1}", s.ship.speed);
                return Ok(Exit::Quit);
            }
        } else if s.demo_video {
            // El video: un cuadro guardado cada dos (30 por segundo, con dt de 1/60), hasta que el piloto termina
            show_stats = demo.show_stats();
            if frame == 0 {
                std::fs::create_dir_all(captures.join("demo-video"))?;
                // Empieza en el espacio, del lado iluminado del planeta, a unos 9 radios, mirándolo
                let p = &s.system.planets[DEMO_PLANET];
                let (c, r) = (p.position(), p.config.radius);
                let sunward = (s.system.center() - c).normalized();
                let side = Vector3::up().cross(sunward).normalized();
                s.ship.position = c + (sunward * 0.8 + side * 0.6).normalized() * (r * 9.0) + Vector3::up() * (r * 1.2);
                s.ship.look_at(c);
                s.ship.speed = 0.0;
                s.ship.carried_by = None;
                s.target = Some(Destination::Planet(DEMO_PLANET));
            }
            if frame % 2 == 0 { capture = Some(format!("demo-video/frame_{:05}.png", frame / 2)); }
            if demo.done() { println!("DEMO_VIDEO_DONE frames={}", frame / 2); return Ok(Exit::Quit); }
        } else if s.launch_smoke {
            unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {frame:0,type_:1,params:[KeyboardKey::KEY_W as i32,0,0,0]}); }
            if (2..=12).contains(&frame) {
                unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {frame:0,type_:2,params:[KeyboardKey::KEY_W as i32,0,0,0]}); }
            }
            if let Some(game) = &s.game {
                if s.ships.ships[s.ship_index].id != game.active_ship() { return Err("El modelo activo no coincide con la selección".into()); }
            }
            if frame == 30 { capture = Some(format!("launch-{}.png", s.ships.ships[s.ship_index].id)); }
            if frame == 61 { println!("LAUNCH_SMOKE_PASS ship={}", s.ships.ships[s.ship_index].id); return Ok(Exit::Quit); }
        } else if smoke {
            let n = s.system.planets.len() as u32;
            let ships = s.ships.ships.len() as u32;
            match frame {
                0 => {
                    s.view = View::Overview;
                    s.focus = None;
                    s.orbit.distance = OVERVIEW * outer;
                }
                10 => capture = Some("overview.png".into()),
                f if (20..20 + n * 15).contains(&f) => {
                    let i = ((f - 20) / 15) as usize;
                    s.focus = Some(Destination::Planet(i));
                    s.orbit.distance = s.system.planets[i].config.radius * 3.4;
                    s.orbit.pitch = 0.35;
                    // Desde el lado del Sol (el iluminado), un poco de costado
                    s.orbit.yaw = std::f32::consts::PI - s.system.planets[i].orbit_angle + 0.6;
                    if (f - 20) % 15 == 12 {
                        capture = Some(format!("planet-{}.png", s.system.planets[i].config.id));
                    }
                }
                f if f == 20 + n * 15 => {
                    s.view = View::Ship;
                    park_ship(&mut s.ship, &s.system.planets[southpark], 4.0, true);
                    snap = true;
                }
                f if f == 25 + n * 15 => {
                    let smilingfriends = s.system.planets.iter().position(|p| p.config.id == "smilingfriends").unwrap();
                    park_ship(&mut s.ship, &s.system.planets[smilingfriends], 4.0, true);
                    snap = true;
                }
                f if f == 27 + n * 15 => capture = Some("flight-smilingfriends-label.png".into()),
                f if f == 29 + n * 15 => {
                    park_ship(&mut s.ship, &s.system.planets[southpark], 4.0, true);
                    snap = true;
                }
                f if f == 35 + n * 15 => capture = Some("ship.png".into()),
                f if f == 40 + n * 15 => {
                    park_ship(&mut s.ship, &s.system.planets[southpark], 1.7, true);
                    snap = true;
                }
                f if f == 55 + n * 15 => capture = Some("explore.png".into()),
                f if f == 58 + n * 15 => {
                    // Aproximación: nave y horizonte juntos para revisar su escala relativa.
                    park_ship(&mut s.ship, &s.system.planets[southpark], 1.12, true);
                    snap = true;
                    capture = Some("scale-approach.png".into());
                }
                f if (60 + n * 15..60 + n * 15 + ships * 10).contains(&f) => {
                    // Cada nave de costado, para revisar hacia dónde apunta la nariz (con la cámara en +Z, a la izquierda)
                    let k = ((f - 60 - n * 15) / 10) as usize;
                    if (f - 60 - n * 15) % 10 == 0 {
                        s.ship_index = k;
                        ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
                        // Por encima del Sol, lejos de su superficie (el Sol se escala con el mundo)
                        s.ship.position = Vector3::new(0.0, s.system.sun.radius * 2.5, 0.0);
                        s.ship.yaw = std::f32::consts::FRAC_PI_2; // nariz hacia -X
                        s.ship.pitch = 0.0;
                        s.ship.speed = 0.0;
                        let len = s.ships.ships[k].length;
                        side_camera = Some(Camera3D::perspective(
                            s.ship.position + Vector3::new(0.0, len * 0.4, len * 2.2),
                            s.ship.position,
                            Vector3::up(),
                            45.0,
                        ));
                    }
                    let phase = (f - 60 - n * 15) % 10;
                    if s.ships.ships[k].id == "smilingfriends" && (phase == 4 || phase == 6) {
                        let direction = if phase == 4 { s.ship.forward() } else { -s.ship.forward() };
                        let len = s.ships.ships[k].length;
                        side_camera = Some(Camera3D::perspective(
                            s.ship.position + direction * (len * 1.30) + Vector3::up() * (len * 0.55),
                            s.ship.position + Vector3::up() * (len * 0.25),
                            Vector3::up(),
                            45.0,
                        ));
                        capture = Some(format!("ship-smilingfriends-{}.png", if phase == 4 { "front" } else { "back" }));
                    }
                    if phase == 8 {
                        let len = s.ships.ships[k].length;
                        side_camera = Some(Camera3D::perspective(
                            s.ship.position + Vector3::new(0.0, len * 0.4, len * 2.2),
                            s.ship.position,
                            Vector3::up(),
                            45.0,
                        ));
                        capture = Some(format!("ship-{}.png", s.ships.ships[k].id));
                    }
                }
                f if (60 + n * 15 + ships * 10..105 + n * 15 + ships * 10).contains(&f) => {
                    // Sol desde lejos, a media distancia y cerca, sin mover el sistema.
                    let step = f - (60 + n * 15 + ships * 10);
                    let k = (step / 15) as usize;
                    let (name, radii) = [("far", 14.0), ("medium", 4.0), ("near", 1.8)][k];
                    let center = s.system.center();
                    side_camera = Some(Camera3D::perspective(
                        center + Vector3::new(0.6, 0.25, 1.0).normalized() * s.system.sun.radius * radii,
                        center,
                        Vector3::up(),
                        60.0,
                    ));
                    if step % 15 == 12 {
                        capture = Some(format!("sun-{name}.png"));
                    }
                }
                f if f >= 105 + n * 15 + ships * 10 => return Ok(Exit::Quit),
                _ => {}
            }
        }

        // --- Teclas comunes ------------------------------------------------------------------------------------
        if pressed(&rl, KeyboardKey::KEY_C) {
            s.view = if s.view == View::Ship { View::Overview } else { View::Ship };
            if s.view == View::Ship {
                rl.disable_cursor();
                snap = true;
            } else {
                rl.enable_cursor();
            }
        }
        if pressed(&rl, KeyboardKey::KEY_N) {
            // Development keeps the complete catalog; adventures respect ownership.
            let next = (1..=s.ships.ships.len()).map(|step| (s.ship_index + step) % s.ships.ships.len())
                .find(|i| s.game.as_ref().is_none_or(|game| game.is_unlocked(&s.ships.ships[*i].id))).unwrap_or(s.ship_index);
            if next != s.ship_index {
                if let Some(game) = &mut s.game {
                    game.activate(&s.ships.ships[next].id)?;
                    if !smoke { crate::gameplay::save::store(&crate::gameplay::save::path(root), game, &s.token_presentation)?; }
                }
                s.ship_index = next;
                s.ships.active_ship = s.ships.ships[next].id.clone();
                ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
                notice = Some((s.ships.ships[next].name.clone(), 2.0));
            }
        }

        // --- Actualización -------------------------------------------------------------------------------------
        let mut in_reach = None;
        match s.view {
            View::Overview => {
                if pressed(&rl, KeyboardKey::KEY_P) {
                    s.paused = !s.paused;
                }
                if pressed(&rl, KeyboardKey::KEY_UP) {
                    s.time_scale = (s.time_scale * 2.0).min(16.0);
                }
                if pressed(&rl, KeyboardKey::KEY_DOWN) {
                    s.time_scale = (s.time_scale / 2.0).max(0.125);
                }
                if pressed(&rl, KeyboardKey::KEY_TAB) {
                    s.focus = Destination::next(s.focus, s.system.planets.len());
                    s.orbit.distance = match s.focus {
                        Some(destination) => destination.radius(s) * 5.0,
                        None => OVERVIEW * outer,
                    };
                }
                if !smoke && !help_open && !tokens_blocking && !cinematic_was_blocking && !help_icon_hit && !token_hud_hit && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let delta = rl.get_mouse_delta();
                    s.orbit.yaw += delta.x * 0.005;
                    s.orbit.pitch = (s.orbit.pitch + delta.y * 0.005).clamp(-1.45, 1.45);
                }
                let wheel = if smoke || tokens_blocking || cinematic_was_blocking { 0.0 } else { rl.get_mouse_wheel_move() };
                if wheel != 0.0 {
                    let nearest = s.focus.map_or(s.system.sun.radius, |destination| destination.radius(s)) * 1.6;
                    s.orbit.distance = (s.orbit.distance * (1.0 - wheel * 0.1)).clamp(nearest, outer * MAX_ZOOM_OUT);
                }
                if !s.paused && !help_open && !tokens_blocking {
                    s.system.update(dt * s.time_scale);
                }
            }
            View::Ship => {
                if pressed(&rl, KeyboardKey::KEY_ESCAPE) {
                    if rl.is_cursor_hidden() {
                        rl.enable_cursor();
                    } else {
                        rl.disable_cursor();
                        discard_mouse_frames = 2;
                    }
                }
                if pressed(&rl, KeyboardKey::KEY_TAB) {
                    s.target = Destination::next(s.target, s.system.planets.len());
                }
                let axis = |neg: bool, pos: bool| (pos as i32 - neg as i32) as f32;
                let key = |k| down(&rl, k);
                let mouse = if discard_mouse_frames > 0 {
                    discard_mouse_frames -= 1;
                    (0.0, 0.0)
                } else if !smoke && !tokens_blocking && rl.is_cursor_hidden() {
                    let m = rl.get_mouse_delta();
                    (m.x, m.y)
                } else {
                    (0.0, 0.0)
                };
                // Piloto automático: apunta adonde va a estar el objetivo cuando la nave llegue
                let aim_at = s.target.filter(|_| key(KeyboardKey::KEY_B)).map(|destination| {
                    match destination {
                        Destination::Planet(i) => s.ship.autopilot_aim(i, &s.ships.flight, &s.system),
                        Destination::Station => s.ship.autopilot_to(s.station.navigation_target(s.ship.position), &s.ships.flight, &s.system),
                    }
                });
                let manual_throttle = axis(
                    key(KeyboardKey::KEY_S) || key(KeyboardKey::KEY_DOWN),
                    key(KeyboardKey::KEY_W) || key(KeyboardKey::KEY_UP),
                );
                if manual_throttle != 0.0 || s.ship.bumped {
                    launch_coast = false;
                }
                let mut input = FlightInput {
                    ascend: key(KeyboardKey::KEY_SPACE),
                    throttle: if intro_time.is_some() && fade <= 0.0 {
                        0.45
                    } else if launch_coast && s.ship.speed > 0.0 {
                        (s.ship.speed / s.ships.flight.cruise_speed).clamp(0.01, 1.0)
                    } else {
                        manual_throttle
                    },
                    boost: key(KeyboardKey::KEY_LEFT_SHIFT) || key(KeyboardKey::KEY_RIGHT_SHIFT),
                    turn: (
                        axis(key(KeyboardKey::KEY_A) || key(KeyboardKey::KEY_LEFT), key(KeyboardKey::KEY_D) || key(KeyboardKey::KEY_RIGHT)),
                        axis(key(KeyboardKey::KEY_F), key(KeyboardKey::KEY_R)),
                    ),
                    mouse,
                    aim_at,
                };
                if s.demo_video {
                    input = demo.input(&s.ship, &s.system, &s.ships.flight, dt);
                }
                if s.target==Some(Destination::Station) && input.aim_at.is_some() {
                    s.station.assist_approach(&s.ship,&mut input,&s.ships.flight);
                }
                if !help_open && !tokens_blocking { s.system.update(dt); }
                if !help_open && !tokens_blocking && leaving.is_none() && !entering_station {
                    let before = s.ship.position;
                    s.ship.update(dt, &input, &s.ships.flight, &s.system);
                    if s.ship.bumped {
                        let body=s.system.planets.iter().find(|p| p.push_out(s.ship.position,s.ships.flight.radius+0.1).is_some())
                            .map(|p| p.config.name.as_str()).unwrap_or("Sol");
                        debug_last=format!("controlador: {body}");
                    }
                    if let Some(out) = s.station.push_out(before, s.ship.position, station_ship_radius(s, &ship_model)) {
                        debug_last=format!("estacion: celdas/marco, desplazamiento {:.2}",(out-s.ship.position).length());
                        s.ship.position = out;
                        s.ship.speed = 0.0;
                        s.ship.bumped = true;
                    }
                } else if !tokens_blocking {
                    s.ship.resolve_collisions(&s.ships.flight, &s.system);
                }
                if s.ship.bumped {
                    shake = 0.25;
                }
                in_reach = s.system.planet_in_reach(s.ship.position);
                if !rl.is_key_down(KeyboardKey::KEY_E) && !rl.is_key_down(KeyboardKey::KEY_ENTER)
                    && !rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    interaction_armed = true;
                }
                let mut explore = interaction_armed && (pressed(&rl, KeyboardKey::KEY_E) || pressed(&rl, KeyboardKey::KEY_ENTER));
                if let Some(i) = in_reach {
                    // También se puede hacer clic en el botón con el mouse suelto
                    let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
                    let button = Rectangle::new(w / 2.0 - 170.0, h - 150.0, 340.0, 64.0);
                    if !smoke && accepts_input
                        && !rl.is_cursor_hidden()
                        && interaction_armed && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
                        && button.check_collision_point_rec(rl.get_mouse_position())
                    {
                        explore = true;
                    }
                    if explore {
                        let p = &s.system.planets[i].config;
                        if p.minigame.is_some() {
                            leaving = Some(i);
                        } else {
                            notice = Some((format!("El minijuego de {} todavía no está listo", p.name), 3.0));
                        }
                    }
                }
                if station_reentry_delay<=0.0 && s.station.in_reach(s.ship.position, station_ship_radius(s, &ship_model)) && explore && in_reach.is_none() {
                    interaction_armed=false;
                    entering_station=true;
                    s.ship.speed=0.0;
                }
            }
        }
        if let Some((_, t)) = &mut notice {
            *t -= dt;
        }
        if notice.as_ref().is_some_and(|(_, t)| *t <= 0.0) {
            notice = None;
        }
        shake = (shake - dt).max(0.0);
        // Fundido: aclara al entrar y oscurece al salir hacia el minijuego
        station_reentry_delay=(station_reentry_delay-dt).max(0.0);
        fade = if leaving.is_some() || entering_station { (fade + dt / FADE).min(1.0) } else { (fade - dt / FADE).max(0.0) };
        if entering_station && fade>=1.0 { rl.enable_cursor(); return Ok(Exit::Dock); }
        if let Some(i) = leaving {
            if fade >= 1.0 {
                rl.enable_cursor();
                return Ok(Exit::Visit(i));
            }
        }

        // --- Cámara --------------------------------------------------------------------------------------------
        let ship_len = s.ships.ships[s.ship_index].length;
        // Trasladar la cámara con la nave evita que el turbo la deje muy atrás.
        chase += s.ship.position - previous_ship_position;
        previous_ship_position = s.ship.position;
        let camera = match s.view {
            View::Overview => {
                let target = s.focus.map_or(s.system.center(), |destination| destination.position(s));
                let aspect = rl.get_screen_width() as f32 / rl.get_screen_height().max(1) as f32;
                s.orbit.camera(target, aspect)
            }
            View::Ship => {
                if let Some(elapsed) = intro_time.filter(|elapsed| *elapsed < ENTRY_STATION_HOLD + ENTRY_CAMERA_MOVE - 0.15) {
                    let t = ((elapsed - ENTRY_STATION_HOLD) / ENTRY_CAMERA_MOVE).clamp(0.0, 1.0);
                    let eased = t * t * (3.0 - 2.0 * t);
                    let turn = (entry_goal.yaw - entry_start_yaw + std::f32::consts::PI)
                        .rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
                    s.ship.yaw = entry_start_yaw + turn * eased;
                    s.ship.pitch = entry_start_pitch + (entry_goal.pitch - entry_start_pitch) * eased;
                }
                // Cámara de persecución: detrás y un poco arriba, siguiendo con suavidad
                let fwd = s.ship.forward();
                let wanted = chase_position(s.ship.position, fwd, s.ships.ships[s.ship_index].camera_length());
                if snap {
                    chase = wanted;
                } else {
                    chase += (wanted - chase) * (1.0 - (-7.0 * dt).exp());
                }
                // Al girar junto a una superficie, la cámara también queda afuera.
                let margin = NEAR_PLANE as f32 * 2.0;
                let sun_clearance = s.system.sun.radius + margin;
                if chase.length() < sun_clearance {
                    chase = chase.normalized() * sun_clearance;
                    debug_camera="Sol";
                }
                for planet in &s.system.planets {
                    if let Some(out) = planet.push_out(chase, margin) {
                        debug_camera="planeta";
                        chase = out;
                    }
                }
                if let Some(out) = s.station.push_out(chase, chase, margin) { chase = out; debug_camera="estacion: celdas/marco"; }
                let jitter = if shake > 0.0 {
                    let t = rl.get_time() as f32 * 60.0;
                    Vector3::new(t.sin(), (t * 1.3).cos(), 0.0) * shake * 0.6
                } else {
                    Vector3::zero()
                };
                let flight_target = s.ship.position + fwd * ship_len * 2.0;
                if let Some(elapsed) = intro_time {
                    // Begin with the station framing the ship, then move directly
                    // into the downward chase view as the ship turns toward the planets.
                    let t = ((elapsed - ENTRY_STATION_HOLD) / ENTRY_CAMERA_MOVE).clamp(0.0, 1.0);
                    let eased = t * t * (3.0 - 2.0 * t);
                    let radius = s.ships.ships[s.ship_index].camera_length() * 3.94;
                    let start_forward = Vector3::new(-entry_start_yaw.sin(), 0.0, -entry_start_yaw.cos());
                    let start_camera = s.ship.position + start_forward * radius
                        + Vector3::up() * s.ships.ships[s.ship_index].camera_length() * 1.28;
                    let position = start_camera * (1.0 - eased) + wanted * eased;
                    let target = (s.ship.position + Vector3::up() * ship_len * 0.95) * (1.0 - eased)
                        + flight_target * eased;
                    let next = elapsed + if fade <= 0.0 { dt } else { 0.0 };
                    intro_time = (next < ENTRY_STATION_HOLD + ENTRY_CAMERA_MOVE).then_some(next);
                    if intro_time.is_none() { chase = wanted; }
                    Camera3D::perspective(position, target, Vector3::up(), 44.0)
                } else if s.demo_video && demo.look() > 0.0 {
                    // El video, en la vuelta: la cámara se corre hacia afuera y mira en parte al planeta, para que se
                    // vea mientras la nave lo rodea
                    let k = demo.look();
                    let c = s.system.planets[DEMO_PLANET].position();
                    let out = (s.ship.position - c).normalized();
                    let len = s.ships.ships[s.ship_index].camera_length();
                    let position = chase + (out * len * 2.5 + Vector3::up() * len * 0.8) * k;
                    let target = flight_target + (c - flight_target) * (0.55 * k);
                    Camera3D::perspective(position, target, Vector3::up(), 44.0 + 16.0 * k)
                } else {
                    Camera3D::perspective(chase + jitter, flight_target, Vector3::up(), 44.0)
                }
            }
        };
        snap = false;
        // The launch safety smoke uses the real chase view from frame zero.
        // The entry shot only moves the camera; the ship follows the same safe path.
        if let Some(plan) = &departure {
            if smoke && (s.launch_smoke || s.docking_smoke.is_some()) && frame < 30 {
                if s.ship.bumped || (s.station.center()-s.ship.position).dot(s.ship.forward()) >= 0.0
                    || (camera.position-s.ship.position).dot(s.ship.forward()) >= 0.0
                    || s.station.push_out(camera.position,camera.position,NEAR_PLANE as f32*2.0).is_some() {
                    return Err("Colisión u orientación incorrecta al iniciar el vuelo".into());
                }
                if frame==1 && (s.ship.position-plan.ship.position).length()>0.01 {
                    return Err("El primer frame desplazó la nave sin acelerar".into());
                }
                if frame==0 && (camera.position-plan.camera_position).length()>0.01 {
                    return Err("La cámara inicial necesitó una corrección de posición".into());
                }
                if frame==14 && (s.ship.position-plan.ship.position).dot(plan.ship.forward())<=0.0 {
                    return Err("W no alejó la nave de la estación".into());
                }
            }
        }
        let camera = side_camera.unwrap_or(camera);
        if ending_ready {
            music.set_paused(true);
            if s.token_presentation.ending.elapsed == 0.0 {
                if let Some(r) = &riser { r.play(); }
            }
            let menu = super::ending::run(&mut rl, &thread, root, s.game.as_ref().unwrap(), &mut s.token_presentation,
                &s.system, &s.ship, &sun, &mut sun_shader, &planets, ship_model.as_ref(), &mut shader,
                &sky, &mut nebula, &mut bloom, &ui_bold, camera, s.ending_smoke)?;
            return Ok(if menu { Exit::Menu } else { Exit::Quit });
        }
        // También cubre el sistema cuando la nave se aleja más allá de la última órbita.
        let far = far_plane.max(camera.position.length() + outer * 2.0);
        unsafe { ffi::rlSetClipPlanes(NEAR_PLANE, far as f64) };

        // --- Proyecciones para el dibujo 2D (antes de empezar a dibujar) -------------------------------------
        let time = rl.get_time() as f32;
        let forward = (camera.target - camera.position).normalized();
        let right = forward.cross(Vector3::up()).normalized();
        let cam_up = right.cross(forward);
        let sky_frame = sky.project(&rl, camera);
        let in_front = |p: Vector3| (p - camera.position).dot(forward) > 0.0;
        let center = s.system.center();
        let sun_glow = in_front(center).then(|| {
            let at = rl.get_world_to_screen(center, camera);
            let edge = rl.get_world_to_screen(center + cam_up * s.system.sun.radius, camera);
            (at, (at - edge).length())
        });
        // Por planeta: dónde cae en la pantalla, su radio en píxeles (None si está detrás de la cámara) y hacia qué
        // lado de la pantalla queda (para la flecha del objetivo)
        let markers: Vec<Marker> = s
            .system
            .planets
            .iter()
            .map(|p| {
                let pos = p.position();
                let rel = pos - camera.position;
                let at = rl.get_world_to_screen(pos, camera);
                let edge = rl.get_world_to_screen(pos + cam_up * p.config.radius, camera);
                Marker {
                    at,
                    radius: in_front(pos).then(|| (at - edge).length()),
                    angle: (-rel.dot(cam_up)).atan2(rel.dot(right)),
                }
            })
            .collect();
        let visual_bounds: Vec<Option<DiskBounds>> = s.system.planets.iter().zip(&planets)
            .map(|(planet, (near, _))| {
                (planet.config.id == "smilingfriends")
                    .then(|| near.as_ref().and_then(|model| projected_disk_bounds(&rl, camera, planet, model)))
                    .flatten()
            })
            .collect();
        let (width, height) = (rl.get_screen_width(), rl.get_screen_height());
        let mut render_stats = RenderStats { planets: vec![PlanetRender::Culled(0); s.system.planets.len()] };
        let draw_mouse = rl.get_mouse_position();

        shader.begin_frame(center, camera.position, 0.16);
        sun_shader.begin_frame(center, camera.position, time);

        if s.token_presentation.abduction().is_some() {
            abductions.prepare(&mut rl, &thread)?;
        }
        bloom.prepare(&mut rl, &thread, width, height)?;
        // Con el renderizador por software, la escena 3D se arma en la CPU antes de empezar a dibujar
        if let Some(scene) = soft.as_mut().filter(|_| software_on) {
            scene.render(&mut rl, &thread, s, camera, center, time, culling, lod_on, &mut render_stats)?;
        }
        // --- Dibujo --------------------------------------------------------------------------------------------
        let mut d = crate::graphics::window::begin_drawing(&mut rl, &thread);
        if let Some(scene) = soft.as_ref().filter(|_| software_on) {
            d.clear_background(Color::BLACK);
            scene.renderer.draw(&mut d, width, height);
        } else {
        // El cielo y la escena 3D van a una textura, para sumarles el bloom (la interfaz va después, sin bloom)
        {
        let mut d = d.begin_texture_mode(&thread, &mut bloom.scene);
        d.clear_background(SKY_BACKGROUND);
        nebula.draw(&mut d, camera, width, height);
        sky.draw(&mut d, &sky_frame, time);
        if let Some((at, radius)) = sun_glow {
            draw_sun_glow(&mut d, at, radius, Vector2::new(width as f32, height as f32));
        }
        for (i, (planet, marker)) in s.system.planets.iter().zip(&markers).enumerate() {
            draw_planet_glow(&mut d, &planet.config.id, marker, visual_bounds[i],
                Vector2::new(width as f32, height as f32));
        }
        {
            let mut d3 = d.begin_mode3D(camera);
            if show_orbits {
                for p in &s.system.planets {
                    let radius = p.config.orbit_radius;
                    let point = |k: usize| {
                        let angle = k as f32 * std::f32::consts::TAU / ORBIT_SEGMENTS as f32;
                        center + Vector3::new(radius * angle.cos(), 0.0, -radius * angle.sin())
                    };
                    for k in 0..ORBIT_SEGMENTS {
                        d3.draw_line_3D(point(k), point(k + 1), Color::new(96, 139, 194, 64));
                    }
                }
            }
            d3.draw_model_ex(&sun, center, Vector3::up(), s.system.sun_spin_angle.to_degrees(), Vector3::one() * s.system.sun.radius, Color::WHITE);
            let station_pos = s.station.center();
            let station_model = if (camera.position - station_pos).length() > s.station.lod_distance {
                &station_far
            } else { &station_near };
            if (station_pos - camera.position).dot(forward) > -s.station.collision_radius {
                station_model.draw(&mut d3, &mut shader, s.station.rotation()*Matrix::translate(station_pos.x, station_pos.y, station_pos.z));
                crate::graphics::station_dock::draw(&mut d3,&s.station);
            }
            for (i, (p, (near, far))) in s.system.planets.iter().zip(&planets).enumerate() {
                let pos = p.position();
                let to = pos - camera.position;
                let full = near.as_ref().map_or(0, SpaceModel::triangles);
                // Descarte: un planeta fuera de la vista (su esfera fuera de la pirámide de la cámara) no se manda a
                // dibujar. Con F5 se apaga, para comparar
                if culling && !in_view(camera, width as f32 / height as f32, pos, p.config.radius * 1.6) {
                    render_stats.planets[i] = PlanetRender::Culled(full);
                    continue;
                }
                let world = p.orientation() * Matrix::translate(pos.x, pos.y, pos.z);
                // Nivel de detalle: lejos, la versión liviana (si tiene); con F6 se apaga, para comparar
                let lod = if lod_on && to.length() > p.config.radius * FAR_LOD { far.as_ref().or(near.as_ref()) } else { near.as_ref() };
                let used = lod.map_or(0, SpaceModel::triangles);
                render_stats.planets[i] = if used < full { PlanetRender::Far(used, full) } else { PlanetRender::Near(used) };
                shader.set_planet_fill(planet_fill(&p.config.id));
                shader.set_planet_toon(p.config.id == "smilingfriends");
                // Smiling Friends no brilla en el bloom, salvo Satanás (sus ojos y su fuego)
                shader.bloom_only = (p.config.id == "smilingfriends").then_some(&["Satan", "FireRed"][..]);
                let radius = p.config.radius;
                match p.config.id.as_str() {
                    "regularshow" => {
                        let up = Vector3::up().transform_with(p.orientation());
                        shader.set_planet_key(pos + up * (radius * 0.75),
                            Vector4::new(0.85, 0.95, 1.0, 0.38), radius * 1.35, false);
                    }
                    "halloween" => {
                        let sunward = (center - pos).normalized();
                        shader.set_planet_key(pos + (sunward + Vector3::up() * 0.65).normalized() * (radius * 1.6),
                            Vector4::new(1.0, 0.45, 0.25, 0.38), radius * 2.0, true);
                    }
                    _ => shader.set_planet_key(Vector3::zero(), Vector4::new(0.0, 0.0, 0.0, 0.0), 1.0, false),
                }
                match lod {
                    Some(model) => model.draw(&mut d3, &mut shader, world),
                    None => d3.draw_model_ex(&placeholder, pos, Vector3::up(), p.spin_angle.to_degrees(), Vector3::one() * p.config.radius, color(&p.config.color)),
                }
            }
            shader.set_planet_fill(Vector3::zero());
            shader.set_planet_toon(false);
            shader.bloom_only = None;
            shader.set_planet_key(Vector3::zero(), Vector4::new(0.0, 0.0, 0.0, 0.0), 1.0, false);
            let sp = &s.ship;
            let ship_world = Matrix::rotate_z(sp.bank) * Matrix::rotate_x(sp.pitch) * Matrix::rotate_y(sp.yaw)
                * Matrix::translate(sp.position.x, sp.position.y, sp.position.z);
            match &ship_model {
                Some(model) => model.draw(&mut d3, &mut shader, ship_world),
                None if s.ships.ships[s.ship_index].model.is_empty() => crate::graphics::ship_placeholder::draw(&mut d3, sp.position, Vector3::new(sp.pitch.to_degrees(), sp.yaw.to_degrees(), sp.bank.to_degrees()), ship_len),
                None => d3.draw_sphere(sp.position, ship_len * 0.3, Color::new(220, 220, 230, 255)),
            }
            if s.collision_debug {
                crate::graphics::station_debug::draw(&mut d3,&s.station,sp.position,station_ship_radius(s,&ship_model));
            }
        }
        }
        bloom.composite(&mut d, &thread);
        }
        if let Some(scene) = soft.as_ref().filter(|_| software_on) {
            let st = scene.renderer.stats;
            let label = format!("RENDER POR SOFTWARE (F7)  ·  {}x{}  ·  {} triángulos rasterizados de {}  ·  {:.1} ms",
                scene.renderer.width, scene.renderer.height, st.rasterized, st.submitted, st.ms);
            if s.demo_video && frame % 60 == 0 { println!("SOFT_FRAME_MS {:.1} tris={}", st.ms, st.rasterized); }
            let w = text_width(&ui_bold, &label, 18.0);
            draw_ui(&mut d, &ui_bold, &label, (width - w) / 2, 14, 18.0, super::holo::CYAN);
        }

        // --- Interfaz ------------------------------------------------------------------------------------------
        let white = Color::new(230, 232, 245, 255);
        let dim = Color::new(160, 168, 200, 255);
        // Distinct cyan diamond instead of a planetary ring. Off-screen target remains locatable.
        let station_at = rl_screen_marker(s.station.exit_position(), camera, width, height);
        let station_selected = if s.view == View::Ship { s.target } else { s.focus } == Some(Destination::Station);
        if station_at.1 || station_selected {
            let at = station_at.0;
            let c = Color::new(109, 222, 239, 255);
            for (a, b) in [(Vector2::new(0.0,-10.0), Vector2::new(10.0,0.0)),
                (Vector2::new(10.0,0.0), Vector2::new(0.0,10.0)),
                (Vector2::new(0.0,10.0), Vector2::new(-10.0,0.0)),
                (Vector2::new(-10.0,0.0), Vector2::new(0.0,-10.0))] { d.draw_line_v(at+a, at+b, c); }
            let label = format!("{} / Hangar  {:.0}", s.station.name, (s.station.exit_position() - s.ship.position).length());
            let label_w = text_width(&ui_regular, &label, 18.0);
            let label_x = (at.x as i32 - label_w / 2).clamp(8, (width-label_w-8).max(8));
            draw_ui(&mut d, &ui_regular, &label, label_x, at.y as i32 + 16, 18.0, c);
        }
        if s.view == View::Ship {
            for (i, (p, m)) in s.system.planets.iter().zip(&markers).enumerate() {
                let at = m.at;
                let is_target = s.target == Some(Destination::Planet(i));
                let c = if is_target { super::holo::ACCENT } else { Color::new(150, 170, 220, 170) };
                let distance = ((p.position() - s.ship.position).length() - p.config.radius).max(0.0);
                let logo = planet_logos.get(p.config.id.as_str());
                let visible_bounds = visual_bounds[i];
                let on_screen = m.radius.is_some() && at.x > 0.0 && at.y > 0.0 && at.x < width as f32 && at.y < height as f32;
                if let (true, Some(radius)) = (on_screen, m.radius) {
                    let r = radius.max(14.0) + 6.0;
                    let y = visible_bounds.map_or(at.y + r + 4.0, |bounds| bounds.bottom - 2.0);
                    if let Some(logo) = logo {
                        let height = draw_planet_logo(&mut d, logo, at.x, y);
                        let distance_label = format!("{distance:.0}");
                        // Del color del logo; la del objetivo, entera y en negrita
                        let (font, lc) = logo_tint(p.config.id.as_str(), is_target, c, &ui_regular, &ui_bold);
                        let w = text_width(font, &distance_label, 18.0);
                        draw_ui(&mut d, font, &distance_label, at.x as i32 - w / 2, (y + height + 2.0) as i32, 18.0, lc);
                    } else {
                        let label = format!("{}  {:.0}", p.config.name, distance);
                        let w = text_width(&ui_regular, &label, 18.0);
                        draw_ui(&mut d, &ui_regular, &label, at.x as i32 - w / 2, y as i32, 18.0, c);
                    }
                } else if is_target {
                    // Flecha en el borde de la pantalla hacia el objetivo
                    let angle = m.angle;
                    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
                    let rad = cx.min(cy) * 0.8;
                    let tip = Vector2::new(cx + angle.cos() * rad, cy + angle.sin() * rad);
                    let back = Vector2::new(angle.cos(), angle.sin()) * -18.0;
                    let side = Vector2::new(-angle.sin(), angle.cos()) * 10.0;
                    // La flecha, del color del logo del objetivo
                    let arrow = logo_color(p.config.id.as_str()).unwrap_or(c);
                    d.draw_triangle(tip, tip + back - side, tip + back + side, arrow);
                    d.draw_triangle(tip, tip + back + side, tip + back - side, arrow);
                    let label = format!("{distance:.0}");
                    let label_x = tip.x + back.x * 2.5;
                    let label_y = tip.y + back.y * 2.5 - 8.0;
                    if let Some(logo) = logo {
                        draw_planet_logo(&mut d, logo, label_x, label_y - 42.0);
                    } else {
                        let label = format!("{}  {:.0}", p.config.name, distance);
                        let w = text_width(&ui_regular, &label, 18.0);
                        draw_ui(&mut d, &ui_regular, &label, label_x as i32 - w / 2, label_y as i32, 18.0, c);
                    }
                    if logo.is_some() {
                        let (font, lc) = logo_tint(p.config.id.as_str(), is_target, c, &ui_regular, &ui_bold);
                        let w = text_width(font, &label, 18.0);
                        draw_ui(&mut d, font, &label, label_x as i32 - w / 2, label_y as i32, 18.0, lc);
                    }
                }
            }
            let (cx, cy) = (width / 2, height / 2);
            d.draw_line(cx - 8, cy, cx - 3, cy, white);
            d.draw_line(cx + 3, cy, cx + 8, cy, white);
            d.draw_line(cx, cy - 8, cx, cy - 3, white);
            d.draw_line(cx, cy + 3, cx, cy + 8, white);
            let ship_name = &s.ships.ships[s.ship_index].name;
            let boosting = s.ship.speed > s.ships.flight.cruise_speed + 1.0;
            // Una sombra suave detrás del encabezado, para leerlo sobre un planeta claro
            d.draw_rectangle_gradient_h(0, 0, 520, 150, Color::new(1, 6, 20, 150), Color::new(1, 6, 20, 0));
            draw_ui(&mut d, &ui_bold, ship_name, 16, 14, 25.0, white);
            // La misión: en qué paso va (cuántas medallas lleva) y qué hacer ahora, en una línea que cambia sola
            let won = s.game.as_ref().map_or(0, |g| g.progress().token_count());
            let label = if won >= 6 { "MISIÓN COMPLETA".to_owned() } else { format!("MISIÓN {}/6", won + 1) };
            draw_ui(&mut d, &ui_bold, &label, 16, 46, 17.0, super::holo::CYAN);
            let (step, color) = mission_step(s, in_reach);
            draw_ui(&mut d, &ui_regular, &step, 16, 66, 22.0, color.unwrap_or(white));
            let speed = format!("Velocidad {:.0}{}", s.ship.speed, if boosting { "  ·  TURBO" } else { "" });
            draw_ui(&mut d, &ui_regular, &speed, 16, 94, 18.0, dim);
            if let Some(i) = s.ship.carried_by {
                draw_ui(&mut d, &ui_regular, &format!("Acompañando la órbita de {}", s.system.planets[i].config.name), 16, 116, 18.0, dim);
            }
            if let Some(i) = in_reach {
                let p = &s.system.planets[i].config;
                let (w, h) = (width as f32, height as f32);
                let button = Rectangle::new(w / 2.0 - 190.0, h - 170.0, 380.0, 84.0);
                let title = format!("Explorar {}", p.name);
                let sub = if p.minigame.is_some() { "Explorar planeta" } else { "Minijuego en construcción" };
                super::holo::button(&mut d, &ui_bold, &ui_regular, button, "E", &title, Some(sub),
                    button.check_collision_point_rec(draw_mouse));
            }
            if station_reentry_delay<=0.0 && s.station.in_reach(s.ship.position, station_ship_radius(s, &ship_model)) {
                let button = Rectangle::new(width as f32 / 2.0 - 210.0, height as f32 - 150.0, 420.0, 60.0);
                super::holo::button(&mut d, &ui_bold, &ui_regular, button, "E", "Entrar a la estación espacial", None, false);
            }
        } else {
            let focus = s.focus.map_or("Sol", |destination| destination.name(s));
            let status = format!("Vista general   Tiempo x{}{}   Siguiendo: {focus}", s.time_scale, if s.paused { "  (pausa)" } else { "" });
            draw_ui(&mut d, &ui_bold, &status, 16, 14, 23.0, white);
            for (i, (p, m)) in s.system.planets.iter().zip(&markers).enumerate() {
                if let Some(r) = m.radius {
                    if let Some(logo) = planet_logos.get(p.config.id.as_str()) {
                        let max_height = if logo.width < logo.height { 34.0 } else { 29.0 };
                        let scale = (88.0 / logo.width as f32).min(max_height / logo.height as f32);
                        let logo_height = logo.height as f32 * scale;
                        let top = if p.config.id == "starwars" {
                            // This innermost planet sits beneath the sun in the entry overview.
                            m.at.y + r + 6.0
                        } else if p.config.id == "smilingfriends" {
                            visual_bounds[i].map_or(m.at.y - r - 5.0, |bounds| bounds.top - 24.0) - logo_height
                        } else {
                            m.at.y - r - 5.0 - logo_height
                        };
                        draw_planet_logo(&mut d, logo, m.at.x, top);
                    } else {
                        let w = text_width(&ui_bold, &p.config.name, 21.0);
                        draw_ui(&mut d, &ui_bold, &p.config.name, m.at.x as i32 - w / 2, (m.at.y - r) as i32 - 26, 21.0, white);
                    }
                }
            }
        }
        if let Some((text, t)) = &notice {
            let a = (t.min(1.0) * 255.0) as u8;
            let w = text_width(&ui_bold, text, 25.0);
            let accent = super::holo::ACCENT;
            draw_ui(&mut d, &ui_bold, text, width / 2 - w / 2, 70, 25.0, Color::new(accent.r, accent.g, accent.b, a));
        }
        let token_progress = s.game.as_ref().map_or(&empty_progress, |game| game.progress());
        token_textures.draw_hud(&mut d, &ui_bold, token_progress, width, height);
        if collection_progress > 0.0 {
            token_textures.draw_collection(&mut d, &ui_regular, &ui_bold, token_progress, width, height, collection_progress, None);
        }
        token_textures.draw_ceremony(&mut d, &ui_regular, &ui_bold, token_progress, &s.token_presentation, width, height);
        draw_help_overlay(&mut d, &ui_regular, &ui_bold, width, height, help_progress, help_open);
        if let Some(video) = abduction_video.as_ref().filter(|_| video_playing) {
            video.draw(&mut d, width, height);
        } else if let Some(franchise) = s.token_presentation.abduction() {
            abductions.set_hexbound_thanks(!s.token_presentation.abduction_is_initial());
            abductions.draw(&mut d, &thread, franchise, s.token_presentation.elapsed(), width, height);
        }
        token_textures.draw_ship_notice(&mut d, &ui_bold, &s.token_presentation, width, height);
        // (En el video no: guardar cada cuadro lo hace parecer lento)
        if !s.token_presentation.is_busy() && !s.demo_video { d.draw_fps(width - 96, 14); }
        if show_stats && !s.token_presentation.is_busy() {
            draw_render_stats(&mut d, &ui_regular, &ui_bold, &render_stats, s, culling, lod_on, width);
        }
        if s.collision_debug && !s.token_presentation.is_busy() {
            crate::graphics::station_debug::hud(&mut d,&s.station,s.ship.position,station_ship_radius(s,&ship_model),&debug_last,debug_camera);
        }
        if fade > 0.0 {
            d.draw_rectangle(0, 0, width, height, Color::new(0, 0, 0, (fade * 255.0) as u8));
        }
        if let Some(name) = &capture {
            save_frame(&captures.join(name), width, height)?;
        }
        if !smoke && d.is_key_pressed(KeyboardKey::KEY_F12) {
            let path = root.join("target/solarsystem.png");
            save_frame(&path, width, height)?;
            println!("Captura: {}", path.display());
        }
        drop(d);
        frame += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::rewards::record_result;

    #[test]
    fn el_objetivo_pasa_al_siguiente_planeta_sin_ganar() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let system = SolarSystem::new(&SolarSystemConfig::load(&root.join("config/solar-system.json")).unwrap());
        let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
        let mut game = GameState::new_game("minecraft", &catalog).unwrap();
        let id = |i: usize| system.planets[i].config.minigame.clone().unwrap();
        // Al empezar, el primer planeta; si se pierde, sigue ahí; si se gana, pasa al siguiente que falta
        assert_eq!(next_objective(&system, Some(&game), 0), Destination::Planet(0));
        record_result(&mut game, &id(0), MinigameResult::Defeat, &catalog).unwrap();
        assert_eq!(next_objective(&system, Some(&game), 0), Destination::Planet(0));
        record_result(&mut game, &id(0), MinigameResult::Victory, &catalog).unwrap();
        assert_eq!(next_objective(&system, Some(&game), 0), Destination::Planet(1));
        // Saltea los ya ganados, dando la vuelta
        record_result(&mut game, &id(2), MinigameResult::Victory, &catalog).unwrap();
        assert_eq!(next_objective(&system, Some(&game), 2), Destination::Planet(3));
        assert_eq!(next_objective(&system, Some(&game), 5), Destination::Planet(5));
        // Con todos ganados, la estación
        for i in 0..system.planets.len() {
            record_result(&mut game, &id(i), MinigameResult::Victory, &catalog).unwrap();
        }
        assert_eq!(next_objective(&system, Some(&game), 3), Destination::Station);
    }
}
