use planet_hoppers::{
    gameplay::station::Station,
    ships::{FlightInput, Ship, ShipsConfig},
    solar_system::{SolarSystem, SolarSystemConfig},
};
use raylib::prelude::Vector3;
use std::path::PathBuf;

fn setup() -> (Station, SolarSystem, ShipsConfig) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    (
        Station::load(&root).unwrap(),
        SolarSystem::new(&SolarSystemConfig::load(&root.join("config/solar-system.json")).unwrap()),
        ShipsConfig::load(&root.join("config/ships.json")).unwrap(),
    )
}

#[test]
fn station_is_fixed_above_every_orbit_and_sun_with_a_safe_entry_anchor() {
    let (station, mut system, ships) = setup();
    let initial = station.center();
    assert!(initial.y - station.collision_radius > system.sun.radius);
    for _ in 0..600 {
        system.update(1.0);
        assert_eq!(station.center(), initial);
        for p in &system.planets {
            assert!((p.position() - initial).length() > station.collision_radius + p.config.radius);
        }
    }
    assert!(station.in_reach(
        station.entry_anchor(ships.flight.radius),
        ships.flight.radius
    ));
    assert!(!station.in_reach(station.approach(), ships.flight.radius));
    assert!(station
        .push_out(station.approach(), station.approach(), ships.flight.radius)
        .is_none());
    assert!(system.planet_in_reach(station.approach()).is_none());
}

#[test]
fn station_collision_blocks_turbo_tunnelling_and_recovers_embedded_ship_or_camera() {
    let (station, _, ships) = setup();
    let (lo, hi) = station.hull.boxes().next().unwrap();
    let c = station.collision_point((lo + hi) * 0.5);
    let offset = station.exit_forward() * 3000.0;
    let hit = station
        .push_out(c + offset, c - offset, ships.flight.radius)
        .unwrap();
    assert!((hit - c).dot(offset) > 0.0);
    assert!(station.push_out(hit, hit, ships.flight.radius).is_none());
    assert!(station
        .push_out(c + offset, c + offset * 2.0, 1.0)
        .is_none());
    let recovered = station.push_out(c, c, 1.0).unwrap();
    assert!(station.push_out(recovered, recovered, 1.0).is_none());
}

#[test]
fn vertical_ascent_is_independent_of_heading_and_frame_rate_and_respects_turbo() {
    let (_, system, ships) = setup();
    let start = Vector3::new(10000.0, 10000.0, 10000.0);
    for boost in [false, true] {
        let mut positions = Vec::new();
        for fps in [30, 60, 144] {
            let mut ship = Ship::new(start, 1.2);
            ship.pitch = -0.7;
            for _ in 0..fps {
                ship.update(
                    1.0 / fps as f32,
                    &FlightInput {
                        ascend: true,
                        boost,
                        ..Default::default()
                    },
                    &ships.flight,
                    &system,
                );
            }
            let moved = ship.position - start;
            assert!(moved.x.abs() < 0.001 && moved.z.abs() < 0.001);
            let speed = if boost {
                ships.flight.boost_speed
            } else {
                ships.flight.cruise_speed
            };
            assert!((moved.y - speed).abs() < 0.15);
            assert_eq!(ship.pitch, -0.7);
            positions.push(ship.position);
        }
        assert!((positions[0] - positions[2]).length() < 0.15);
    }
}

#[test]
fn shared_autopilot_reaches_the_station_safely_in_normal_and_turbo_flight() {
    let (station, mut system, ships) = setup();
    for boost in [false, true] {
        let mut ship = Ship::new(system.departure_position(), 0.0);
        for _ in 0..12000 {
            if station.in_reach(ship.position, ships.flight.radius) {
                break;
            }
            let aim_at = Some(ship.autopilot_to(
                station.navigation_target(ship.position),
                &ships.flight,
                &system,
            ));
            let before = ship.position;
            system.update(1.0 / 60.0);
            let mut input = FlightInput {
                throttle: 1.0,
                boost,
                aim_at,
                ..Default::default()
            };
            station.assist_approach(&ship, &mut input, &ships.flight);
            ship.update(1.0 / 60.0, &input, &ships.flight, &system);
            assert!(station
                .push_out(before, ship.position, ships.flight.radius)
                .is_none());
            assert!(ship.position.length() >= system.sun.radius + ships.flight.radius);
        }
        assert!(
            station.in_reach(ship.position, ships.flight.radius),
            "no llegó con turbo={boost}"
        );
    }
}

#[test]
fn docking_waypoint_follows_translation_rotation_and_fitted_scale() {
    let (mut station, _, ships) = setup();
    for angles in [[0.0, -90.0, 0.0], [22.0, 37.0, 13.0], [0.0, 180.0, 0.0]] {
        station.rotation_degrees = angles;
        station.position = [400.0, 8200.0, -700.0];
        let expected = station.center()
            + Vector3::new(
                station.dock.local_position[0],
                station.dock.local_position[1],
                station.dock.local_position[2],
            )
            .transform_with(station.rotation())
                * station.length;
        assert!((expected - station.exit_position()).length() < 0.001);
        let offset = station.dock_target() - station.exit_position();
        assert!((offset.length() - station.dock.departure_clearance).abs() < 0.001);
        assert!(offset.normalized().dot(station.exit_forward()) > 0.999);
        assert!(station.in_reach(
            station.entry_anchor(ships.flight.radius),
            ships.flight.radius
        ));
        assert!(station
            .push_out(
                station.dock_target(),
                station.dock_target(),
                ships.flight.radius
            )
            .is_none());
        assert!(!station.in_reach(
            station.exit_position() - station.exit_forward() * 2.0,
            ships.flight.radius
        ));
        assert!(!station.in_reach(
            station.entry_anchor(ships.flight.radius)
                + station.exit_right() * station.dock.width * 3.0,
            ships.flight.radius
        ));
    }
}

#[test]
fn empty_space_outside_the_door_is_free_but_its_frame_is_solid() {
    let (station, _, ships) = setup();
    let p = station.dock_target();
    assert!(station
        .push_out(station.approach(), p, ships.flight.radius)
        .is_none());
    assert!(station
        .push_out(p, p + station.exit_right() * 1000.0, ships.flight.radius)
        .is_none());
    assert!(station
        .push_out(p, p + station.exit_up() * 50.0, ships.flight.radius)
        .is_none());
    assert!(station
        .push_out(p, p + station.exit_forward() * 1000.0, ships.flight.radius)
        .is_none());
    let post = station.exit_position()
        + station.exit_right() * (station.dock.width * 0.5 + 0.5)
        + station.exit_forward() * 4.0;
    assert!(station
        .push_out(
            post + station.exit_forward() * 50.0,
            post,
            ships.flight.radius
        )
        .is_some());
}
