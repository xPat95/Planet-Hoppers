use planet_hoppers::{
    gameplay::planet_visit::PlanetReturnPose,
    minigames::{
        common::host::MinigameExit,
        minecraft::visit::{VillageAction, VillageMenu},
    },
    scenes::minigame_host::{self, MinigameVisits},
    ships::{Ship, ShipsConfig},
    solar_system::{SolarSystem, SolarSystemConfig},
};
use raylib::prelude::Vector3;
use std::path::PathBuf;

fn system() -> SolarSystem {
    SolarSystem::new(
        &SolarSystemConfig::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/solar-system.json"),
        )
        .unwrap(),
    )
}

#[test]
fn minecraft_planet_dispatches_existing_overworld_host_by_identity() {
    let s = system();
    let planet = s.planets.iter().find(|p| p.config.id == "minecraft").unwrap();
    assert_eq!(planet.config.minigame.as_deref(), Some("minecraft"));
    for p in &s.planets {
        let point = p.position() + p.spin_axis() * p.config.radius * 1.6;
        let reached = s.planet_in_reach(point).unwrap();
        assert_eq!(s.planets[reached].config.id, p.config.id);
    }
    // Help returns before creating a window; this still exercises the real central dispatch.
    assert_eq!(
        minigame_host::run(
            "minecraft",
            &mut MinigameVisits::default(),
            vec!["--help".to_string()].into_iter()
        )
        .unwrap(),
        MinigameExit::WindowClosed
    );
}

#[test]
fn village_return_is_direct_from_pause_without_abandonment_warning() {
    let mut menu = VillageMenu::Closed;
    assert!(!menu.apply(VillageAction::RequestReturn));
    assert_eq!(menu, VillageMenu::Closed);
    assert!(!menu.apply(VillageAction::Pause));
    assert!(!menu.apply(VillageAction::Continue));
    assert_eq!(menu, VillageMenu::Closed);
    menu.apply(VillageAction::Pause);
    assert!(menu.apply(VillageAction::RequestReturn));
    assert_eq!(menu, VillageMenu::Closed);
    assert!(!menu.apply(VillageAction::RequestReturn));
}

#[test]
fn return_keeps_approach_and_orientation_for_every_ship_after_orbital_motion() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    for model in &ships.ships {
        let mut system = system();
        let index = system.planets.iter().position(|p| p.config.id == "minecraft").unwrap();
        let mut ship = Ship::new(
            system.planets[index].position()
                + system.planets[index].spin_axis() * system.planets[index].config.radius * 1.7,
            0.81,
        );
        ship.pitch = 0.3;
        ship.bank = -0.2;
        ship.speed = 120.0;
        ship.carried_by = Some(index);
        ship.bumped = true;
        let offset = ship.position - system.planets[index].position();
        let pose = PlanetReturnPose::capture(&ship, &system.planets[index]);
        system.update(120.0);
        pose.restore(&mut ship, &system, index, model.length, |_| true)
            .unwrap();
        assert!((ship.position - system.planets[index].position() - offset).length() < 0.01);
        assert_eq!((ship.yaw, ship.pitch, ship.bank), (0.81, 0.3, -0.2));
        assert_eq!(ship.speed, 0.0);
        assert!(!ship.bumped && ship.carried_by.is_none());
        assert!(system
            .planets
            .iter()
            .all(|p| p.push_out(ship.position, model.length).is_none()));
        assert_eq!(system.planet_in_reach(ship.position), Some(index));
    }
}

#[test]
fn unsafe_approach_is_replaced_near_same_planet_without_changing_heading() {
    let mut system = system();
    let index = system.planets.iter().position(|p| p.config.id == "minecraft").unwrap();
    let mut ship = Ship::new(system.planets[index].position(), 1.2);
    let pose = PlanetReturnPose::capture(&ship, &system.planets[index]);
    system.update(30.0);
    // Simulate additional blocked space around the planet, such as a station collider.
    let center = system.planets[index].position();
    let minimum = system.planets[index].config.radius * 2.4;
    pose.restore(&mut ship, &system, index, 10.0, |point| {
        (point - center).length() > minimum
    })
    .unwrap();
    assert!((ship.position - center).length() > minimum);
    assert!((ship.position - center).length() < system.planets[index].config.radius * 3.0);
    assert_eq!(ship.yaw, 1.2);
    assert!(system
        .planets
        .iter()
        .all(|p| p.push_out(ship.position, 10.0).is_none()));
    assert!(ship.position.length() > system.sun.radius + 10.0);
    assert!(pose
        .restore(&mut ship, &system, index, 10.0, |_| false)
        .is_err());
    assert_ne!(ship.position, Vector3::zero());
}
