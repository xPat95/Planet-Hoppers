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
// Original short struck-string tones, generated once in RAM. No added music,
// external assets, dependencies, persistent timers or audio streams.
fn piano_note(frequency:f32)->Vec<u8> {
    let rate=22050_u32;let count=(rate as f32*0.45) as u32;let size=count*2;
    let mut bytes=Vec::with_capacity(size as usize+44);
    bytes.extend_from_slice(b"RIFF");bytes.extend_from_slice(&(size+36).to_le_bytes());bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());bytes.extend_from_slice(&1_u16.to_le_bytes());bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());bytes.extend_from_slice(&(rate*2).to_le_bytes());bytes.extend_from_slice(&2_u16.to_le_bytes());bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");bytes.extend_from_slice(&size.to_le_bytes());
    for i in 0..count {
        let t=i as f32/rate as f32;let phase=std::f32::consts::TAU*frequency*t;
        let signal=phase.sin()*(-7.*t).exp()+0.35*(phase*2.001).sin()*(-13.*t).exp()+0.16*(phase*3.004).sin()*(-22.*t).exp();
        let envelope=(1.-(-600.*t).exp())*((0.45-t)/0.05).clamp(0.,1.);
        bytes.extend_from_slice(&((signal*envelope*18000.) as i16).to_le_bytes());
    }
    bytes
}
pub struct AudioController<'a> {
    intro: Option<Music<'a>>,
    exploration: Option<Music<'a>>,
    challenge: Option<Music<'a>>,
    voice: Option<Music<'a>>,
    sounds: BTreeMap<&'static str, Sound<'a>>,
    piano_keys: Vec<Sound<'a>>,
    piano_lit: Option<usize>,
    config: AudioConfig,
    mode: MusicMode,
    background_muted: bool,
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
        let mut piano_keys=vec![];
        if let Some(d)=device {
            // Three bounded voices per pitch allow fast consecutive presses
            // without cutting off the preceding note's release envelope.
            for frequency in [261.63,293.66,329.63,349.23,392.0].into_iter().cycle().take(15) {
                match d.new_wave_from_memory(".wav",&piano_note(frequency)).and_then(|wave|d.new_sound_from_wave(&wave)) {
                    Ok(sound)=>{sound.set_volume(config.sfx_volume*0.85);piano_keys.push(sound);},
                    Err(e)=>{audit.warnings.push(format!("Piano key: {e}"));piano_keys.clear();break;}
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
            piano_keys,
            piano_lit: None,
            config,
            mode: MusicMode::Silent,
            background_muted: false,
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
        let mode=if self.background_muted && matches!(mode,MusicMode::Exploration|MusicMode::Challenge) {MusicMode::Silent} else {mode};
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
    /// Save Kenny's puzzle preference affects music, never functional effects/voices.
    pub fn set_background_muted(&mut self,muted:bool) {
        self.background_muted=muted;
        if muted && matches!(self.mode,MusicMode::Exploration|MusicMode::Challenge) {self.set_music(MusicMode::Silent);}
    }
    /// One restart for a newly opened puzzle, not one for every frame or Start click.
    pub fn restart_challenge_music(&mut self) {
        if let Some(m)=&self.challenge {m.stop_stream();}
        self.music_started[1]=false;
        if self.mode==MusicMode::Challenge {self.mode=MusicMode::Silent;}
        self.count("puzzle_music_start");
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
    /// Temporary multiplier; the configured user volume remains unchanged.
    pub fn duck_exploration(&self, active: bool) {
        if let Some(music) = &self.exploration {
            music.set_volume(self.config.music_volume * if self.background_muted {0.0} else if active { 0.2 } else { 1.0 });
        }
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
        for sound in &self.piano_keys {sound.stop();}
        for sound in self.sounds.values() {
            sound.stop();
        }
    }
    pub fn play_piano_key(&self,index:usize) {
        if self.background_muted || index>=5 {return;}
        let sound=self.piano_keys.iter().skip(index).step_by(5).find(|s|!s.is_playing()).or_else(||self.piano_keys.get(index));
        if let Some(sound)=sound {sound.play();}
    }
    pub fn sync_piano(&mut self,run:Option<&crate::minigames::southpark::puzzles::PuzzleRun>) {
        use crate::minigames::southpark::puzzles::Board;
        // Let the final pressed note finish its short envelope; pause, mute and
        // leaving still stop it immediately, and no new notes start on results.
        if run.is_some_and(|r|matches!(r.board,Board::Piano(_)) && r.result.is_some() && r.result_time<0.45 && !r.paused && !r.leaving && !self.background_muted) {return;}
        let piano=run.filter(|r|r.interactive() && !self.background_muted).and_then(|r|if let Board::Piano(p)=&r.board {Some(p)}else{None});
        let Some(p)=piano else {for sound in &self.piano_keys {sound.stop();}self.piano_lit=None;return;};
        let next=p.lit();
        if next!=self.piano_lit {
            if let Some(i)=next {self.play_piano_key(i);}
            self.piano_lit=next;
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
