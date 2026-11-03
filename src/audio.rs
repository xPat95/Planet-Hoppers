//! Shared raylib audio lifetime and playback policy. Missing devices/files are
//! optional; no decoding failure may prevent exploring the town.
use raylib::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum AudioCue {
    Danger,
    Success,
    DeathVoice,
    Victory,
    GameOver,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicMode {
    Exploration,
    Challenge,
    Silent,
    Intro,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct VoiceClock {
    pub position: Option<f32>,
    pub finished: bool,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub voice_volume: f32,
    pub footsteps_volume: f32,
}
impl Default for AudioConfig {
    fn default() -> Self {
        serde_json::from_str(include_str!("../config/southpark-audio.json"))
            .expect("Bundled audio config")
    }
}
impl AudioConfig {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let c: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if ![
            c.music_volume,
            c.sfx_volume,
            c.voice_volume,
            c.footsteps_volume,
        ]
        .iter()
        .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
        {
            return Err("Audio volumes must be in [0,1]".into());
        }
        Ok(c)
    }
}
#[derive(Default, Serialize)]
pub struct AudioAudit {
    pub loaded: BTreeMap<String, f32>,
    pub warnings: Vec<String>,
    pub plays: BTreeMap<String, u32>,
    pub music_transitions: Vec<String>,
}
pub struct AudioController<'a> {
    intro: Option<Music<'a>>,
    exploration: Option<Music<'a>>,
    challenge: Option<Music<'a>>,
    voice: Option<Music<'a>>,
    sounds: BTreeMap<&'static str, Sound<'a>>,
    config: AudioConfig,
    mode: MusicMode,
    music_started: [bool; 3],
    voice_started: Option<Instant>,
    intro_started: Option<Instant>,
    paused_at: Option<Instant>,
    voice_paused: bool,
    pub audit: AudioAudit,
}
impl<'a> AudioController<'a> {
    pub fn load(device: Option<&'a RaylibAudio>, root: &Path, config: AudioConfig) -> Self {
        let mut audit = AudioAudit::default();
        if device.is_none() {
            audit
                .warnings
                .push("Audio device unavailable; subtitles remain enabled".into());
        }
        let mut music = |relative: &str, looping: bool| {
            let path = root.join(relative);
            let loaded = device.and_then(|d| match d.new_music(&path.to_string_lossy()) {
                Ok(mut m) => {
                    m.looping = looping;
                    audit.loaded.insert(relative.into(), m.get_time_length());
                    Some(m)
                }
                Err(e) => {
                    audit.warnings.push(format!("{}: {e}", path.display()));
                    None
                }
            });
            loaded
        };
        let intro = music(crate::scenes::southpark_video::AUDIO, false);
        let exploration = music("assets/audio/music/South Park/exploration.mp3", true);
        let challenge = music("assets/audio/music/South Park/save_kenny.mp3", true);
        let voice = music("assets/audio/sfx/South Park/they_killed_kenny.mp3", false);
        let mut sounds = BTreeMap::new();
        for name in ["danger", "footsteps", "success", "victory", "game_over"] {
            let relative = format!("assets/audio/sfx/South Park/{name}.mp3");
            let path = root.join(&relative);
            if let Some(d) = device {
                match d.new_sound(&path.to_string_lossy()) {
                    Ok(s) => {
                        audit.loaded.insert(
                            relative,
                            s.frame_count() as f32 / s.stream.sampleRate as f32,
                        );
                        s.set_volume(if name == "footsteps" {
                            config.footsteps_volume
                        } else {
                            config.sfx_volume
                        });
                        sounds.insert(name, s);
                    }
                    Err(e) => audit.warnings.push(format!("{}: {e}", path.display())),
                }
            }
        }
        if let Some(v) = &voice {
            v.set_volume(config.voice_volume);
        }
        for warning in &audit.warnings {
            eprintln!("AUDIO_WARNING {warning}");
        }
        println!(
            "AUDIO_LOADED {}",
            serde_json::to_string(&audit).unwrap_or_default()
        );
        Self {
            intro,
            exploration,
            challenge,
            voice,
            sounds,
            config,
            mode: MusicMode::Silent,
            music_started: [false; 3],
            voice_started: None,
            intro_started: None,
            paused_at: None,
            voice_paused: false,
            audit,
        }
    }
    fn count(&mut self, name: &str) {
        *self.audit.plays.entry(name.into()).or_default() += 1;
    }
    pub fn set_music(&mut self, mode: MusicMode) {
        if mode == self.mode {
            return;
        }
        if let Some(m) = match self.mode {
            MusicMode::Exploration => self.exploration.as_ref(),
            MusicMode::Challenge => self.challenge.as_ref(),
            MusicMode::Intro => self.intro.as_ref(),
            _ => None,
        } {
            if self.mode == MusicMode::Intro {
                m.stop_stream();
            } else {
                m.pause_stream();
            }
        }
        self.mode = mode;
        self.intro_started = (mode == MusicMode::Intro).then(Instant::now);
        let (music, index, name) = match mode {
            MusicMode::Exploration => (self.exploration.as_ref(), 0, "exploration"),
            MusicMode::Challenge => (self.challenge.as_ref(), 1, "save_kenny"),
            MusicMode::Silent => (None, 0, "silent"),
            MusicMode::Intro => (self.intro.as_ref(), 2, "intro"),
        };
        if let Some(m) = music {
            m.set_volume(self.config.music_volume);
            if self.music_started[index] && mode != MusicMode::Intro {
                m.resume_stream();
            } else {
                m.play_stream();
                self.music_started[index] = true;
            }
        }
        self.audit.music_transitions.push(name.into());
    }
    pub fn cue(&mut self, cue: AudioCue) {
        // Alerts/results replace one another; footsteps keep their single
        // channel until movement stops. Voice stops every effect below.
        for (name, sound) in &self.sounds {
            if *name != "footsteps" {
                sound.stop();
            }
        }
        let name = match cue {
            AudioCue::Danger => "danger",
            AudioCue::Success => "success",
            AudioCue::Victory => "victory",
            AudioCue::GameOver => "game_over",
            AudioCue::DeathVoice => "they_killed_kenny",
        };
        if cue == AudioCue::DeathVoice {
            self.stop_effects();
            if let Some(v) = &self.voice {
                v.stop_stream();
                v.play_stream();
            }
            self.voice_started = Some(Instant::now());
        } else if let Some(s) = self.sounds.get(name) {
            s.stop();
            s.play();
        }
        self.count(name);
    }
    pub fn update(&self) {
        for m in [&self.intro, &self.exploration, &self.challenge, &self.voice]
            .into_iter()
            .flatten()
        {
            if m.is_stream_playing() {
                m.update_stream();
            }
        }
    }
    pub fn intro_clock(&self) -> VoiceClock {
        let (Some(started), Some(m)) = (self.intro_started, self.intro.as_ref()) else {
            return VoiceClock::default();
        };
        let length = m.get_time_length();
        // Device/stream loss must still fall back to the timed cinematic.
        if started.elapsed().as_secs_f32() > length + 2.0 {
            return VoiceClock::default();
        }
        let finished = !m.is_stream_playing() && started.elapsed().as_secs_f32() > 0.1;
        VoiceClock {
            position: Some(if finished {
                length
            } else {
                m.get_time_played()
            }),
            finished,
        }
    }
    pub fn intro_duration(&self) -> Option<f32> {
        self.intro
            .as_ref()
            .map(Music::get_time_length)
            .filter(|v| v.is_finite() && *v > 0.0)
    }
    pub fn voice_duration(&self) -> Option<f32> {
        self.voice
            .as_ref()
            .map(Music::get_time_length)
            .filter(|v| v.is_finite() && *v > 0.0)
    }
    pub fn voice_clock(&self) -> VoiceClock {
        let Some(started) = self.voice_started else {
            return VoiceClock::default();
        };
        let Some(v) = &self.voice else {
            return VoiceClock::default();
        };
        // Device loss must not strand a match in dialogue forever.
        if started.elapsed().as_secs_f32() > v.get_time_length() + 2.0 {
            return VoiceClock {
                position: None,
                finished: false,
            };
        }
        VoiceClock {
            position: Some(v.get_time_played()),
            finished: !v.is_stream_playing() && started.elapsed().as_secs_f32() > 0.1,
        }
    }
    pub fn footsteps(&mut self, speed: f32) {
        if let Some(s) = self.sounds.get("footsteps") {
            if speed < 0.1 {
                s.stop();
            } else {
                s.set_pitch(if speed > 5.0 { 1.25 } else { 1.0 });
                if !s.is_playing() {
                    s.play();
                    self.count("footsteps");
                }
            }
        }
    }
    pub fn stop_effects(&self) {
        for sound in self.sounds.values() {
            sound.stop();
        }
    }
    /// Preserve the dialogue stream and its fallback clock across a pause.
    /// Music is paused/resumed by set_music; gameplay effects stop immediately.
    pub fn set_paused(&mut self, paused: bool) {
        if paused && self.paused_at.is_none() {
            self.paused_at = Some(Instant::now());
            self.voice_paused = self.voice.as_ref().is_some_and(|v| v.is_stream_playing());
            if self.voice_paused {
                if let Some(v) = &self.voice { v.pause_stream(); }
            }
            self.stop_effects();
        } else if !paused {
            if let Some(at) = self.paused_at.take() {
                if let Some(started) = &mut self.voice_started { *started += at.elapsed(); }
                if self.voice_paused {
                    if let Some(v) = &self.voice { v.resume_stream(); }
                }
                self.voice_paused = false;
            }
        }
    }
    pub fn silence_voice(&mut self) {
        if let Some(v) = &self.voice {
            v.stop_stream();
        }
        self.voice_started = None;
        self.voice_paused = false;
    }
}
