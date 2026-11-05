//! Sistema solar jugable (fases 1 a 6). Une la lógica de `solar_system` (órbitas, rotación, proximidad) y `ships`
//! (vuelo y choques) con la representación de `graphics::space`, y coordina las visitas: "Explorar planeta" cierra
//! esta ventana, abre el minijuego y al terminar vuelve al mismo sistema solar, con la nave junto al planeta.
use super::minigame_host::{self, MinigameVisits};
use crate::gameplay::planet_visit::PlanetReturnPose;
use crate::gameplay::station::{chase_position, DepartureBounds, Station};
use crate::minigames::common::host::{MinigameCompletion, MinigameExit, MinigameResult};
use crate::graphics::resources::save_frame;
use crate::graphics::space::{draw_sun_glow, Fit, Sky, SpaceModel, SpaceShader, SunShader, SKY_BACKGROUND};
use crate::ships::{FlightInput, Ship, ShipsConfig};
use crate::gameplay::selection::{GameState, ShipCatalog, SelectionTransition};
use crate::solar_system::{hex_color, Planet, SolarSystem, SolarSystemConfig};
use raylib::{ffi, prelude::*};
use std::path::{Path, PathBuf};

const HELP_SHIP: &str = "W/S: acelerar / frenar | Mouse o A/D: girar | Espacio: subir | Shift: turbo | Tab: objetivo | B: piloto automático\n\
                         E: explorar / entrar a estación | C: vista general | N: otra nave | O: órbitas | Esc: soltar el mouse | Ctrl+Q: salir";
const HELP_OVERVIEW: &str = "Arrastrar: girar cámara | Rueda: acercar | Tab: seguir objetivo | P: pausa | Arriba/Abajo: velocidad del tiempo\n\
                             C: volver a la nave | O: órbitas | F12: captura | Ctrl+Q: salir";

/// El plano cercano conserva el detalle de las naves; el lejano se calcula con el sistema.
const NEAR_PLANE: f64 = 0.5;
/// Distancia de la vista general y alejamiento máximo, en múltiplos de la órbita más lejana.
const OVERVIEW: f32 = 2.7;
const MAX_ZOOM_OUT: f32 = 3.5;
const ORBIT_SEGMENTS: usize = 256;
/// Más lejos que esto (en radios del planeta) se usa la versión liviana del modelo.
const FAR_LOD: f32 = 9.0;
/// Duración del fundido a negro al entrar o salir (segundos).
const FADE: f32 = 0.6;

#[derive(Clone, Copy, PartialEq)]
enum View {
    Ship,
    Overview,
}

#[derive(Clone, Copy, PartialEq)]
enum Destination {
    Planet(usize),
    Station,
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
        Camera3D::perspective(target + offset, target, Vector3::up(), 45.0)
    }
}

/// Lo que se conserva entre ventanas: al volver de un minijuego todo sigue donde estaba.
struct Session {
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
    show_orbits: bool,
    minigames: MinigameVisits,
    last_completion: Option<MinigameCompletion>,
    return_smoke: bool,
    launch_smoke: bool,
    station_smoke: bool,
    collision_debug: bool,
    docking_smoke: Option<u32>,
    departing: bool,
    game: Option<GameState>,
    returning_from: Option<usize>,
    return_pose: Option<PlanetReturnPose>,
    minecraft_smoke: bool,
    starwars_smoke: bool,
    /// Prueba de ida y vuelta a un minijuego (--smoke-visit): cuántas visitas van.
    visit_smoke: Option<u32>,
}

/// Planeta visto desde la cámara en este cuadro.
struct Marker {
    at: Vector2,
    radius: Option<f32>,
    angle: f32,
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
    Visit(usize),
    Dock,
}

fn color(hex: &str) -> Color {
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
    run_session(args, Some(game))?.ok_or_else(|| "Se perdió el estado de la aventura".into())
}

fn run_session(args: impl Iterator<Item = String>, game: Option<GameState>) -> Result<Option<GameState>, Box<dyn std::error::Error>> {
    let mut game = game;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut config_path = root.join("config/solar-system.json");
    let mut ships_path = root.join("config/ships.json");
    let mut smoke = false;
    let mut visit_smoke = false;
    let mut minecraft_smoke = false;
    let mut starwars_smoke = false;
    let mut return_smoke = false;
    let mut launch_smoke = false;
    let mut station_smoke = false;
    let mut collision_debug = false;
    let mut docking_smoke = false;
    let mut overview = false;
    let mut args = args;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--smoke" => smoke = true,
            "--smoke-launch" => { smoke = true; launch_smoke = true; }
            "--smoke-station" => { smoke = true; station_smoke = true; }
            "--debug-collisions" => collision_debug = true,
            "--smoke-docking" => { smoke = true; docking_smoke = true; }
            "--smoke-returns" => { smoke = true; visit_smoke = true; return_smoke = true; }
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
                println!("--smoke-visit: ida y vuelta a South Park (con su prueba de entrada) y captura al volver");
                println!("--smoke-starwars: visita al túnel con la nave activa y regreso al sistema solar");
                println!("--smoke-minecraft: dos visitas a la aldea y Diamond Rush, regreso confirmado y misma nave");
                println!("--smoke-returns: victoria, derrota, pausa/reinicio/abandono y cuatro visitas a South Park");
                println!("--smoke-station: estación fija, aproximación y controles de ascenso/piloto");
                println!("--smoke-docking: tres entradas al hangar, selección, cambio, cancelación y despegue");
                return Ok(game);
            }
            _ => return Err(format!("Opción desconocida: {arg}").into()),
        }
    }
    let config = SolarSystemConfig::load(&config_path)?;
    let mut ships = ShipsConfig::load(&ships_path)?;
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
    let outer = outer_orbit(&system);
    let station = Station::load(&root)?;
    // The actual departure is resolved after loading the fitted visual bounds.
    let mut ship = Ship::new(station.dock_target(), 0.0);
    ship.look_at(ship.position + station.exit_forward());
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
        target: Some(Destination::Planet(0)),
        show_orbits: true,
        minigames: MinigameVisits::default(),
        last_completion: None,
        return_smoke,
        launch_smoke,
        station_smoke,
        collision_debug,
        docking_smoke: docking_smoke.then_some(0),
        departing: true,
        game,
        returning_from: None,
        return_pose: None,
        minecraft_smoke,
        starwars_smoke,
        visit_smoke: visit_smoke.then_some(0),
    };
    loop {
        match fly(&root, &mut session, smoke)? {
            Exit::Quit => return Ok(session.game),
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
                let args = if session.starwars_smoke {
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
                let mut on_result = |result| {
                    if let Some(game) = &mut session.game {
                        crate::gameplay::rewards::record_result(game, &id, result, &catalog)?;
                    }
                    Ok(())
                };
                match minigame_host::run_with_ship_and_results(&id, &mut session.minigames, &selected_ship, &root, args.into_iter(), &mut on_result)? {
                    MinigameExit::WindowClosed if session.visit_smoke.is_none() => return Ok(session.game),
                    MinigameExit::Returned(result) => {
                        let completion = MinigameCompletion { planet_id: planet.id.clone(), minigame_id: id, result };
                        println!("MINIGAME_RETURN {}", serde_json::to_string(&completion)?);
                        session.last_completion = Some(completion);
                    }
                    _ => {}
                }
                if (session.return_smoke || session.minecraft_smoke || session.starwars_smoke) && (session.ship_index != ship_index
                    || session.system.planets.iter().map(|p| p.orbit_angle).collect::<Vec<_>>() != orbit_angles) {
                    return Err("La visita alteró la nave o las órbitas del anfitrión".into());
                }
                session.view = View::Ship;
                session.target = Some(Destination::Planet(i));
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
    let far_plane = outer_orbit(&s.system) * (2.0 * MAX_ZOOM_OUT + 2.0);
    {
        let mut d = rl.begin_drawing(&thread);
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
    let sky = Sky::new();
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
        let accepts_input = (!smoke || s.launch_smoke || s.station_smoke || s.minecraft_smoke || s.docking_smoke.is_some()) && leaving.is_none() && !entering_station;
        let pressed = |rl: &RaylibHandle, k: KeyboardKey| accepts_input && rl.is_key_pressed(k);
        let down = |rl: &RaylibHandle, k: KeyboardKey| accepts_input && rl.is_key_down(k);
        if pressed(&rl,KeyboardKey::KEY_F3) {s.collision_debug=!s.collision_debug;}
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
            let destination = if s.starwars_smoke { s.system.planets.iter().position(|p|p.config.id=="starwars").unwrap() } else if s.minecraft_smoke { s.system.planets.iter().position(|p| p.config.id == "minecraft").unwrap() } else { southpark };
            let total = if s.minecraft_smoke { 2 } else if s.return_smoke { 4 } else { 1 };
            if s.minecraft_smoke {
                let event = if visits > 0 && frame == 0 || visits < total && frame == 30 { Some(2) }
                    else if frame == 10 || frame == 31 { Some(1) } else { None };
                if let Some(kind) = event { unsafe { ffi::PlayAutomationEvent(ffi::AutomationEvent {
                    frame: 0, type_: kind, params: [KeyboardKey::KEY_E as i32, 0, 0, 0],
                }); } }
                if visits > 0 && frame == 12 && leaving.is_some() {
                    return Err("Held E re-entered Minecraft before a new press".into());
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
                    if s.game.is_none() { s.ship_index = visits as usize % s.ships.ships.len(); }
                    ship_model = load_ship_model(&mut rl, &thread, root, &shader, s);
                    park_ship(&mut s.ship, &s.system.planets[destination], 1.7, true);
                    snap = true;
                }
                (_, 30) if visits < total => {
                    capture = Some("before-visit.png".into());
                    if s.system.planet_in_reach(s.ship.position) != Some(destination) {
                        return Err("La nave no quedó al alcance de South Park".into());
                    }
                    if !s.minecraft_smoke { leaving = Some(destination); }
                }
                (_, 60) if visits == total => capture = Some("after-visit.png".into()),
                (_, 61) if visits == total => {
                    if s.return_smoke && s.last_completion.as_ref().map(|c| c.result) != Some(MinigameResult::Abandoned) {
                        return Err("Falta resultado del último regreso".into());
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
        if pressed(&rl, KeyboardKey::KEY_O) {
            s.show_orbits = !s.show_orbits;
        }
        if pressed(&rl, KeyboardKey::KEY_N) {
            // Development keeps the complete catalog; adventures respect ownership.
            let next = (1..=s.ships.ships.len()).map(|step| (s.ship_index + step) % s.ships.ships.len())
                .find(|i| s.game.as_ref().is_none_or(|game| game.is_unlocked(&s.ships.ships[*i].id))).unwrap_or(s.ship_index);
            if next != s.ship_index {
                if let Some(game) = &mut s.game { game.activate(&s.ships.ships[next].id)?; }
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
                if !smoke && rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                    let delta = rl.get_mouse_delta();
                    s.orbit.yaw += delta.x * 0.005;
                    s.orbit.pitch = (s.orbit.pitch + delta.y * 0.005).clamp(-1.45, 1.45);
                }
                let wheel = if smoke { 0.0 } else { rl.get_mouse_wheel_move() };
                if wheel != 0.0 {
                    let nearest = s.focus.map_or(s.system.sun.radius, |destination| destination.radius(s)) * 1.6;
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
                } else if !smoke && rl.is_cursor_hidden() {
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
                let mut input = FlightInput {
                    ascend: key(KeyboardKey::KEY_SPACE),
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
                if s.target==Some(Destination::Station) && input.aim_at.is_some() {
                    s.station.assist_approach(&s.ship,&mut input,&s.ships.flight);
                }
                s.system.update(dt);
                if leaving.is_none() && !entering_station {
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
                } else {
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
                    if !smoke
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
                Camera3D::perspective(chase + jitter, s.ship.position + fwd * ship_len * 2.0, Vector3::up(), 47.0)
            }
        };
        snap = false;
        // Launch starts in the real chase view, behind the nose. No reverse
        // reveal camera or interpolation through the station during takeoff.
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
        let (width, height) = (rl.get_screen_width(), rl.get_screen_height());

        shader.begin_frame(center, camera.position, 0.16);
        sun_shader.begin_frame(center, camera.position, time);

        // --- Dibujo --------------------------------------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(SKY_BACKGROUND);
        sky.draw(&mut d, &sky_frame);
        if let Some((at, radius)) = sun_glow {
            draw_sun_glow(&mut d, at, radius, Vector2::new(width as f32, height as f32));
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
            let station_pos = s.station.center();
            let station_model = if (camera.position - station_pos).length() > s.station.lod_distance {
                &station_far
            } else { &station_near };
            if (station_pos - camera.position).dot(forward) > -s.station.collision_radius {
                station_model.draw(&mut d3, &mut shader, s.station.rotation()*Matrix::translate(station_pos.x, station_pos.y, station_pos.z));
                crate::graphics::station_dock::draw(&mut d3,&s.station);
            }
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
                None if s.ships.ships[s.ship_index].model.is_empty() => crate::graphics::ship_placeholder::draw(&mut d3, sp.position, Vector3::new(sp.pitch.to_degrees(), sp.yaw.to_degrees(), sp.bank.to_degrees()), ship_len),
                None => d3.draw_sphere(sp.position, ship_len * 0.3, Color::new(220, 220, 230, 255)),
            }
            if s.collision_debug {
                crate::graphics::station_debug::draw(&mut d3,&s.station,sp.position,station_ship_radius(s,&ship_model));
            }
            if sp.speed > 1.0 {
                // Compact supplemental exhaust; the GLB's own engine effects remain.
                let k = (sp.speed / s.ships.flight.boost_speed).min(1.0);
                let rear = ship_model.as_ref().map_or(ship_len * 0.5, |m| m.half_extents.z);
                for (offset, radius, alpha) in crate::graphics::ship_presentation::exhaust_spheres(ship_len, rear, k) {
                    let at = sp.position - fwd * offset;
                    d3.draw_sphere(at, radius, Color::new(140, 210, 255, alpha));
                }
            }
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
            let label_x = (at.x as i32 - d.measure_text(&label,16)/2).clamp(8, (width-d.measure_text(&label,16)-8).max(8));
            d.draw_text(&label, label_x, at.y as i32 + 16, 16, c);
        }
        if s.view == View::Ship {
            for (i, (p, m)) in s.system.planets.iter().zip(&markers).enumerate() {
                let at = m.at;
                let is_target = s.target == Some(Destination::Planet(i));
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
            let target = s.target.map_or("ninguno", |destination| destination.name(s));
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
                let sub = if p.minigame.is_some() { "[E] Explorar planeta" } else { "[E]  (minijuego en construcción)" };
                let sw = d.measure_text(sub, 16);
                d.draw_text(sub, (w / 2.0) as i32 - sw / 2, (h - 110.0) as i32, 16, dim);
            }
            if station_reentry_delay<=0.0 && s.station.in_reach(s.ship.position, station_ship_radius(s, &ship_model)) {
                let text = "[E] Entrar a la estación espacial";
                d.draw_text(text, (width - d.measure_text(text, 20)) / 2, height - 112, 20, Color::new(109,222,239,255));
            }
        } else {
            let focus = s.focus.map_or("Sol", |destination| destination.name(s));
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
        if s.collision_debug {
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
