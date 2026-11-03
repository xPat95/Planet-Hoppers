use planet_hoppers::{
    graphics::billboard::{alpha_visible, BillboardPose},
    minigames::{
        common::{
            collision::{Aabb, Obstacle},
            interaction::{select_target_with, InteractionTarget},
            manifest::SceneManifest,
        },
        southpark::{
            ike::{self, IkeEvent},
            locations::WorldLocations,
            session::SessionState,
            DevConfig, SouthParkScene,
        },
    },
    scenes::southpark_video::VideoFrames,
};
use raylib::prelude::*;
use std::{collections::HashSet, path::PathBuf};

fn scene() -> (SouthParkScene, WorldLocations) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut config = DevConfig::load(&root.join("config/southpark-dev.json")).unwrap();
    config.test_objects = false;
    let scene = SouthParkScene::load(&manifest, config).unwrap();
    let locations = WorldLocations::build(&manifest, &scene.world, &scene.roads).unwrap();
    (scene, locations)
}

#[test]
fn spawn_is_random_valid_grounded_accessible_and_separate_from_characters() {
    let (scene, locations) = scene();
    let mut positions = HashSet::new();
    for mut seed in 1..65 {
        let p = ike::choose_position(&scene, &locations, &mut seed).expect("safe spawn");
        assert_eq!(p.y, scene.world.ground_y);
        assert!(scene.world.can_occupy(p, 0.9, 1.8));
        assert!((p - scene.kenny.position()).length() > 8.0);
        assert!((p - scene.player.feet).length() > 4.0);
        assert!(p.x > scene.world.terrain.min.x && p.x < scene.world.terrain.max.x);
        assert!(p.z > scene.world.terrain.min.z && p.z < scene.world.terrain.max.z);
        positions.insert((p.x.to_bits(), p.z.to_bits()));
    }
    assert!(positions.len() >= 2);
}

#[test]
fn playback_repeats_without_relocating_and_cannot_overlap() {
    let position = Vector3::new(5.0, 0.0, 8.0);
    let mut event = IkeEvent {
        position: Some(position),
        ..Default::default()
    };
    assert!(!event.begin());
    event.sync(SessionState::Exploring, false);
    assert!(event.visible() && event.begin());
    assert!(!event.visible() && !event.begin());
    event.finish(); // Natural completion or decoder/audio error takes the same restoration path.
    assert!(event.visible());
    assert_eq!(event.position, Some(position));
    assert!(event.begin());
    event.finish();
    assert!(event.begin());
}

#[test]
fn every_non_exploration_state_and_intro_disable_video_and_interaction() {
    for state in [
        SessionState::Briefing,
        SessionState::Preparing,
        SessionState::Active,
        SessionState::Success,
        SessionState::DeathAnimation,
        SessionState::DeathDialogue,
        SessionState::Restoring,
        SessionState::Victory,
        SessionState::Defeat,
        SessionState::Paused,
        SessionState::Interlude,
        SessionState::Relocating,
        SessionState::Searching,
        SessionState::MapOpen,
    ] {
        let mut event = IkeEvent {
            position: Some(Vector3::zero()),
            ..Default::default()
        };
        event.sync(SessionState::Exploring, false);
        assert!(event.begin());
        event.sync(state, false);
        assert!(!event.playing && !event.visible() && !event.begin());
        if matches!(state, SessionState::Paused | SessionState::MapOpen) {
            assert!(event.position.is_some());
        } else {
            assert!(event.position.is_none());
        }
    }
    let mut event = IkeEvent {
        position: Some(Vector3::zero()),
        ..Default::default()
    };
    event.sync(SessionState::Exploring, true);
    assert!(!event.begin() && !event.visible());
}

#[test]
fn ike_raycast_rejects_transparency_range_misses_and_solid_walls() {
    let (mut scene, _) = scene();
    let feet = scene.player.feet + Vector3::new(0.0, 0.0, 2.0);
    let origin = feet + Vector3::new(0.0, 0.48, -2.0);
    let mut image = Image::load_image(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(ike::SPRITE)
            .to_string_lossy(),
    )
    .unwrap();
    image.alpha_crop(0.125);
    let alpha: Vec<_> = image.get_image_data().iter().map(|p| p.a).collect();
    assert!(alpha.contains(&0) && alpha.contains(&255));
    let width = ike::HEIGHT * image.width as f32 / image.height as f32;
    let pose = BillboardPose::new(feet, origin, width, ike::HEIGHT, 0.0, 0.0);
    let targets = [InteractionTarget {
        id: "easter-egg:ike".into(),
        label: "Ike".into(),
        bounds: pose.bounds(),
        action: "video".into(),
    }];
    // Aim at an actual opaque pixel, avoiding any assumption about PNG margins.
    let pixel = alpha
        .iter()
        .enumerate()
        .filter(|(_, a)| **a == 255)
        .min_by_key(|(i, _)| {
            let x = *i as i32 % image.width;
            let y = *i as i32 / image.width;
            (x - image.width / 2).abs() + (y - image.height / 2).abs()
        })
        .unwrap()
        .0;
    let u = (pixel % image.width as usize) as f32 / image.width as f32;
    let v = (pixel / image.width as usize) as f32 / image.height as f32;
    let aim = pose.origin + pose.right * ((u - 0.5) * width) + pose.up * ((1.0 - v) * ike::HEIGHT);
    let direction = (aim - origin).normalized();
    scene.world.obstacles.clear();
    let select = |world: &_| {
        select_target_with(&targets, world, origin, direction, 4.5, |_, o, d, r, _| {
            let (distance, u, v) = pose.ray_uv(o, d, r)?;
            alpha_visible(&alpha, image.width as usize, image.height as usize, (u, v))
                .then_some(distance)
        })
    };
    assert_eq!(select(&scene.world), Some(0));
    assert!(pose.ray_uv(origin, direction, 0.5).is_none());
    assert!(pose.ray_uv(origin, -direction, 4.5).is_none());
    assert!(!alpha_visible(
        &alpha,
        image.width as usize,
        image.height as usize,
        (-0.1, 0.5)
    ));
    let midpoint = origin + direction;
    scene.world.obstacles.push(Obstacle {
        id: "wall".into(),
        bounds: Aabb::new(
            midpoint - Vector3::one() * 0.4,
            midpoint + Vector3::one() * 0.4,
        ),
    });
    assert_eq!(select(&scene.world), None);
}

#[test]
fn derived_clip_matches_source_metadata_and_keeps_aspect() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let clip = VideoFrames::load(&root.join(ike::VIDEO)).unwrap();
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.join("assets/video/south park/kick_the_baby.json")).unwrap(),
    )
    .unwrap();
    // Dimensions belong to the prepared source, not a hard-coded 16:9 canvas.
    assert_eq!(i64::from(clip.width), metadata["width"].as_i64().unwrap());
    assert_eq!(i64::from(clip.height), metadata["height"].as_i64().unwrap());
    assert_eq!(u64::from(clip.fps), metadata["fps"].as_u64().unwrap());
    assert!(
        (clip.duration() - metadata["duration_seconds"].as_f64().unwrap() as f32).abs() < 0.001
    );
    assert!(root.join(ike::AUDIO).is_file());
    assert_eq!(
        metadata["source"],
        "assets/video/south park/Kick_the_baby.mp4"
    );
}
