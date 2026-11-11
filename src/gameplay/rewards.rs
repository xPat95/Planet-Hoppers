//! Idempotent victory rewards. No navigation, rendering or persistence effects.
use super::{
    progress::Franchise,
    selection::{GameState, ShipCatalog},
};
use crate::minigames::common::host::MinigameResult;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RewardOutcome {
    /// True only for the first victory of this franchise.
    pub token_obtained: bool,
    /// True only when that victory adds previously missing ship ownership.
    pub ship_unlocked: bool,
    /// True only for the victory that grants the sixth token, not later calls.
    pub progress_completed: bool,
}

/// Record an attempt outcome supplied by the minigame rules or secret completion code.
///
/// Callers report Victory when the challenge finishes or the player completes
/// the explicit secret shortcut. Visiting a planet, returning to the host or
/// collecting a partial objective is not a win.
/// IDs must match the existing selection catalog (e.g. `smilingfriends`). Invalid
/// identities/catalog entries return an error without changing the adventure.
/// Repeated victories, defeats and abandonments return an empty outcome.
/// The active and initial ships are never changed.
pub fn record_result(
    game: &mut GameState,
    franchise_id: &str,
    result: MinigameResult,
    catalog: &ShipCatalog,
) -> Result<RewardOutcome, String> {
    let franchise = Franchise::from_id(franchise_id)?;
    if catalog.choice(franchise_id).is_none() {
        return Err(format!("Franquicia ausente del catálogo: {franchise_id}"));
    }
    if result != MinigameResult::Victory || game.progress.is_completed(franchise) {
        return Ok(RewardOutcome::default());
    }

    // Resolve the only fallible mutation before granting the token.
    let ship_unlocked = game.unlock(franchise_id, catalog)?;
    let token_obtained = game.progress.complete(franchise);
    Ok(RewardOutcome {
        token_obtained,
        ship_unlocked,
        progress_completed: game.progress.is_complete(),
    })
}

/// Presentation cheat: grant only missing tokens and their ships, atomically.
/// Active/initial ship selection and existing progress remain unchanged.
pub fn complete_for_presentation(game: &mut GameState, catalog: &ShipCatalog) -> Result<usize, String> {
    let mut completed = game.clone();
    let mut granted = 0;
    for franchise in Franchise::ALL {
        if !completed.progress().is_completed(franchise) {
            record_result(&mut completed, franchise.as_str(), MinigameResult::Victory, catalog)?;
            granted += 1;
        }
    }
    *game = completed;
    Ok(granted)
}
