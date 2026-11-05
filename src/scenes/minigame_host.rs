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
    if id == "starwars" {
        crate::minigames::starwars::run(root, selected, args)
    } else {
        run(id, visits, args)
    }
}
