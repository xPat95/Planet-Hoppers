use planet_hoppers::solar_system::{hex_color, SolarSystem, SolarSystemConfig, PLANET_IDS};
use std::{f32::consts::TAU, path::PathBuf};

fn config() -> SolarSystemConfig {
    SolarSystemConfig::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/solar-system.json"))
        .expect("config/solar-system.json válido")
}

#[test]
fn el_orden_orbital_es_el_definitivo_con_la_escala_comun() {
    let config = config();
    let actual: Vec<_> = config.planets.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(actual, ["starwars", "smilingfriends", "minecraft", "regularshow", "southpark", "halloween"]);
    let radii: Vec<_> = config.planets.iter().map(|p| p.orbit_radius).collect();
    assert_eq!(radii, [1320.0, 1896.0, 2472.0, 3048.0, 3624.0, 4200.0]);
    assert!(config.planets.windows(2).all(|pair| pair[1].orbit_radius - pair[0].orbit_radius == 576.0));
    let angles: Vec<_> = config.planets.iter().map(|p| p.start_angle_degrees).collect();
    assert_eq!(angles, [300.0, 240.0, 180.0, 120.0, 60.0, 0.0]);
    // The six starting positions occupy the full circle instead of forming a row.
    assert!(angles.windows(2).all(|pair| pair[0] - pair[1] == 60.0));
}

#[test]
fn la_escala_conserva_proporciones_y_un_punto_de_partida_seguro() {
    let config = config();
    let system = SolarSystem::new(&config);
    assert_eq!(config.sun.radius, 540.0);
    let radii: Vec<_> = config.planets.iter().map(|p| p.radius).collect();
    assert_eq!(radii, [264.0, 264.0, 192.0, 240.0, 216.0, 216.0]);
    let start = system.departure_position();
    assert!(start.length() > config.sun.radius + 100.0);
    for p in &system.planets {
        assert!(p.shape_distance(start, 5.0) > 1.0);
    }
    assert_eq!(system.planet_in_reach(start), None);
}

#[test]
fn la_proximidad_conserva_la_identidad_y_el_minijuego_en_las_nuevas_orbitas() {
    let mut system = SolarSystem::new(&config());
    system.update(120.0);
    for id in PLANET_IDS {
        let planet = system.planet(id).unwrap();
        let point = planet.position() + planet.spin_axis() * planet.config.radius * 1.5;
        let index = system.planet_in_reach(point).expect("planeta al alcance");
        let reached = &system.planets[index].config;
        assert_eq!(reached.id, id);
        // Each connected planet dispatches by its own identity.
        let expected = if matches!(id, "southpark" | "smilingfriends" | "minecraft" | "starwars") { Some(id) } else { None };
        assert_eq!(reached.minigame.as_deref(), expected, "{id}");
        assert!(reached.model.as_deref().unwrap().contains(&format!("/planets/{id}/")));
    }
}

#[test]
fn hay_seis_planetas_en_orbitas_separadas_alrededor_del_sol() {
    let config = config();
    let system = SolarSystem::new(&config);
    assert_eq!(system.planets.len(), 6);
    for id in PLANET_IDS {
        assert!(system.planet(id).is_some(), "falta {id}");
    }
    let mut last = config.sun.radius;
    for p in &system.planets {
        // Cada planeta está sobre su círculo, en el plano XZ, sin tocar al Sol ni a su vecino
        let pos = p.position();
        assert!((pos.x.hypot(pos.z) - p.config.orbit_radius).abs() < 1e-3);
        assert_eq!(pos.y, 0.0);
        assert!(p.config.orbit_radius - p.config.radius > last);
        last = p.config.orbit_radius + p.config.radius;
    }
}

#[test]
fn los_de_afuera_tardan_mas_en_dar_la_vuelta() {
    let system = SolarSystem::new(&config());
    for pair in system.planets.windows(2) {
        assert!(pair[0].config.orbit_speed_degrees.abs() > pair[1].config.orbit_speed_degrees.abs());
    }
}

#[test]
fn el_movimiento_no_depende_de_los_fps() {
    let config = config();
    let mut slow = SolarSystem::new(&config);
    let mut fast = SolarSystem::new(&config);
    for _ in 0..30 {
        slow.update(1.0 / 30.0);
    }
    for _ in 0..144 {
        fast.update(1.0 / 144.0);
    }
    for (a, b) in slow.planets.iter().zip(&fast.planets) {
        // Se conserva la tolerancia en unidades de diseño: f32 redondea en un mundo mayor.
        let tolerance = 1e-2 * planet_hoppers::solar_system::scale::WORLD_SCALE;
        assert!((a.position() - b.position()).length() < tolerance, "{}", a.config.id);
        // Diferencia de ángulos dando la vuelta (0 y 2π son el mismo giro)
        let diff = (a.spin_angle - b.spin_angle).rem_euclid(TAU);
        assert!(diff.min(TAU - diff) < 1e-3, "{}", a.config.id);
    }
}

#[test]
fn trasladan_y_rotan_a_su_velocidad() {
    let mut system = SolarSystem::new(&config());
    let start: Vec<f32> = system.planets.iter().map(|p| p.orbit_angle).collect();
    system.update(2.0);
    for (p, a0) in system.planets.iter().zip(start) {
        let expected = (a0 + 2.0 * p.config.orbit_speed_degrees.to_radians()).rem_euclid(TAU);
        assert!((p.orbit_angle - expected).abs() < 1e-4, "{}", p.config.id);
        let spin = (2.0 * p.config.rotation_speed_degrees.to_radians()).rem_euclid(TAU);
        assert!((p.spin_angle - spin).abs() < 1e-4, "{}", p.config.id);
        assert!((0.0..TAU).contains(&p.spin_angle));
    }
}

#[test]
fn rechaza_orbitas_que_chocan() {
    let mut bad = config();
    bad.planets[1].orbit_radius = bad.planets[0].orbit_radius + 0.5;
    assert!(bad.validate().is_err());
    let mut missing = config();
    missing.planets.pop();
    assert!(missing.validate().is_err());
    assert!(hex_color("#5daa3c").is_ok());
    assert!(hex_color("verde").is_err());
}
