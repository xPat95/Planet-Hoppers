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

#[test]
fn five_kind_catalog_is_unique_and_parseable() {
    for (name, kind) in ["bus", "ufo", "sewer", "piano", "snowman"]
        .into_iter()
        .zip(ThreatKind::ALL)
    {
        assert_eq!(ThreatKind::parse(name), Some(kind));
    }
}

#[test]
fn piano_only_falls_after_kenny_enters_impact_zone_then_hits_ground() {
    let mut s = scene();
    active(&mut s, ThreatKind::Piano);
    while s.threats.state == ThreatState::Active {
        assert!((s.threats.piano_position().y - s.threats.config.piano_height).abs() < 0.001);
        s.update_kenny(1.0 / 60.0);
    }
    assert_eq!(s.threats.state, ThreatState::Resolving);
    assert!((s.kenny.position() - s.threats.piano_impact()).length() < 0.01);
    let mut last = s.threats.piano_position().y;
    while s.threats.state == ThreatState::Resolving {
        s.update_kenny(1.0 / 60.0);
        let y = s.threats.piano_position().y;
        assert!(y >= 0.0 && y <= last);
        last = y;
    }
    assert_eq!(s.threats.result, Some(ThreatResult::Died));
    assert_eq!(last, 0.0);
    tick(&mut s, 0.8);
    assert!(s.threats.sprite_height_scale() < 0.4);
}

#[test]
fn piano_safety_is_single_use_freezes_sway_and_prevents_later_fall() {
    let mut s = scene();
    active(&mut s, ThreatKind::Piano);
    tick(&mut s, 3.0);
    assert!(s.threats.piano_sway().abs() > 0.001);
    assert!(act(&mut s, "threat:piano-lock"));
    assert!(!act(&mut s, "threat:piano-lock"));
    let p = s.threats.piano_position();
    tick(&mut s, 3.0);
    assert_eq!(s.threats.piano_position(), p);
    assert_eq!(s.threats.piano_sway(), 0.0);
    assert_eq!(s.threats.result, Some(ThreatResult::Survived));
}

#[test]
fn heaters_are_independent_single_use_and_third_starts_melting() {
    let mut s = scene();
    active(&mut s, ThreatKind::Snowman);
    assert!(!act(&mut s, "threat:heater-3"));
    assert!(!act(&mut s, "threat:heater-200"));
    for i in 0..3 {
        assert!(act(&mut s, &format!("threat:heater-{i}")));
        assert!(!act(&mut s, &format!("threat:heater-{i}")));
        assert_eq!(s.threats.heaters.iter().filter(|b| **b).count(), i + 1);
        if i < 2 {
            assert_eq!(s.threats.state, ThreatState::Active);
        }
    }
    assert_eq!(s.threats.state, ThreatState::Resolving);
    let p = s.threats.snow_position();
    tick(&mut s, 0.9);
    assert_eq!(s.threats.snow_position(), p);
    assert!(s.threats.snow_scale().y < 0.6 && s.threats.snow_scale().y > 0.0);
    tick(&mut s, 1.3);
    assert_eq!(s.threats.result, Some(ThreatResult::Survived));
    assert!(!s.kenny.is_paused());
}

#[test]
fn pursuit_is_continuous_clear_of_city_and_fails_by_contact() {
    for heaters in 0..3 {
        let mut s = scene();
        active(&mut s, ThreatKind::Snowman);
        for i in 0..heaters {
            assert!(act(&mut s, &format!("threat:heater-{i}")));
        }
        let mut static_world = s.world.clone();
        static_world
            .obstacles
            .retain(|o| o.id != "southpark-kenny" && !o.id.starts_with(PREFIX));
        let mut previous = s.threats.snow_position();
        for _ in 0..600 {
            s.update_kenny(0.1);
            let p = s.threats.snow_position();
            assert!((p - previous).length() <= s.threats.config.snowman_speed * 0.1 + 0.002);
            assert!(static_world.can_occupy(p, 1.05, 3.3));
            assert!(static_world.can_occupy(s.kenny.position(), 0.28, 1.35));
            assert_eq!(p.y, 0.0);
            previous = p;
            if s.threats.result.is_some() {
                break;
            }
        }
        assert_eq!(s.threats.result, Some(ThreatResult::Died));
        assert!((s.kenny.position() - s.threats.snow_position()).length() <= 1.15);
        let p = s.threats.snow_position();
        tick(&mut s, 2.0);
        assert_eq!(s.threats.snow_position(), p);
    }
}

#[test]
fn new_event_paths_match_at_different_frame_rates() {
    for kind in [ThreatKind::Piano, ThreatKind::Snowman] {
        let mut poses = vec![];
        for fps in [30, 60, 144] {
            let mut s = scene();
            active(&mut s, kind);
            for _ in 0..fps * 5 {
                s.update_kenny(1.0 / fps as f32);
            }
            poses.push((s.kenny.position(), s.threats.snow_position()));
        }
        for pose in &poses[1..] {
            assert!((pose.0 - poses[0].0).length() < 0.01);
            assert!((pose.1 - poses[0].1).length() < 0.01);
        }
    }
}

#[test]
fn new_event_result_cleanup_restores_kenny_and_removes_all_props() {
    for kind in [ThreatKind::Piano, ThreatKind::Snowman] {
        for win in [false, true] {
            let mut s = scene();
            active(&mut s, kind);
            if win {
                for target in s.threats.targets() {
                    assert!(act(&mut s, &target.id));
                }
            }
            while s.threats.result.is_none() {
                s.update_kenny(0.1);
            }
            tick(&mut s, 6.2);
            assert!(s.threats.released);
            assert!(s.threats.visible_kenny());
            assert!(!s.kenny.is_paused());
            assert!(s.world.can_occupy(s.player.feet, 0.35, 1.8));
            assert!(!s.world.obstacles.iter().any(|o| o.id.starts_with(PREFIX)));
            assert!(!s.targets.iter().any(|o| o.id.starts_with(PREFIX)));
            assert_eq!(s.kenny.position().y, 0.0);
            assert!(s.threats.result.is_some());
            s.restart_threat().unwrap();
            assert_eq!(s.threats.state, ThreatState::Preparing);
            assert!(!s.threats.released && !s.threats.piano_locked);
            assert_eq!(s.threats.heaters, [false; 3]);
            s.clear_threat().unwrap();
            assert!(!s.kenny.is_paused());
        }
    }
}

#[test]
fn new_targets_use_shared_range_and_occlusion() {
    for kind in [ThreatKind::Piano, ThreatKind::Snowman] {
        let mut s = scene();
        active(&mut s, kind);
        for target in s.threats.targets() {
            let p = target.bounds.center();
            let origin = p + Vector3::new(-2.5, 0.5, -1.0);
            let select = |s: &SouthParkScene, range: f32| {
                select_target_with(
                    &s.targets,
                    &s.world,
                    origin,
                    (p - origin).normalized(),
                    range,
                    |i, o, d, r, b| {
                        if s.targets[i].id == target.id {
                            s.threats.refine(&target.id, o, d, r, b)
                        } else {
                            None
                        }
                    },
                )
            };
            let index = select(&s, 4.5).expect("Target should be visible from street");
            assert_eq!(s.targets[index].id, target.id);
            assert!(select(&s, 0.5).is_none());
            let c = (origin + p) * 0.5;
            s.world.obstacles.push(Obstacle {
                id: "test-wall".into(),
                bounds: Aabb::new(c - Vector3::one() * 0.2, c + Vector3::one() * 0.2),
            });
            assert!(select(&s, 4.5).is_none());
            s.world.obstacles.pop();
        }
    }
}

#[test]
fn wide_pursuer_uses_graph_detour_instead_of_crossing_a_wall() {
    use planet_hoppers::minigames::{
        common::collision::CollisionWorld,
        southpark::navigation::{event_route, RoadNetwork},
    };
    let a = Vector3::new(-5.0, 0.0, 0.0);
    let b = Vector3::new(5.0, 0.0, 0.0);
    let c = Vector3::new(-5.0, 0.0, 5.0);
    let d = Vector3::new(5.0, 0.0, 5.0);
    let world = CollisionWorld::new(
        Aabb::new(
            Vector3::new(-20.0, 0.0, -20.0),
            Vector3::new(20.0, 0.0, 20.0),
        ),
        0.0,
        vec![Obstacle {
            id: "wall".into(),
            bounds: Aabb::new(Vector3::new(-1.0, 0.0, -3.0), Vector3::new(1.0, 5.0, 3.0)),
        }],
    );
    let graph = RoadNetwork {
        nodes: vec![a, c, d, b],
        neighbors: vec![vec![1], vec![0, 2], vec![1, 3], vec![2]],
    };
    let path = event_route(&graph, &world, a, b, 1.05, 3.3).unwrap();
    assert!(path.iter().any(|p| p.z > 3.0));
    for p in path.windows(2) {
        assert!((world.move_body(p[0], p[1] - p[0], 1.05, 3.3) - p[1]).length() < 0.002);
    }
}
