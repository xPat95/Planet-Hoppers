//! Drives the normal menu/raycast/input path, never injects threat results.
use crate::{
    audio::AudioController,
    minigames::southpark::{
        session::{SaveKennySession, SessionState},
        threats::ThreatKind,
        KennyDialogue, SouthParkScene,
    },
};
use raylib::{ffi, prelude::*};
use serde_json::json;
use std::{collections::BTreeSet, path::Path};

pub struct SessionSmoke {
    phase: u8,
    cooldown: f32,
    pause_frames: u32,
    pause_time: f32,
    captures: BTreeSet<&'static str>,
    passed: Vec<ThreatKind>,
    retries: Vec<ThreatKind>,
    last_state: SessionState,
    last_saves: u8,
    victory: bool,
    defeat: bool,
    replayed: bool,
    paused: bool,
    mapped: Vec<ThreatKind>,
    map_frame: u32,
    map_frozen: Option<Vec<f32>>,
    map_checks: usize,
    zones: Vec<String>,
}
impl SessionSmoke {
    pub fn new() -> Self {
        Self {
            phase: 0,
            cooldown: 0.0,
            pause_frames: 0,
            pause_time: 0.0,
            captures: BTreeSet::new(),
            passed: vec![],
            retries: vec![],
            last_state: SessionState::Exploring,
            last_saves: 0,
            victory: false,
            defeat: false,
            replayed: false,
            paused: false,
            mapped: vec![],
            map_frame: 0,
            map_frozen: None,
            map_checks: 0,
            zones: vec![],
        }
    }
    pub fn done(&self) -> bool {
        self.phase == 7
    }
    pub fn before_frame(
        &mut self,
        rl: &RaylibHandle,
        scene: &mut SouthParkScene,
        s: &SaveKennySession,
    ) -> Result<(), String> {
        for key in [
            KeyboardKey::KEY_E,
            KeyboardKey::KEY_ENTER,
            KeyboardKey::KEY_P,
            KeyboardKey::KEY_M,
            KeyboardKey::KEY_ESCAPE,
            KeyboardKey::KEY_R,
            KeyboardKey::KEY_W,
        ] {
            event(1, [key as i32, 0, 0, 0]);
        }
        event(5, [0, 0, 0, 0]);
        use SessionState::*;
        if s.dynamic() && s.state == MapOpen {
            self.map_frame += 1;
            if self.map_frozen.as_ref() != Some(&snapshot(scene)) {
                return Err("World/animation advanced while map open".into());
            }
            press(KeyboardKey::KEY_W);
            if self.map_frame == 2 {
                event(8, [0, 1, 0, 0]);
            }
            if (3..=4).contains(&self.map_frame) {
                event(
                    7,
                    [
                        rl.get_screen_width() / 2 + self.map_frame as i32 * 30,
                        rl.get_screen_height() / 2 + 20,
                        0,
                        0,
                    ],
                );
                event(6, [0, 0, 0, 0]);
            }
            if self.map_frame == 6 {
                press(KeyboardKey::KEY_R);
            }
            if self.map_frame >= 8 {
                press(if self.map_checks.is_multiple_of(2) {
                    KeyboardKey::KEY_M
                } else {
                    KeyboardKey::KEY_ESCAPE
                });
            }
            return Ok(());
        }
        self.cooldown -= 0.1;
        if self.cooldown > 0.0 {
            return Ok(());
        }
        if s.dynamic()
            && s.state == Active
            && self.phase == 0
            && !self.mapped.contains(&s.current())
        {
            self.mapped.push(s.current());
            self.map_frame = 0;
            self.map_frozen = Some(snapshot(scene));
            press(KeyboardKey::KEY_M);
            return Ok(());
        }
        match self.phase {
            0 if s.state == Victory && s.elapsed > 0.4 => {
                press(KeyboardKey::KEY_ENTER);
                self.phase = 1;
                self.cooldown = 0.4;
                return Ok(());
            }
            1 if s.dynamic() && s.state == Interlude => {
                if s.lives != 3 || s.saves != 0 {
                    return Err("Replay did not reset match".into());
                }
                self.replayed = true;
                press(KeyboardKey::KEY_P);
                return Ok(());
            }
            1 if s.state == Briefing || (s.dynamic() && s.state == Paused) => {
                if s.lives != 3 || s.saves != 0 {
                    return Err("Replay did not reset match".into());
                }
                self.replayed = true;
                click_right(rl);
                self.phase = 2;
                self.cooldown = 0.4;
                return Ok(());
            }
            2 if s.state == Briefing || (s.dynamic() && s.state == Interlude) => {
                self.phase = 3;
            }
            3 if s.state == Defeat && s.elapsed > 0.5 => {
                click_right(rl);
                self.phase = 4;
                self.cooldown = 0.4;
                return Ok(());
            }
            4 if s.state == Briefing || (s.dynamic() && s.state == Interlude) => {
                self.phase = 5;
            }
            5 if s.state == Preparing => {
                press(KeyboardKey::KEY_P);
                self.phase = 6;
                self.pause_time = scene.threats.elapsed;
                return Ok(());
            }
            6 if s.state == Paused => {
                self.pause_frames += 1;
                if (scene.threats.elapsed - self.pause_time).abs() > 0.001 {
                    return Err("Threat advanced behind pause menu".into());
                }
                if self.pause_frames >= 6 {
                    self.paused = true;
                    click_right(rl);
                    self.cooldown = 0.4;
                }
                return Ok(());
            }
            6 if s.state == Exploring => {
                self.phase = 7;
                return Ok(());
            }
            _ => {}
        }
        if s.state == Searching {
            if scene.threats.elapsed != 0.0 || scene.threats.state_time != 0.0 {
                return Err("Search consumed mortal time".into());
            }
            if s.elapsed > 2.0 {
                let p = scene.threats.stage(s.current()) + Vector3::new(-16.0, 0.0, -2.0);
                if !scene.world.can_occupy(p, 0.35, 1.8) {
                    return Err("World smoke entrance blocked".into());
                }
                scene.player.feet = p;
                aim(scene, scene.kenny.position() + Vector3::up());
            }
            return Ok(());
        }
        if s.state == Exploring {
            if scene.dialogue == KennyDialogue::Invitation {
                press(KeyboardKey::KEY_ENTER);
            } else {
                let p = scene.kenny.position();
                let feet = [
                    Vector3::new(-2.3, 0.0, 0.0),
                    Vector3::new(2.3, 0.0, 0.0),
                    Vector3::new(0.0, 0.0, 2.3),
                ]
                .iter()
                .map(|d| p + *d)
                .find(|q| scene.world.can_occupy(*q, 0.35, 1.8))
                .ok_or("No smoke approach to Kenny")?;
                scene.player.feet = feet;
                aim(scene, p + Vector3::up() * 0.675);
                press(KeyboardKey::KEY_E);
            }
            self.cooldown = 0.4;
        } else if (s.state == Briefing && s.elapsed > 0.5)
            || (!s.dynamic() && s.state == Success && s.elapsed > s.config.success_hold_seconds)
        {
            press(KeyboardKey::KEY_ENTER);
            self.cooldown = 0.4;
        } else if s.state == Active {
            if self.phase == 3 {
                aim(scene, scene.kenny.position() + Vector3::up() * 0.7);
                return Ok(());
            }
            if scene.threats.elapsed < 2.0 {
                return Ok(());
            }
            if let Some(target) = scene.threats.targets().into_iter().next() {
                let feet = match s.current() {
                    ThreatKind::Ufo => {
                        scene.threats.stage(ThreatKind::Ufo) - Vector3::right() * 5.0
                    }
                    ThreatKind::Sewer if scene.threats.carrying_lid => {
                        scene.threats.hole_position() + Vector3::new(-2.0, 0.0, 2.0)
                    }
                    _ => {
                        target.bounds.center() + Vector3::new(-2.5, -target.bounds.center().y, -1.0)
                    }
                };
                if !scene.world.can_occupy(feet, 0.35, 1.8) {
                    return Err("Smoke target approach blocked".into());
                }
                scene.player.feet = feet;
                aim(scene, target.bounds.center());
                press(KeyboardKey::KEY_E);
                self.cooldown = 0.6;
            }
        }
        Ok(())
    }
    pub fn capture(&mut self, s: &SaveKennySession) -> Option<&'static str> {
        use SessionState::*;
        let name = match s.state {
            MapOpen if self.map_frame >= 4 => "map-zoom.png",
            MapOpen => "map.png",
            Searching if s.lives == 2 => "lives.png",
            Searching => "search.png",
            Briefing if s.lives == 2 => "lives.png",
            Briefing => "briefing.png",
            Active if s.elapsed > 1.0 => "hud.png",
            DeathDialogue if s.voice_time >= 0.7 && s.voice_time < s.config.kyle_switch_seconds => {
                "stan.png"
            }
            DeathDialogue if s.voice_time >= s.config.kyle_switch_seconds + 0.3 => "kyle.png",
            Victory => "victory.png",
            Defeat => "defeat.png",
            Paused => "pause.png",
            _ => return None,
        };
        self.captures.insert(name).then_some(name)
    }
    pub fn after_frame(
        &mut self,
        scene: &SouthParkScene,
        s: &SaveKennySession,
        _audio: &AudioController,
    ) -> Result<(), String> {
        if self.last_state == SessionState::MapOpen && s.state != SessionState::MapOpen {
            if !unsafe { ffi::IsCursorHidden() } {
                return Err("FPS cursor was not recaptured after map".into());
            }
            self.map_checks += 1;
            self.map_frozen = None;
        }
        if s.state == SessionState::Searching && self.last_state != SessionState::Searching {
            if let Some(site) = &s.site {
                self.zones.push(site.id.clone());
            }
        }
        if self.last_state != s.state {
            println!(
                "SESSION_TRACE phase={} {:?} {:?} lives={} saves={}",
                self.phase,
                s.state,
                s.current(),
                s.lives,
                s.saves
            );
        }
        if let Some(error) = &s.error {
            return Err(format!("Session transition failed: {error}"));
        }
        if s.state == SessionState::Success && s.saves > self.last_saves {
            self.passed.push(s.current());
        }
        if s.state == SessionState::Preparing
            && self.last_state != SessionState::Preparing
            && self.phase == 3
        {
            self.retries.push(s.current());
        }
        if s.state == SessionState::Victory {
            self.victory = true;
            if s.saves != 5 {
                return Err("Premature victory".into());
            }
        }
        if s.state == SessionState::Defeat {
            self.defeat = true;
            if s.lives != 0 {
                return Err("Premature defeat".into());
            }
        }
        if matches!(s.state, SessionState::Victory | SessionState::Defeat)
            && scene.threats.running()
        {
            return Err("Danger remained after match".into());
        }
        self.last_state = s.state;
        self.last_saves = s.saves;
        Ok(())
    }
    pub fn finish(
        &self,
        scene: &SouthParkScene,
        s: &SaveKennySession,
        audio: &AudioController,
        portraits: usize,
        out: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let distinct: BTreeSet<_> = self.passed.iter().map(|k| format!("{k:?}")).collect();
        let voice_plays = audio
            .audit
            .plays
            .get("they_killed_kenny")
            .copied()
            .unwrap_or(0);
        if !self.done()
            || !self.victory
            || !self.defeat
            || !self.replayed
            || !self.paused
            || distinct.len() != 5
            || self.retries.len() != 3
            || !self.retries.iter().all(|k| *k == self.retries[0])
            || voice_plays != 3
            || audio.audit.plays.get("victory") != Some(&1)
            || audio.audit.plays.get("game_over") != Some(&1)
            || audio.audit.plays.get("success") != Some(&5)
            || audio.audit.loaded.len() != 8
            || portraits != 2
            || !s.assert_exploration_clean(scene)
            || self.captures.len() != if s.dynamic() { 10 } else { 8 }
            || (s.dynamic() && self.map_checks != 5)
        {
            return Err(format!(
                "Incomplete session smoke: passed={:?}, retries={:?}, audio={}, captures={:?}",
                self.passed,
                self.retries,
                serde_json::to_string(&audio.audit)?,
                self.captures
            )
            .into());
        }
        let report = json!({"status":"PASS","five_distinct_successes":self.passed,"three_retries_same_pending_threat":self.retries,
            "victory":self.victory,"defeat":self.defeat,"replay_reset":self.replayed,"pause_freezes_threat":self.paused,
            "returned_to_exploration":true,"portraits_loaded":portraits,"audio":audio.audit,
            "kyle_switch_seconds":s.config.kyle_switch_seconds,"voice_duration_seconds":audio.voice_duration(),
            "captures":self.captures,"manual_listening_verified":false,"physical_input_verified":false,
            "map_pause_resume_checks":self.map_checks,"event_sites":self.zones,"automatic_progression":s.dynamic(),
            "note":"Real renderer, native input events and shared raycasting; test player placement/aiming are automated."});
        std::fs::write(
            out.join("runtime-validation.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("SESSION_SMOKE_PASS {report}");
        Ok(())
    }
}
fn event(kind: u32, params: [i32; 4]) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: kind,
            params,
        });
    }
}
fn press(key: KeyboardKey) {
    event(2, [key as i32, 0, 0, 0]);
}
fn click_right(rl: &RaylibHandle) {
    event(
        7,
        [
            rl.get_screen_width() / 2 + 144,
            rl.get_screen_height() / 2 + 82,
            0,
            0,
        ],
    );
    event(6, [0, 0, 0, 0]);
}
fn aim(scene: &mut SouthParkScene, p: Vector3) {
    let d = (p - scene.player.eye()).normalized();
    scene.player.yaw = d.z.atan2(d.x);
    scene.player.pitch = d.y.asin();
}
fn snapshot(s: &SouthParkScene) -> Vec<f32> {
    let mut v = vec![
        s.player.yaw,
        s.player.pitch,
        s.kenny.animation.phase,
        s.kenny.animation.weight,
        s.threats.elapsed,
        s.threats.state_time,
        s.threats.progress,
    ];
    for p in [
        s.player.feet,
        s.kenny.position(),
        s.threats.bus_position(),
        s.threats.ufo_position(),
        s.threats.piano_position(),
        s.threats.snow_position(),
    ] {
        v.extend([p.x, p.y, p.z]);
    }
    v
}
