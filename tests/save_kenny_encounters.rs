use planet_hoppers::minigames::{
    common::manifest::SceneManifest,
    southpark::{
        encounters::{script, Adventure, MEMBERS},
        locations::WorldLocations,
        puzzles::{Board, BUS_SOLUTION, HEATER_SOLUTION},
        session::{SaveKennySession, SessionState, VoiceClock},
        threats::ThreatState,
        DevConfig, SouthParkScene,
    },
};
use raylib::prelude::Vector3;
use std::path::Path;

#[test]
fn exploration_encounters_all_2d_puzzles_and_victory() {
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
    let exploration_kenny = scene.kenny.position();
    for _ in 0..300 {
        s.update(1.0, &mut scene, VoiceClock::default()).unwrap();
    }
    assert_eq!(s.state, SessionState::Exploring);
    assert_eq!(scene.kenny.position(), exploration_kenny, "Mission exploration must not move the legacy 3D Kenny actor");
    assert!(!scene.world.obstacles.iter().any(|o|o.id==planet_hoppers::minigames::southpark::kenny::KENNY_ID), "No invisible Kenny collider in exploration");
    assert_eq!(s.lives, 3);
    // A failed encounter consumes one life after the existing Stan/Kyle reaction,
    // returns to exploration, and stays available for retry.
    s.begin_encounter(&mut scene, 0).unwrap();
    let feet = scene.player.feet;
    let kenny = scene.kenny.position();
    for _ in 0..5 {
        s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
    }
    s.puzzle.as_mut().unwrap().start();
    for _ in 0..1800 {
        s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
        if s.state == SessionState::Restoring {
            break;
        }
    }
    assert_eq!(s.state, SessionState::Restoring);
    for _ in 0..100 {
        s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
    }
    assert_eq!(s.lives, 2); // waiting on the failure page cannot charge another life
    assert_eq!(scene.player.feet, feet);
    assert_eq!(scene.kenny.position(), kenny);
    s.leave_puzzle();
    for _ in 0..5 {
        s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
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
        assert!(s.blocks_controls());
        assert!(!s.open_map());
        let position = scene.player.feet;
        let kenny = scene.kenny.position();
        let yaw = scene.player.yaw;
        // Reading the illustrated instructions never consumes the puzzle timer.
        s.update(1000.0, &mut scene, VoiceClock::default()).unwrap();
        assert_eq!(scene.threats.state, ThreatState::Inactive);
        for _ in 0..5 {
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
        }
        assert_eq!(s.puzzle.as_ref().unwrap().elapsed, 0.0);
        s.puzzle.as_mut().unwrap().start();
        assert_eq!(s.state, SessionState::Active);
        for _ in 0..1500 {
            let p = s.puzzle.as_mut().unwrap();
            match &p.board {
                Board::Heaters(h) => {
                    if let Some(i) = (0..16).find(|i| h.tiles[*i] != HEATER_SOLUTION[*i]) {
                        p.click(i);
                    }
                }
                Board::Ufo(u) => {
                    let target = u.target;
                    p.click(target);
                }
                Board::Bus(b) => {
                    let next = BUS_SOLUTION[b.entered.len().min(3)];
                    p.click(next);
                }
                Board::Piano(k) => {
                    if !k.demonstrating {
                        let next = k.notes[k.input];
                        p.click(next);
                    }
                }
                Board::Sewers(sewers) => {
                    let open = sewers.open;
                    for (i, value) in open.into_iter().enumerate() {
                        if value {
                            p.click(i);
                        }
                    }
                }
            }
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
            if s.state == SessionState::Success {
                break;
            }
        }
        assert_eq!(s.state, SessionState::Success);
        assert_eq!(
            (scene.player.feet, scene.player.yaw, scene.kenny.position()),
            (position, yaw, kenny)
        );
        let saves = s.saves;
        for _ in 0..100 {
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
        }
        assert_eq!(s.saves, saves);
        s.leave_puzzle();
        for _ in 0..5 {
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
        }
        assert!(s.puzzle.is_none());
        assert!(s.adventure.as_ref().unwrap().completed[group]);
        assert!(s.begin_encounter(&mut scene, group).is_err());
        assert_eq!(s.lives, 3);
    }
    assert_eq!(s.saves, 5);
    assert_eq!(s.state, SessionState::Victory);
    use planet_hoppers::minigames::common::host::{AttemptOutcome, MinigameResult};
    assert_eq!(s.attempt_outcome(), Some(MinigameResult::Victory));
}

#[test]
fn three_puzzle_failures_use_original_death_reaction_and_end_the_attempt() {
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
    s.begin_encounter(&mut scene, 0).unwrap();
    for attempt in 0..3 {
        let p = s.puzzle.as_mut().unwrap();
        p.tick(0.32);
        p.start();
        p.tick(20.0);
        assert_eq!(p.result, Some(false));
        for _ in 0..150 {
            s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
        }
        assert_eq!(s.lives, 2 - attempt);
        assert_eq!(s.saves, 0);
        let cues = s.take_cues();
        assert_eq!(
            cues.iter()
                .filter(|c| **c == planet_hoppers::audio::AudioCue::DeathVoice)
                .count(),
            1
        );
        assert!(!s.adventure.as_ref().unwrap().completed[0]);
        if attempt < 2 {
            assert_eq!(s.state, SessionState::Restoring);
            s.restart_puzzle();
            assert_eq!(s.state, SessionState::Active);
            assert!(!s.puzzle.as_ref().unwrap().started);
        }
    }
    assert_eq!(s.state, SessionState::Defeat);
    s.restart_puzzle();
    assert_eq!(s.state, SessionState::Defeat);
    s.leave_puzzle();
    for _ in 0..5 {
        s.update(0.1, &mut scene, VoiceClock::default()).unwrap();
    }
    assert!(s.puzzle.is_none());
    assert_eq!(s.state, SessionState::Defeat);
    use planet_hoppers::minigames::common::host::{AttemptOutcome, MinigameResult};
    assert_eq!(s.attempt_outcome(), Some(MinigameResult::Defeat));
}
