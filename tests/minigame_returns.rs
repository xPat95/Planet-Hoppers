use planet_hoppers::{
    minigames::{
        common::{
            host::{MinigameCompletion, MinigameResult, ReturnConfirmation},
            manifest::SceneManifest,
        },
        southpark::{
            locations::WorldLocations,
            session::{SaveKennySession, SessionState, VoiceClock},
            threats::PREFIX,
            DevConfig, SouthParkScene,
        },
    },
    scenes::southpark_entry::restart_attempt,
};
use std::path::PathBuf;

fn fixture() -> (SceneManifest, DevConfig, SouthParkScene) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )
    .unwrap();
    let mut config = DevConfig::load(&root.join("config/southpark-dev.json")).unwrap();
    config.test_objects = false;
    let scene = SouthParkScene::load(&manifest, config.clone()).unwrap();
    (manifest, config, scene)
}

fn snapshot(scene: &SouthParkScene) -> Vec<f32> {
    let mut values = vec![
        scene.kenny.animation.phase,
        scene.kenny.animation.weight,
        scene.threats.elapsed,
        scene.threats.state_time,
        scene.threats.progress,
        scene.player.yaw,
        scene.player.pitch,
    ];
    for p in [
        scene.player.feet,
        scene.kenny.position(),
        scene.threats.bus_position(),
        scene.threats.ufo_position(),
        scene.threats.piano_position(),
        scene.threats.snow_position(),
    ] {
        values.extend([p.x, p.y, p.z]);
    }
    values
}

fn balance(c: &planet_hoppers::minigames::southpark::session::SessionConfig) -> [f32; 14] {
    [
        c.death_animation_seconds,
        c.voice_fallback_seconds,
        c.kyle_switch_seconds,
        c.success_hold_seconds,
        c.text_characters_per_second,
        c.world_interval_seconds,
        c.activation_max_seconds,
        c.arrival_distance,
        c.relocation_seconds,
        c.zone_detection_distance,
        c.search_min_seconds,
        c.search_max_seconds,
        c.search_orientation_seconds,
        c.search_route_factor,
    ]
}

#[test]
fn abandonment_requires_a_pending_confirmation_and_cancel_discards_it() {
    let mut confirmation = ReturnConfirmation::default();
    assert_eq!(confirmation.confirm(), None);
    confirmation.request();
    confirmation.cancel();
    assert_eq!(confirmation.confirm(), None);
    confirmation.request();
    assert_eq!(confirmation.confirm(), Some(MinigameResult::Abandoned));
    assert_eq!(confirmation.confirm(), None);
}

#[test]
fn result_contract_preserves_identity_for_all_six_minigames() {
    for id in [
        "halloween",
        "starwars",
        "smilingfriends",
        "minecraft",
        "regularshow",
        "southpark",
    ] {
        for result in [
            MinigameResult::Victory,
            MinigameResult::Defeat,
            MinigameResult::Abandoned,
        ] {
            let completion = MinigameCompletion {
                planet_id: id.into(),
                minigame_id: id.into(),
                result,
            };
            let encoded = serde_json::to_string(&completion).unwrap();
            assert_eq!(
                serde_json::from_str::<MinigameCompletion>(&encoded).unwrap(),
                completion
            );
        }
    }
}

#[test]
fn pause_freezes_every_gameplay_phase_and_restores_its_exact_clock() {
    for phase in [
        SessionState::Exploring,
        SessionState::Briefing,
        SessionState::Searching,
        SessionState::Preparing,
        SessionState::Active,
        SessionState::Success,
        SessionState::Interlude,
        SessionState::Relocating,
        SessionState::DeathAnimation,
        SessionState::DeathDialogue,
        SessionState::Restoring,
    ] {
        let (_, _, mut scene) = fixture();
        let mut session = SaveKennySession::default();
        session.start(&mut scene, 12).unwrap();
        session.continue_match(&mut scene).unwrap();
        session.state = phase;
        session.elapsed = 0.375;
        session.activation_elapsed = 7.125;
        session.voice_time = 0.25;
        scene.kenny.animation.weight = 0.8;
        let before = snapshot(&scene);
        session.pause();
        assert_eq!(session.state, SessionState::Paused, "{phase:?}");
        for _ in 0..600 {
            session
                .update(
                    0.1,
                    &mut scene,
                    VoiceClock {
                        position: Some(99.0),
                        finished: true,
                    },
                )
                .unwrap();
        }
        assert_eq!(snapshot(&scene), before, "{phase:?}");
        assert_eq!(session.activation_elapsed, 7.125);
        assert_eq!(session.voice_time, 0.25);
        session.continue_match(&mut scene).unwrap();
        assert_eq!(session.state, phase);
        assert_eq!(session.elapsed, 0.375);
    }
}

#[test]
fn retry_rebuilds_the_whole_attempt_without_changing_balance() {
    for dynamic in [false, true] {
        let (manifest, config, mut scene) = fixture();
        let mut session = SaveKennySession::default();
        if dynamic {
            session.enable_world(
                WorldLocations::build(&manifest, &scene.world, &scene.roads).unwrap(),
                42,
            );
        }
        let original_balance = balance(&session.config);
        session.start(&mut scene, 13).unwrap();
        if !dynamic {
            session.continue_match(&mut scene).unwrap();
        }
        session.lives = 1;
        session.saves = 3;
        session.activation_elapsed = 19.0;
        session.voice_time = 2.0;
        session.error = Some("previous attempt".into());
        scene.confirmation = Some(("previous attempt".into(), 10.0));
        scene.kenny.animation.weight = 1.0;
        session.pause();
        restart_attempt(&mut scene, &mut session, &manifest, &config, 14).unwrap();
        assert_eq!(
            session.state,
            if dynamic {
                SessionState::Interlude
            } else {
                SessionState::Briefing
            }
        );
        assert_eq!((session.lives, session.saves), (3, 0));
        assert_eq!(session.activation_elapsed, 0.0);
        assert_eq!(session.voice_time, 0.0);
        assert_eq!(session.elapsed, 0.0);
        assert!(session.site.is_none() && session.error.is_none());
        assert!(scene.confirmation.is_none() && scene.threats.kind.is_none());
        assert!(!scene
            .world
            .obstacles
            .iter()
            .any(|o| o.id.starts_with(PREFIX)));
        assert_eq!(scene.kenny.animation.weight, 0.0);
        assert_eq!(balance(&session.config), original_balance);
    }
}
