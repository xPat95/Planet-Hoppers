//! In-process dispatch. Each adapter owns one window at a time and returns the shared result.
use super::{
    minecraft_rush, smilingfriends_entry, southpark_dev, southpark_entry::SouthParkVisits,
};
use crate::minigames::common::host::MinigameExit;
use crate::minigames::minecraft::visit::MinecraftVisits;

#[derive(Default)]
pub struct MinigameVisits {
    pub southpark: SouthParkVisits,
    pub minecraft: MinecraftVisits,
}

pub fn run(
    id: &str,
    visits: &mut MinigameVisits,
    args: impl Iterator<Item = String>,
) -> Result<MinigameExit, Box<dyn std::error::Error>> {
    match id {
        "southpark" => southpark_dev::run_visit_with_args(&mut visits.southpark, args),
        "smilingfriends" => smilingfriends_entry::run(args),
        "minecraft" => minecraft_rush::run_visit_with_state(args.collect(), &mut visits.minecraft),
        "regularshow" => crate::minigames::regularshow::run_visit(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")), args, &mut |_| Ok(())),
        "halloween" => crate::minigames::halloween::run_visit(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")), args, &mut |_| Ok(())),
        "starwars" => Err("Star Wars requiere la nave activa; usar run_with_ship".into()),
        _ => Err(format!("Minijuego no implementado: {id}").into()),
    }
}

/// Only CPU metadata crosses the window boundary. Never pass a GPU Model here.
pub fn run_with_ship(
    id: &str,
    visits: &mut MinigameVisits,
    selected: &crate::ships::ShipModel,
    root: &std::path::Path,
    args: impl Iterator<Item = String>,
) -> Result<MinigameExit, Box<dyn std::error::Error>> {
    run_with_ship_and_results(id, visits, selected, root, args, &mut |_| Ok(()))
}

/// Reports actual attempts immediately; navigation continues to use MinigameExit.
pub fn run_with_ship_and_results(
    id: &str,
    visits: &mut MinigameVisits,
    selected: &crate::ships::ShipModel,
    root: &std::path::Path,
    args: impl Iterator<Item = String>,
    observer: &mut crate::minigames::common::host::ResultObserver<'_>,
) -> Result<MinigameExit, Box<dyn std::error::Error>> {
    match id {
        "southpark" => southpark_dev::run_visit_with_results(&mut visits.southpark, args, observer),
        "smilingfriends" => smilingfriends_entry::run_with_results(args, observer),
        "minecraft" => minecraft_rush::run_visit_with_results(args.collect(), &mut visits.minecraft, observer),
        "regularshow" => crate::minigames::regularshow::run_visit(root, args, observer),
        "starwars" => crate::minigames::starwars::run_with_results(root, selected, args, observer),
        "halloween" => crate::minigames::halloween::run_visit(root, args, observer),
        _ => Err(format!("Minijuego no implementado: {id}").into()),
    }
}
