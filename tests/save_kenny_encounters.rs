use planet_hoppers::minigames::{
    common::manifest::SceneManifest,
    southpark::{
        encounters::{script, Adventure, MEMBERS},
        locations::WorldLocations,
        session::{SaveKennySession, SessionState, VoiceClock},
        threats::{ThreatKind, ThreatState},
        DevConfig, SouthParkScene,
    },
};
use raylib::prelude::Vector3;
use std::path::Path;

#[test]
fn exploration_encounters_all_existing_challenges_and_victory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut config = DevConfig::load(&root.join("config/southpark-dev.json")).unwrap();
    config.test_objects = false;
    let mut scene = SouthParkScene::load(&manifest, config).unwrap();
    let locations = WorldLocations::build(&manifest, &scene.world, &scene.roads).unwrap();
    let mut s = SaveKennySession::default();
    s.adventure = Some(Adventure::build(&scene, &locations.sites).unwrap());
    s.enable_world(locations, 1);
    assert_eq!(
        s.adventure
            .as_ref()
            .unwrap()
            .groups
            .iter()
            .map(|g| g.positions.len())
            .sum::<usize>(),
        8
    );
    for i in 0..5 {
        assert!(script(i)
            .iter()
            .all(|l| MEMBERS[i].contains(&l.speaker) && !l.en.is_empty() && !l.es.is_empty()));
    }
    for _ in 0..300 {
        s.update(1.0, &mut scene, VoiceClock::default()).unwrap();
    }
    assert_eq!(s.state, SessionState::Exploring);
    assert_eq!(s.lives, 3);
    // A failed encounter consumes one life after the existing Stan/Kyle reaction,
    // returns to exploration, and stays available for retry.
    s.begin_encounter(&mut scene, 0).unwrap();
    scene.player.feet = scene.threats.stage(s.current()) + Vector3::new(-16.0, 0.0, -2.0);
    for _ in 0..1800 {
        s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
        if s.state == SessionState::Exploring {
            break;
        }
    }
    assert_eq!(s.state, SessionState::Exploring);
    assert_eq!(s.lives, 2);
    assert!(!s.adventure.as_ref().unwrap().completed[0]);
    assert!(s.adventure.as_ref().unwrap().active.is_none());
    assert!(s
        .take_cues()
        .contains(&planet_hoppers::audio::AudioCue::DeathVoice));
    s.start(&mut scene, 1).unwrap();
    assert_eq!(s.lives, 3);
    for group in [3, 0, 4, 1, 2] {
        let npc = s.adventure.as_ref().unwrap().groups[group].positions[0];
        scene.player.feet = npc + Vector3::new(-1.0, 0.0, 0.0);
        assert_eq!(s.adventure.as_ref().unwrap().nearby(&scene), Some(group));
        s.begin_encounter(&mut scene, group).unwrap();
        assert!(s.begin_encounter(&mut scene, group).is_err());
        // Searching has no deadline, even with a long frame and an open map.
        scene.player.feet = Vector3::new(-43.2, 0.0, -42.9);
        s.update(1000.0, &mut scene, VoiceClock::default()).unwrap();
        assert_eq!(scene.threats.state, ThreatState::Preparing);
        scene.player.feet = scene.threats.stage(s.current()) + Vector3::new(-16.0, 0.0, -2.0);
        for _ in 0..200 {
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
            if s.state == SessionState::Active {
                break;
            }
        }
        assert_eq!(s.state, SessionState::Active);
        let ids: Vec<String> = match s.current() {
            ThreatKind::Bus => vec!["threat:brake".into()],
            ThreatKind::Piano => vec!["threat:piano-lock".into()],
            ThreatKind::Sewer => vec!["threat:pickup".into(), "threat:place".into()],
            ThreatKind::Ufo => (0..3).map(|i| format!("threat:light-{i}")).collect(),
            ThreatKind::Snowman => (0..3).map(|i| format!("threat:heater-{i}")).collect(),
        };
        for id in ids {
            assert!(scene.threats.interact(&id, &mut scene.kenny), "{id}");
        }
        for _ in 0..1500 {
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
            if matches!(s.state, SessionState::Exploring | SessionState::Victory) {
                break;
            }
        }
        assert!(s.adventure.as_ref().unwrap().completed[group]);
        assert!(s.begin_encounter(&mut scene, group).is_err());
        assert_eq!(s.lives, 3);
    }
    assert_eq!(s.saves, 5);
    assert_eq!(s.state, SessionState::Victory);
}
