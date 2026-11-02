use planet_hoppers::{
    gameplay::selection::{
        GameState, SelectionMode, SelectionTransition, ShipCatalog, ShipSelection,
        INITIAL_SHIP_ORDER,
    },
    graphics::hangar::HangarCamera,
    ships::ShipsConfig,
};
use raylib::prelude::*;
use std::path::PathBuf;

fn setup() -> (ShipCatalog, ShipsConfig) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
    (catalog, ships)
}

#[test]
fn all_six_initial_choices_are_free_and_unlock_exactly_one_ship() {
    let (catalog, _) = setup();
    assert_eq!(
        catalog
            .choices
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>(),
        INITIAL_SHIP_ORDER
    );
    for index in 0..6 {
        let selection = ShipSelection {
            index,
            mode: SelectionMode::Initial,
        };
        assert!(selection.can_confirm(&catalog, None));
        let SelectionTransition::StartAdventure(game) = selection.confirm(&catalog, None).unwrap()
        else {
            panic!("wrong transition")
        };
        let id = &catalog.choices[index].id;
        assert_eq!(game.first_ship(), id);
        assert_eq!(game.active_ship(), id);
        assert_eq!(game.unlocked_count(), 1);
        for choice in &catalog.choices {
            assert_eq!(game.is_unlocked(&choice.id), &choice.id == id);
        }
    }
}

#[test]
fn browsing_and_cancelling_do_not_mutate_the_current_adventure() {
    let (catalog, _) = setup();
    let game = GameState::new_game("minecraft", &catalog).unwrap();
    let before = game.clone();
    let mut selection = ShipSelection::new(SelectionMode::Change, Some(&game), &catalog);
    for _ in 0..30 {
        selection.navigate(1, &catalog);
    }
    for _ in 0..30 {
        selection.navigate(-1, &catalog);
    }
    assert_eq!(selection.index, 3);
    assert_eq!(game, before);
    selection.index = 0;
    selection.navigate(-1, &catalog);
    assert_eq!(selection.index, 5);
    selection.navigate(1, &catalog);
    assert_eq!(selection.index, 0);
}

#[test]
fn change_mode_previews_locked_ships_but_cannot_activate_them() {
    let (catalog, _) = setup();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    let selection = ShipSelection {
        index: 1,
        mode: SelectionMode::Change,
    };
    let original = game.clone();
    assert!(!selection.can_confirm(&catalog, Some(&game)));
    assert!(selection.confirm(&catalog, Some(&game)).is_err());
    assert_eq!(game, original);
    assert!(game.unlock("starwars", &catalog).unwrap());
    assert!(!game.unlock("starwars", &catalog).unwrap());
    assert!(selection.can_confirm(&catalog, Some(&game)));
    let SelectionTransition::ChangedShip(changed) =
        selection.confirm(&catalog, Some(&game)).unwrap()
    else {
        panic!("must retain the current adventure")
    };
    assert_eq!(changed.active_ship(), "starwars");
    assert_eq!(changed.first_ship(), "minecraft");
    assert_eq!(changed.unlocked_count(), 2);
    assert!(changed.is_unlocked("minecraft"));
    assert_eq!(game.active_ship(), "minecraft");
}

#[test]
fn new_game_discards_old_ownership_only_after_confirmation() {
    let (catalog, _) = setup();
    let mut previous = GameState::new_game("minecraft", &catalog).unwrap();
    previous.unlock("starwars", &catalog).unwrap();
    let selection = ShipSelection {
        index: 5,
        mode: SelectionMode::Initial,
    };
    let SelectionTransition::StartAdventure(game) =
        selection.confirm(&catalog, Some(&previous)).unwrap()
    else {
        panic!("new game expected")
    };
    assert_eq!(game.active_ship(), "southpark");
    assert_eq!(game.unlocked_count(), 1);
    assert_eq!(previous.unlocked_count(), 2);
    assert!(GameState::new_game("unknown", &catalog).is_err());
    assert!(previous.unlock("unknown", &catalog).is_err());
}

#[test]
fn launch_resolves_each_identity_without_changing_flight_scale_or_models() {
    let (catalog, original) = setup();
    for choice in &catalog.choices {
        let mut ships = original.clone();
        let game = GameState::new_game(&choice.id, &catalog).unwrap();
        catalog.configure_active(&game, &mut ships).unwrap();
        let active = ships.active().unwrap();
        assert_eq!(active.id, choice.id);
        if let Some(id) = &choice.ship_id {
            let expected = original.ships.iter().find(|s| &s.id == id).unwrap();
            assert_eq!(active.model, expected.model);
            assert_eq!(active.length, expected.length);
            assert_eq!(
                active.model_rotation_degrees,
                expected.model_rotation_degrees
            );
        } else {
            assert_eq!(choice.id, "halloween");
            assert!(active.model.is_empty());
            assert_eq!(active.length, 4.0);
        }
        assert_eq!(ships.flight.cruise_speed, original.flight.cruise_speed);
        assert_eq!(ships.flight.boost_speed, original.flight.boost_speed);
        assert_eq!(ships.flight.acceleration, original.flight.acceleration);
        assert_eq!(ships.flight.radius, original.flight.radius);
    }
}

#[test]
fn exhibition_camera_fits_rotating_bounds_between_header_and_footer() {
    let settings = HangarCamera::default();
    for viewport in [
        Vector2::new(960.0, 640.0),
        Vector2::new(1280.0, 800.0),
        Vector2::new(1920.0, 1080.0),
    ] {
        for half in [
            Vector3::new(3.0, 0.5, 1.0),
            Vector3::new(1.0, 3.0, 1.0),
            Vector3::new(3.0, 3.0, 3.0),
        ] {
            let center = Vector3::new(0.0, 4.0, 0.0);
            let camera = settings.camera(center, half, viewport, Vector2::new(151.0, 156.0));
            assert_eq!(camera.position, settings.position);
            assert_eq!(camera.target, settings.target);
            assert!(camera.fovy >= settings.fov);
            let forward = (camera.target - camera.position).normalized();
            let right = forward.cross(Vector3::up()).normalized();
            let up = right.cross(forward).normalized();
            let tangent = (camera.fovy.to_radians() * 0.5).tan();
            for step in 0..8 {
                let angle = step as f32 * std::f32::consts::TAU / 8.0;
                for sx in [-1.0, 1.0] {
                    for sy in [-1.0, 1.0] {
                        for sz in [-1.0, 1.0] {
                            let (x, z) = (sx * half.x, sz * half.z);
                            let point = center
                                + Vector3::new(
                                    x * angle.cos() - z * angle.sin(),
                                    sy * half.y,
                                    x * angle.sin() + z * angle.cos(),
                                );
                            let delta = point - camera.position;
                            let depth = delta.dot(forward);
                            assert!(depth > 0.0);
                            let ndc_x =
                                delta.dot(right) / (depth * tangent * viewport.x / viewport.y);
                            let ndc_y = delta.dot(up) / (depth * tangent);
                            let x = (ndc_x + 1.0) * viewport.x * 0.5;
                            let y = (1.0 - ndc_y) * viewport.y * 0.5;
                            assert!((90.0..=viewport.x - 90.0).contains(&x));
                            assert!((151.0..=viewport.y - 156.0).contains(&y));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn docked_fleet_changes_are_confirmed_and_preserve_the_first_ship() {
    let (catalog, ships) = setup();
    let game = GameState::new_game("minecraft", &catalog).unwrap();
    let mut selector = ShipSelection::new(SelectionMode::Docked, Some(&game), &catalog);
    assert_eq!(catalog.choices[selector.index].id, game.active_ship());
    selector.navigate(-2, &catalog);
    assert!(selector.can_confirm(&catalog, Some(&game)));
    assert_eq!(game.active_ship(), "minecraft");
    let SelectionTransition::ChangedShip(next) = selector.confirm(&catalog, Some(&game)).unwrap()
    else {
        panic!("wrong transition")
    };
    assert_eq!(next.first_ship(), "minecraft");
    assert_eq!(next.active_ship(), "starwars");
    assert!(next.is_unlocked("minecraft") && next.is_unlocked("starwars"));
    let mut configured = ships;
    catalog.configure_active(&next, &mut configured).unwrap();
    assert_eq!(configured.active_ship, "starwars");
    assert!(configured.active().unwrap().model.contains("mandalorian"));
}
