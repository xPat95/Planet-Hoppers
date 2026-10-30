use planet_hoppers::minigames::{
    common::{
        collision::{Aabb, CollisionWorld, Obstacle},
        fps::{FpsController, PlayerInput},
        interaction::{select_target, InteractionTarget},
        manifest::{from_blender, from_blender_bounds, SceneManifest},
    },
    southpark::{DevConfig, SouthParkScene},
};
use raylib::prelude::{Vector2, Vector3};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn manifest() -> SceneManifest {
    SceneManifest::load(&root().join("assets/models/minigames/southpark/southpark_town.scene.json"))
        .unwrap()
}
fn config() -> DevConfig {
    DevConfig::load(&root().join("config/southpark-dev.json")).unwrap()
}
fn box_at(min: [f32; 3], max: [f32; 3]) -> Aabb {
    Aabb::new(
        Vector3::new(min[0], min[1], min[2]),
        Vector3::new(max[0], max[1], max[2]),
    )
}
fn empty() -> CollisionWorld {
    CollisionWorld::new(
        box_at([-100.0, 0.0, -100.0], [100.0, 0.0, 100.0]),
        0.0,
        vec![],
    )
}
fn obstacle(bounds: Aabb) -> Obstacle {
    Obstacle {
        id: "test-wall".into(),
        bounds,
    }
}
fn player() -> FpsController {
    FpsController::new(
        Vector3::new(0.0, 1.7, 0.0),
        Vector3::right(),
        1.7,
        config().player,
    )
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.002, "{a} differs from {b}");
}
fn target(x: f32) -> InteractionTarget {
    InteractionTarget {
        id: format!("target-{x}"),
        label: "Probe".into(),
        bounds: box_at([x, 0.0, -0.5], [x + 0.5, 2.0, 0.5]),
        action: "dev-confirm".into(),
    }
}

#[test]
fn manifest_reuses_all_395_solids_and_converts_axes() {
    let manifest = manifest();
    let world = manifest.collision_world().unwrap();
    assert_eq!(
        manifest
            .colliders
            .iter()
            .filter(|c| c.role == "solid")
            .count(),
        395
    );
    assert_eq!(world.obstacles.len(), 399); // plus four terrain boundaries
    assert_eq!(from_blender([1.0, 2.0, 3.0]), Vector3::new(1.0, 3.0, -2.0));
    let b = from_blender_bounds([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
    assert_eq!(b.min, Vector3::new(1.0, 3.0, -5.0));
    assert_eq!(b.max, Vector3::new(4.0, 6.0, -2.0));
    close(world.terrain.size().x, 720.0);
    close(world.terrain.size().z, 335.0);
}

#[test]
fn spawn_is_clear_grounded_and_faces_east() {
    let scene = SouthParkScene::load(&manifest(), config()).unwrap();
    assert!(scene.world.can_occupy(scene.player.feet, 0.35, 1.8));
    close(scene.player.eye().y, 1.7);
    close(scene.player.feet.y, 0.0);
    close(scene.player.direction().x, 1.0);
    assert_eq!(scene.targets.len(), 3);
}

#[test]
fn temporary_objects_can_be_removed_without_changing_world_assets() {
    let mut c = config();
    c.test_objects = false;
    let scene = SouthParkScene::load(&manifest(), c).unwrap();
    assert!(scene.targets.is_empty());
    assert_eq!(scene.world.obstacles.len(), 399);
}

#[test]
fn movement_is_frame_rate_independent_and_diagonal_is_normalized() {
    let mut distances = Vec::new();
    for fps in [30, 60, 144] {
        let mut p = player();
        for _ in 0..fps * 2 {
            p.update(
                PlayerInput {
                    forward: 1.0,
                    right: 1.0,
                    ..Default::default()
                },
                1.0 / fps as f32,
                &empty(),
            );
        }
        distances.push(p.feet.length());
        close(p.feet.length(), 9.0);
    }
    close(distances[0], distances[2]);
}

#[test]
fn yaw_controls_forward_and_strafe_without_vertical_motion() {
    let mut p = player();
    p.yaw = std::f32::consts::FRAC_PI_2;
    p.pitch = 1.0;
    p.update(
        PlayerInput {
            forward: 1.0,
            ..Default::default()
        },
        0.1,
        &empty(),
    );
    close(p.feet.x, 0.0);
    close(p.feet.z, 0.45);
    close(p.feet.y, 0.0);
    p.update(
        PlayerInput {
            right: 1.0,
            ..Default::default()
        },
        0.1,
        &empty(),
    );
    close(p.feet.x, -0.45);
    close(p.feet.z, 0.45);
}

#[test]
fn pitch_is_clamped_and_mouse_delta_is_not_scaled_by_fps() {
    let mut a = player();
    let mut b = player();
    let input = PlayerInput {
        mouse: Vector2::new(50.0, -1e6),
        ..Default::default()
    };
    a.update(input, 1.0 / 30.0, &empty());
    b.update(input, 1.0 / 144.0, &empty());
    close(a.yaw, b.yaw);
    close(a.pitch, 85.0_f32.to_radians());
    a.update(
        PlayerInput {
            mouse: Vector2::new(0.0, 1e6),
            ..Default::default()
        },
        0.01,
        &empty(),
    );
    close(a.pitch, -85.0_f32.to_radians());
}

#[test]
fn thin_wall_blocks_even_a_hundred_metre_sprint_step() {
    let mut world = empty();
    world
        .obstacles
        .push(obstacle(box_at([0.0, 0.0, -10.0], [0.02, 8.0, 10.0])));
    let feet = world.move_body(
        Vector3::new(-3.0, 0.0, 0.0),
        Vector3::new(100.0, 0.0, 0.0),
        0.35,
        1.8,
    );
    assert!(feet.x <= -0.35);
    assert!(world.can_occupy(feet, 0.35, 1.8));
}

#[test]
fn diagonal_motion_slides_along_walls() {
    let mut world = empty();
    world
        .obstacles
        .push(obstacle(box_at([0.0, 0.0, -10.0], [2.0, 8.0, 10.0])));
    let feet = world.move_body(
        Vector3::new(-2.0, 0.0, -3.0),
        Vector3::new(4.0, 0.0, 5.0),
        0.35,
        1.8,
    );
    assert!(feet.x <= -0.35);
    close(feet.z, 2.0);
    assert!(world.can_occupy(feet, 0.35, 1.8));
}

#[test]
fn corner_contact_does_not_enter_a_box() {
    let mut world = empty();
    world
        .obstacles
        .push(obstacle(box_at([0.0, 0.0, 0.0], [2.0, 8.0, 2.0])));
    let mut feet = Vector3::new(-3.0, 0.0, -3.0);
    for _ in 0..300 {
        feet = world.move_body(feet, Vector3::new(0.4, 0.0, 0.4), 0.35, 1.8);
        assert!(world.can_occupy(feet, 0.35, 1.8));
    }
}

#[test]
fn all_four_terrain_edges_block_the_entire_player_volume() {
    for direction in [
        Vector3::right(),
        -Vector3::right(),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, -1.0),
    ] {
        let world = empty();
        let feet = world.move_body(Vector3::zero(), direction * 10000.0, 0.35, 1.8);
        assert!(feet.x.abs() <= 99.65 && feet.z.abs() <= 99.65);
        assert!(world.can_occupy(feet, 0.35, 1.8));
    }
}

#[test]
fn floor_height_never_changes_from_vertical_displacements() {
    let feet = empty().move_body(
        Vector3::new(0.0, -100.0, 0.0),
        Vector3::new(1.0, -500.0, 0.0),
        0.35,
        1.8,
    );
    close(feet.y, 0.0);
    close(feet.x, 1.0);
}

#[test]
fn parallel_rays_hit_miss_and_inside_origins_are_handled() {
    let a = box_at([2.0, 0.0, -1.0], [3.0, 2.0, 1.0]);
    close(
        a.ray(Vector3::new(0.0, 1.0, 0.0), Vector3::right(), 4.0)
            .unwrap()
            .distance,
        2.0,
    );
    assert!(a
        .ray(Vector3::new(0.0, 3.0, 0.0), Vector3::right(), 4.0)
        .is_none());
    close(
        a.ray(Vector3::new(2.5, 1.0, 0.0), Vector3::right(), 4.0)
            .unwrap()
            .distance,
        0.0,
    );
}

#[test]
fn ray_selection_chooses_closest_target_and_limits_distance() {
    let targets = [target(3.0), target(1.0)];
    let origin = Vector3::new(0.0, 1.7, 0.0);
    assert_eq!(
        select_target(&targets, &empty(), origin, Vector3::right(), 4.5),
        Some(1)
    );
    assert_eq!(
        select_target(&targets, &empty(), origin, Vector3::right(), 0.9),
        None
    );
    assert_eq!(
        select_target(&targets, &empty(), origin, -Vector3::right(), 4.5),
        None
    );
    assert_eq!(
        select_target(&targets, &empty(), origin, Vector3::zero(), 4.5),
        None
    );
}

#[test]
fn selection_uses_a_true_3d_ray_not_just_horizontal_projection() {
    assert_eq!(
        select_target(
            &[target(2.0)],
            &empty(),
            Vector3::new(0.0, 1.7, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            4.5
        ),
        None
    );
}

#[test]
fn opaque_obstacle_blocks_interaction_but_target_itself_does_not() {
    let t = target(3.0);
    let origin = Vector3::new(0.0, 1.7, 0.0);
    let mut world = empty();
    world.obstacles.push(obstacle(t.bounds));
    assert_eq!(
        select_target(
            std::slice::from_ref(&t),
            &world,
            origin,
            Vector3::right(),
            4.5
        ),
        Some(0)
    );
    world
        .obstacles
        .push(obstacle(box_at([1.0, 0.0, -1.0], [1.2, 5.0, 1.0])));
    assert_eq!(
        select_target(&[t], &world, origin, Vector3::right(), 4.5),
        None
    );
}

#[test]
fn all_three_scene_probes_are_reachable_and_actions_confirm() {
    let mut scene = SouthParkScene::load(&manifest(), config()).unwrap();
    for index in 0..3 {
        let direction = scene.targets[index].bounds.center() - scene.player.eye();
        assert_eq!(
            select_target(
                &scene.targets,
                &scene.world,
                scene.player.eye(),
                direction,
                scene.interaction_distance
            ),
            Some(index)
        );
        scene.interact(index);
    }
    assert_eq!(scene.interaction_count, 3);
    assert!(scene.confirmation.is_some());
    scene.tick_confirmation(3.0);
    assert!(scene.confirmation.is_none());
}

#[test]
fn repeated_running_stops_at_the_probe_and_cannot_creep_through() {
    let mut scene = SouthParkScene::load(&manifest(), config()).unwrap();
    let start = scene.player.feet;
    for _ in 0..600 {
        scene.player.update(
            PlayerInput {
                forward: 1.0,
                sprint: true,
                ..Default::default()
            },
            1.0 / 60.0,
            &scene.world,
        );
    }
    let distance = (scene.player.feet - start).length();
    assert!((2.49..2.501).contains(&distance));
    assert!(scene.world.can_occupy(scene.player.feet, 0.35, 1.8));
}

#[test]
fn real_city_street_lanes_allow_a_human_volume() {
    let world = manifest().collision_world().unwrap();
    let data: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root().join("assets/models/minigames/southpark/southpark_town.scene.json"))
            .unwrap(),
    )
    .unwrap();
    let mut samples = 0;
    for road in data["roads"].as_array().unwrap() {
        let points = road["points"].as_array().unwrap();
        for pair in points.windows(2) {
            let point = |v: &serde_json::Value| {
                from_blender([
                    v[0].as_f64().unwrap() as f32,
                    v[1].as_f64().unwrap() as f32,
                    0.0,
                ])
            };
            let (a, b) = (point(&pair[0]), point(&pair[1]));
            let delta = b - a;
            let side = Vector3::new(-delta.z, 0.0, delta.x).normalized();
            for step in 0..=delta.length().ceil() as u32 {
                for offset in [-2.0, 0.0, 2.0] {
                    let feet = a + delta * (step as f32 / delta.length()).min(1.0) + side * offset;
                    assert!(
                        world.can_occupy(feet, 0.35, 1.8),
                        "Street {} blocked at {:?}",
                        road["name"],
                        feet
                    );
                    samples += 1;
                }
            }
        }
    }
    assert!(samples > 7000);
}

#[test]
fn actual_buildings_trees_mountains_and_sign_block_swept_movement() {
    let world = manifest().collision_world().unwrap();
    let mut tested = 0;
    let mut kinds = std::collections::BTreeSet::new();
    for solid in world
        .obstacles
        .iter()
        .filter(|o| !o.id.starts_with("boundary-"))
    {
        let center = solid.bounds.center();
        let start = Vector3::new(solid.bounds.min.x - 1.0, 0.0, center.z);
        if !world.can_occupy(start, 0.35, 1.8) {
            continue;
        }
        let feet = world.move_body(
            start,
            Vector3::new(solid.bounds.size().x + 2.0, 0.0, 0.0),
            0.35,
            1.8,
        );
        assert!(
            feet.x <= solid.bounds.min.x - 0.35 + 0.001,
            "Crossed {}",
            solid.id
        );
        assert!(world.can_occupy(feet, 0.35, 1.8));
        tested += 1;
        kinds.insert(if solid.id.contains("Pine_") {
            "tree"
        } else if solid.id.contains("Mountain_") {
            "mountain"
        } else if solid.id.contains("Sign") {
            "sign"
        } else {
            "building"
        });
    }
    assert!(tested > 300);
    assert_eq!(kinds.len(), 4);
}
