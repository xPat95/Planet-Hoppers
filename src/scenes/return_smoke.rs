//! Real renderer/input round trips. Uses existing threat/raycast automation;
//! victory and defeat are reached by playing, never by assigning terminal states.
use super::session_smoke::{click_pause, click_right, event, press, snapshot, SessionSmoke};
use crate::minigames::{
    common::host::{MinigameExit, MinigameResult, ReturnConfirmation},
    southpark::{
        session::{SaveKennySession, SessionState},
        SouthParkScene,
    },
};
use raylib::prelude::*;
use std::{collections::BTreeSet, path::Path};

pub struct ReturnSmoke {
    goal: MinigameResult,
    driver: SessionSmoke,
    stage: u8,
    frames_paused: u32,
    frozen: Vec<f32>,
    clock: f64,
    terminal_seen: bool,
    captures: BTreeSet<String>,
}
impl ReturnSmoke {
    pub fn new(goal: MinigameResult) -> Self {
        Self {
            goal,
            driver: if goal == MinigameResult::Defeat {
                SessionSmoke::losing()
            } else {
                SessionSmoke::new()
            },
            stage: 0,
            frames_paused: 0,
            frozen: vec![],
            clock: 0.0,
            terminal_seen: false,
            captures: BTreeSet::new(),
        }
    }
    pub fn before_frame(
        &mut self,
        rl: &RaylibHandle,
        scene: &mut SouthParkScene,
        s: &SaveKennySession,
        confirmation: &ReturnConfirmation,
        intro: bool,
    ) -> Result<(), String> {
        for key in [
            KeyboardKey::KEY_ENTER,
            KeyboardKey::KEY_SPACE,
            KeyboardKey::KEY_ESCAPE,
            KeyboardKey::KEY_P,
            KeyboardKey::KEY_E,
            KeyboardKey::KEY_M,
            KeyboardKey::KEY_W,
        ] {
            event(1, [key as i32, 0, 0, 0]);
        }
        event(5, [0, 0, 0, 0]);
        if intro {
            press(KeyboardKey::KEY_SPACE);
            return Ok(());
        }
        use SessionState::*;
        if (self.goal == MinigameResult::Victory && s.state == Victory)
            || (self.goal == MinigameResult::Defeat && s.state == Defeat)
        {
            self.terminal_seen = true;
            if (s.state == Victory && s.saves != 5) || (s.state == Defeat && s.lives != 0) {
                return Err("Terminal result is not earned".into());
            }
            if s.elapsed > 0.4 {
                click_right(rl);
            }
            return Ok(());
        }
        if self.goal == MinigameResult::Abandoned {
            if (1..=3).contains(&self.stage)
                && (snapshot(scene) != self.frozen || s.activation_elapsed != self.clock)
            {
                return Err(
                    "Gameplay/activation clock changed behind pause or confirmation".into(),
                );
            }
            match self.stage {
                0 if s.state == Searching => {
                    self.frozen = snapshot(scene);
                    self.clock = s.activation_elapsed;
                    press(KeyboardKey::KEY_ESCAPE);
                    self.stage = 1;
                }
                1 if s.state == Paused => {
                    self.frames_paused += 1;
                    if self.frames_paused >= 20 {
                        click_pause(rl, 2);
                        self.stage = 2;
                    }
                }
                2 if confirmation.pending => {
                    press(KeyboardKey::KEY_ESCAPE);
                    self.frames_paused = 0;
                    self.stage = 3;
                }
                3 if !confirmation.pending && s.state == Paused => {
                    // A physical key must be released for a frame between two presses.
                    self.frames_paused += 1;
                    if self.frames_paused >= 2 {
                        press(KeyboardKey::KEY_ESCAPE);
                        self.stage = 4;
                    }
                }
                4 if s.state == Searching => {
                    if snapshot(scene) != self.frozen || s.activation_elapsed != self.clock {
                        return Err("Resume consumed the reaction window".into());
                    }
                    if !rl.is_cursor_hidden() {
                        return Err("Resume lost cursor capture".into());
                    }
                    press(KeyboardKey::KEY_P);
                    self.stage = 5;
                }
                5 if s.state == Paused => {
                    click_pause(rl, 1);
                    self.stage = 6;
                }
                6 if s.state == Interlude => {
                    if s.lives != 3
                        || s.saves != 0
                        || s.activation_elapsed != 0.0
                        || scene.threats.kind.is_some()
                        || scene.confirmation.is_some()
                    {
                        return Err("Restart did not clear the attempt".into());
                    }
                    press(KeyboardKey::KEY_ESCAPE);
                    self.stage = 7;
                }
                7 if s.state == Paused => {
                    click_pause(rl, 2);
                    self.stage = 8;
                }
                8 if confirmation.pending => {
                    press(KeyboardKey::KEY_ENTER);
                    self.stage = 9;
                }
                _ if self.stage == 0 => self.driver.before_frame(rl, scene, s)?,
                _ => {}
            }
        } else {
            self.driver.before_frame(rl, scene, s)?;
        }
        Ok(())
    }
    pub fn after_frame(
        &mut self,
        scene: &SouthParkScene,
        s: &SaveKennySession,
        audio: &crate::audio::AudioController,
    ) -> Result<(), String> {
        self.driver.after_frame(scene, s, audio)
    }
    pub fn capture(
        &mut self,
        s: &SaveKennySession,
        confirmation: &ReturnConfirmation,
    ) -> Option<String> {
        let name = match s.state {
            SessionState::Victory => "victory",
            SessionState::Defeat => "defeat",
            SessionState::Paused if confirmation.pending => "confirm-return",
            SessionState::Paused => "pause",
            SessionState::Interlude if self.stage == 7 => "restarted",
            _ => return None,
        };
        let name = format!("{name}.png");
        self.captures.insert(name.clone()).then_some(name)
    }
    pub fn finish(
        &self,
        exit: MinigameExit,
        path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if exit != MinigameExit::Returned(self.goal)
            || (self.goal == MinigameResult::Abandoned && self.stage != 9)
            || (self.goal != MinigameResult::Abandoned && !self.terminal_seen)
        {
            return Err(format!(
                "Incomplete return test: expected {:?}, got {exit:?}, stage={}",
                self.goal, self.stage
            )
            .into());
        }
        let report = serde_json::json!({"status":"PASS", "result":self.goal, "pause_cancel_resume_restart_confirm":self.goal == MinigameResult::Abandoned, "terminal_reached_by_gameplay":self.terminal_seen, "physical_input_verified":false});
        std::fs::write(
            path.join("return-validation.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("RETURN_SMOKE_PASS {report}");
        Ok(())
    }
}
