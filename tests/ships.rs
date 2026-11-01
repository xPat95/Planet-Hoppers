use planet_hoppers::ships::{FlightInput, Ship, ShipsConfig};
use planet_hoppers::solar_system::{SolarSystem, SolarSystemConfig};
use raylib::prelude::Vector3;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn setup() -> (SolarSystem, ShipsConfig) {
    let system = SolarSystem::new(&SolarSystemConfig::load(&root().join("config/solar-system.json")).unwrap());
    (system, ShipsConfig::load(&root().join("config/ships.json")).unwrap())
}

/// Punto al costado de un planeta, a `radii` radios de su centro, del lado del Sol.
fn beside(system: &SolarSystem, i: usize, radii: f32) -> Vector3 {
    let p = &system.planets[i];
    p.position() - p.position().normalized() * p.config.radius * radii
}

#[test]
fn las_naves_del_config_existen() {
    let (_, ships) = setup();
    assert!(ships.active().is_some());
    for s in &ships.ships {
        assert!(root().join(&s.model).is_file(), "falta el modelo de {}", s.id);
    }
}

#[test]
fn la_escala_de_vuelo_conserva_naves_y_tiempos_de_aceleracion() {
    let (_, ships) = setup();
    let base: ShipsConfig = serde_json::from_str(&std::fs::read_to_string(root().join("config/ships.json")).unwrap()).unwrap();
    let factor = planet_hoppers::solar_system::scale::travel_scale();
    assert_eq!(ships.flight.cruise_speed, base.flight.cruise_speed * factor);
    assert_eq!(ships.flight.boost_speed, base.flight.boost_speed * factor);
    assert_eq!(ships.flight.acceleration, base.flight.acceleration * factor);
    assert_eq!(ships.flight.radius, base.flight.radius);
    for (actual, original) in ships.ships.iter().zip(&base.ships) {
        assert_eq!(actual.id, original.id);
        assert_eq!(actual.length, original.length);
        assert_eq!(actual.model, original.model);
    }
    let before = base.flight.boost_speed / (base.flight.acceleration * 2.0);
    let after = ships.flight.boost_speed / (ships.flight.acceleration * 2.0);
    assert!((before - after).abs() < 1e-5);
}

#[test]
fn avanza_hacia_donde_apunta() {
    let (system, ships) = setup();
    let mut ship = Ship::new(Vector3::new(0.0, system.sun.radius * 4.0, 0.0), 0.3);
    let start = ship.position;
    let input = FlightInput { throttle: 1.0, ..Default::default() };
    for _ in 0..120 {
        ship.update(1.0 / 60.0, &input, &ships.flight, &system);
    }
    let moved = ship.position - start;
    assert!(moved.length() > 10.0);
    assert!(moved.normalized().dot(ship.forward()) > 0.999);
    assert!(ship.speed <= ships.flight.cruise_speed + 1e-3);
}

#[test]
fn el_vuelo_no_depende_de_los_fps() {
    let (system, ships) = setup();
    let input = FlightInput { throttle: 1.0, turn: (1.0, 0.0), ..Default::default() };
    let mut a = Ship::new(Vector3::new(0.0, system.sun.radius * 4.0, 0.0), 0.0);
    let mut b = a.clone();
    for _ in 0..30 {
        a.update(1.0 / 30.0, &input, &ships.flight, &system);
    }
    for _ in 0..144 {
        b.update(1.0 / 144.0, &input, &ships.flight, &system);
    }
    assert!((a.yaw - b.yaw).abs() < 1e-3);
    let travel = planet_hoppers::solar_system::scale::travel_scale();
    assert!((a.speed - b.speed).abs() < 0.5 * travel);
    assert!((a.position - b.position).length() < travel, "{:?} {:?}", a.position, b.position);
}

#[test]
fn no_atraviesa_el_sol_ni_los_planetas() {
    let (system, ships) = setup();
    let mut ship = Ship::new(Vector3::new(0.0, 0.0, system.sun.radius + 100.0), 0.0); // Directo al Sol
    let input = FlightInput { throttle: 1.0, boost: true, ..Default::default() };
    for _ in 0..600 {
        ship.update(1.0 / 60.0, &input, &ships.flight, &system);
        assert!(ship.position.length() >= system.sun.radius + ships.flight.radius - 1e-3);
    }
    for i in 0..system.planets.len() {
        let target = system.planets[i].position();
        let mut ship = Ship::new(beside(&system, i, 4.0), 0.0);
        ship.look_at(target);
        for _ in 0..600 {
            ship.update(1.0 / 60.0, &input, &ships.flight, &system);
            assert!(system.planets[i].shape_distance(ship.position, ships.flight.radius) >= 1.0 - 1e-3);
        }
    }
}

#[test]
fn acompana_al_planeta_en_su_orbita() {
    let (mut system, ships) = setup();
    let i = 5; // South Park: el más lejano
    let mut ship = Ship::new(beside(&system, i, 2.0), 0.0);
    let offset = ship.position - system.planets[i].position();
    for _ in 0..180 {
        system.update(1.0 / 60.0);
        ship.update(1.0 / 60.0, &FlightInput::default(), &ships.flight, &system);
    }
    assert_eq!(ship.carried_by, Some(i));
    let drift = (ship.position - system.planets[i].position() - offset).length();
    assert!(drift < 0.5, "se separó {drift}");
}

#[test]
fn puede_alcanzar_cualquier_planeta() {
    let (system, ships) = setup();
    // La nave es mucho más rápida que cualquier planeta en su órbita
    let fastest = system.planets.iter().map(|p| p.velocity().length()).fold(0.0, f32::max);
    assert!(ships.flight.cruise_speed > fastest * 4.0, "{fastest}");
    // La misma predicción que usa la escena: todos los destinos, en normal y turbo,
    // también con el dt máximo del juego (0,1 s).
    for boost in [false, true] {
        for dt in [1.0 / 60.0, 0.1] {
            for i in 0..system.planets.len() {
                // Salida inicial y viajes desde cada uno de los otros planetas.
                for origin in 0..=system.planets.len() {
                    if origin == i { continue; }
                    let mut system = system.clone();
                    let start = if origin == system.planets.len() {
                        system.departure_position()
                    } else {
                        beside(&system, origin, 2.2)
                    };
                    let mut ship = Ship::new(start, 0.0);
                    ship.look_at(system.planets[0].position());
                    let mut reached = false;
                    for _ in 0..(180.0 / dt) as usize {
                        let aim = ship.autopilot_aim(i, &ships.flight, &system);
                        let input = FlightInput { throttle: 1.0, boost, aim_at: Some(aim), ..Default::default() };
                        system.update(dt);
                        ship.update(dt, &input, &ships.flight, &system);
                        if system.planet_in_reach(ship.position) == Some(i) {
                            reached = true;
                            break;
                        }
                    }
                    assert!(reached, "{} origin={origin} boost={boost} dt={dt}", system.planets[i].config.id);
                }
            }
        }
    }
}

#[test]
fn explorar_solo_cerca_del_planeta() {
    let (system, _) = setup();
    for i in 0..system.planets.len() {
        assert_eq!(system.planet_in_reach(beside(&system, i, 1.5)), Some(i));
        assert_eq!(system.planet_in_reach(beside(&system, i, 3.5)), None);
    }
    assert_eq!(system.planet_in_reach(Vector3::zero()), None);
}
