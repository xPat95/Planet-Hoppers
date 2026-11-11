//! Automated native-renderer inspection of real puzzle outcomes, not fake results.
use crate::minigames::southpark::{
    puzzles::{Board, HEATER_SOLUTION},
    session::{SaveKennySession, SessionState},
    SouthParkScene,
};
use raylib::prelude::*;
use std::{collections::HashSet, path::Path};

#[derive(Default)]
pub struct OutcomeSmoke {
    case: usize,
    started: bool,
    captured: bool,
    leaving: bool,
    frames: u32,
    images: HashSet<String>,
}
impl OutcomeSmoke {
    pub fn before_frame(
        &mut self,
        rl: &mut RaylibHandle,
        s: &mut SaveKennySession,
        scene: &mut SouthParkScene,
    ) -> Result<bool, String> {
        self.frames += 1;
        if self.frames > 16000 {
            return Err(format!(
                "Puzzle result smoke timeout: case {} {:?}",
                self.case, s.state
            ));
        }
        if self.captured && !self.leaving {
            s.leave_puzzle();
            self.leaving = true;
        }
        if self.leaving && s.puzzle.is_none() {
            self.case += 1;
            self.started = false;
            self.captured = false;
            self.leaving = false;
        }
        if self.case >= 10 {
            return Ok(true);
        }
        if !self.started {
            s.start(scene, 1)?;
            rl.set_window_size(
                if self.case == 9 { 800 } else { 1280 },
                if self.case == 9 { 600 } else { 800 },
            );
            s.adventure.as_mut().unwrap().spanish = self.case % 2 == 1;
            s.begin_encounter(scene, [1, 0, 2, 3, 4][self.case % 5])?;
            self.started = true;
        }
        let Some(p) = &mut s.puzzle else {
            return Ok(false);
        };
        if p.ready() && !p.started {
            p.start();
        }
        if !p.interactive() {
            return Ok(false);
        }
        if self.case < 5 {
            match &p.board {
                Board::Heaters(h) => {
                    if let Some(i) = (0..16).find(|i| h.tiles[*i] != HEATER_SOLUTION[*i]) {
                        p.click(i);
                    }
                }
                Board::Ufo(u) => {
                    let i = u.target;
                    if (u.reaction-u.window(p.elapsed)).abs()<0.065 {p.click(i);}
                }
                Board::Bus(_) => {
                    use crate::minigames::southpark::murdoku::{SOLUTION,DANGEROUS};
                    for (i,cell) in SOLUTION.into_iter().enumerate() {p.select_bus_piece(i);p.place_bus_piece(cell);}
                    p.accuse_bus_driver(DANGEROUS);p.confirm_bus();
                }
                Board::Piano(k) => {
                    if !k.demonstrating {
                        let i = k.notes[k.input];
                        p.click(i);
                    }
                }
                Board::Sewers(sewers) => {
                    if sewers.target<12 && (sewers.approach-sewers.window()).abs()<0.09 {let i=sewers.target;p.click(i);}
                }
            }
        } else {
            match &p.board {
                Board::Bus(_) => {
                    for (i,cell) in crate::minigames::southpark::murdoku::SOLUTION.into_iter().enumerate() {p.select_bus_piece(i);p.place_bus_piece(cell);}
                    p.accuse_bus_driver(1);p.confirm_bus();
                }
                Board::Piano(k) => {
                    if !k.demonstrating {
                        let i = (k.notes[k.input] + 1) % 5;
                        p.click(i);
                    }
                }
                _ => {} // timed dangers are allowed to reach Kenny naturally
            }
        }
        Ok(false)
    }
    pub fn capture(
        &mut self,
        s: &SaveKennySession,
        root: &Path,
        w: i32,
        h: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(p) = &s.puzzle else {
            return Ok(());
        };
        let name = if s.state == SessionState::Success && p.result_time > 1.0 {
            Some("result")
        } else if s.state == SessionState::Success && p.result_time > 0.3 {
            Some("rescued")
        } else if s.state == SessionState::DeathAnimation && s.elapsed > 0.8 {
            Some("danger")
        } else if s.state == SessionState::DeathDialogue
            && s.voice_time > s.config.kyle_switch_seconds + 0.15
        {
            Some("kyle")
        } else if s.state == SessionState::DeathDialogue
            && s.voice_time > 0.7
            && s.voice_time < s.config.kyle_switch_seconds
        {
            Some("stan")
        } else if matches!(s.state, SessionState::Restoring | SessionState::Defeat) {
            Some("result")
        } else {
            None
        };
        if let Some(name) = name {
            let filename = format!("outcome-{}-{name}.png", self.case);
            if self.images.insert(filename.clone()) {
                crate::graphics::resources::save_frame(
                    &root.join("target/southpark-puzzles-smoke").join(filename),
                    w,
                    h,
                )?;
            }
            if name == "result" {
                if p.result != Some(self.case < 5) {
                    return Err("Unexpected puzzle outcome".into());
                }
                if self.case >= 5
                    && (!self
                        .images
                        .contains(&format!("outcome-{}-stan.png", self.case))
                        || !self
                            .images
                            .contains(&format!("outcome-{}-kyle.png", self.case)))
                {
                    return Err("Missing Stan/Kyle reaction".into());
                }
                self.captured = true;
            }
        }
        Ok(())
    }
}
