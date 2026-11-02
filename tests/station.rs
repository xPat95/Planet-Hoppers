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
    assert!(station.in_reach(station.approach()));
    assert!(station
        .push_out(station.approach(), station.approach(), ships.flight.radius)
        .is_none());
    assert!(system.planet_in_reach(station.approach()).is_none());
}

#[test]
fn station_collision_blocks_turbo_tunnelling_and_recovers_embedded_ship_or_camera() {
    let (station, _, ships) = setup();
    let c = station.center();
    let offset = Vector3::new(0.0, 0.0, 3000.0);
    let hit = station
        .push_out(c + offset, c - offset, ships.flight.radius)
        .unwrap();
    assert!(hit.z > c.z);
    assert!((hit - c).length() >= station.collision_radius + ships.flight.radius);
    assert!(station
        .push_out(c + offset, c + offset * 2.0, 1.0)
        .is_none());
    assert!((station.push_out(c, c, 1.0).unwrap() - c).length() > station.collision_radius);
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
            if station.in_reach(ship.position) {
                break;
            }
            let aim_at = Some(ship.autopilot_to(station.approach(), &ships.flight, &system));
            let before = ship.position;
            system.update(1.0 / 60.0);
            ship.update(
                1.0 / 60.0,
                &FlightInput {
                    throttle: 1.0,
                    boost,
                    aim_at,
                    ..Default::default()
                },
                &ships.flight,
                &system,
            );
            assert!(station
                .push_out(before, ship.position, ships.flight.radius)
                .is_none());
            assert!(ship.position.length() >= system.sun.radius + ships.flight.radius);
        }
        assert!(
            station.in_reach(ship.position),
            "no llegó con turbo={boost}"
        );
    }
}
