//! Regular Show: El Martillo, integrated without launching an external executable.
mod app;
mod media;
mod render;
pub mod simulation;
mod support;
mod video;

pub use app::{autoplay, run_visit};

impl crate::minigames::common::host::AttemptOutcome for simulation::Game {
    fn attempt_outcome(&self) -> Option<crate::minigames::common::host::MinigameResult> {
        use crate::minigames::common::host::MinigameResult;
        match self.phase {
            simulation::Phase::ArcadeClear => Some(MinigameResult::Victory),
            simulation::Phase::Defeat => Some(MinigameResult::Defeat),
            _ => None,
        }
    }
}
