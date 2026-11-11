use planet_hoppers::{
    gameplay::{
        save,
        selection::{GameState, ShipCatalog},
    },
    scenes::token_collection::TokenPresentation,
    ships::ShipsConfig,
};
use std::path::Path;

#[test]
fn chef_seen_is_per_adventure_backward_compatible_and_not_a_reward() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
    let fresh = GameState::new_game("southpark", &catalog).unwrap();
    assert!(!fresh.southpark_chef_intro_seen);
    let mut seen = fresh.clone();
    seen.southpark_chef_intro_seen = true;
    assert_eq!(seen.progress(), fresh.progress());
    assert_eq!(seen.unlocked_count(), fresh.unlocked_count());
    let path = root.join("target/chef-save-test.json");
    save::store(&path, &seen, &TokenPresentation::default()).unwrap();
    assert!(
        save::load(&path, &catalog)
            .unwrap()
            .unwrap()
            .game
            .southpark_chef_intro_seen
    );
    let mut legacy = serde_json::to_value(&seen).unwrap();
    legacy
        .as_object_mut()
        .unwrap()
        .remove("southpark_chef_intro_seen");
    let loaded: GameState = serde_json::from_value(legacy).unwrap();
    assert!(!loaded.southpark_chef_intro_seen);
    assert_eq!(GameState::new_game("southpark", &catalog).unwrap(), fresh);
}
