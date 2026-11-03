//! Minecraft adapter. One stream survives surface/mine transitions; shared audit format.
use super::{
    mining::MiningEvent,
    rush::{RushCue, RushState},
};
use crate::audio::AudioAudit;
use raylib::prelude::*;
use std::{collections::BTreeMap, path::Path};
pub struct MinecraftAudio<'a> {
    music: Option<Music<'a>>,
    sounds: BTreeMap<&'static str, Sound<'a>>,
    pub audit: AudioAudit,
    pub music_starts: u32,
    elapsed: f32,
    last_hit: f32,
    step: f32,
    muted: bool,
    context: &'static str,
}
impl<'a> MinecraftAudio<'a> {
    pub fn load(device: Option<&'a RaylibAudio>, root: &Path, muted: bool) -> Self {
        let mut audit = AudioAudit::default();
        let mut music = None;
        let mut sounds = BTreeMap::new();
        let mut starts = 0;
        if let Some(d) = device {
            let path = "assets/audio/music/Minecraft/Minecraft.mp3";
            match d.new_music(&root.join(path).to_string_lossy()) {
                Ok(mut m) => {
                    m.looping = true;
                    m.set_volume(if muted { 0.0 } else { 0.22 });
                    audit.loaded.insert(path.into(), m.get_time_length());
                    m.play_stream();
                    starts = 1;
                    music = Some(m);
                }
                Err(e) => audit.warnings.push(format!("{path}: {e}")),
            }
            for (name, ext) in [
                ("block_break", "wav"),
                ("danger", "mp3"),
                ("diamond_collect", "wav"),
                ("footsteps", "mp3"),
                ("game_over", "mp3"),
                ("pickaxe_hit", "mp3"),
                ("success", "mp3"),
                ("victory", "mp3"),
            ] {
                let path = format!("assets/audio/sfx/Minecraft/{name}.{ext}");
                match d.new_sound(&root.join(&path).to_string_lossy()) {
                    Ok(s) => {
                        audit
                            .loaded
                            .insert(path, s.frame_count() as f32 / s.stream.sampleRate as f32);
                        s.set_volume(if muted {
                            0.0
                        } else {
                            match name {
                                "footsteps" => 0.16,
                                "pickaxe_hit" => 0.32,
                                _ => 0.6,
                            }
                        });
                        sounds.insert(name, s);
                    }
                    Err(e) => audit.warnings.push(format!("{path}: {e}")),
                }
            }
        } else {
            audit.warnings.push("No audio device available".into());
        }
        Self {
            music,
            sounds,
            audit,
            music_starts: starts,
            elapsed: 0.0,
            last_hit: -1.0,
            step: 0.0,
            muted,
            context: "",
        }
    }
    fn play(&mut self, name: &'static str) {
        if let Some(s) = self.sounds.get(name) {
            let retrigger = matches!(name, "pickaxe_hit" | "block_break" | "diamond_collect");
            if retrigger || !s.is_playing() {
                if retrigger {
                    s.stop();
                }
                s.play();
                *self.audit.plays.entry(name.into()).or_default() += 1;
            }
        }
    }
    pub fn update(
        &mut self,
        dt: f32,
        moving: bool,
        sprint: bool,
        state: Option<RushState>,
        warning: bool,
    ) {
        self.elapsed += dt.max(0.0);
        let context = match state {
            None => "surface",
            Some(RushState::Loading) => "loading",
            Some(RushState::Running) if warning => "mine-low",
            Some(RushState::Running) => "mine",
            Some(RushState::Paused) => "paused",
            Some(RushState::Victory) => "victory",
            Some(RushState::Defeat) => "defeat",
        };
        if self.context != context {
            self.audit.music_transitions.push(context.into());
            self.context = context;
        }
        if let Some(m) = &self.music {
            m.update_stream();
            m.set_volume(if self.muted {
                0.0
            } else if warning
                || matches!(
                    state,
                    Some(RushState::Victory | RushState::Defeat | RushState::Paused)
                )
            {
                0.12
            } else {
                0.22
            });
        }
        if moving {
            self.step -= dt;
            if self.step <= 0.0 {
                self.play("footsteps");
                self.step = if sprint { 0.34 } else { 0.52 };
            }
        } else {
            self.step = 0.0;
            if let Some(s) = self.sounds.get("footsteps") {
                s.stop();
            }
        }
    }
    pub fn mining(&mut self, events: &[MiningEvent]) {
        for e in events {
            match e {
                MiningEvent::Impact { .. } => {
                    if self.elapsed - self.last_hit >= 0.12 {
                        self.play("pickaxe_hit");
                        self.last_hit = self.elapsed;
                    }
                }
                MiningEvent::BlockDestroyed { .. } => self.play("block_break"),
                MiningEvent::DiamondCollected { .. } => self.play("diamond_collect"),
                MiningEvent::SessionReset => self.reset_effects(),
                _ => {}
            }
        }
    }
    pub fn cues(&mut self, cues: &[RushCue]) {
        for c in cues {
            match c {
                RushCue::Danger => self.play("danger"),
                RushCue::Success => self.play("success"),
                RushCue::Victory => self.play("victory"),
                RushCue::GameOver => self.play("game_over"),
                RushCue::Reset => self.reset_effects(),
            }
        }
    }
    pub fn reset_effects(&mut self) {
        for s in self.sounds.values() {
            s.stop();
        }
        self.step = 0.0;
        self.last_hit = -1.0;
    }
    pub fn music_playing(&self) -> bool {
        self.music.as_ref().is_some_and(|m| m.is_stream_playing())
    }
    pub fn stop(&mut self) {
        self.reset_effects();
        if let Some(music) = &self.music {
            music.stop_stream();
        }
    }
}
