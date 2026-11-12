use planet_hoppers::{
    gameplay::{
        progress::Franchise,
        rewards::record_result,
        selection::{GameState, ShipCatalog},
    },
    minigames::common::host::MinigameResult,
    scenes::{
        ending::{absorb, protagonist, ready, EndingState},
        token_collection::TokenPresentation,
    },
    ships::ShipsConfig,
};
use raylib::prelude::Vector3;
use std::path::Path;

fn catalog() -> ShipCatalog {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap()
}

#[test]
fn ending_waits_for_all_six_rewards_including_last_toast() {
    let catalog = catalog();
    for last in Franchise::ALL {
        let mut game = GameState::new_game("minecraft", &catalog).unwrap();
        let mut ui = TokenPresentation::initial_shown();
        for f in Franchise::ALL
            .into_iter()
            .filter(|f| *f != last)
            .chain([last])
        {
            let outcome =
                record_result(&mut game, f.as_str(), MinigameResult::Victory, &catalog).unwrap();
            ui.enqueue(f, outcome, "Nave ganada");
            assert!(!ready(&game, &ui));
            while ui.pending_count() > 0 || ui.phase().is_some() {
                assert!(!ready(&game, &ui));
                ui.advance(100.0, true);
            }
            assert_eq!(ready(&game, &ui), f == last);
        }
        let again =
            record_result(&mut game, last.as_str(), MinigameResult::Victory, &catalog).unwrap();
        assert!(!ui.enqueue(last, again, "Nave"));
        ui.ending = EndingState {
            elapsed: 36.0,
            finished: true,
        };
        let restored: TokenPresentation =
            serde_json::from_str(&serde_json::to_string(&ui).unwrap()).unwrap();
        assert!(restored.ending.finished);
        assert_eq!(restored.ending.elapsed, 36.0);
        // Saved pre-ending adventures remain compatible.
        let mut legacy = serde_json::to_value(&ui).unwrap();
        legacy.as_object_mut().unwrap().remove("ending");
        let legacy: TokenPresentation = serde_json::from_value(legacy).unwrap();
        assert!(!legacy.ending.finished);
        assert_eq!(legacy.ending.elapsed, 0.0);
    }
}

#[test]
fn home_identity_survives_all_thirty_six_ship_combinations() {
    let catalog = catalog();
    for origin in Franchise::ALL {
        let mut game = GameState::new_game(origin.as_str(), &catalog).unwrap();
        for current in Franchise::ALL {
            game.unlock(current.as_str(), &catalog).unwrap();
            game.activate(current.as_str()).unwrap();
            assert_eq!(protagonist(&game).unwrap(), origin);
            assert_eq!(game.active_ship(), current.as_str());
        }
    }
}

#[test]
fn spiral_accelerates_contracts_and_vanishes_without_changing_source_pose() {
    let center = Vector3::new(5.0, 2.0, -8.0);
    let start = center + Vector3::new(100.0, 6.0, 50.0);
    assert_eq!(absorb(start, center, 0.0), (start, 1.0));
    assert_eq!(absorb(start, center, 1.0), (center, 0.0));
    let mut previous_radius = (start - center).length();
    let mut previous_scale = 1.0;
    for i in 1..=100 {
        let (pos, scale) = absorb(start, center, i as f32 / 100.0);
        assert!((pos - center).length() <= previous_radius + 0.001);
        assert!(scale <= previous_scale);
        previous_radius = (pos - center).length();
        previous_scale = scale;
    }
    assert!(
        (absorb(start, center, 0.25).0 - center).length()
            - (absorb(start, center, 0.5).0 - center).length()
            < (absorb(start, center, 0.5).0 - center).length()
                - (absorb(start, center, 0.75).0 - center).length()
    );
}
