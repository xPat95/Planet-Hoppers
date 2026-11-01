use planet_hoppers::solar_system::{hex_color, SolarSystem, SolarSystemConfig, PLANET_IDS};
use std::{f32::consts::TAU, path::PathBuf};

fn config() -> SolarSystemConfig {
    SolarSystemConfig::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/solar-system.json"))
        .expect("config/solar-system.json válido")
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
        // Solo cambia el redondeo de f32 al sumar más pasos: centésimas de unidad en órbitas de decenas
        assert!((a.position() - b.position()).length() < 1e-2, "{}", a.config.id);
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
