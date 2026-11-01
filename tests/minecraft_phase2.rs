use planet_hoppers::minigames::{
    common::{fps::PlayerInput, interaction::select_target},
    minecraft::{MenuAction, MinecraftConfig, MinecraftScene, SurfaceState},
};
use raylib::prelude::{Vector2, Vector3};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn scene() -> MinecraftScene {
    MinecraftScene::load(
        &root().join("assets/models/minigames/minecraft/minecraft_village.scene.json"),
        MinecraftConfig::load(&root().join("config/minecraft-dev.json")).unwrap(),
    )
    .unwrap()
}
fn face_mine(s: &mut MinecraftScene) {
    s.player.feet = Vector3::new(8.5, 0.0, -22.5);
    s.player.yaw = -std::f32::consts::FRAC_PI_2;
    s.player.pitch = 0.0;
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.002, "{a} != {b}");
}

#[test]
fn phase1_manifest_loads_with_safe_grounded_spawn_and_all_collisions() {
    let s = scene();
    assert_eq!(s.world.obstacles.len(), 87); // 83 source AABBs + 4 shared boundaries.
    assert_eq!(s.targets.len(), 2);
    assert!(s.world.can_occupy(s.player.feet, 0.35, 1.8));
    close(s.player.eye().y, 1.7);
    close(s.player.feet.y, 0.0);
    close(s.player.direction().z, -1.0);
    assert!(s
        .manifest
        .visual_path(&root().join("assets/models/minigames/minecraft/minecraft_village.scene.json"))
        .unwrap()
        .is_file());
}

#[test]
fn every_phase1_road_is_traversable_by_the_swept_player_on_three_bands() {
    let s = scene();
    for road in &s.manifest.roads {
        for pair in road.points.windows(2) {
            let a = planet_hoppers::minigames::common::manifest::from_blender(pair[0]);
            let b = planet_hoppers::minigames::common::manifest::from_blender(pair[1]);
            let delta = b - a;
            if delta.length() < 0.001 {
                continue;
            }
            let normal = Vector3::new(-delta.z, 0.0, delta.x).normalized();
            for side in [-1.1, 0.0, 1.1] {
                let start = a + normal * side;
                let end = b + normal * side;
                assert!(s.world.can_occupy(start, 0.35, 1.8), "{} start", road.name);
                let result = s.world.move_body(start, end - start, 0.35, 1.8);
                assert!(
                    (result - end).length() < 0.002,
                    "{} band {side}: {result:?} != {end:?}",
                    road.name
                );
            }
        }
    }
}

#[test]
fn large_sprint_displacement_cannot_tunnel_through_any_house_or_oak() {
    let s = scene();
    for house in s
        .world
        .obstacles
        .iter()
        .filter(|o| o.id.starts_with("House_") || o.id == "Village_Hall")
    {
        let start = Vector3::new(house.bounds.center().x, 0.0, house.bounds.max.z + 2.0);
        assert!(s.world.can_occupy(start, 0.35, 1.8));
        let end = s
            .world
            .move_body(start, Vector3::new(0.0, 0.0, -100.0), 0.35, 1.8);
        close(end.z, house.bounds.max.z + 0.3501);
        assert!(s.world.can_occupy(end, 0.35, 1.8));
    }
    let end = s.world.move_body(
        Vector3::new(21.0, 0.0, -0.5),
        Vector3::new(100.0, 0.0, 0.0),
        0.35,
        1.8,
    );
    close(end.x, 22.6499);
    let edge = s.world.move_body(
        Vector3::new(0.0, 0.0, 20.0),
        Vector3::new(0.0, 0.0, 1000.0),
        0.35,
        1.8,
    );
    assert!(edge.z < 26.0 && edge.y == 0.0);
}

#[test]
fn shared_fps_is_frame_independent_with_clamped_mouse_pitch() {
    let mut slow = scene();
    let mut fast = scene();
    for _ in 0..60 {
        slow.update(
            PlayerInput {
                forward: 1.0,
                ..Default::default()
            },
            1.0 / 30.0,
            true,
        );
    }
    for _ in 0..288 {
        fast.update(
            PlayerInput {
                forward: 1.0,
                ..Default::default()
            },
            1.0 / 144.0,
            true,
        );
    }
    assert!((slow.player.feet - fast.player.feet).length() < 0.003);
    close(slow.player.feet.y, 0.0);
    let before = slow.player.yaw;
    slow.update(
        PlayerInput {
            mouse: Vector2::new(100.0, 100000.0),
            ..Default::default()
        },
        0.0,
        true,
    );
    close(slow.player.yaw - before, 0.2);
    close(slow.player.pitch, -85.0_f32.to_radians());
}

#[test]
fn mine_raycast_checks_range_direction_and_solid_occlusion() {
    let mut s = scene();
    face_mine(&mut s);
    let select =
        |origin, direction, range| select_target(&s.targets, &s.world, origin, direction, range);
    assert_eq!(select(s.player.eye(), s.player.direction(), 4.5), Some(0));
    assert_eq!(select(s.player.eye(), s.player.direction(), 1.0), None);
    assert_eq!(select(s.player.eye(), -s.player.direction(), 4.5), None);
    // The hill and tunnel end must occlude an entrance target viewed from behind.
    let behind = Vector3::new(8.5, 1.7, -34.0);
    assert_eq!(
        select(
            behind,
            (s.targets[0].bounds.center() - behind).normalized(),
            20.0
        ),
        None
    );
}

#[test]
fn mine_menu_freezes_player_and_pending_request_preserves_return_pose() {
    let mut s = scene();
    face_mine(&mut s);
    s.update(
        PlayerInput {
            interact: true,
            ..Default::default()
        },
        0.0,
        true,
    );
    assert_eq!(s.state, SurfaceState::MineMenu);
    let pose = s.pose();
    s.update(
        PlayerInput {
            forward: 1.0,
            sprint: true,
            mouse: Vector2::new(100.0, 100.0),
            interact: true,
            ..Default::default()
        },
        0.1,
        true,
    );
    assert_eq!(s.player.feet, pose.feet);
    close(s.player.yaw, pose.yaw);
    close(s.player.pitch, pose.pitch);
    s.menu_action(MenuAction::Start);
    assert_eq!(s.state, SurfaceState::ChallengePending);
    assert_eq!(s.pending_request.unwrap().return_pose.feet, pose.feet);
    s.menu_action(MenuAction::Start);
    assert_eq!(s.state, SurfaceState::ChallengePending);
    s.menu_action(MenuAction::Cancel);
    assert_eq!(s.state, SurfaceState::Exploring);
    assert!(s.pending_request.is_none());
    assert_eq!(s.player.feet, pose.feet);
}

#[test]
fn uncaptured_controls_cannot_move_look_or_interact_and_cancel_is_repeatable() {
    let mut s = scene();
    face_mine(&mut s);
    let before = s.pose();
    s.update(
        PlayerInput {
            interact: true,
            forward: 1.0,
            mouse: Vector2::new(70.0, 80.0),
            ..Default::default()
        },
        0.1,
        false,
    );
    assert_eq!(s.state, SurfaceState::Exploring);
    assert_eq!(s.player.feet, before.feet);
    close(s.player.yaw, before.yaw);
    for _ in 0..3 {
        s.update(
            PlayerInput {
                interact: true,
                ..Default::default()
            },
            0.0,
            true,
        );
        assert_eq!(s.state, SurfaceState::MineMenu);
        s.menu_action(MenuAction::Cancel);
        assert_eq!(s.state, SurfaceState::Exploring);
    }
}

#[test]
fn original_table_can_be_inspected_without_inventories_or_new_colliders() {
    let mut s = scene();
    s.player.feet = Vector3::new(3.5, 0.0, -23.0);
    s.player.yaw = -std::f32::consts::FRAC_PI_2;
    s.player.pitch = -0.4;
    let count = s.world.obstacles.len();
    s.update(
        PlayerInput {
            interact: true,
            ..Default::default()
        },
        0.0,
        true,
    );
    assert_eq!(s.selected, Some(1));
    assert!(s.confirmation.is_some());
    assert_eq!(s.state, SurfaceState::Exploring);
    assert_eq!(s.world.obstacles.len(), count);
}
