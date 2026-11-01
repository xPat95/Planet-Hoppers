use planet_hoppers::minigames::{
    common::{
        manifest::SceneManifest,
        world_map::{circle_marker, MapBounds, MapView},
    },
    southpark::{
        kenny::KENNY_ID,
        locations::WorldLocations,
        session::{SaveKennySession, SessionState, VoiceClock},
        threats::{ThreatKind, ThreatResult, ThreatState},
        DevConfig, SouthParkScene,
    },
};
use raylib::prelude::{Vector2, Vector3};
use std::{path::PathBuf, sync::OnceLock};
fn scene() -> SouthParkScene {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut c = DevConfig::load(&root.join("config/southpark-dev.json")).unwrap();
    c.test_objects = false;
    SouthParkScene::load(&manifest, c).unwrap()
}
fn catalog() -> WorldLocations {
    static CACHE: OnceLock<WorldLocations> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let w = scene();
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let m = SceneManifest::load(
                &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
            )
            .unwrap();
            WorldLocations::build(&m, &w.world, &w.roads).unwrap()
        })
        .clone()
}
fn tick(s: &mut SaveKennySession, w: &mut SouthParkScene, n: usize) {
    for _ in 0..n {
        s.update(0.1, w, VoiceClock::default()).unwrap();
    }
}
fn search(s: &mut SaveKennySession, w: &mut SouthParkScene) {
    for _ in 0..200 {
        if s.state == SessionState::Searching {
            return;
        }
        s.update(0.1, w, VoiceClock::default()).unwrap();
    }
    panic!("No automatic search");
}
fn arrive(s: &mut SaveKennySession, w: &mut SouthParkScene) {
    w.player.feet = w.threats.stage(s.current()) + Vector3::new(-16.0, 0.0, -2.0);
    assert!(w.world.can_occupy(w.player.feet, 0.35, 1.8));
    for _ in 0..80 {
        s.update(0.1, w, VoiceClock::default()).unwrap();
        if s.state == SessionState::Active {
            return;
        }
    }
    panic!("Could not enter verified event");
}
fn bounds() -> MapBounds {
    MapBounds {
        min: Vector2::new(-350.0, -175.0),
        max: Vector2::new(370.0, 160.0),
    }
}

#[test]
fn five_sites_per_kind_are_accessible_and_spread_across_town() {
    let sites = catalog();
    assert_eq!(sites.exploration_spawns.len(), 9);
    for kind in ThreatKind::ALL {
        assert_eq!(
            sites
                .sites
                .iter()
                .filter(|s| s.compatible.contains(&kind))
                .count(),
            5
        );
    }
    let mut world = scene().world;
    world.obstacles.retain(|o| o.id != KENNY_ID);
    for p in &sites.exploration_spawns {
        assert!(world.can_occupy(Vector3::new(p[0], p[1], p[2]), 0.28, 1.35));
    }
    assert!(sites.sites.iter().any(|s| s.centre[0] < -140.0));
    assert!(sites.sites.iter().any(|s| s.centre[2] > 60.0));
}
#[test]
fn location_selection_never_repeats_last_when_alternatives_exist() {
    let sites = catalog();
    let mut seed = 18;
    let mut last = None;
    for kind in ThreatKind::ALL.into_iter().cycle().take(100) {
        let site = sites.choose(kind, last.as_deref(), &mut seed).unwrap();
        assert_ne!(Some(&site.id), last.as_ref());
        last = Some(site.id);
    }
}
#[test]
fn random_spawns_remain_safe_and_allow_the_existing_walk() {
    let mut world = scene();
    let mut s = SaveKennySession::default();
    s.enable_world(catalog(), 131);
    let mut selected = std::collections::BTreeSet::new();
    for _ in 0..25 {
        s.randomize_exploration(&mut world).unwrap();
        let p = world.kenny.position();
        selected.insert(format!("{p:?}"));
        assert_eq!(p.y, 0.0);
        assert!((p - world.player.feet).length() > 1.0);
        tick(&mut s, &mut world, 20);
        assert!((world.kenny.position() - p).length() > 0.1);
    }
    assert!(selected.len() >= 5);
}
#[test]
fn every_translated_layout_preserves_success_and_failure() {
    for site in catalog().sites {
        for &kind in &site.compatible {
            let mut w = scene();
            for win in [true, false] {
                w.start_threat_at(kind, site.translation()).unwrap();
                for _ in 0..1200 {
                    if w.threats.result.is_some() {
                        break;
                    }
                    if win {
                        for target in w.threats.targets() {
                            w.threats.interact(&target.id, &mut w.kenny);
                        }
                    }
                    w.update_kenny(0.1);
                }
                assert_eq!(
                    w.threats.result,
                    Some(if win {
                        ThreatResult::Survived
                    } else {
                        ThreatResult::Died
                    }),
                    "{kind:?} {}",
                    site.id
                );
                w.clear_threat().unwrap();
            }
        }
    }
}
#[test]
fn distant_search_is_finite_and_does_not_teleport_player() {
    let mut w = scene();
    let original = w.player.feet;
    let mut s = SaveKennySession::default();
    s.enable_world(catalog(), 44);
    s.start(&mut w, 12).unwrap();
    search(&mut s, &mut w);
    assert!(s.search_budget > 0.0 && s.search_budget < 20.0);
    let mut began = false;
    for _ in 0..1600 {
        s.update(0.1, &mut w, VoiceClock::default()).unwrap();
        began |= s.state == SessionState::Active;
        if s.state == SessionState::DeathDialogue {
            break;
        }
    }
    assert!(began);
    assert_eq!(s.state, SessionState::DeathDialogue);
    assert_eq!(w.player.feet, original);
    assert_eq!((s.lives, s.saves), (3, 0));
}
#[test]
fn complete_match_advances_automatically_without_continue_calls() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    s.enable_world(catalog(), 75);
    s.start(&mut w, 18).unwrap();
    let mut passed = vec![];
    let mut last = 0;
    for _ in 0..3000 {
        if s.state == SessionState::Searching {
            arrive(&mut s, &mut w);
        }
        if s.state == SessionState::Active {
            for t in w.threats.targets() {
                w.threats.interact(&t.id, &mut w.kenny);
            }
        }
        s.update(0.1, &mut w, VoiceClock::default()).unwrap();
        if s.saves > last {
            passed.push(s.current());
            last = s.saves;
        }
        if s.state == SessionState::Victory {
            break;
        }
    }
    assert_eq!(s.state, SessionState::Victory);
    assert_eq!((s.saves, s.lives), (5, 3));
    assert!(ThreatKind::ALL.iter().all(|k| passed.contains(k)));
}
#[test]
fn map_freezes_each_threat_and_restores_exact_previous_state() {
    for kind in ThreatKind::ALL {
        let mut s = SaveKennySession::default();
        let mut w = scene();
        w.start_threat(kind).unwrap();
        s.state = SessionState::Preparing;
        tick(&mut s, &mut w, 40);
        if kind == ThreatKind::Piano {
            while w.threats.state != ThreatState::Resolving {
                s.update(0.1, &mut w, VoiceClock::default()).unwrap();
            }
        }
        let state = s.state;
        let time = s.elapsed;
        let npc = w.kenny.position();
        let phase = w.kenny.animation.phase;
        let frozen = (
            w.threats.elapsed,
            w.threats.state_time,
            w.threats.progress,
            w.threats.piano_position(),
            w.threats.snow_position(),
        );
        assert!(s.open_map());
        assert!(!s.permits_interaction());
        tick(&mut s, &mut w, 500);
        assert_eq!(
            frozen,
            (
                w.threats.elapsed,
                w.threats.state_time,
                w.threats.progress,
                w.threats.piano_position(),
                w.threats.snow_position()
            )
        );
        assert_eq!(w.kenny.position(), npc);
        assert_eq!(w.kenny.animation.phase, phase);
        s.continue_match(&mut w).unwrap();
        assert_eq!(s.state, state);
        assert_eq!(s.elapsed, time);
        s.update(0.1, &mut w, VoiceClock::default()).unwrap();
        assert!(w.threats.elapsed > frozen.0 || w.threats.state_time > frozen.1);
    }
}
#[test]
fn menu_priority_and_marker_visibility_are_explicit() {
    let mut s = SaveKennySession::default();
    assert!(!s.location_visible());
    assert!(s.open_map());
    assert!(!s.in_match());
    for state in [
        SessionState::Briefing,
        SessionState::Paused,
        SessionState::DeathAnimation,
        SessionState::DeathDialogue,
        SessionState::Victory,
        SessionState::Defeat,
        SessionState::Relocating,
    ] {
        s.state = state;
        assert!(!s.open_map());
    }
    s.state = SessionState::Searching;
    assert!(s.location_visible());
    s.state = SessionState::Success;
    assert!(!s.location_visible());
}
#[test]
fn coordinate_projection_has_no_axis_flip_and_is_reversible() {
    let b = bounds();
    assert_eq!(
        b.project(Vector3::new(-350.0, 0.0, -175.0)),
        Vector2::zero()
    );
    assert_eq!(b.project(Vector3::new(370.0, 0.0, 160.0)), Vector2::one());
    let player = Vector3::new(-43.2, 0.0, -42.9);
    assert!((b.unproject(b.project(player)) - player).length() < 0.001);
    let centre = Vector2::new(100.0, 100.0);
    for (target, expected) in [
        (player + Vector3::right() * 10.0, Vector2::new(110.0, 100.0)),
        (
            player + Vector3::new(0.0, 0.0, -10.0),
            Vector2::new(100.0, 90.0),
        ),
    ] {
        assert!((circle_marker(player, target, centre, 50.0, 50.0).0 - expected).length() < 0.001);
    }
    let (edge, out) = circle_marker(
        player,
        player + Vector3::right() * 1000.0,
        centre,
        50.0,
        50.0,
    );
    assert!(out);
    assert!((edge - centre).length() <= 50.001);
}
#[test]
fn map_zoom_pan_and_recenter_are_bounded() {
    let mut v = MapView::new(bounds());
    let size = Vector2::new(1000.0, 600.0);
    let centre = v.centre;
    v.zoom_by(10000.0, size);
    assert_eq!(v.zoom, 6.0);
    v.pan(Vector2::new(100000.0, -100000.0), size);
    assert!(v.centre.x >= v.bounds.min.x && v.centre.y <= v.bounds.max.y);
    v.recenter(Vector3::new(-43.2, 0.0, -42.9), size);
    assert!((v.centre - Vector2::new(-43.2, -42.9)).length() < 0.001);
    v.zoom_by(f32::NAN, size);
    assert_eq!(v.zoom, 6.0);
    v.zoom_by(-10000.0, size);
    assert_eq!(v.zoom, 1.0);
    assert_eq!(v.centre, centre);
}
#[test]
fn dynamic_return_to_exploration_cleans_and_places_kenny_nearby() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    s.enable_world(catalog(), 31);
    s.start(&mut w, 42).unwrap();
    search(&mut s, &mut w);
    s.explore(&mut w).unwrap();
    assert_eq!(s.state, SessionState::Relocating);
    tick(&mut s, &mut w, 8);
    assert!(s.assert_exploration_clean(&w));
    assert!((4.0..=24.0).contains(&(w.kenny.position() - w.player.feet).length()));
}
