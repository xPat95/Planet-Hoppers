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
fn avanza_hacia_donde_apunta() {
    let (system, ships) = setup();
    let mut ship = Ship::new(Vector3::new(0.0, 200.0, 0.0), 0.3);
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
    let mut a = Ship::new(Vector3::new(0.0, 200.0, 0.0), 0.0);
    let mut b = a.clone();
    for _ in 0..30 {
        a.update(1.0 / 30.0, &input, &ships.flight, &system);
    }
    for _ in 0..144 {
        b.update(1.0 / 144.0, &input, &ships.flight, &system);
    }
    assert!((a.yaw - b.yaw).abs() < 1e-3);
    assert!((a.speed - b.speed).abs() < 0.5);
    assert!((a.position - b.position).length() < 1.0, "{:?} {:?}", a.position, b.position);
}

#[test]
fn no_atraviesa_el_sol_ni_los_planetas() {
    let (system, ships) = setup();
    let mut ship = Ship::new(Vector3::new(0.0, 0.0, 120.0), 0.0); // Mira hacia -Z: directo al Sol
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
    let i = 5; // Star Wars: el más lejano
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
    // Con piloto automático llega al alcance de "Explorar" del planeta más lejano
    let (mut system, i) = (system, 5);
    let mut ship = Ship::new(Vector3::new(0.0, 30.0, 80.0), 0.0);
    let mut reached = false;
    for _ in 0..60 * 60 {
        let p = &system.planets[i];
        let eta = (p.position() - ship.position).length() / ships.flight.boost_speed;
        let aim = p.position() + p.velocity() * eta;
        let near = system.planet_in_reach(ship.position) == Some(i);
        let input = FlightInput { throttle: if near { 0.0 } else { 1.0 }, boost: true, aim_at: Some(aim), ..Default::default() };
        system.update(1.0 / 60.0);
        ship.update(1.0 / 60.0, &input, &ships.flight, &system);
        if system.planet_in_reach(ship.position) == Some(i) {
            reached = true;
            break;
        }
    }
    assert!(reached);
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
