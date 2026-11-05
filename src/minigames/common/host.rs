//! Navigation and attempt reporting contracts. No reward rules or persistence.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinigameResult {
    Victory,
    Defeat,
    Abandoned,
}

pub type ResultObserver<'a> =
    dyn FnMut(MinigameResult) -> Result<(), Box<dyn std::error::Error>> + 'a;

/// Read-only terminal outcome from the actual challenge rules.
pub trait AttemptOutcome {
    fn attempt_outcome(&self) -> Option<MinigameResult>;
}

/// Lives outside the challenge state so rebuilding that state cannot erase a win.
#[derive(Default)]
pub struct AttemptReporter {
    reported: bool,
}

impl AttemptReporter {
    /// Observe after simulation and before any restart/return input can discard it.
    /// A new nonterminal attempt rearms reporting; terminal frames are delivered once.
    pub fn observe(
        &mut self,
        attempt: &impl AttemptOutcome,
        observer: &mut ResultObserver<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(result) = attempt.attempt_outcome() {
            if !self.reported {
                observer(result)?;
                self.reported = true;
            }
        } else {
            self.reported = false;
        }
        Ok(())
    }
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
