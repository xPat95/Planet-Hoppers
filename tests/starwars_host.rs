use planet_hoppers::{
    gameplay::selection::ShipCatalog,
    minigames::common::host::MinigameExit,
    scenes::minigame_host::{self, MinigameVisits},
    ships::ShipsConfig,
    solar_system::{SolarSystem, SolarSystemConfig},
};
use std::path::Path;
#[test]
fn death_star_dispatches_with_the_resolved_session_ship() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let solar =
        SolarSystem::new(&SolarSystemConfig::load(&root.join("config/solar-system.json")).unwrap());
    let planet = solar
        .planets
        .iter()
        .find(|p| p.config.id == "starwars")
        .unwrap();
    assert_eq!(planet.config.minigame.as_deref(), Some("starwars"));
    let config = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &config).unwrap();
    for choice in &catalog.choices {
        let selected = catalog.model(choice, &config);
        assert!(root.join(&selected.model).is_file());
        if choice.id == "starwars" {
            assert!(selected.model.contains("mandalorian"));
            assert_eq!(selected.id, "starwars");
        }
        let result = minigame_host::run_with_ship(
            "starwars",
            &mut MinigameVisits::default(),
            &selected,
            root,
            ["--help".to_owned()].into_iter(),
        )
        .unwrap();
        assert_eq!(result, MinigameExit::ReturnedWithoutResult);
    }
}

/// Run explicitly on the graphics machine; normal cargo test stays window-free.
#[test]
#[ignore = "requires OpenGL and audio device"]
fn obj_ship_uses_the_same_loader_and_safe_startup() {
    use planet_hoppers::{minigames::starwars, ships::ShipModel};
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model = ShipModel {
        id: "starwars_obj".into(),
        name: "Mandaloriano OBJ".into(),
        model: "assets/models/ships/starwars/mandalorian/exports/mandalorian_n1_ship.obj".into(),
        length: 5.0,
        model_rotation_degrees: [0.0; 3],
    };
    let result = starwars::run(root, &model, ["--check-startup".to_owned()].into_iter()).unwrap();
    assert_eq!(result, MinigameExit::ReturnedWithoutResult);
}
