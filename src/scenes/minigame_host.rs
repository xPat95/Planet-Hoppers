//! In-process dispatch. Each adapter owns one window at a time and returns the shared result.
use super::{smilingfriends_entry, southpark_dev, southpark_entry::SouthParkVisits};
use crate::minigames::common::host::MinigameExit;

#[derive(Default)]
pub struct MinigameVisits {
    pub southpark: SouthParkVisits,
}

pub fn run(
    id: &str,
    visits: &mut MinigameVisits,
    args: impl Iterator<Item = String>,
) -> Result<MinigameExit, Box<dyn std::error::Error>> {
    match id {
        "southpark" => southpark_dev::run_visit_with_args(&mut visits.southpark, args),
        "smilingfriends" => smilingfriends_entry::run(args),
        _ => Err(format!("Minijuego no implementado: {id}").into()),
    }
}
