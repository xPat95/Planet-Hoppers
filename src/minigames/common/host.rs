//! Contract between a minigame and its host. No rewards or saved attempt state.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinigameResult {
    Victory,
    Defeat,
    Abandoned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinigameExit {
    Returned(MinigameResult),
    /// Navigation back to the host without a challenge result (free exploration).
    ReturnedWithoutResult,
    WindowClosed,
}

/// The host keeps the originating identity for future progression consumers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MinigameCompletion {
    pub planet_id: String,
    pub minigame_id: String,
    pub result: MinigameResult,
}

#[derive(Default)]
pub struct ReturnConfirmation {
    pub pending: bool,
}
impl ReturnConfirmation {
    pub fn request(&mut self) {
        self.pending = true;
    }
    pub fn cancel(&mut self) {
        self.pending = false;
    }
    pub fn confirm(&mut self) -> Option<MinigameResult> {
        std::mem::take(&mut self.pending).then_some(MinigameResult::Abandoned)
    }
}
