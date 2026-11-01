use planet_hoppers::{
    minigames::{
        common::{fps::PlayerInput, interaction::select_target_with, manifest::SceneManifest},
        southpark::{
            kenny::KENNY_ID,
            locations::WorldLocations,
            navigation::{event_route, route_length},
            session::{SaveKennySession, SessionState, VoiceClock},
            threats::{ThreatKind, ThreatResult},
            DevConfig, SouthParkScene,
        },
    },
    scenes::southpark_entry::{EntryConfig, SouthParkEntry, SouthParkVisits},
};
use raylib::prelude::*;
use serde_json::json;
use std::{path::PathBuf, sync::OnceLock};
fn scene() -> SouthParkScene {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut cfg = DevConfig::load(&root.join("config/southpark-dev.json")).unwrap();
    cfg.test_objects = false;
    SouthParkScene::load(&manifest, cfg).unwrap()
}
fn catalog() -> WorldLocations {
    static C: OnceLock<WorldLocations> = OnceLock::new();
    C.get_or_init(|| {
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
fn setup(kind: ThreatKind, site: usize) -> (SaveKennySession, SouthParkScene) {
    setup_at(kind, site, false)
}
fn setup_at(kind: ThreatKind, site: usize, near: bool) -> (SaveKennySession, SouthParkScene) {
    let mut w = scene();
    let mut s = SaveKennySession::default();
    let mut c = catalog();
    let selected = c.sites[site].clone();
    if near {
        w.player.feet = Vector3::new(selected.entry[0], 0.0, selected.entry[2] - 3.0);
        assert!(w.world.can_occupy(w.player.feet, 0.35, 1.8));
    }
    c.sites = vec![selected];
    s.enable_world(c, 11);
    s.start(&mut w, 15).unwrap();
    s.order[0] = kind;
    for _ in 0..100 {
        s.update(0.1, &mut w, VoiceClock::default()).unwrap();
        if s.state == SessionState::Searching {
            return (s, w);
        }
    }
    panic!("No search");
}
fn step(s: &mut SaveKennySession, w: &mut SouthParkScene) {
    s.update(0.05, w, VoiceClock::default()).unwrap();
}
#[test]
fn total_deadline_includes_interval_relocation_search_and_preparation() {
    let mut report = vec![];
    for kind in ThreatKind::ALL {
        for site in 0..5 {
            for dt in [1.0 / 30.0, 1.0 / 60.0, 1.0 / 144.0, 0.073] {
                let mut w = scene();
                let mut s = SaveKennySession::default();
                let mut locations = catalog();
                locations.sites = vec![locations.sites[site].clone()];
                s.enable_world(locations, 44);
                s.start(&mut w, 12).unwrap();
                s.order[0] = kind;
                let mut time = 0.0_f64;
                let mut moved = false;
                for _ in 0..4000 {
                    let before = time;
                    s.update(dt, &mut w, VoiceClock::default()).unwrap();
                    time += f64::from(dt);
                    if s.state == SessionState::Active {
                        // The update that crosses the deadline already advances real danger.
                        assert!(
                            w.threats.elapsed > 0.0,
                            "{kind:?}/{site}: active but frozen"
                        );
                        assert!(
                            before < 20.000001,
                            "{kind:?}/{site}: motion started at {before}"
                        );
                        assert!(time <= 20.0 + f64::from(dt) + 0.000001);
                        if kind == ThreatKind::Piano {
                            assert!(w.threats.piano_sway().abs() > 0.0);
                        }
                        report.push(json!({"kind":format!("{kind:?}"),"site":site,"dt":dt,
                            "motion_update_start_seconds":before,"observed_active_seconds":time,
                            "hazard_elapsed":w.threats.elapsed}));
                        moved = true;
                        break;
                    }
                }
                assert!(moved, "{kind:?}/{site}: no motion");
            }
        }
    }
    std::fs::write(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/kenny-deadline.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
}
#[test]
fn deadline_is_not_extended_by_a_long_frame_or_by_retrying_after_death() {
    let mut w = scene();
    let mut s = SaveKennySession::default();
    s.enable_world(catalog(), 44);
    s.start(&mut w, 12).unwrap();
    s.update(19.9, &mut w, VoiceClock::default()).unwrap();
    s.update(0.1, &mut w, VoiceClock::default()).unwrap();
    assert_eq!(s.state, SessionState::Active);
    assert!(w.threats.elapsed > 0.0);
    while s.state != SessionState::Restoring {
        step(&mut s, &mut w);
    }
    assert_eq!(s.lives, 2);
    let pending = s.current();
    step(&mut s, &mut w); // Restoring enters the new pending event; controls are still blocked.
    assert_eq!(s.activation_elapsed, 0.0);
    let mut time = 0.0_f64;
    while s.state != SessionState::Active && time < 20.1 {
        step(&mut s, &mut w);
        time += 0.05;
    }
    assert_eq!(s.state, SessionState::Active);
    assert!(time <= 20.05);
    assert_eq!(s.current(), pending);
    assert_eq!((s.lives, s.saves), (2, 0));
    assert!(w.threats.elapsed > 0.0);
}
#[test]
fn deadline_counts_success_hold_and_survives_map_and_pause_without_resetting() {
    let mut w = scene();
    let mut s = SaveKennySession::default();
    s.enable_world(catalog(), 44);
    s.start(&mut w, 12).unwrap();
    while s.state != SessionState::Searching {
        step(&mut s, &mut w);
    }
    let clock = s.activation_elapsed;
    assert!(s.open_map());
    for _ in 0..400 {
        step(&mut s, &mut w);
    }
    assert_eq!(s.activation_elapsed, clock);
    s.continue_match(&mut w).unwrap();
    s.pause();
    for _ in 0..400 {
        step(&mut s, &mut w);
    }
    assert_eq!(s.activation_elapsed, clock);
    s.continue_match(&mut w).unwrap();
    while s.state != SessionState::Active {
        step(&mut s, &mut w);
    }
    for _ in 0..100 {
        for t in w.threats.targets() {
            w.threats.interact(&t.id, &mut w.kenny);
        }
        step(&mut s, &mut w);
        if s.state == SessionState::Success {
            break;
        }
    }
    assert_eq!(s.state, SessionState::Success);
    assert_eq!(s.activation_elapsed, 0.0);
    let mut time = 0.0;
    while s.state != SessionState::Active && time < 20.1 {
        step(&mut s, &mut w);
        time += 0.05;
    }
    assert_eq!(s.state, SessionState::Active);
    assert!(time <= 20.05);
    assert!(w.threats.elapsed > 0.0);
    assert_eq!((s.lives, s.saves), (3, 1));
}
#[test]
fn intro_visit_skip_and_camera_clearance() {
    let mut w = scene();
    let mut s = SaveKennySession::default();
    s.place_kenny_near_player(&mut w).unwrap();
    assert!((4.0..=14.0).contains(&(w.kenny.position() - w.player.feet).length()));
    let cfg = EntryConfig::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/southpark-entry.json"),
    )
    .unwrap();
    for skip in [false, true] {
        let mut visits = SouthParkVisits::default();
        let mut e = SouthParkEntry::new(&visits, cfg.clone(), Some(27.376326), &w).unwrap();
        assert_eq!(e.duration, 27.376326);
        assert!(e.full_intro);
        e.validate(&w.world).unwrap();
        assert!((e.camera_at(1.0).position - w.player.eye()).length() < 0.001);
        for _ in 0..280 {
            e.update(0.1, skip, &mut visits);
        }
        assert!(!e.active && visits.intro_seen);
        let next = SouthParkEntry::new(&visits, cfg.clone(), Some(27.376326), &w).unwrap();
        assert!(!next.full_intro);
        assert_eq!(next.duration, 0.7);
        s.start(&mut w, 12).unwrap();
        assert!(visits.intro_seen); // Match controller cannot reset visit memory.
    }
}
#[test]
fn cinematic_uses_audio_clock_and_does_not_cut_the_last_seconds() {
    let w = scene();
    let cfg = EntryConfig::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/southpark-entry.json"),
    )
    .unwrap();
    let mut visits = SouthParkVisits::default();
    let mut entry = SouthParkEntry::new(&visits, cfg, Some(27.376326), &w).unwrap();
    for position in [0.0, 24.0, 27.0, 27.376326] {
        entry.update_audio(
            0.1,
            false,
            &mut visits,
            VoiceClock {
                position: Some(position),
                finished: false,
            },
        );
        assert!(entry.active && !visits.intro_seen);
        assert_eq!(entry.elapsed, position);
    }
    entry.update_audio(
        0.0,
        false,
        &mut visits,
        VoiceClock {
            position: Some(27.376326),
            finished: true,
        },
    );
    assert!(!entry.active && visits.intro_seen);
    assert!((entry.camera().position - w.player.eye()).length() < 0.001);
}
#[test]
fn all_five_idle_threats_expire_and_cost_exactly_one_life() {
    let mut report = vec![];
    for kind in ThreatKind::ALL {
        for site in 0..5 {
            let (mut s, mut w) = setup(kind, site);
            let budget = s.search_budget;
            let start = w.player.feet;
            let mut active = false;
            let mut voice = 0;
            let mut seconds = 0.0;
            for _ in 0..3600 {
                step(&mut s, &mut w);
                seconds += 0.05;
                active |= s.state == SessionState::Active;
                voice += s
                    .take_cues()
                    .iter()
                    .filter(|c| **c == planet_hoppers::audio::AudioCue::DeathVoice)
                    .count();
                if s.state == SessionState::Restoring {
                    break;
                }
            }
            assert!(active, "{kind:?}/{site}");
            assert_eq!(w.threats.result, Some(ThreatResult::Died));
            assert_eq!((s.lives, s.saves, voice), (2, 0, 1), "{kind:?}/{site}");
            assert_eq!(w.player.feet, start);
            assert!(seconds >= budget);
            report.push(json!({"kind":format!("{kind:?}"),"site":site,"search_seconds":budget,
                "route_metres":s.search_route_metres,"death_sequence_seconds":seconds,"lives":s.lives,"voice_cues":voice}));
        }
    }
    std::fs::write(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/phase6b-idle.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
}
fn route(w: &SouthParkScene, a: Vector3, b: Vector3) -> Vec<Vector3> {
    let mut world = w.world.clone();
    world.obstacles.retain(|o| o.id != KENNY_ID);
    event_route(&w.roads, &world, a, b, 0.35, 1.8).unwrap()
}
fn walk(w: &mut SouthParkScene, path: &[Vector3], index: &mut usize) {
    while *index < path.len() && (path[*index] - w.player.feet).length() < 0.12 {
        *index += 1;
    }
    if *index >= path.len() {
        return;
    }
    let d = path[*index] - w.player.feet;
    w.player.yaw = d.z.atan2(d.x);
    let distance = d.length();
    w.player.update(
        PlayerInput {
            forward: 1.0,
            sprint: true,
            ..Default::default()
        },
        0.05_f32.min(distance / w.player.config.run_speed),
        &w.world,
    );
}
#[test]
fn nearby_rescues_remain_possible_by_running_and_shared_raycasting() {
    let mut report = vec![];
    for kind in ThreatKind::ALL {
        for site in 0..5 {
            let (mut s, mut w) = setup_at(kind, site, true);
            let entry = s.search_entry;
            let mut path = route(&w, w.player.feet, entry);
            let mut index = 0;
            w.threats
                .validate_access(kind, &w.world, &w.roads, entry)
                .unwrap();
            let mut target_key = String::new();
            let mut actions = 0;
            let mut travel = 0.0;
            let mut reaction = 0.5;
            for _ in 0..3600 {
                if matches!(
                    s.state,
                    SessionState::Searching | SessionState::Preparing | SessionState::Active
                ) {
                    if s.state == SessionState::Active {
                        if let Some(target) = w.threats.targets().first() {
                            if target.id != target_key {
                                target_key = target.id.clone();
                                let mut candidates = vec![];
                                for radius in [2.5, 4.0] {
                                    for angle in 0..16 {
                                        let a = angle as f32 * std::f32::consts::TAU / 16.0;
                                        let dest = Vector3::new(
                                            target.bounds.center().x + radius * a.cos(),
                                            0.0,
                                            target.bounds.center().z + radius * a.sin(),
                                        );
                                        if !w.world.can_occupy(dest, 0.35, 1.8) {
                                            continue;
                                        }
                                        let eye = dest + Vector3::up() * 1.7;
                                        let toward = target.bounds.center() - eye;
                                        if w.world
                                            .ray_distance_ignoring(
                                                eye,
                                                toward.normalized(),
                                                toward.length(),
                                                Some(&target.id),
                                            )
                                            .is_some_and(|d| d < toward.length() - 0.1)
                                        {
                                            continue;
                                        }
                                        let mut clean = w.world.clone();
                                        clean.obstacles.retain(|o| o.id != KENNY_ID);
                                        if let Ok(r) = event_route(
                                            &w.roads,
                                            &clean,
                                            w.player.feet,
                                            dest,
                                            0.35,
                                            1.8,
                                        ) {
                                            candidates.push(r);
                                        }
                                    }
                                }
                                path = candidates
                                    .into_iter()
                                    .min_by(|a, b| route_length(a).total_cmp(&route_length(b)))
                                    .expect("reachable mechanism");
                                index = 0;
                            }
                        }
                    }
                    let old = w.player.feet;
                    walk(&mut w, &path, &mut index);
                    travel += (w.player.feet - old).length();
                    reaction -= 0.05;
                    if s.state == SessionState::Active && reaction <= 0.0 {
                        let targets = w.threats.targets();
                        if let Some(t) = targets.first() {
                            let eye = w.player.eye();
                            let direction = (t.bounds.center() - eye).normalized();
                            if let Some(hit) = select_target_with(
                                &targets,
                                &w.world,
                                eye,
                                direction,
                                w.threats.range(),
                                |i, o, d, r, b| w.threats.refine(&targets[i].id, o, d, r, b),
                            ) {
                                if w.threats.interact(&targets[hit].id, &mut w.kenny) {
                                    actions += 1;
                                    reaction = 0.5;
                                }
                            }
                        }
                    }
                }
                step(&mut s, &mut w);
                if matches!(
                    s.state,
                    SessionState::Success | SessionState::DeathAnimation
                ) {
                    break;
                }
            }
            assert_eq!(s.state,SessionState::Success,"{kind:?}/{site}, p={:?}, target={target_key}, actions={actions}, search={}, route={}, hazard={} next={:?} index={index}",w.player.feet,s.search_budget,route_length(&path),w.threats.elapsed,path.get(index));
            assert_eq!((s.saves, s.lives), (1, 3));
            report.push(json!({"kind":format!("{kind:?}"),"site":site,"run_metres":travel,"actions":actions,"hazard_seconds":w.threats.elapsed,"search_seconds":s.search_budget}));
        }
    }
    std::fs::write(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/phase6b-rescue.json"),
        serde_json::to_string_pretty(&report).unwrap(),
    )
    .unwrap();
}
#[test]
fn map_preserves_search_deadline_and_early_activation_is_one_way() {
    let (mut s, mut w) = setup(ThreatKind::Ufo, 0);
    for _ in 0..20 {
        step(&mut s, &mut w);
    }
    let remaining = s.search_remaining();
    assert!(s.open_map());
    for _ in 0..4000 {
        step(&mut s, &mut w);
    }
    assert_eq!(s.search_remaining(), remaining);
    s.continue_match(&mut w).unwrap();
    let site = s.site.as_ref().unwrap();
    w.player.feet = Vector3::new(site.entry[0], 0.0, site.entry[2]);
    step(&mut s, &mut w);
    assert_eq!(s.state, SessionState::Preparing);
    w.player.feet = Vector3::new(-43.2, 0.0, -42.9);
    for _ in 0..80 {
        step(&mut s, &mut w);
    }
    assert_eq!(s.state, SessionState::Active);
    assert!(w.threats.elapsed > 0.0);
}
