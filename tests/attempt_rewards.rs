use planet_hoppers::{
    gameplay::{
        progress::Franchise,
        rewards::record_result,
        selection::{GameState, ShipCatalog},
    },
    minigames::{
        common::host::{AttemptOutcome, AttemptReporter, MinigameResult},
        minecraft::{
            mine::MineScene,
            mining::{MiningConfig, MiningSession},
            rush::{RushConfig, RushSession},
            visit::MinecraftVisits,
            MinecraftConfig,
        },
        smilingfriends::{Dialogue, Game, Step},
    },
    scenes::minigame_host::{self, MinigameVisits},
    ships::ShipsConfig,
};
use std::path::Path;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn adventure() -> (GameState, ShipCatalog, ShipsConfig) {
    let ships = ShipsConfig::load(&root().join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root().join("config/ship-selection.json"), &ships).unwrap();
    let game = GameState::new_game("halloween", &catalog).unwrap();
    (game, catalog, ships)
}

fn observe(
    reporter: &mut AttemptReporter,
    attempt: &impl AttemptOutcome,
    game: &mut GameState,
    catalog: &ShipCatalog,
    id: &str,
) {
    reporter
        .observe(attempt, &mut |result| {
            record_result(game, id, result, catalog)?;
            Ok(())
        })
        .unwrap();
}

#[test]
fn minecraft_win_survives_retry_loss_and_navigation_with_the_last_result() {
    let (mut game, catalog, _) = adventure();
    let mut scene = MineScene::load(
        &root().join("assets/models/minigames/minecraft/rush/minecraft_mine.scene.json"),
        MinecraftConfig::load(&root().join("config/minecraft-dev.json")).unwrap(),
    )
    .unwrap();
    let mut mining = MiningSession::new(
        &scene,
        MiningConfig::load(&root().join("config/minecraft-mining.json")).unwrap(),
    );
    let mut attempt =
        RushSession::new(RushConfig::load(&root().join("config/minecraft-rush.json")).unwrap());
    let mut visits = MinecraftVisits::default();
    let mut reporter = AttemptReporter::default();
    visits.start_attempt();
    attempt.ready();
    mining.diamonds = attempt.config.required_diamonds;
    attempt.inspect_world(&mut scene, &mut mining, false);
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    assert_eq!(game.progress().token_count(), 0); // Partial objective is not victory.
    attempt.inspect_world(&mut scene, &mut mining, true);
    visits.record_cues(&attempt.drain_cues());
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    assert_eq!(game.progress().token_count(), 1); // Before navigation or restart.
    assert!(game.is_unlocked("minecraft"));
    attempt.restart(&mut scene, &mut mining);
    visits.start_attempt();
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    attempt.advance(attempt.config.initial_seconds + 1.0);
    visits.record_cues(&attempt.drain_cues());
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    assert_eq!(
        visits.exit_since(0),
        planet_hoppers::minigames::common::host::MinigameExit::Returned(MinigameResult::Defeat)
    );
    assert!(game.progress().is_completed(Franchise::Minecraft));
    assert_eq!(game.progress().token_count(), 1);
    // A second victory and a later abandoned attempt cannot replace or duplicate it.
    attempt.restart(&mut scene, &mut mining);
    visits.start_attempt();
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    mining.diamonds = attempt.config.required_diamonds;
    attempt.inspect_world(&mut scene, &mut mining, true);
    visits.record_cues(&attempt.drain_cues());
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    attempt.restart(&mut scene, &mut mining);
    visits.start_attempt();
    observe(&mut reporter, &attempt, &mut game, &catalog, "minecraft");
    visits.finish(MinigameResult::Abandoned);
    assert_eq!(
        visits.exit_since(0),
        planet_hoppers::minigames::common::host::MinigameExit::Returned(MinigameResult::Abandoned)
    );
    assert_eq!(game.progress().token_count(), 1);
    assert_eq!(game.active_ship(), "halloween");
}

#[test]
fn smilingfriends_real_choices_award_before_navigation_and_only_once() {
    let (mut game, catalog, _) = adventure();
    let mut attempt =
        Game::new(Dialogue::load(&root().join("config/smilingfriends-dialogue.json")).unwrap());
    let mut reporter = AttemptReporter::default();
    let mut notifications = Vec::new();
    let mut observer = |result| {
        notifications.push(result);
        record_result(&mut game, "smilingfriends", result, &catalog)?;
        Ok(())
    };
    for _ in 0..500 {
        reporter.observe(&attempt, &mut observer).unwrap();
        if attempt.finished() {
            break;
        }
        if let Step::Prompt { encounter, round } = attempt.step {
            let choices = &attempt.dialogue.encounters[encounter].rounds[round].choices;
            let index = (0..choices.len())
                .max_by_key(|&i| choices[i].laugh)
                .unwrap();
            attempt.choose(index);
        } else {
            attempt.advance();
        }
    }
    assert_eq!(attempt.step, Step::Victory);
    for _ in 0..10 {
        reporter.observe(&attempt, &mut observer).unwrap();
    }
    assert_eq!(notifications, vec![MinigameResult::Victory]);
    assert!(game.progress().is_completed(Franchise::SmilingFriends));
    assert!(game.is_unlocked("smilingfriends"));
    assert_eq!(game.active_ship(), "halloween");
}

#[test]
fn exploration_and_real_smilingfriends_defeat_grant_nothing() {
    let (mut game, catalog, _) = adventure();
    let before = game.clone();
    let mut attempt =
        Game::new(Dialogue::load(&root().join("config/smilingfriends-dialogue.json")).unwrap());
    let mut reporter = AttemptReporter::default();
    for _ in 0..500 {
        observe(
            &mut reporter,
            &attempt,
            &mut game,
            &catalog,
            "smilingfriends",
        );
        if attempt.finished() {
            break;
        }
        if let Step::Prompt { encounter, round } = attempt.step {
            let choices = &attempt.dialogue.encounters[encounter].rounds[round].choices;
            let index = (0..choices.len())
                .min_by_key(|&i| choices[i].laugh)
                .unwrap();
            attempt.choose(index);
        } else {
            attempt.advance();
        }
    }
    assert_eq!(attempt.step, Step::Defeat);
    assert_eq!(game, before);
    record_result(
        &mut game,
        "smilingfriends",
        MinigameResult::Abandoned,
        &catalog,
    )
    .unwrap();
    assert_eq!(game, before);
}

#[test]
fn pending_minigames_and_help_cannot_emit_rewards() {
    let (mut game, catalog, ships) = adventure();
    let before = game.clone();
    let selected = catalog.model(catalog.choice("halloween").unwrap(), &ships);
    for id in [
        "halloween",
        "regularshow",
        "minecraft",
        "southpark",
        "starwars",
    ] {
        let result = minigame_host::run_with_ship_and_results(
            id,
            &mut MinigameVisits::default(),
            &selected,
            root(),
            ["--help".to_owned()].into_iter(),
            &mut |result| {
                record_result(&mut game, id, result, &catalog)?;
                Ok(())
            },
        );
        if matches!(id, "halloween" | "regularshow") {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
        assert_eq!(game, before);
    }
}

#[test]
fn reporter_retries_failed_delivery_without_losing_the_terminal_result() {
    let mut attempt =
        Game::new(Dialogue::load(&root().join("config/smilingfriends-dialogue.json")).unwrap());
    attempt.step = Step::Victory;
    let mut reporter = AttemptReporter::default();
    assert!(reporter
        .observe(&attempt, &mut |_| Err("delivery failed".into()))
        .is_err());
    let mut delivered = 0;
    for _ in 0..2 {
        reporter
            .observe(&attempt, &mut |_| {
                delivered += 1;
                Ok(())
            })
            .unwrap();
    }
    assert_eq!(delivered, 1);
}
