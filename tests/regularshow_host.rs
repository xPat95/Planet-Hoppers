use planet_hoppers::{
    gameplay::{
        rewards::record_result,
        selection::{GameState, ShipCatalog},
    },
    minigames::{
        common::host::{AttemptReporter, MinigameExit, MinigameResult},
        regularshow::{autoplay, simulation::*},
    },
    scenes::minigame_host::{self, MinigameVisits},
    ships::ShipsConfig,
    solar_system::SolarSystemConfig,
};
use std::path::Path;

#[test]
fn regular_show_planet_dispatches_and_owns_all_media_in_the_main_project() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let solar = SolarSystemConfig::load(&root.join("config/solar-system.json")).unwrap();
    let planet = solar
        .planets
        .iter()
        .find(|p| p.id == "regularshow")
        .unwrap();
    assert_eq!(planet.minigame.as_deref(), Some("regularshow"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let selected = ships.active().unwrap();
    let result = minigame_host::run_with_ship(
        "regularshow",
        &mut MinigameVisits::default(),
        selected,
        root,
        ["--help".into()].into_iter(),
    )
    .unwrap();
    assert_eq!(result, MinigameExit::ReturnedWithoutResult);
    for asset in [
        "audio/musica_fondo_martillo.mp3",
        "audio/musica_fondo_n3_martillo.mp3",
        "video/transicion.sfv",
        "video/transicion.ogg",
        "video/celebracion.sfv",
        "video/celebracion.ogg",
    ] {
        assert!(
            root.join("assets/minigames/regularshow")
                .join(asset)
                .is_file(),
            "{asset}"
        );
    }
}

#[test]
fn only_the_complete_arcade_awards_a_token_and_retries_preserve_it() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
    let mut adventure = GameState::new_game("minecraft", &catalog).unwrap();
    let mut g = Game::new();
    let mut reporter = AttemptReporter::default();
    let mut outcomes = Vec::new();
    let mut cleared = 0;
    g.start();
    for _ in 0..36_000 {
        reporter
            .observe(&g, &mut |result| {
                outcomes.push(result);
                record_result(&mut adventure, "regularshow", result, &catalog)?;
                Ok(())
            })
            .unwrap();
        if g.phase == Phase::ArcadeClear {
            break;
        }
        assert_ne!(g.phase, Phase::Defeat);
        if g.phase == Phase::StageClear {
            cleared += 1;
            assert_eq!(adventure.progress().token_count(), 0);
            g.advance();
        }
        g.update(1.0 / 60.0, autoplay(&g));
    }
    assert_eq!(cleared, 2);
    assert_eq!(outcomes, [MinigameResult::Victory]);
    assert_eq!(adventure.progress().token_count(), 1);
    assert!(adventure.is_unlocked("regularshow"));
    assert_eq!(adventure.active_ship(), "minecraft");
    g.retry();
    // A real later loss must not erase or duplicate the already granted victory.
    for _ in 0..36_000 {
        reporter
            .observe(&g, &mut |result| {
                outcomes.push(result);
                record_result(&mut adventure, "regularshow", result, &catalog)?;
                Ok(())
            })
            .unwrap();
        if g.phase == Phase::Defeat {
            break;
        }
        g.update(1.0 / 60.0, [Input::default(); 2]);
    }
    assert_eq!(outcomes, [MinigameResult::Victory, MinigameResult::Defeat]);
    assert_eq!(adventure.progress().token_count(), 1);
    assert_eq!(adventure.active_ship(), "minecraft");
}
