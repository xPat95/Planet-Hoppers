use planet_hoppers::minigames::{
    common::{
        collision::{Aabb, Obstacle},
        interaction::select_target_with,
        manifest::SceneManifest,
    },
    southpark::{
        threats::{ThreatKind, ThreatResult, ThreatState, PREFIX},
        DevConfig, SouthParkScene,
    },
};
use raylib::prelude::Vector3;
use std::path::PathBuf;

fn scene() -> SouthParkScene {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let m = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut c = DevConfig::load(&root.join("config/southpark-dev.json")).unwrap();
    c.test_objects = false;
    SouthParkScene::load(&m, c).unwrap()
}
fn tick(s: &mut SouthParkScene, seconds: f32) {
    for _ in 0..(seconds * 60.0).ceil() as usize {
        s.update_kenny(1.0 / 60.0);
    }
}
fn active(s: &mut SouthParkScene, kind: ThreatKind) {
    s.start_threat(kind).unwrap();
    for _ in 0..2000 {
        if s.threats.state == ThreatState::Active {
            return;
        }
        s.update_kenny(1.0 / 60.0);
    }
    panic!("Preparation did not finish");
}
fn act(s: &mut SouthParkScene, id: &str) -> bool {
    s.threats.interact(id, &mut s.kenny)
}

#[test]
fn each_zone_and_bus_sweep_avoid_the_real_city() {
    let s = scene();
    for kind in [ThreatKind::Bus, ThreatKind::Ufo, ThreatKind::Sewer] {
        s.threats.validate_zone(kind, &s.world).unwrap();
    }
    assert!((s.threats.panel_position() - s.player.feet).length() > s.interaction_distance + 3.0);
}
#[test]
fn only_one_event_can_run_and_preparation_disallows_early_actions() {
    let mut s = scene();
    s.start_threat(ThreatKind::Bus).unwrap();
    let p = s.kenny.position();
    assert!(s.start_threat(ThreatKind::Ufo).is_err());
    assert_eq!(s.threats.kind, Some(ThreatKind::Bus));
    assert_eq!(s.kenny.position(), p);
    assert!(!act(&mut s, "threat:brake"));
    assert!(s.threats.targets().is_empty());
}
#[test]
fn brake_stops_bus_before_contact_and_result_is_public() {
    let mut s = scene();
    active(&mut s, ThreatKind::Bus);
    tick(&mut s, 5.0);
    let before = s.threats.bus_position();
    assert!(act(&mut s, "threat:brake"));
    tick(&mut s, 20.0);
    assert_eq!(s.threats.bus_position(), before);
    assert_eq!(s.threats.result, Some(ThreatResult::Survived));
    assert_eq!(s.threats.state, ThreatState::Succeeded);
    assert!(!s.kenny.is_paused());
}
#[test]
fn bus_contact_fails_then_restart_restores_everything() {
    let mut s = scene();
    active(&mut s, ThreatKind::Bus);
    tick(&mut s, 18.0);
    assert_eq!(s.threats.result, Some(ThreatResult::Died));
    assert_eq!(s.threats.state, ThreatState::Failed);
    assert!(!s.threats.visible_kenny());
    assert_eq!(s.kenny.take_threat_contact().as_deref(), Some("bus"));
    s.restart_threat().unwrap();
    assert_eq!(s.threats.state, ThreatState::Preparing);
    assert_eq!(s.threats.result, None);
    assert!(s.threats.visible_kenny());
    assert_eq!(s.kenny.position().y, 0.0);
}
#[test]
fn bus_motion_is_continuous_and_matches_active_time_at_different_fps() {
    let mut positions = Vec::new();
    for fps in [30, 60, 144] {
        let mut s = scene();
        active(&mut s, ThreatKind::Bus);
        let mut previous = s.threats.bus_position();
        for _ in 0..fps * 8 {
            s.update_kenny(1.0 / fps as f32);
            let p = s.threats.bus_position();
            assert!((p - previous).length() < 4.0 / fps as f32 + 0.001);
            previous = p;
        }
        positions.push(previous);
    }
    assert!((positions[0] - positions[2]).length() < 0.01);
}
#[test]
fn three_ufo_lights_are_independent_and_cannot_be_counted_twice() {
    let mut s = scene();
    active(&mut s, ThreatKind::Ufo);
    tick(&mut s, 4.0);
    assert!(s.kenny.position().y > 0.5);
    for i in 0..3 {
        let id = format!("threat:light-{i}");
        assert!(act(&mut s, &id));
        assert!(!act(&mut s, &id));
    }
    assert_eq!(s.threats.state, ThreatState::Resolving);
    assert_eq!(s.threats.result, None);
    tick(&mut s, 3.0);
    assert_eq!(s.threats.result, Some(ThreatResult::Survived));
    assert_eq!(s.threats.state, ThreatState::Succeeded);
    assert!(s.kenny.position().y.abs() < 0.001);
}
#[test]
fn ufo_abduction_fails_at_height_limit_and_can_restart() {
    let mut s = scene();
    active(&mut s, ThreatKind::Ufo);
    tick(&mut s, 20.0);
    assert_eq!(s.threats.result, Some(ThreatResult::Died));
    assert!(s.kenny.position().y >= s.threats.config.ufo_max_lift);
    assert!(!s.threats.visible_kenny());
    s.restart_threat().unwrap();
    assert_eq!(s.threats.lights, [true; 3]);
    assert_eq!(s.kenny.position().y, 0.0);
}
#[test]
fn ufo_sphere_refinement_rejects_empty_corners_of_its_aabb() {
    let mut s = scene();
    active(&mut s, ThreatKind::Ufo);
    let p = s.threats.light_position(0);
    assert!(s
        .threats
        .refine(
            "threat:light-0",
            p + Vector3::new(0.0, 0.0, 3.0),
            Vector3::new(0.0, 0.0, -1.0),
            14.0,
            2.75
        )
        .is_some());
    assert!(s
        .threats
        .refine(
            "threat:light-0",
            p + Vector3::new(0.24, 0.24, 3.0),
            Vector3::new(0.0, 0.0, -1.0),
            14.0,
            2.75
        )
        .is_none());
    for i in 0..3 {
        assert!(s.threats.light_position(i).y < 6.0);
    }
}
#[test]
fn sewer_requires_pickup_then_place_and_safely_resolves() {
    let mut s = scene();
    active(&mut s, ThreatKind::Sewer);
    assert!(!act(&mut s, "threat:place"));
    assert!(act(&mut s, "threat:pickup"));
    assert!(!act(&mut s, "threat:pickup"));
    assert!(s.threats.carrying_lid);
    assert!(s.threats.lid_position().is_none());
    assert!(s.threats.targets().iter().all(|t| t.id == "threat:place"));
    assert!(act(&mut s, "threat:place"));
    tick(&mut s, 1.0);
    assert!(s.threats.covered);
    assert!(!s.threats.carrying_lid);
    assert_eq!(s.threats.result, Some(ThreatResult::Survived));
}
#[test]
fn uncovered_hole_causes_vertical_fall_and_reset_clears_carrying() {
    let mut s = scene();
    active(&mut s, ThreatKind::Sewer);
    assert!(act(&mut s, "threat:pickup"));
    tick(&mut s, 20.0);
    assert_eq!(s.threats.state, ThreatState::Failed);
    assert!(s.kenny.position().y < -2.0);
    assert!(!s.threats.visible_kenny());
    s.restart_threat().unwrap();
    assert!(!s.threats.carrying_lid && !s.threats.covered);
    assert!(s.threats.lid_position().is_some());
}
#[test]
fn clearing_any_event_restores_exploration_without_stale_targets_or_colliders() {
    for kind in [ThreatKind::Bus, ThreatKind::Ufo, ThreatKind::Sewer] {
        let mut s = scene();
        let original = s.kenny.position();
        active(&mut s, kind);
        tick(&mut s, 2.0);
        s.clear_threat().unwrap();
        assert_eq!(s.threats.state, ThreatState::Inactive);
        assert!((s.kenny.position() - original).length() < 0.01);
        assert!(!s.kenny.is_paused());
        assert!(!s.targets.iter().any(|t| t.id.starts_with(PREFIX)));
        assert!(!s.world.obstacles.iter().any(|o| o.id.starts_with(PREFIX)));
        tick(&mut s, 2.0);
        assert!((s.kenny.position() - original).length() > 0.1);
    }
}
#[test]
fn emergency_panel_uses_shared_raycast_range_and_wall_occlusion() {
    let mut s = scene();
    active(&mut s, ThreatKind::Bus);
    let target = s.threats.targets().remove(0);
    let p = target.bounds.center();
    let origin = p + Vector3::new(-3.0, 0.2, 0.0);
    let dir = (p - origin).normalized();
    let select = |s: &SouthParkScene, range: f32| {
        select_target_with(&s.targets, &s.world, origin, dir, range, |i, o, d, r, b| {
            if s.targets[i].id.starts_with(PREFIX) {
                s.threats.refine(&s.targets[i].id, o, d, r, b)
            } else {
                None
            }
        })
    };
    let chosen = select(&s, 4.5).unwrap();
    assert_eq!(s.targets[chosen].id, "threat:brake");
    assert!(select(&s, 1.0).is_none());
    let center = (p + origin) * 0.5;
    s.world.obstacles.push(Obstacle {
        id: "wall".into(),
        bounds: Aabb::new(
            center - Vector3::new(0.1, 2.0, 2.0),
            center + Vector3::new(0.1, 2.0, 2.0),
        ),
    });
    assert!(select(&s, 4.5).is_none());
}
#[test]
fn player_remains_outside_the_bus_when_it_moves_into_the_player_lane() {
    let mut s = scene();
    active(&mut s, ThreatKind::Bus);
    s.player.feet = Vector3::new(5.0, 0.0, -42.9);
    tick(&mut s, 8.0);
    assert!(s.world.can_occupy(
        s.player.feet,
        s.player.config.radius,
        s.player.config.body_height
    ));
    assert!((s.player.feet.z + 42.9).abs() > 1.0);
}

#[test]
fn lid_and_hole_only_accept_the_circular_surface_not_box_corners() {
    let mut s = scene();
    active(&mut s, ThreatKind::Sewer);
    for (id, p) in [
        ("threat:pickup", s.threats.cover_position()),
        ("threat:place", s.threats.hole_position()),
    ] {
        assert!(s
            .threats
            .refine(id, p + Vector3::up() * 3.0, -Vector3::up(), 4.5, 2.8)
            .is_some());
        assert!(s
            .threats
            .refine(
                id,
                p + Vector3::new(0.8, 3.0, 0.8),
                -Vector3::up(),
                4.5,
                2.8
            )
            .is_none());
    }
}
