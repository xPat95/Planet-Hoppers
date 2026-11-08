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
    assert_eq!(actual, ["halloween", "starwars", "smilingfriends", "minecraft", "regularshow", "southpark"]);
    let radii: Vec<_> = config.planets.iter().map(|p| p.orbit_radius).collect();
    assert_eq!(radii, [1800.0, 2760.0, 3840.0, 4920.0, 6000.0, 7080.0]);
    assert!(config.planets.windows(2).all(|pair| pair[0].orbit_radius < pair[1].orbit_radius));
    let angles: Vec<_> = config.planets.iter().map(|p| p.start_angle_degrees).collect();
    // Preserve current angles by identity: angular placement is not orbital order.
    assert_eq!(angles, [0.0, 300.0, 240.0, 180.0, 120.0, 60.0]);
}

#[test]
fn la_escala_conserva_proporciones_y_un_punto_de_partida_seguro() {
    let config = config();
    let system = SolarSystem::new(&config);
    assert_eq!(config.sun.radius, 540.0);
    let radii: Vec<_> = config.planets.iter().map(|p| p.radius).collect();
    assert_eq!(radii, [216.0, 264.0, 264.0, 192.0, 240.0, 216.0]);
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
        let expected = if matches!(id, "southpark" | "smilingfriends" | "minecraft" | "starwars" | "regularshow") { Some(id) } else { None };
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
fn conserva_las_velocidades_actuales_por_franquicia() {
    let system = SolarSystem::new(&config());
    // Reordering the orbits must not reassign speeds to a different franchise.
    for (id, orbital, spin) in [
        ("halloween", 0.55, -7.0), ("starwars", 2.2, 5.0),
        ("smilingfriends", 1.5, 6.0), ("minecraft", 1.1, 10.0),
        ("regularshow", 0.85, 9.0), ("southpark", 0.7, 8.0),
    ] {
        let p = system.planet(id).unwrap();
        assert_eq!(p.config.orbit_speed_degrees, orbital / planet_hoppers::solar_system::scale::travel_scale());
        assert_eq!(p.config.rotation_speed_degrees, spin);
    }
}

#[test]
fn radios_e_identidades_se_conservan_durante_toda_la_simulacion() {
    use planet_hoppers::{gameplay::progress::Franchise, scenes::token_collection::TOKENS};
    let mut system = SolarSystem::new(&config());
    let expected = [
        ("halloween", "Halloween", 1800.0), ("starwars", "Star Wars", 2760.0),
        ("smilingfriends", "Smiling Friends", 3840.0), ("minecraft", "Minecraft", 4920.0),
        ("regularshow", "Un Show Más", 6000.0), ("southpark", "South Park", 7080.0),
    ];
    for step in 0..3600 {
        system.update([1.0 / 144.0, 1.0 / 30.0, 0.1, 1.0, 60.0][step % 5]);
        for (planet, (id, name, radius)) in system.planets.iter().zip(expected) {
            assert_eq!(planet.config.id, id);
            assert_eq!(planet.config.name, name);
            assert_eq!(planet.config.orbit_radius, radius);
            assert!((planet.position().length() - radius).abs() < 0.002, "{id}: step={step}");
            assert!(planet.config.model.as_deref().unwrap().contains(&format!("/planets/{id}/")));
            if let Some(far) = &planet.config.model_far { assert!(far.contains(&format!("/planets/{id}/"))); }
            assert_eq!(planet.config.minigame.as_deref(), if id == "halloween" { None } else { Some(id) });
            let franchise = Franchise::from_id(id).unwrap();
            let art = TOKENS.iter().find(|art| art.franchise == franchise).unwrap();
            assert_eq!(art.png, format!("assets/ui/tokens/{id}.png"));
        }
    }
}

#[test]
fn rechaza_reasignar_franquicias_a_otras_orbitas_aunque_no_haya_colisiones() {
    let mut bad = config();
    // Exchange whole planet configs but leave the radii sorted and collision-free.
    let radii: Vec<_> = bad.planets.iter().map(|p| p.orbit_radius).collect();
    bad.planets.rotate_left(1);
    for (planet, radius) in bad.planets.iter_mut().zip(radii) { planet.orbit_radius = radius; }
    let error = bad.validate().unwrap_err();
    assert!(error.contains("Orden orbital"), "{error}");
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
