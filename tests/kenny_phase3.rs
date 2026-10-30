use planet_hoppers::{
    graphics::billboard::{alpha_visible, BillboardPose},
    minigames::{
        common::{
            collision::{Aabb, Obstacle},
            interaction::select_target_with,
            manifest::SceneManifest,
        },
        southpark::{
            kenny::{WalkAnimation, KENNY_ID},
            navigation::{clear_segment, NPC_HEIGHT},
            DevConfig, KennyDialogue, SouthParkScene,
        },
    },
};
use raylib::prelude::*;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn scene() -> SouthParkScene {
    let manifest = SceneManifest::load(
        &root().join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    SouthParkScene::load(
        &manifest,
        DevConfig::load(&root().join("config/southpark-dev.json")).unwrap(),
    )
    .unwrap()
}
fn pixels() -> (Vec<u8>, usize, usize) {
    let image = Image::load_image(
        root()
            .join("assets/textures/minigames/South Park/KennyMcCormick.png")
            .to_str()
            .unwrap(),
    )
    .unwrap();
    (
        image.get_image_data().iter().map(|p| p.a).collect(),
        image.width() as usize,
        image.height() as usize,
    )
}

#[test]
fn png_keeps_real_alpha_and_original_aspect_ratio() {
    let (alpha, width, height) = pixels();
    assert_eq!((width, height), (271, 367));
    assert!(alpha.contains(&0) && alpha.contains(&255));
    assert!(!alpha_visible(&alpha, width, height, (0.0, 0.0)));
    assert!(alpha_visible(&alpha, width, height, (0.5, 0.5)));
    let pose = BillboardPose::new(
        Vector3::zero(),
        Vector3::new(0.0, 2.0, 5.0),
        NPC_HEIGHT * width as f32 / height as f32,
        NPC_HEIGHT,
        0.0,
        0.0,
    );
    assert!((pose.width / pose.height - width as f32 / height as f32).abs() < 1e-6);
}

#[test]
fn billboard_stays_vertical_from_all_sides_and_camera_heights() {
    for camera in [
        Vector3::new(5.0, 1.7, 0.0),
        Vector3::new(-5.0, 10.0, 0.0),
        Vector3::new(0.0, 30.0, 5.0),
        Vector3::new(0.0, -2.0, -5.0),
        Vector3::new(0.0, 10.0, 0.0),
    ] {
        let pose = BillboardPose::new(Vector3::zero(), camera, 1.0, NPC_HEIGHT, 0.0, 0.0);
        assert!(pose.normal.y.abs() < 1e-7);
        assert!((pose.up - Vector3::up()).length() < 1e-6);
        assert!(
            pose.corners()
                .iter()
                .map(|p| p.y)
                .fold(f32::INFINITY, f32::min)
                .abs()
                < 1e-6
        );
    }
}

#[test]
fn walking_roll_is_local_to_plane_and_feet_do_not_sink() {
    for angle in [-10.0_f32, 0.0, 10.0] {
        let pose = BillboardPose::new(
            Vector3::zero(),
            Vector3::new(3.0, 10.0, 5.0),
            1.0,
            NPC_HEIGHT,
            angle.to_radians(),
            0.02,
        );
        assert!(pose.normal.y.abs() < 1e-6);
        assert!(pose.right.dot(pose.normal).abs() < 1e-6 && pose.up.dot(pose.normal).abs() < 1e-6);
        assert!(
            (pose
                .corners()
                .iter()
                .map(|p| p.y)
                .fold(f32::INFINITY, f32::min)
                - 0.02)
                .abs()
                < 1e-6
        );
    }
}

#[test]
fn animation_tracks_distance_and_decays_smoothly_when_stopped() {
    let mut animation = WalkAnimation::default();
    let mut maximum: f32 = 0.0;
    for _ in 0..180 {
        animation.update(1.0 / 60.0, 1.3 / 60.0);
        maximum = maximum.max(animation.roll().abs());
    }
    assert!((8.0..=10.01).contains(&maximum.to_degrees()));
    let phase = animation.phase;
    let weight = animation.weight;
    animation.update(1.0 / 60.0, 0.0);
    assert!(animation.weight < weight && animation.weight > 0.1);
    for _ in 0..120 {
        animation.update(1.0 / 60.0, 0.0);
    }
    assert_eq!(animation.phase, phase);
    assert!(animation.roll().abs() < 0.00001 && animation.bounce() < 0.00001);
}

#[test]
fn animation_phase_is_independent_of_frame_rate() {
    let mut results = Vec::new();
    for fps in [30, 60, 144] {
        let mut a = WalkAnimation::default();
        for _ in 0..fps * 2 {
            a.update(1.0 / fps as f32, 1.3 / fps as f32);
        }
        results.push(a.phase);
    }
    assert!((results[0] - results[2]).abs() < 0.0001);
}

#[test]
fn road_graph_uses_real_crossings_and_only_clear_connections() {
    let s = scene();
    assert!(s.roads.nodes.len() > 600);
    let mut edges = 0;
    for (i, neighbors) in s.roads.neighbors.iter().enumerate() {
        for &j in neighbors {
            assert!(s.roads.neighbors[j].contains(&i));
            assert!(clear_segment(
                &s.world,
                s.roads.nodes[i],
                s.roads.nodes[j],
                Some(KENNY_ID)
            ));
            edges += 1;
        }
    }
    assert!(edges > 1200);
    assert!(s.roads.neighbors.iter().any(|n| n.len() >= 3));
}

#[test]
fn initial_npc_is_near_player_and_outside_scenery() {
    let s = scene();
    assert!((s.kenny.position() - s.player.feet).length() < 4.0);
    assert!(!s.kenny.body(&s.world).overlaps(s.world.body(
        s.player.feet,
        s.player.config.radius,
        s.player.config.body_height
    )));
    assert!(!s
        .world
        .obstacles
        .iter()
        .filter(|o| o.id != KENNY_ID)
        .any(|o| s.kenny.body(&s.world).overlaps(o.bounds)));
}

#[test]
fn autonomous_walk_is_deterministic_continuous_and_never_enters_buildings() {
    let mut a = scene();
    let mut b = scene();
    let mut distance = 0.0;
    let mut stopped = 0;
    for _ in 0..14400 {
        let previous = a.kenny.position();
        a.update_kenny(1.0 / 60.0);
        b.update_kenny(1.0 / 60.0);
        assert!((a.kenny.position() - b.kenny.position()).length() < 1e-6);
        let step = (a.kenny.position() - previous).length();
        assert!(step <= 1.3 / 60.0 + 0.001);
        distance += step;
        assert!(a.kenny.position().y.abs() < 0.0001);
        assert!(!a
            .world
            .obstacles
            .iter()
            .filter(|o| o.id != KENNY_ID)
            .any(|o| a.kenny.body(&a.world).overlaps(o.bounds)));
        if !a.kenny.is_walking() {
            stopped += 1;
        }
    }
    assert!(distance > 100.0, "Only walked {distance} m");
    assert!(stopped > 30);
}

#[test]
fn npc_motion_matches_at_different_update_rates() {
    let mut positions = Vec::new();
    for fps in [30, 60, 144] {
        let mut s = scene();
        for _ in 0..fps * 12 {
            s.update_kenny(1.0 / fps as f32);
        }
        positions.push(s.kenny.position());
    }
    assert!((positions[0] - positions[2]).length() < 0.025);
}

#[test]
fn dialogue_pauses_animation_and_cancel_resumes_the_route() {
    let mut s = scene();
    for _ in 0..60 {
        s.update_kenny(1.0 / 60.0);
    }
    let position = s.kenny.position();
    s.interact(s.kenny_target());
    assert_eq!(s.dialogue, KennyDialogue::Invitation);
    for _ in 0..120 {
        s.update_kenny(1.0 / 60.0);
    }
    assert_eq!(s.kenny.position(), position);
    assert!(s.kenny.animation.weight < 0.0001);
    s.close_dialogue();
    for _ in 0..60 {
        s.update_kenny(1.0 / 60.0);
    }
    assert!((s.kenny.position() - position).length() > 0.5);
}

#[test]
fn accept_is_only_preparation_and_can_be_closed() {
    let mut s = scene();
    s.interact(s.kenny_target());
    s.accept_challenge();
    assert_eq!(s.dialogue, KennyDialogue::Preparing);
    assert!(s.kenny.is_paused());
    s.close_dialogue();
    assert_eq!(s.dialogue, KennyDialogue::Closed);
    assert!(!s.kenny.is_paused());
}

#[test]
fn shared_raycast_refines_to_sprite_alpha_and_obeys_opaque_occlusion() {
    let manifest = SceneManifest::load(
        &root().join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut config = DevConfig::load(&root().join("config/southpark-dev.json")).unwrap();
    config.test_objects = false;
    let mut s = SouthParkScene::load(&manifest, config).unwrap();
    let (alpha, w, h) = pixels();
    let position = s.kenny.position();
    let camera = position + Vector3::new(0.0, 0.7, 3.0);
    let pose = BillboardPose::new(
        position,
        camera,
        NPC_HEIGHT * w as f32 / h as f32,
        NPC_HEIGHT,
        0.0,
        0.0,
    );
    let index = s.kenny_target();
    s.targets[index].bounds = pose.bounds();
    let select = |s: &SouthParkScene, aim: Vector3, range: f32| {
        select_target_with(
            &s.targets,
            &s.world,
            camera,
            (aim - camera).normalized(),
            range,
            |i, o, d, r, broad| {
                if i == index {
                    pose.ray_uv(o, d, r)
                        .and_then(|(t, u, v)| alpha_visible(&alpha, w, h, (u, v)).then_some(t))
                } else {
                    Some(broad)
                }
            },
        )
    };
    let middle = position + Vector3::up() * (NPC_HEIGHT * 0.5);
    assert_eq!(select(&s, middle, 4.5), Some(index));
    assert_eq!(select(&s, middle, 2.0), None);
    assert_eq!(
        select(
            &s,
            pose.corners()[3] + pose.right * 0.005 - pose.up * 0.005,
            4.5
        ),
        None
    );
    s.world.obstacles.push(Obstacle {
        id: "blocking-wall".into(),
        bounds: Aabb::new(
            camera + Vector3::new(-2.0, -1.0, -1.5),
            camera + Vector3::new(2.0, 2.0, -1.2),
        ),
    });
    assert_eq!(select(&s, middle, 4.5), None);
}

#[test]
fn temporary_route_and_safe_reposition_never_accept_unsafe_locations() {
    let mut s = scene();
    let original = s.kenny.position();
    assert!(!s
        .kenny
        .reposition_safe(Vector3::new(10000.0, 0.0, 10000.0), &s.world));
    let wall = s
        .world
        .obstacles
        .iter()
        .find(|o| o.id.starts_with("COL_SP_B"))
        .unwrap()
        .bounds
        .center();
    assert!(!s
        .kenny
        .reposition_safe(Vector3::new(wall.x, 0.0, wall.z), &s.world));
    assert_eq!(s.kenny.position(), original);
    assert!(!s
        .kenny
        .set_temporary_route(&[Vector3::new(10000.0, 0.0, 0.0)], &s.world));
    assert!(s
        .kenny
        .set_temporary_route(&[original + Vector3::new(0.0, 0.0, 0.2)], &s.world));
    for _ in 0..12 {
        s.update_kenny(1.0 / 60.0);
    }
    assert!((s.kenny.position() - original).length() > 0.1);
}

#[test]
fn future_threat_interfaces_only_expose_requests_and_notifications() {
    let mut s = scene();
    s.kenny.request_danger_animation(true);
    assert!(s.kenny.danger_animation_requested);
    s.kenny.notify_threat_contact("future-bus");
    assert_eq!(s.kenny.take_threat_contact().as_deref(), Some("future-bus"));
    assert_eq!(s.kenny.take_threat_contact(), None);
    assert_eq!(s.dialogue, KennyDialogue::Closed);
}

#[test]
fn a_player_blocking_a_route_causes_wait_and_a_safe_new_approach() {
    let manifest = SceneManifest::load(
        &root().join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut config = DevConfig::load(&root().join("config/southpark-dev.json")).unwrap();
    config.test_objects = false;
    let mut s = SouthParkScene::load(&manifest, config).unwrap();
    let start = s.kenny.position();
    for _ in 0..45 {
        s.update_kenny(1.0 / 60.0);
    }
    let heading = (s.kenny.position() - start).normalized();
    s.player.feet = s.kenny.position() + heading * 1.3;
    assert!(s.world.can_occupy(
        s.player.feet,
        s.player.config.radius,
        s.player.config.body_height
    ));
    let mut walked = 0.0;
    for _ in 0..900 {
        let before = s.kenny.position();
        s.update_kenny(1.0 / 60.0);
        let step = (s.kenny.position() - before).length();
        walked += step;
        assert!(step < 1.3 / 60.0 + 0.001);
        assert!(!s.kenny.body(&s.world).overlaps(s.world.body(
            s.player.feet,
            s.player.config.radius,
            s.player.config.body_height
        )));
    }
    assert!(walked > 3.0, "NPC stayed blocked: {walked}");
}
