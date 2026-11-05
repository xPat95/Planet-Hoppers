use planet_hoppers::{
    gameplay::{selection::ShipCatalog, station::chase_position},
    graphics::ship_presentation::{exhaust_spheres, hangar_nose, hangar_start_yaw},
    ships::ShipsConfig,
};
use raylib::prelude::{Matrix, Vector3};
use std::path::Path;

#[test]
fn six_exhibits_show_a_thirty_degree_front_and_rotate_toward_the_viewer() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
    let camera = Vector3::new(10.8, 0.0, 12.6);
    let viewer = camera.normalized();
    assert_eq!(catalog.choices.len(), 6);
    for choice in &catalog.choices {
        let ship = catalog.model(choice, &ships);
        let initial = hangar_start_yaw(&ship, camera);
        let nose =
            |angle: f32| hangar_nose(&ship).transform_with(Matrix::rotate_y(angle.to_radians()));
        let start_alignment = nose(initial).dot(viewer);
        assert!(
            (start_alignment.acos().to_degrees() - 30.0).abs() < 0.01,
            "{}",
            ship.id
        );
        assert!(nose(initial + 10.0).dot(viewer) > start_alignment);
        assert!(nose(initial + 30.0).dot(viewer) > 0.9999);
        assert!((nose(initial + 360.0) - nose(initial)).length() < 0.001);
    }
}

#[test]
fn flight_camera_preserves_ship_clearance_with_narrower_lens() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    for model in &ships.ships {
        let position = Vector3::new(100.0, 30.0, -20.0);
        let forward = Vector3::new(-0.6, 0.0, -0.8);
        let camera = chase_position(position, forward, model.camera_length());
        let rear = -(camera - position).dot(forward);
        let expected_ratio = if model.model.contains("/starwars/") {
            0.732
        } else {
            1.22
        };
        assert!((rear / (model.length * 3.0) - expected_ratio).abs() < 0.0001);
        // Even a sphere enclosing a cube of this maximum dimension clears near=.5.
        assert!((camera - position).length() > model.length * 0.5 * 3.0_f32.sqrt() + 0.5);
    }
}

#[test]
fn exhaust_stays_behind_the_hull_and_shrinks_at_cruise_and_boost() {
    for length in [4.0, 5.0, 6.0] {
        for throttle in [0.0, 0.2, 0.5, 1.0] {
            let rear = length * 0.5;
            let spheres = exhaust_spheres(length, rear, throttle);
            for &(offset, radius, alpha) in &spheres {
                assert!(offset - radius > rear);
                assert!(radius > 0.0 && alpha <= 90);
            }
            let extent = spheres
                .iter()
                .map(|(at, r, _)| at + r)
                .fold(0.0_f32, f32::max)
                - rear;
            let old_extent = length
                * (0.55 + 0.7 * (0.3 + throttle) + (0.12 - 0.7 * 0.05) * (0.6 + throttle))
                - rear;
            assert!(extent / old_extent < 0.30);
            assert!(spheres[0].1 / (length * 0.12 * (0.6 + throttle)) < 0.30);
        }
    }
}

#[test]
fn n1_flight_nose_matches_controller_without_altering_hangar_configuration() {
    use planet_hoppers::ships::Ship;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    for model in ships
        .ships
        .iter()
        .filter(|m| m.model.contains("/starwars/"))
    {
        assert_eq!(model.model_rotation_degrees, [0.0; 3]);
        assert_eq!(hangar_nose(model), Vector3::new(0.0, 0.0, 1.0));
        let r = model.flight_rotation_degrees();
        let import = Matrix::rotate_x(r[0].to_radians())
            * Matrix::rotate_y(r[1].to_radians())
            * Matrix::rotate_z(r[2].to_radians());
        for yaw in [0.0, 0.7, 3.14, 5.1] {
            for pitch in [-1.2, 0.0, 0.8] {
                let mut ship = Ship::new(Vector3::zero(), yaw);
                ship.pitch = pitch;
                ship.bank = 0.4;
                let world =
                    Matrix::rotate_z(ship.bank) * Matrix::rotate_x(pitch) * Matrix::rotate_y(yaw);
                let nose = Vector3::new(0.0, 0.0, 1.0).transform_with(import * world);
                assert!(nose.dot(ship.forward()) > 0.9999);
            }
        }
    }
}
