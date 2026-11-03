use planet_hoppers::minigames::{
    common::host::{MinigameExit, MinigameResult, ReturnConfirmation},
    minecraft::{
        mine::MineScene,
        mining::{MiningConfig, MiningSession},
        rush::{RushConfig, RushCue, RushSession, RushState},
        visit::{AttemptEvent, MinecraftVisits, VillageAction, VillageMenu},
        MinecraftConfig,
    },
};
use std::path::PathBuf;

fn fixture() -> (MineScene, MiningSession, RushSession) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scene = MineScene::load(
        &root.join("assets/models/minigames/minecraft/rush/minecraft_mine.scene.json"),
        MinecraftConfig::load(&root.join("config/minecraft-dev.json")).unwrap(),
    )
    .unwrap();
    let mining = MiningSession::new(
        &scene,
        MiningConfig::load(&root.join("config/minecraft-mining.json")).unwrap(),
    );
    let game =
        RushSession::new(RushConfig::load(&root.join("config/minecraft-rush.json")).unwrap());
    (scene, mining, game)
}

#[test]
fn exploration_exit_is_navigation_without_a_fabricated_abandonment() {
    let mut visits = MinecraftVisits::default();
    let mut menu = VillageMenu::Closed;
    menu.apply(VillageAction::Pause);
    assert!(menu.apply(VillageAction::RequestReturn));
    assert_eq!(visits.exit_since(0), MinigameExit::ReturnedWithoutResult);
    assert!(visits.active_attempt().is_none() && visits.completed().is_empty());
    assert!(visits.drain_events().is_empty());
}

#[test]
fn genuine_victory_is_recorded_at_completion_and_survives_navigation_exactly_once() {
    let (mut scene, mut mining, mut game) = fixture();
    let mut visits = MinecraftVisits::default();
    let id = visits.start_attempt();
    game.ready();
    mining.diamonds = game.config.required_diamonds;
    game.inspect_world(&mut scene, &mut mining, false);
    visits.record_cues(&game.drain_cues());
    assert!(visits.completed().is_empty()); // Eight diamonds alone are not victory.
    assert_eq!(visits.active_attempt(), Some(id));
    game.inspect_world(&mut scene, &mut mining, true);
    let cues = game.drain_cues();
    visits.record_cues(&cues);
    assert_eq!(game.state, RushState::Victory);
    assert_eq!(visits.active_attempt(), None);
    assert_eq!(visits.completed()[0].result, MinigameResult::Victory);
    let remaining = game.remaining;
    game.advance(1000.0);
    game.inspect_world(&mut scene, &mut mining, true);
    visits.record_cues(&cues); // A repeated terminal notification cannot produce another completion.
    visits.record_cues(&game.drain_cues());
    assert_eq!(game.remaining, remaining);
    assert_eq!(visits.finish(MinigameResult::Abandoned), None);
    for _ in 0..3 {
        assert_eq!(
            visits.exit_since(0),
            MinigameExit::Returned(MinigameResult::Victory)
        );
    }
    assert_eq!(visits.completed().len(), 1);
    assert_eq!(
        visits.drain_events(),
        vec![
            AttemptEvent::Started { attempt_id: id },
            AttemptEvent::Finished(visits.completed()[0])
        ]
    );
    assert!(visits.drain_events().is_empty());
    // A later visit with no new challenge preserves history without reporting that victory again.
    assert_eq!(
        visits.exit_since(visits.completed().len()),
        MinigameExit::ReturnedWithoutResult
    );
}

#[test]
fn loss_and_confirmed_abandonment_never_register_victory() {
    let (mut scene, mut mining, mut game) = fixture();
    let mut visits = MinecraftVisits::default();
    visits.start_attempt();
    game.ready();
    game.advance(game.config.initial_seconds + 1.0);
    visits.record_cues(&game.drain_cues());
    assert_eq!(
        visits.exit_since(0),
        MinigameExit::Returned(MinigameResult::Defeat)
    );
    assert!(visits.active_attempt().is_none());
    game.restart(&mut scene, &mut mining);
    let id = visits.start_attempt();
    visits.record_cues(&game.drain_cues());
    let mut confirmation = ReturnConfirmation::default();
    game.pause();
    confirmation.request();
    let remaining = game.remaining;
    game.advance(20.0);
    confirmation.cancel();
    assert_eq!(confirmation.confirm(), None);
    game.resume();
    assert_eq!(game.state, RushState::Running);
    assert_eq!(game.remaining, remaining);
    assert_eq!(visits.active_attempt(), Some(id));
    assert_eq!(visits.completed().len(), 1);
    game.pause();
    confirmation.request();
    visits.finish(confirmation.confirm().unwrap());
    assert!(visits.active_attempt().is_none());
    assert_eq!(visits.completed()[1].result, MinigameResult::Abandoned);
    assert!(visits
        .completed()
        .iter()
        .all(|r| r.result != MinigameResult::Victory));
}

#[test]
fn retry_resets_real_gameplay_and_keeps_prior_wins_without_reusing_them() {
    let (mut scene, mut mining, mut game) = fixture();
    let mut visits = MinecraftVisits::default();
    let mut ids = Vec::new();
    for _ in 0..2 {
        game.restart(&mut scene, &mut mining);
        ids.push(visits.start_attempt());
        visits.record_cues(&game.drain_cues());
        assert_eq!(game.state, RushState::Running);
        assert_eq!(game.remaining, game.config.initial_seconds);
        assert_eq!(mining.diamonds, 0);
        assert!(!game.objective_met && !game.warning);
        assert!(mining.blocks.iter().all(|b| b.alive));
        mining.diamonds = game.config.required_diamonds;
        game.inspect_world(&mut scene, &mut mining, true);
        visits.record_cues(&game.drain_cues());
    }
    assert_ne!(ids[0], ids[1]);
    assert_eq!(visits.completed().len(), 2);
    assert!(visits
        .completed()
        .iter()
        .all(|r| r.result == MinigameResult::Victory));
    let events = visits.drain_events();
    assert_eq!(
        events
            .iter()
            .filter(|e| matches!(e, AttemptEvent::Finished(_)))
            .count(),
        2
    );
    assert!(visits.drain_events().is_empty());
}

#[test]
fn restarting_an_unfinished_attempt_marks_only_that_attempt_abandoned() {
    let mut visits = MinecraftVisits::default();
    let old = visits.start_attempt();
    let new = visits.start_attempt();
    assert_eq!(visits.completed()[0].attempt_id, old);
    assert_eq!(visits.completed()[0].result, MinigameResult::Abandoned);
    assert_eq!(visits.active_attempt(), Some(new));
    visits.record_cues(&[RushCue::Reset, RushCue::Success]);
    assert_eq!(visits.completed().len(), 1);
}
