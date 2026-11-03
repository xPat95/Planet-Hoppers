//! Navigation and session attempt results have separate lifetimes. No rewards/persistence.
use super::rush::RushCue;
use crate::minigames::common::host::{MinigameExit, MinigameResult};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct CompletedAttempt {
    pub attempt_id: u64,
    pub result: MinigameResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum AttemptEvent {
    Started { attempt_id: u64 },
    Finished(CompletedAttempt),
}

/// Owned by the existing solar session across Minecraft visits, not by the village UI.
#[derive(Default)]
pub struct MinecraftVisits {
    next_attempt: u64,
    active: Option<u64>,
    completed: Vec<CompletedAttempt>,
    events: Vec<AttemptEvent>,
}
impl MinecraftVisits {
    pub fn start_attempt(&mut self) -> u64 {
        // Restarting an unfinished attempt ends that attempt before starting the next.
        self.finish(MinigameResult::Abandoned);
        self.next_attempt += 1;
        self.active = Some(self.next_attempt);
        self.events.push(AttemptEvent::Started {
            attempt_id: self.next_attempt,
        });
        self.next_attempt
    }
    pub fn active_attempt(&self) -> Option<u64> {
        self.active
    }
    pub fn completed(&self) -> &[CompletedAttempt] {
        &self.completed
    }
    pub fn finish(&mut self, result: MinigameResult) -> Option<CompletedAttempt> {
        let attempt = CompletedAttempt {
            attempt_id: self.active.take()?,
            result,
        };
        self.completed.push(attempt);
        self.events.push(AttemptEvent::Finished(attempt));
        println!(
            "MINECRAFT_ATTEMPT_FINISHED {}",
            serde_json::to_string(&attempt).expect("Attempt JSON")
        );
        Some(attempt)
    }
    /// Consume the explicit outcome cues emitted by the rules, never a scene transition.
    pub fn record_cues(&mut self, cues: &[RushCue]) {
        for cue in cues {
            match cue {
                RushCue::Victory => {
                    self.finish(MinigameResult::Victory);
                }
                RushCue::GameOver => {
                    self.finish(MinigameResult::Defeat);
                }
                _ => {}
            }
        }
    }
    /// Future reward consumer: each event can be taken once, with a session-unique ID.
    /// No tokens are granted here; completed history survives event consumption.
    pub fn drain_events(&mut self) -> Vec<AttemptEvent> {
        std::mem::take(&mut self.events)
    }
    pub fn exit_since(&self, visit_start: usize) -> MinigameExit {
        self.completed
            .get(visit_start..)
            .and_then(|attempts| attempts.last())
            .map_or(MinigameExit::ReturnedWithoutResult, |attempt| {
                MinigameExit::Returned(attempt.result)
            })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VillageMenu {
    #[default]
    Closed,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VillageAction {
    None,
    Pause,
    Continue,
    RequestReturn,
}

impl VillageMenu {
    /// Village exploration has no active challenge to abandon: return directly.
    pub fn apply(&mut self, action: VillageAction) -> bool {
        match (*self, action) {
            (Self::Closed, VillageAction::Pause) => *self = Self::Paused,
            (Self::Paused, VillageAction::Continue) => *self = Self::Closed,
            (Self::Paused, VillageAction::RequestReturn) => {
                *self = Self::Closed;
                return true;
            }
            _ => {}
        }
        false
    }
}
