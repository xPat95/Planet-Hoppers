use planet_hoppers::{
    gameplay::station::Station,
    ships::{FlightInput, Ship, ShipsConfig},
    solar_system::{SolarSystem, SolarSystemConfig},
};
use raylib::prelude::{Matrix, Vector3};
use std::path::Path;

fn station() -> Station {
    Station::load(Path::new(env!("CARGO_MANIFEST_DIR"))).unwrap()
}

#[test]
fn old_spherical_barrier_was_not_a_world_limit_and_empty_space_is_now_free() {
    let station = station();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let mut system =
        SolarSystem::new(&SolarSystemConfig::load(&root.join("config/solar-system.json")).unwrap());
    assert!(
        500.0 < station.collision_radius,
        "regression probe must be inside the old sphere"
    );
    for offset in [
        Vector3::up() * 500.0,
        Vector3::up() * 1000.0,
        Vector3::up() * 10000.0,
        Vector3::new(500.0, 0.0, 0.0),
    ] {
        let p = station.center() + offset;
        // Independently outside occupied model bounds, but inside the old sphere.
        let local = station.collision_local(p);
        let [lo, hi] = station.hull.bounds;
        assert!(
            local.x < lo.x
                || local.x > hi.x
                || local.y < lo.y
                || local.y > hi.y
                || local.z < lo.z
                || local.z > hi.z
        );
        let mut ship = Ship::new(p, 0.0);
        for _ in 0..120 {
            system.update(1.0 / 60.0);
            ship.update(1.0 / 60.0, &FlightInput::default(), &ships.flight, &system);
        }
        assert_eq!(ship.position, p, "controller/world barrier");
        assert!(!ship.bumped);
        assert!(
            station.push_out(p, p, 4.5).is_none(),
            "empty-space station barrier: {p:?}"
        );
    }
}

#[test]
fn transformed_real_surface_blocks_sweeps_and_recovers_without_jitter() {
    let mut station = station();
    for rotation in [[0.0, -90.0, 0.0], [22.0, 37.0, 13.0], [0.0, 0.0, 90.0]] {
        station.rotation_degrees = rotation;
        station.position = [400.0, 8200.0, -700.0];
        for length in [700.0, 1400.0, 2380.0] {
            station.length = length;
            for (lo, hi) in station.hull.boxes().step_by(1300) {
                let inside = station.collision_point((lo + hi) * 0.5);
                for radius in [1.0, 2.86, 4.41] {
                    let recovered = station
                        .push_out(inside, inside, radius)
                        .expect("occupied cell");
                    assert!(
                        station.push_out(recovered, recovered, radius).is_none(),
                        "unstable recovery"
                    );
                    for direction in [
                        Vector3::up(),
                        Vector3::new(1.0, 0.0, 0.0),
                        Vector3::new(0.0, 0.0, 1.0),
                    ] {
                        let start = inside + direction * 5000.0;
                        let hit = station
                            .push_out(start, inside - direction * 5000.0, radius)
                            .expect("tunnelled through real geometry");
                        assert!(station.push_out(hit, hit, radius).is_none());
                    }
                }
            }
        }
    }
}

#[test]
fn collision_alignment_uses_the_visual_fit_origin() {
    let mut station = station();
    let raw = station.hull.raw_center;
    let raw = Vector3::new(raw[0], raw[1], raw[2]);
    let fit = Matrix::translate(-raw.x, -raw.y, -raw.z)
        * Matrix::scale(
            station.hull.raw_scale,
            station.hull.raw_scale,
            station.hull.raw_scale,
        )
        * Matrix::translate(0.0, -19.0, 0.0);
    station.align_collision(fit);
    assert!(
        (station.collision_point(Vector3::zero())
            - station.center()
            - Vector3::new(0.0, -19.0, 0.0).transform_with(station.rotation()))
        .length()
            < 0.01
    );
}
