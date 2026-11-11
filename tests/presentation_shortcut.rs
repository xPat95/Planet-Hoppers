use planet_hoppers::{
    gameplay::{
        progress::Franchise,
        rewards::{complete_for_presentation, record_result},
        save,
        selection::{GameState, ShipCatalog},
    },
    graphics::window,
    minigames::common::{host::MinigameResult, shortcuts::PresentationShortcut},
    scenes::token_collection::TokenPresentation,
    ships::ShipsConfig,
};
use raylib::{ffi, prelude::*};
use std::path::Path;

fn catalog() -> ShipCatalog {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap()
}

#[test]
fn every_partial_collection_receives_exactly_its_missing_tokens() {
    let catalog = catalog();
    for initial in Franchise::ALL {
        for mask in 0u32..64 {
            let mut game = GameState::new_game(initial.as_str(), &catalog).unwrap();
            for (index, franchise) in Franchise::ALL.into_iter().enumerate() {
                if mask & (1 << index) != 0 {
                    record_result(
                        &mut game,
                        franchise.as_str(),
                        MinigameResult::Victory,
                        &catalog,
                    )
                    .unwrap();
                }
            }
            assert_eq!(
                complete_for_presentation(&mut game, &catalog).unwrap(),
                6 - mask.count_ones() as usize
            );
            assert_eq!(game.progress().token_count(), 6);
            assert!(game.progress().is_complete());
            assert_eq!(game.unlocked_count(), 6);
            assert_eq!(game.active_ship(), initial.as_str());
            assert_eq!(game.first_ship(), initial.as_str());
            game.validate(&catalog).unwrap();
            let before = game.clone();
            assert_eq!(complete_for_presentation(&mut game, &catalog).unwrap(), 0);
            assert_eq!(game, before);
        }
    }
}

#[test]
fn invalid_catalog_does_not_leave_partially_granted_rewards() {
    let mut catalog = catalog();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    let before = game.clone();
    catalog.choices.retain(|choice| choice.id != "regularshow");
    assert!(complete_for_presentation(&mut game, &catalog).is_err());
    assert_eq!(game, before);
}

#[test]
fn cnp_requires_control_order_and_timely_letters_and_can_restart() {
    let mut shortcut = PresentationShortcut::default();
    for letter in ['C', 'N', 'P'] {
        assert!(!shortcut.update(false, Some(letter), 0.0));
    }
    for interruption in [
        (false, None, 1.1),
        (true, Some('P'), 1.1),
        (true, None, 7.0),
    ] {
        assert!(!shortcut.update(true, Some('C'), 1.0));
        assert!(!shortcut.update(interruption.0, interruption.1, interruption.2));
        assert!(!shortcut.update(true, Some('N'), interruption.2 + 0.1));
        assert!(!shortcut.update(true, Some('P'), interruption.2 + 0.2));
    }
    assert!(!shortcut.update(true, Some('C'), 8.0));
    assert!(!shortcut.update(true, None, 8.1));
    assert!(!shortcut.update(true, Some('N'), 8.2));
    assert!(shortcut.update(true, Some('P'), 8.3));
    assert!(!shortcut.update(true, Some('P'), 8.4));
}

#[test]
fn completed_save_reloads_without_replaying_pending_abductions() {
    let catalog = catalog();
    let mut game = GameState::new_game("southpark", &catalog).unwrap();
    let mut presentation = TokenPresentation::initial_shown();
    let outcome = record_result(&mut game, "starwars", MinigameResult::Victory, &catalog).unwrap();
    presentation.enqueue(Franchise::StarWars, outcome, "Nave");
    presentation.advance(0.0, true);
    assert!(presentation.is_busy());
    assert_eq!(complete_for_presentation(&mut game, &catalog).unwrap(), 5);
    presentation.finish_for_presentation();
    assert_eq!(presentation.pending_count(), 0);
    assert!(!presentation.is_busy());
    assert!(presentation.required_abductions().is_empty());
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("target/cnp-save-{}", std::process::id()));
    let path = directory.join("adventure.json");
    save::store(&path, &game, &presentation).unwrap();
    let restored = save::load(&path, &catalog).unwrap().unwrap();
    assert_eq!(restored.game, game);
    assert_eq!(restored.game.progress().token_count(), 6);
    assert!(!restored.presentation.is_busy());
    assert_eq!(restored.presentation.pending_count(), 0);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

fn key(key: KeyboardKey, down: bool) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: if down { 2 } else { 1 },
            params: [key as i32, 0, 0, 0],
        });
    }
}

#[test]
#[ignore = "requires OpenGL; run with --ignored --test-threads=1"]
fn native_cnp_detects_either_control_key_without_consuming_scene_input() {
    let (mut rl, thread) = window::build(raylib::init().size(640, 480).title("CNP QA"), true);
    rl.set_exit_key(None);
    for control in [
        KeyboardKey::KEY_LEFT_CONTROL,
        KeyboardKey::KEY_RIGHT_CONTROL,
    ] {
        let mut shortcut = PresentationShortcut::default();
        key(control, true);
        for (index, letter) in [KeyboardKey::KEY_C, KeyboardKey::KEY_N, KeyboardKey::KEY_P]
            .into_iter()
            .enumerate()
        {
            key(letter, true);
            assert_eq!(shortcut.poll(&rl), index == 2);
            assert!(rl.is_key_pressed(letter));
            key(letter, false);
            window::begin_drawing(&mut rl, &thread).clear_background(Color::BLACK);
        }
        key(control, false);
    }
}
