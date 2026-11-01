//! Sistema solar jugable (fases 1 a 6). Une la lógica de `solar_system` (órbitas, rotación, proximidad) y `ships`
//! (vuelo y choques) con la representación de `graphics::space`, y coordina las visitas: "Explorar planeta" cierra
//! esta ventana, abre el minijuego y al terminar vuelve al mismo sistema solar, con la nave junto al planeta.
use super::southpark_dev;
use super::southpark_entry::{SouthParkExit, SouthParkVisits};
use crate::graphics::resources::save_frame;
use crate::graphics::space::{draw_sun_glow, Fit, Sky, SpaceModel, SpaceShader, SunShader};
use crate::ships::{FlightInput, Ship, ShipsConfig};
use crate::solar_system::{hex_color, Planet, SolarSystem, SolarSystemConfig};
use raylib::{ffi, prelude::*};
use std::path::{Path, PathBuf};

const HELP_SHIP: &str = "W/S: acelerar / frenar | Mouse o A/D: girar | Shift: turbo | Tab: objetivo | Espacio: piloto automático\n\
                         E: explorar planeta | C: vista general | N: otra nave | O: órbitas | Esc: soltar el mouse | Ctrl+Q: salir";
const HELP_OVERVIEW: &str = "Arrastrar: girar cámara | Rueda: acercar | Tab: seguir planeta | Espacio: pausa | Arriba/Abajo: velocidad del tiempo\n\
                             C: volver a la nave | O: órbitas | F12: captura | Ctrl+Q: salir";

/// Planos de recorte: el sistema mide más de mil unidades de punta a punta (raylib usa 1000 por defecto).
const NEAR_PLANE: f64 = 0.5;
const FAR_PLANE: f64 = 12_000.0;
/// Distancia de la vista general y alejamiento máximo, en múltiplos de la órbita más lejana.
const OVERVIEW: f32 = 1.9;
const MAX_ZOOM_OUT: f32 = 3.5;
const ORBIT_SEGMENTS: usize = 256;
/// Más lejos que esto (en radios del planeta) se usa la versión liviana del modelo.
const FAR_LOD: f32 = 9.0;
/// Duración del fundido a negro al entrar o salir (segundos).
const FADE: f32 = 0.6;
const BACKGROUND: Color = Color::new(1, 1, 4, 255);

#[derive(Clone, Copy, PartialEq)]
enum View {
    Ship,
    Overview,
}

/// Cámara de la vista general: gira alrededor del Sol o del planeta que se sigue.
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
}

impl OrbitCamera {
    fn camera(&self, target: Vector3) -> Camera3D {
        let offset = Vector3::new(
            self.pitch.cos() * self.yaw.cos(),
            self.pitch.sin(),
            self.pitch.cos() * self.yaw.sin(),
        ) * self.distance;
        Camera3D::perspective(target + offset, target, Vector3::up(), 50.0)
    }
}

/// Lo que se conserva entre ventanas: al volver de un minijuego todo sigue donde estaba.
struct Session {
    system: SolarSystem,
    ships: ShipsConfig,
    ship_index: usize,
    ship: Ship,
    view: View,
    orbit: OrbitCamera,
    focus: Option<usize>,
    paused: bool,
    time_scale: f32,
    target: Option<usize>,
    show_orbits: bool,
    southpark: SouthParkVisits,
    returning_from: Option<usize>,
    /// Prueba de ida y vuelta a un minijuego (--smoke-visit): cuántas visitas van.
    visit_smoke: Option<u32>,
}

/// Planeta visto desde la cámara en este cuadro.
struct Marker {
    at: Vector2,
    radius: Option<f32>,
    angle: f32,
}

enum Exit {
    Quit,
    Visit(usize),
}

fn color(hex: &str) -> Color {
    let [r, g, b] = hex_color(hex).unwrap_or([255, 0, 255]);
    Color::new(r, g, b, 255)
}

fn outer_orbit(system: &SolarSystem) -> f32 {
    system.planets.iter().map(|p| p.config.orbit_radius).fold(0.0, f32::max)
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
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = root.join("config/solar-system.json");
    let mut ships_path = root.join("config/ships.json");
    let mut smoke = false;
    let mut visit_smoke = false;
    let mut overview = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--smoke-visit" => {
                smoke = true;
                visit_smoke = true;
            }
            "--overview" => overview = true,
            "--config" => config_path = PathBuf::from(args.next().ok_or("--config necesita la ruta de un JSON")?),
            "--ships" => ships_path = PathBuf::from(args.next().ok_or("--ships necesita la ruta de un JSON")?),
            "--help" | "-h" => {
                println!("cargo run --bin solarsystem -- [--overview] [--config PATH] [--ships PATH] [--smoke]");
                println!("Nave:\n{HELP_SHIP}\nVista general:\n{HELP_OVERVIEW}");
                println!("--smoke: recorrido automático sin ventana visible; guarda capturas en target/solarsystem/");
                println!("--smoke-visit: ida y vuelta a South Park (con su prueba de entrada) y captura al volver");
                return Ok(());
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let config = SolarSystemConfig::load(&config_path)?;
    let ships = ShipsConfig::load(&ships_path)?;
    let system = SolarSystem::new(&config);
    let outer = outer_orbit(&system);
    // La nave empieza entre el Sol y el primer planeta, mirándolo
    let first = system.planets[0].position();
    let mut ship = Ship::new(first * 0.62 + Vector3::new(0.0, 14.0, 0.0), 0.0);
    ship.look_at(first);
    let mut session = Session {
        ship_index: ships.ships.iter().position(|s| s.id == ships.active_ship).unwrap_or(0),
        ships,
        system,
        ship,
        view: if overview { View::Overview } else { View::Ship },
        orbit: OrbitCamera { yaw: 0.6, pitch: 0.75, distance: OVERVIEW * outer },
        focus: None,
        paused: false,
        time_scale: 1.0,
        target: Some(0),
        show_orbits: true,
        southpark: SouthParkVisits::default(),
        returning_from: None,
        visit_smoke: visit_smoke.then_some(0),
    };
    loop {
        match fly(&root, &mut session, smoke)? {
            Exit::Quit => return Ok(()),
            Exit::Visit(i) => {
                // La ventana del sistema solar ya se cerró: el minijuego abre la suya
                if session.system.planets[i].config.minigame.as_deref() == Some("southpark") {
                    // En la prueba, South Park corre su propia prueba de entrada y cierra solo
                    let args: Vec<String> = session.visit_smoke.map(|_| "--smoke-entry".to_string()).into_iter().collect();
                    match southpark_dev::run_visit_with_args(&mut session.southpark, args.into_iter())? {
                        SouthParkExit::WindowClosed if session.visit_smoke.is_none() => return Ok(()),
                        _ => {}
                    }
                }
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
    SpaceModel::load(rl, thread, &root.join(&s.model), shader, s.model_rotation_degrees, Fit::Length(s.length))
        .map_err(|e| eprintln!("Aviso: nave {}: {e}", s.id))
        .ok()
}

/// Una ventana del sistema solar, hasta salir del juego o ir a un minijuego.
fn fly(root: &Path, s: &mut Session, smoke: bool) -> Result<Exit, Box<dyn std::error::Error>> {
    let (mut rl, thread) = raylib::init()
        .size(1280, 800)
        .resizable()
        .msaa_4x()
        .title("Planet-Hoppers | Sistema solar")
        .build();
    rl.set_exit_key(None);
    rl.set_window_min_size(640, 480);
    if smoke {
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    }
    rl.set_target_fps(60);
    unsafe { ffi::rlSetClipPlanes(NEAR_PLANE, FAR_PLANE) };
    {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND);
        d.draw_text("Cargando el sistema solar...", 40, 40, 24, Color::RAYWHITE);
    }

    let mut shader = SpaceShader::load(&mut rl, &thread)?;
    let mut sun_shader = SunShader::load(&mut rl, &thread)?;
    let mut sun = rl.load_model_from_mesh(&thread, unsafe { Mesh::gen_mesh_sphere(&thread, 1.0, 48, 64).make_weak() })?;
    *sun.materials_mut()[0].shader_mut() = unsafe { WeakShader::from_raw(*sun_shader.shader.as_ref()) };
    let placeholder = rl.load_model_from_mesh(&thread, unsafe { Mesh::gen_mesh_sphere(&thread, 1.0, 24, 32).make_weak() })?;
    let planets = load_planet_models(&mut rl, &thread, root, &shader, &mut s.system);
    let mut ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
    let sky = Sky::new();
    let outer = outer_orbit(&s.system);
    let captures = root.join("target/solarsystem");
    if smoke {
        std::fs::create_dir_all(&captures)?;
    }

    if let Some(i) = s.returning_from.take() {
        park_ship(&mut s.ship, &s.system.planets[i], 2.2, false);
    }
    if s.view == View::Ship && !smoke {
        rl.disable_cursor();
    }
    let mut chase = s.ship.position - s.ship.forward() * 20.0;
    let mut snap = true;
    let mut fade = if smoke { 0.0 } else { 1.0_f32 }; // 1 = negro; baja al entrar
    let mut leaving: Option<usize> = None;
    let mut notice: Option<(String, f32)> = None;
    let mut frame = 0u32;
    let mut shake = 0.0_f32;
    let mut side_camera: Option<Camera3D> = None; // Solo para las capturas de prueba

    loop {
        if rl.window_should_close() {
            return Ok(Exit::Quit);
        }
        let ctrl = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL);
        if ctrl && rl.is_key_pressed(KeyboardKey::KEY_Q) {
            return Ok(Exit::Quit);
        }
        let dt = if smoke { 1.0 / 60.0 } else { rl.get_frame_time().min(0.1) };
        // Sin teclado durante las pruebas ni mientras se sale hacia un minijuego
        let accepts_input = !smoke && leaving.is_none();
        let pressed = |rl: &RaylibHandle, k: KeyboardKey| accepts_input && rl.is_key_pressed(k);
        let down = |rl: &RaylibHandle, k: KeyboardKey| accepts_input && rl.is_key_down(k);

        // --- Recorrido automático de prueba (--smoke) -------------------------------------------------------
        let mut capture: Option<String> = None;
        if let Some(visits) = s.visit_smoke {
            // Ida: estacionar junto a South Park y explorar. Vuelta: capturar la nave junto al planeta y salir
            match (visits, frame) {
                (0, 0) => {
                    park_ship(&mut s.ship, &s.system.planets[1], 1.7, true);
                    snap = true;
                }
                (0, 30) => {
                    capture = Some("before-visit.png".into());
                    if s.system.planet_in_reach(s.ship.position) != Some(1) {
                        return Err("La nave no quedó al alcance de South Park".into());
                    }
                    leaving = Some(1);
                }
                (1.., 60) => capture = Some("after-visit.png".into()),
                (1.., 61) => {
                    println!("VISIT_SMOKE_PASS visitas={visits}");
                    return Ok(Exit::Quit);
                }
                _ => {}
            }
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
                    s.focus = Some(i);
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
                    park_ship(&mut s.ship, &s.system.planets[1], 4.0, true);
                    snap = true;
                }
                f if f == 35 + n * 15 => capture = Some("ship.png".into()),
                f if f == 40 + n * 15 => {
                    park_ship(&mut s.ship, &s.system.planets[1], 1.7, true);
                    snap = true;
                }
                f if f == 55 + n * 15 => capture = Some("explore.png".into()),
                f if (60 + n * 15..60 + n * 15 + ships * 10).contains(&f) => {
                    // Cada nave de costado, para revisar hacia dónde apunta la nariz (con la cámara en +Z, a la izquierda)
                    let k = ((f - 60 - n * 15) / 10) as usize;
                    if (f - 60 - n * 15) % 10 == 0 {
                        s.ship_index = k;
                        ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
                        s.ship.position = Vector3::new(0.0, 80.0, 0.0);
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
                    if (f - 60 - n * 15) % 10 == 8 {
                        capture = Some(format!("ship-{}.png", s.ships.ships[k].id));
                    }
                }
                f if f >= 60 + n * 15 + ships * 10 => return Ok(Exit::Quit),
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
        if pressed(&rl, KeyboardKey::KEY_O) {
            s.show_orbits = !s.show_orbits;
        }
        if pressed(&rl, KeyboardKey::KEY_N) {
            // Cambio de nave para probar (la estación y los desbloqueos llegan en la etapa 8)
            s.ship_index = (s.ship_index + 1) % s.ships.ships.len();
            ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
            notice = Some((s.ships.ships[s.ship_index].name.clone(), 2.0));
        }

        // --- Actualización -------------------------------------------------------------------------------------
        let mut in_reach = None;
        match s.view {
            View::Overview => {
                if pressed(&rl, KeyboardKey::KEY_SPACE) {
                    s.paused = !s.paused;
                }
                if pressed(&rl, KeyboardKey::KEY_UP) {
                    s.time_scale = (s.time_scale * 2.0).min(16.0);
                }
                if pressed(&rl, KeyboardKey::KEY_DOWN) {
                    s.time_scale = (s.time_scale / 2.0).max(0.125);
                }
                if pressed(&rl, KeyboardKey::KEY_TAB) {
                    s.focus = match s.focus {
                        None => Some(0),
                        Some(i) if i + 1 < s.system.planets.len() => Some(i + 1),
                        Some(_) => None,
                    };
                    s.orbit.distance = match s.focus {
                        Some(i) => s.system.planets[i].config.radius * 5.0,
                        None => OVERVIEW * outer,
                    };
                }
                if !smoke && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let delta = rl.get_mouse_delta();
                    s.orbit.yaw += delta.x * 0.005;
                    s.orbit.pitch = (s.orbit.pitch + delta.y * 0.005).clamp(-1.45, 1.45);
                }
                let wheel = if smoke { 0.0 } else { rl.get_mouse_wheel_move() };
                if wheel != 0.0 {
                    let nearest = s.focus.map_or(s.system.sun.radius, |i| s.system.planets[i].config.radius) * 1.6;
                    s.orbit.distance = (s.orbit.distance * (1.0 - wheel * 0.1)).clamp(nearest, outer * MAX_ZOOM_OUT);
                }
                if !s.paused {
                    s.system.update(dt * s.time_scale);
                }
            }
            View::Ship => {
                if pressed(&rl, KeyboardKey::KEY_ESCAPE) {
                    if rl.is_cursor_hidden() {
                        rl.enable_cursor();
                    } else {
                        rl.disable_cursor();
                    }
                }
                if pressed(&rl, KeyboardKey::KEY_TAB) {
                    s.target = match s.target {
                        None => Some(0),
                        Some(i) if i + 1 < s.system.planets.len() => Some(i + 1),
                        Some(_) => None,
                    };
                }
                let axis = |neg: bool, pos: bool| (pos as i32 - neg as i32) as f32;
                let key = |k| down(&rl, k);
                let mouse = if !smoke && rl.is_cursor_hidden() {
                    let m = rl.get_mouse_delta();
                    (m.x, m.y)
                } else {
                    (0.0, 0.0)
                };
                // Piloto automático: apunta adonde va a estar el objetivo cuando la nave llegue
                let aim_at = s.target.filter(|_| key(KeyboardKey::KEY_SPACE)).map(|i| {
                    let p = &s.system.planets[i];
                    let distance = (p.position() - s.ship.position).length();
                    let eta = distance / s.ship.speed.abs().max(s.ships.flight.cruise_speed);
                    p.position() + p.velocity() * eta
                });
                let input = FlightInput {
                    throttle: axis(
                        key(KeyboardKey::KEY_S) || key(KeyboardKey::KEY_DOWN),
                        key(KeyboardKey::KEY_W) || key(KeyboardKey::KEY_UP),
                    ),
                    boost: key(KeyboardKey::KEY_LEFT_SHIFT) || key(KeyboardKey::KEY_RIGHT_SHIFT),
                    turn: (
                        axis(key(KeyboardKey::KEY_A) || key(KeyboardKey::KEY_LEFT), key(KeyboardKey::KEY_D) || key(KeyboardKey::KEY_RIGHT)),
                        axis(key(KeyboardKey::KEY_F), key(KeyboardKey::KEY_R)),
                    ),
                    mouse,
                    aim_at,
                };
                s.system.update(dt);
                if leaving.is_none() {
                    s.ship.update(dt, &input, &s.ships.flight, &s.system);
                } else {
                    s.ship.resolve_collisions(&s.ships.flight, &s.system);
                }
                if s.ship.bumped {
                    shake = 0.25;
                }
                in_reach = s.system.planet_in_reach(s.ship.position);
                let mut explore = pressed(&rl, KeyboardKey::KEY_E) || pressed(&rl, KeyboardKey::KEY_ENTER);
                if let Some(i) = in_reach {
                    // También se puede hacer clic en el botón con el mouse suelto
                    let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
                    let button = Rectangle::new(w / 2.0 - 170.0, h - 150.0, 340.0, 64.0);
                    if !smoke
                        && !rl.is_cursor_hidden()
                        && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
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
        fade = if leaving.is_some() { (fade + dt / FADE).min(1.0) } else { (fade - dt / FADE).max(0.0) };
        if let Some(i) = leaving {
            if fade >= 1.0 {
                rl.enable_cursor();
                return Ok(Exit::Visit(i));
            }
        }

        // --- Cámara --------------------------------------------------------------------------------------------
        let ship_len = s.ships.ships[s.ship_index].length;
        let camera = match s.view {
            View::Overview => {
                let target = s.focus.map_or(s.system.center(), |i| s.system.planets[i].position());
                s.orbit.camera(target)
            }
            View::Ship => {
                // Cámara de persecución: detrás y un poco arriba, siguiendo con suavidad
                let fwd = s.ship.forward();
                let wanted = s.ship.position - fwd * ship_len * 3.0 + Vector3::up() * ship_len * 0.9;
                if snap {
                    chase = wanted;
                } else {
                    chase += (wanted - chase) * (1.0 - (-7.0 * dt).exp());
                }
                let jitter = if shake > 0.0 {
                    let t = rl.get_time() as f32 * 60.0;
                    Vector3::new(t.sin(), (t * 1.3).cos(), 0.0) * shake * 0.6
                } else {
                    Vector3::zero()
                };
                Camera3D::perspective(chase + jitter, s.ship.position + fwd * ship_len * 2.0, Vector3::up(), 60.0)
            }
        };
        snap = false;
        let camera = side_camera.unwrap_or(camera);

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
        let (width, height) = (rl.get_screen_width(), rl.get_screen_height());

        shader.begin_frame(center, camera.position, 0.16);
        sun_shader.begin_frame(center, camera.position, time);

        // --- Dibujo --------------------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BACKGROUND);
        sky.draw(&mut d, &sky_frame, time);
        if let Some((at, radius)) = sun_glow {
            draw_sun_glow(&mut d, at, radius);
        }
        {
            let mut d3 = d.begin_mode3D(camera);
            if s.show_orbits {
                for p in &s.system.planets {
                    // Anillos sutiles que se apagan con la distancia a la cámara
                    let r = p.config.orbit_radius;
                    let at = |k: usize| {
                        let a = k as f32 / ORBIT_SEGMENTS as f32 * std::f32::consts::TAU;
                        center + Vector3::new(r * a.cos(), 0.0, -r * a.sin())
                    };
                    let reach = (camera.position - center).length() + outer;
                    for k in 0..ORBIT_SEGMENTS {
                        let far = (at(k) - camera.position).length() / reach;
                        let alpha = (55.0 * (1.2 - far).clamp(0.1, 1.0)) as u8;
                        d3.draw_line_3D(at(k), at(k + 1), Color::new(95, 110, 170, alpha));
                    }
                }
            }
            d3.draw_model_ex(&sun, center, Vector3::up(), s.system.sun_spin_angle.to_degrees(), Vector3::one() * s.system.sun.radius, Color::WHITE);
            for (p, (near, far)) in s.system.planets.iter().zip(&planets) {
                let pos = p.position();
                let to = pos - camera.position;
                if to.dot(forward) < -p.config.radius * 1.5 {
                    continue; // Detrás de la cámara
                }
                let world = p.orientation() * Matrix::translate(pos.x, pos.y, pos.z);
                let lod = if to.length() > p.config.radius * FAR_LOD { far.as_ref().or(near.as_ref()) } else { near.as_ref() };
                match lod {
                    Some(model) => model.draw(&mut d3, &mut shader, world),
                    None => d3.draw_model_ex(&placeholder, pos, Vector3::up(), p.spin_angle.to_degrees(), Vector3::one() * p.config.radius, color(&p.config.color)),
                }
            }
            let sp = &s.ship;
            let fwd = sp.forward();
            let ship_world = Matrix::rotate_z(sp.bank) * Matrix::rotate_x(sp.pitch) * Matrix::rotate_y(sp.yaw)
                * Matrix::translate(sp.position.x, sp.position.y, sp.position.z);
            match &ship_model {
                Some(model) => model.draw(&mut d3, &mut shader, ship_world),
                None => d3.draw_sphere(sp.position, ship_len * 0.3, Color::new(220, 220, 230, 255)),
            }
            if sp.speed > 1.0 {
                // Fuego del motor: más largo con más velocidad
                let k = (sp.speed / s.ships.flight.boost_speed).min(1.0);
                for (i, a) in [(0.0, 150u8), (0.35, 90u8), (0.7, 40u8)] {
                    let at = sp.position - fwd * ship_len * (0.55 + i * (0.3 + k));
                    d3.draw_sphere(at, ship_len * (0.12 - i * 0.05) * (0.6 + k), Color::new(140, 210, 255, a));
                }
            }
        }

        // --- Interfaz ------------------------------------------------------------------------------------------
        let white = Color::new(230, 232, 245, 255);
        let dim = Color::new(160, 168, 200, 255);
        if s.view == View::Ship {
            for (i, (p, m)) in s.system.planets.iter().zip(&markers).enumerate() {
                let at = m.at;
                let is_target = s.target == Some(i);
                let c = if is_target { Color::new(255, 214, 90, 255) } else { Color::new(150, 170, 220, 170) };
                let distance = ((p.position() - s.ship.position).length() - p.config.radius).max(0.0);
                let label = format!("{}  {:.0}", p.config.name, distance);
                let on_screen = m.radius.is_some() && at.x > 0.0 && at.y > 0.0 && at.x < width as f32 && at.y < height as f32;
                if let (true, Some(radius)) = (on_screen, m.radius) {
                    let r = radius.max(14.0) + 6.0;
                    d.draw_circle_lines(at.x as i32, at.y as i32, r, c);
                    let w = d.measure_text(&label, 16);
                    d.draw_text(&label, at.x as i32 - w / 2, (at.y + r) as i32 + 4, 16, c);
                } else if is_target {
                    // Flecha en el borde de la pantalla hacia el objetivo
                    let angle = m.angle;
                    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
                    let rad = cx.min(cy) * 0.8;
                    let tip = Vector2::new(cx + angle.cos() * rad, cy + angle.sin() * rad);
                    let back = Vector2::new(angle.cos(), angle.sin()) * -18.0;
                    let side = Vector2::new(-angle.sin(), angle.cos()) * 10.0;
                    d.draw_triangle(tip, tip + back - side, tip + back + side, c);
                    d.draw_triangle(tip, tip + back + side, tip + back - side, c);
                    let w = d.measure_text(&label, 16);
                    d.draw_text(&label, (tip.x + back.x * 2.5) as i32 - w / 2, (tip.y + back.y * 2.5) as i32 - 8, 16, c);
                }
            }
            let (cx, cy) = (width / 2, height / 2);
            d.draw_line(cx - 8, cy, cx - 3, cy, white);
            d.draw_line(cx + 3, cy, cx + 8, cy, white);
            d.draw_line(cx, cy - 8, cx, cy - 3, white);
            d.draw_line(cx, cy + 3, cx, cy + 8, white);
            let ship_name = &s.ships.ships[s.ship_index].name;
            let boosting = s.ship.speed > s.ships.flight.cruise_speed + 1.0;
            d.draw_text(ship_name, 16, 14, 22, white);
            d.draw_text(&format!("Velocidad {:.0}{}", s.ship.speed, if boosting { "  TURBO" } else { "" }), 16, 42, 18, dim);
            let target = s.target.map_or("ninguno".to_string(), |i| s.system.planets[i].config.name.clone());
            d.draw_text(&format!("Objetivo: {target}"), 16, 64, 18, dim);
            if let Some(i) = s.ship.carried_by {
                d.draw_text(&format!("Acompañando la órbita de {}", s.system.planets[i].config.name), 16, 86, 18, dim);
            }
            if let Some(i) = in_reach {
                let p = &s.system.planets[i].config;
                let (w, h) = (width as f32, height as f32);
                let button = Rectangle::new(w / 2.0 - 170.0, h - 150.0, 340.0, 64.0);
                let pulse = 0.75 + 0.25 * (time * 4.0).sin();
                d.draw_rectangle_rounded(button, 0.3, 8, Color::new(20, 24, 48, 220));
                d.draw_rectangle_rounded_lines(button, 0.3, 8, Color::new(255, 214, 90, (255.0 * pulse) as u8));
                let title = format!("Explorar {}", p.name);
                let tw = d.measure_text(&title, 24);
                d.draw_text(&title, (w / 2.0) as i32 - tw / 2, (h - 140.0) as i32, 24, white);
                let sub = if p.minigame.is_some() { "[E] o clic" } else { "[E]  (minijuego en construcción)" };
                let sw = d.measure_text(sub, 16);
                d.draw_text(sub, (w / 2.0) as i32 - sw / 2, (h - 110.0) as i32, 16, dim);
            }
        } else {
            let focus = s.focus.map_or("Sol", |i| s.system.planets[i].config.name.as_str());
            let status = format!("Vista general   Tiempo x{}{}   Siguiendo: {focus}", s.time_scale, if s.paused { "  (pausa)" } else { "" });
            d.draw_text(&status, 16, 14, 20, white);
            for (p, m) in s.system.planets.iter().zip(&markers) {
                if let Some(r) = m.radius {
                    let w = d.measure_text(&p.config.name, 18);
                    d.draw_text(&p.config.name, m.at.x as i32 - w / 2, (m.at.y - r) as i32 - 24, 18, white);
                }
            }
        }
        if let Some((text, t)) = &notice {
            let a = (t.min(1.0) * 255.0) as u8;
            let w = d.measure_text(text, 22);
            d.draw_text(text, width / 2 - w / 2, 70, 22, Color::new(255, 214, 90, a));
        }
        let help = if s.view == View::Ship { HELP_SHIP } else { HELP_OVERVIEW };
        d.draw_text(help, 16, height - 48, 15, Color::new(150, 156, 185, 255));
        d.draw_fps(width - 96, 14);
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
