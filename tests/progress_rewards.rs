use planet_hoppers::{
    gameplay::{
        progress::{Franchise, ProgressState},
        rewards::{record_result, RewardOutcome},
        selection::{GameState, SelectionMode, SelectionTransition, ShipCatalog, ShipSelection},
    },
    minigames::common::host::MinigameResult,
    ships::ShipsConfig,
};
use std::path::Path;

fn catalog() -> ShipCatalog {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap()
}

#[test]
fn canonical_identities_match_the_real_catalog() {
    let catalog = catalog();
    assert_eq!(Franchise::ALL.len(), catalog.choices.len());
    for choice in &catalog.choices {
        let franchise = Franchise::from_id(&choice.id).unwrap();
        assert_eq!(franchise.as_str(), choice.id);
    }
    assert_eq!(ProgressState::default().token_count(), 0);
    assert!(!ProgressState::default().is_complete());
}

#[test]
fn first_victory_grants_one_token_and_unlocks_its_ship() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    let outcome = record_result(&mut game, "southpark", MinigameResult::Victory, &catalog).unwrap();
    assert_eq!(
        outcome,
        RewardOutcome {
            token_obtained: true,
            ship_unlocked: true,
            progress_completed: false,
        }
    );
    assert_eq!(game.progress().token_count(), 1);
    assert!(game.progress().is_completed(Franchise::SouthPark));
    assert_eq!(
        game.progress().completed().collect::<Vec<_>>(),
        vec![Franchise::SouthPark]
    );
    assert!(game.is_unlocked("southpark"));
    assert_eq!(game.unlocked_count(), 2);
}

#[test]
fn repeated_victory_is_a_no_op() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    record_result(&mut game, "halloween", MinigameResult::Victory, &catalog).unwrap();
    let before = game.clone();
    for _ in 0..3 {
        assert_eq!(
            record_result(&mut game, "halloween", MinigameResult::Victory, &catalog).unwrap(),
            RewardOutcome::default()
        );
        assert_eq!(game, before);
    }
}

#[test]
fn every_initial_selection_unlocks_one_ship_without_tokens() {
    let catalog = catalog();
    for index in 0..catalog.choices.len() {
        let selection = ShipSelection {
            index,
            mode: SelectionMode::Initial,
        };
        let SelectionTransition::StartAdventure(game) = selection.confirm(&catalog, None).unwrap()
        else {
            panic!("expected a new adventure");
        };
        assert_eq!(game.progress().token_count(), 0);
        assert!(!game.progress().is_complete());
        assert_eq!(game.unlocked_count(), 1);
        assert!(game.is_unlocked(&catalog.choices[index].id));
    }
}

#[test]
fn winning_the_initial_franchise_grants_a_token_without_another_unlock() {
    let catalog = catalog();
    for franchise in Franchise::ALL {
        let mut game = GameState::new_game(franchise.as_str(), &catalog).unwrap();
        let outcome = record_result(
            &mut game,
            franchise.as_str(),
            MinigameResult::Victory,
            &catalog,
        )
        .unwrap();
        assert!(outcome.token_obtained);
        assert!(!outcome.ship_unlocked);
        assert!(!outcome.progress_completed);
        assert_eq!(game.progress().token_count(), 1);
        assert_eq!(game.unlocked_count(), 1);
    }
}

#[test]
fn defeats_and_abandonments_preserve_the_entire_adventure() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    record_result(&mut game, "southpark", MinigameResult::Victory, &catalog).unwrap();
    let before = game.clone();
    for franchise in Franchise::ALL {
        for result in [MinigameResult::Defeat, MinigameResult::Abandoned] {
            assert_eq!(
                record_result(&mut game, franchise.as_str(), result, &catalog).unwrap(),
                RewardOutcome::default()
            );
            assert_eq!(game, before);
        }
    }
}

#[test]
fn invalid_franchises_and_model_ids_are_rejected_without_mutation() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    let before = game.clone();
    for id in [
        "unknown",
        "",
        "smiling_friends",
        "regular_show",
        "starwars_mandalorian",
    ] {
        for result in [
            MinigameResult::Victory,
            MinigameResult::Defeat,
            MinigameResult::Abandoned,
        ] {
            assert!(record_result(&mut game, id, result, &catalog).is_err());
            assert_eq!(game, before);
        }
    }
}

#[test]
fn missing_catalog_entry_cannot_partially_grant_a_reward() {
    let mut catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    catalog.choices.retain(|choice| choice.id != "southpark");
    let before = game.clone();
    assert!(record_result(&mut game, "southpark", MinigameResult::Victory, &catalog).is_err());
    assert_eq!(game, before);
}

#[test]
fn rewards_preserve_the_explicitly_selected_active_ship() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    game.unlock("starwars", &catalog).unwrap();
    game.activate("starwars").unwrap();
    for franchise in Franchise::ALL {
        record_result(
            &mut game,
            franchise.as_str(),
            MinigameResult::Victory,
            &catalog,
        )
        .unwrap();
        assert_eq!(game.active_ship(), "starwars");
        assert_eq!(game.first_ship(), "minecraft");
    }
}

#[test]
fn direct_unlock_is_idempotent_and_does_not_grant_tokens() {
    let catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    assert!(game.unlock("regularshow", &catalog).unwrap());
    assert!(!game.unlock("regularshow", &catalog).unwrap());
    assert_eq!(game.progress().token_count(), 0);
    assert_eq!(game.active_ship(), "minecraft");
    let outcome =
        record_result(&mut game, "regularshow", MinigameResult::Victory, &catalog).unwrap();
    assert!(outcome.token_obtained);
    assert!(!outcome.ship_unlocked);
    assert_eq!(game.unlocked_count(), 2);
}

#[test]
fn exactly_six_unique_tokens_complete_progress_once_in_any_rotated_order() {
    let catalog = catalog();
    for start in 0..Franchise::ALL.len() {
        let mut game = GameState::new_game("minecraft", &catalog).unwrap();
        for step in 0..Franchise::ALL.len() {
            let id = Franchise::ALL[(start + step) % Franchise::ALL.len()].as_str();
            let outcome = record_result(&mut game, id, MinigameResult::Victory, &catalog).unwrap();
            assert!(outcome.token_obtained);
            assert_eq!(outcome.progress_completed, step == 5);
            assert_eq!(game.progress().token_count(), step + 1);
            assert_eq!(game.progress().is_complete(), step == 5);
            let before = game.clone();
            assert_eq!(
                record_result(&mut game, id, MinigameResult::Victory, &catalog).unwrap(),
                RewardOutcome::default()
            );
            assert_eq!(game, before);
        }
        assert_eq!(game.unlocked_count(), 6);
        assert!(record_result(&mut game, "seventh", MinigameResult::Victory, &catalog).is_err());
        assert_eq!(game.progress().token_count(), 6);
    }
}

#[test]
fn confirming_a_new_game_resets_progress_and_ownership_without_mutating_the_old_game() {
    let catalog = catalog();
    let mut previous = GameState::new_game("minecraft", &catalog).unwrap();
    for franchise in Franchise::ALL {
        record_result(
            &mut previous,
            franchise.as_str(),
            MinigameResult::Victory,
            &catalog,
        )
        .unwrap();
    }
    let selection = ShipSelection {
        index: catalog
            .choices
            .iter()
            .position(|choice| choice.id == "southpark")
            .unwrap(),
        mode: SelectionMode::Initial,
    };
    let SelectionTransition::StartAdventure(next) =
        selection.confirm(&catalog, Some(&previous)).unwrap()
    else {
        panic!("expected a new adventure");
    };
    assert_eq!(next.progress(), &ProgressState::default());
    assert_eq!(next.unlocked_count(), 1);
    assert_eq!(next.first_ship(), "southpark");
    assert_eq!(next.active_ship(), "southpark");
    assert!(previous.progress().is_complete());
    assert_eq!(previous.unlocked_count(), 6);
}
