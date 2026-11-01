use planet_hoppers::{
    audio::{AudioConfig, AudioController},
    minigames::{
        common::{
            collision::{Aabb, Obstacle},
            manifest::SceneManifest,
        },
        southpark::{
            session::{MusicMode, SaveKennySession, SessionCue, SessionState, VoiceClock},
            threats::ThreatKind,
            DevConfig, KennyDialogue, SouthParkScene,
        },
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
fn tick(s: &mut SaveKennySession, world: &mut SouthParkScene, seconds: f32) {
    for _ in 0..(seconds * 60.0).ceil() as usize {
        s.update(1.0 / 60.0, world, VoiceClock::default()).unwrap();
    }
}
fn begin(s: &mut SaveKennySession, w: &mut SouthParkScene) {
    if !s.in_match() {
        s.start(w, 42).unwrap();
    }
    s.continue_match(w).unwrap();
    while s.state == SessionState::Preparing {
        s.update(0.1, w, VoiceClock::default()).unwrap();
    }
    assert_eq!(s.state, SessionState::Active);
}
fn win(s: &mut SaveKennySession, w: &mut SouthParkScene) {
    begin(s, w);
    for _ in 0..1000 {
        if s.state == SessionState::Success {
            return;
        }
        for t in w.threats.targets() {
            w.threats.interact(&t.id, &mut w.kenny);
        }
        s.update(0.1, w, VoiceClock::default()).unwrap();
    }
    panic!("Threat did not resolve successfully");
}
fn reach_death_dialogue(s: &mut SaveKennySession, w: &mut SouthParkScene) {
    begin(s, w);
    for _ in 0..1200 {
        if s.state == SessionState::DeathDialogue {
            return;
        }
        s.update(0.1, w, VoiceClock::default()).unwrap();
    }
    panic!("Threat did not fail");
}

#[test]
fn shuffle_is_seeded_and_always_a_permutation_of_five() {
    let mut variants = std::collections::BTreeSet::new();
    for seed in 0..128 {
        let a = SaveKennySession::shuffled(seed);
        assert_eq!(a, SaveKennySession::shuffled(seed));
        assert!(ThreatKind::ALL
            .iter()
            .all(|k| a.iter().filter(|j| *j == k).count() == 1));
        variants.insert(format!("{a:?}"));
    }
    assert!(variants.len() > 30);
}
#[test]
fn invitation_cancel_preserves_free_exploration_and_start_resets_scores() {
    let mut w = scene();
    let mut s = SaveKennySession::default();
    w.interact(w.kenny_target());
    assert_eq!(w.dialogue, KennyDialogue::Invitation);
    w.close_dialogue();
    assert!(!s.in_match());
    assert!(!w.kenny.is_paused());
    s.lives = 1;
    s.saves = 4;
    s.start(&mut w, 5).unwrap();
    assert_eq!((s.lives, s.saves), (3, 0));
    assert_eq!(s.state, SessionState::Briefing);
    assert_eq!(w.dialogue, KennyDialogue::Closed);
    assert!(w.world.can_occupy(w.player.feet, 0.35, 1.8));
}
#[test]
fn reading_briefing_never_starts_danger_or_consumes_time() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    s.start(&mut w, 18).unwrap();
    let p = w.kenny.position();
    tick(&mut s, &mut w, 90.0);
    assert_eq!(s.state, SessionState::Briefing);
    assert_eq!(w.threats.elapsed, 0.0);
    assert_eq!(w.kenny.position(), p);
    assert!(s.take_cues().is_empty());
    s.continue_match(&mut w).unwrap();
    assert_eq!(s.state, SessionState::Preparing);
    assert_eq!(s.take_cues(), vec![SessionCue::Danger]);
    assert_eq!(w.threats.elapsed, 0.0);
}
#[test]
fn five_successes_are_unique_and_counted_once_before_victory() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    let mut passed = vec![];
    let mut cues = vec![];
    for expected in 1..=5 {
        win(&mut s, &mut w);
        passed.push(s.current());
        assert_eq!(s.saves, expected);
        assert_eq!(s.lives, 3);
        tick(&mut s, &mut w, 1.5);
        cues.extend(s.take_cues());
        assert_eq!(s.saves, expected);
        if expected < 5 {
            assert_eq!(s.state, SessionState::Success);
            s.continue_match(&mut w).unwrap();
        }
    }
    assert_eq!(s.state, SessionState::Victory);
    assert!(ThreatKind::ALL
        .iter()
        .all(|k| passed.iter().filter(|j| *j == k).count() == 1));
    assert_eq!(
        cues.iter().filter(|c| **c == SessionCue::Success).count(),
        5
    );
    assert_eq!(
        cues.iter().filter(|c| **c == SessionCue::Victory).count(),
        1
    );
    assert!(!w.threats.running());
    tick(&mut s, &mut w, 40.0);
    assert!(s.take_cues().is_empty());
}
#[test]
fn a_failed_threat_is_retried_only_after_voice_and_costs_one_life() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    reach_death_dialogue(&mut s, &mut w);
    let pending = s.current();
    let order = s.order;
    assert_eq!((s.lives, s.saves), (3, 0));
    assert_eq!(
        s.take_cues()
            .iter()
            .filter(|c| **c == SessionCue::DeathVoice)
            .count(),
        1
    );
    for _ in 0..100 {
        s.update(
            0.1,
            &mut w,
            VoiceClock {
                position: Some(1.0),
                finished: false,
            },
        )
        .unwrap();
    }
    assert_eq!(s.state, SessionState::DeathDialogue);
    assert_eq!(s.lives, 3);
    s.update(
        0.1,
        &mut w,
        VoiceClock {
            position: Some(3.34),
            finished: true,
        },
    )
    .unwrap();
    assert_eq!(s.lives, 2);
    assert_eq!(s.state, SessionState::Restoring);
    tick(&mut s, &mut w, 2.0);
    assert_eq!((s.lives, s.saves), (2, 0));
    assert_eq!(s.current(), pending);
    assert_eq!(s.order, order);
    assert_eq!(s.state, SessionState::Briefing);
    begin(&mut s, &mut w);
    assert_eq!(w.threats.kind, Some(pending));
}
#[test]
fn three_complete_deaths_trigger_defeat_and_only_one_game_over_cue() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    let mut cues = vec![];
    for remaining in (0..3).rev() {
        reach_death_dialogue(&mut s, &mut w);
        assert_ne!(s.state, SessionState::Defeat);
        tick(&mut s, &mut w, 4.0);
        cues.extend(s.take_cues());
        assert_eq!(s.lives, remaining);
    }
    assert_eq!(s.state, SessionState::Defeat);
    assert_eq!(s.saves, 0);
    assert_eq!(
        cues.iter()
            .filter(|c| **c == SessionCue::DeathVoice)
            .count(),
        3
    );
    assert_eq!(cues.last(), Some(&SessionCue::GameOver));
    tick(&mut s, &mut w, 20.0);
    assert!(s.take_cues().is_empty());
}
#[test]
fn completed_threat_is_not_repeated_when_the_following_one_fails() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    win(&mut s, &mut w);
    let completed = s.current();
    tick(&mut s, &mut w, 1.5);
    s.continue_match(&mut w).unwrap();
    let pending = s.current();
    assert_ne!(completed, pending);
    reach_death_dialogue(&mut s, &mut w);
    tick(&mut s, &mut w, 4.0);
    assert_eq!(s.current(), pending);
    assert_eq!(s.saves, 1);
    assert_eq!(s.lives, 2);
}
#[test]
fn dialogue_uses_recorded_clock_and_changes_portrait_at_configured_boundary() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    reach_death_dialogue(&mut s, &mut w);
    let point = s.config.kyle_switch_seconds;
    s.update(
        0.1,
        &mut w,
        VoiceClock {
            position: Some(point - 0.01),
            finished: false,
        },
    )
    .unwrap();
    assert_eq!(s.dialogue().0, "Stan");
    assert_eq!(s.dialogue().1, "Oh my God! They killed Kenny!");
    s.update(
        0.1,
        &mut w,
        VoiceClock {
            position: Some(point + 0.3),
            finished: false,
        },
    )
    .unwrap();
    assert_eq!(s.dialogue().0, "Kyle");
    assert_eq!(s.dialogue().1, "You bastards!");
    assert!(s.dialogue().2 > 0);
    let pos = w.player.feet;
    let look = (w.player.yaw, w.player.pitch);
    let npc = w.kenny.position();
    tick(&mut s, &mut w, 0.2);
    assert_eq!(w.player.feet, pos);
    assert_eq!((w.player.yaw, w.player.pitch), look);
    assert_eq!(w.kenny.position(), npc);
}
#[test]
fn pause_freezes_danger_and_resume_preserves_its_progress() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    begin(&mut s, &mut w);
    tick(&mut s, &mut w, 1.0);
    let progress = w.threats.elapsed;
    let npc = w.kenny.position();
    s.pause();
    tick(&mut s, &mut w, 100.0);
    assert_eq!(w.threats.elapsed, progress);
    assert_eq!(w.kenny.position(), npc);
    assert!(!s.permits_interaction());
    assert_eq!(s.music(), MusicMode::Silent);
    s.continue_match(&mut w).unwrap();
    tick(&mut s, &mut w, 0.5);
    assert!(w.threats.elapsed > progress);
    assert_eq!(s.music(), MusicMode::Challenge);
}
#[test]
fn every_session_phase_can_return_to_a_clean_explorable_world() {
    for phase in [
        SessionState::Briefing,
        SessionState::Active,
        SessionState::DeathDialogue,
        SessionState::Success,
    ] {
        let mut s = SaveKennySession::default();
        let mut w = scene();
        match phase {
            SessionState::Briefing => s.start(&mut w, 1).unwrap(),
            SessionState::Active => begin(&mut s, &mut w),
            SessionState::Success => win(&mut s, &mut w),
            _ => reach_death_dialogue(&mut s, &mut w),
        }
        s.explore(&mut w).unwrap();
        assert!(s.assert_exploration_clean(&w));
        assert_eq!(s.music(), MusicMode::Exploration);
        let p = w.kenny.position();
        tick(&mut s, &mut w, 3.0);
        assert!((w.kenny.position() - p).length() > 0.1);
        s.start(&mut w, 88).unwrap();
        assert_eq!((s.lives, s.saves), (3, 0));
    }
}
#[test]
fn debug_threat_results_do_not_modify_match_counters() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    w.start_threat(ThreatKind::Bus).unwrap();
    tick(&mut s, &mut w, 25.0);
    assert!(w.threats.result.is_some());
    assert_eq!(s.state, SessionState::Exploring);
    assert_eq!((s.lives, s.saves), (3, 0));
    assert!(s.take_cues().is_empty());
}
#[test]
fn failed_respawn_does_not_charge_another_life_on_each_retry() {
    let mut s = SaveKennySession::default();
    let mut w = scene();
    reach_death_dialogue(&mut s, &mut w);
    s.update(
        0.1,
        &mut w,
        VoiceClock {
            position: Some(3.34),
            finished: true,
        },
    )
    .unwrap();
    assert_eq!(s.lives, 2);
    w.world.obstacles.push(Obstacle {
        id: "blocked-respawn".into(),
        bounds: Aabb::new(
            Vector3::new(-60.0, 0.0, -55.0),
            Vector3::new(-30.0, 5.0, -30.0),
        ),
    });
    for _ in 0..20 {
        assert!(s.update(0.1, &mut w, VoiceClock::default()).is_err());
        assert_eq!(s.lives, 2);
    }
    w.world.obstacles.pop();
    s.update(0.1, &mut w, VoiceClock::default()).unwrap();
    assert_eq!(s.state, SessionState::Briefing);
    assert_eq!(s.lives, 2);
}
#[test]
fn absent_audio_device_is_non_fatal_and_music_requests_are_idempotent() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut audio = AudioController::load(None, &root, AudioConfig::default());
    for _ in 0..100 {
        audio.set_music(MusicMode::Exploration);
        audio.update();
        audio.footsteps(4.5);
    }
    assert_eq!(audio.audit.music_transitions, vec!["exploration"]);
    audio.cue(SessionCue::DeathVoice);
    assert!(audio.voice_clock().position.is_none());
    assert_eq!(audio.audit.plays.get("they_killed_kenny"), Some(&1));
}

#[test]
fn missing_audio_files_degrade_to_subtitles_without_panicking() {
    let Ok(device) = raylib::prelude::RaylibAudio::init_audio_device() else {
        return;
    };
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/nonexistent-phase5-audio-assets");
    let mut audio = AudioController::load(Some(&device), &root, AudioConfig::default());
    assert!(audio.audit.loaded.is_empty());
    assert_eq!(audio.audit.warnings.len(), 9);
    audio.set_music(MusicMode::Challenge);
    audio.cue(SessionCue::DeathVoice);
    audio.update();
    audio.footsteps(8.0);
    assert!(audio.voice_clock().position.is_none());
}
