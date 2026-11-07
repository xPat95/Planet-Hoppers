//! Presentation flow stays separate from the combat simulation.
use super::simulation::{Game, Phase, Stage};
use raylib::prelude::*;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    Transition,
    Celebration,
}
impl Clip {
    pub fn stem(self) -> &'static str {
        match self {
            Self::Transition => "transicion",
            Self::Celebration => "celebracion",
        }
    }
}

pub struct MediaFlow {
    stage: Stage,
    phase: Phase,
}
impl MediaFlow {
    pub fn new(g: &Game) -> Self {
        Self {
            stage: g.stage,
            phase: g.phase,
        }
    }
    pub fn update(&mut self, g: &Game) -> Option<Clip> {
        let clip = if self.stage == Stage::Hammer && g.stage == Stage::LivingRoom {
            Some(Clip::Transition)
        } else if self.phase != Phase::ArcadeClear && g.phase == Phase::ArcadeClear {
            Some(Clip::Celebration)
        } else {
            None
        };
        self.stage = g.stage;
        self.phase = g.phase;
        clip
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Track {
    Main,
    LevelThree,
}

pub fn soundtrack(g: &Game, cinematic: bool) -> Option<Track> {
    if cinematic || matches!(g.phase, Phase::ArcadeClear | Phase::Defeat) {
        None
    } else if g.phase == Phase::Title || g.stage != Stage::LivingRoom {
        Some(Track::Main)
    } else {
        Some(Track::LevelThree)
    }
}

pub struct Soundtrack<'a> {
    main: Music<'a>,
    level_three: Music<'a>,
    current: Option<Track>,
    paused: bool,
}
impl<'a> Soundtrack<'a> {
    pub fn load(audio: &'a RaylibAudio, root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let load = |name: &str| -> Result<Music<'a>, Box<dyn std::error::Error>> {
            let mut track = audio.new_music(&root.join("audio").join(name).to_string_lossy())?;
            track.looping = true;
            track.set_volume(0.55);
            Ok(track)
        };
        Ok(Self {
            main: load("musica_fondo_martillo.mp3")?,
            level_three: load("musica_fondo_n3_martillo.mp3")?,
            current: None,
            paused: false,
        })
    }
    fn track(&self, track: Track) -> &Music<'a> {
        match track {
            Track::Main => &self.main,
            Track::LevelThree => &self.level_three,
        }
    }
    pub fn update(&mut self, g: &Game, cinematic: bool, confirmation: bool) {
        let next = soundtrack(g, cinematic);
        if self.current != next {
            if let Some(old) = self.current {
                self.track(old).stop_stream();
            }
            self.current = next;
            self.paused = false;
            if let Some(new) = next {
                self.track(new).play_stream();
            }
        }
        if let Some(track) = self.current {
            let paused = g.phase == Phase::Paused || confirmation;
            if paused != self.paused {
                if paused {
                    self.track(track).pause_stream();
                } else {
                    self.track(track).resume_stream();
                }
                self.paused = paused;
            }
            if !paused {
                self.track(track).update_stream();
            }
        }
    }
}
impl Drop for Soundtrack<'_> {
    fn drop(&mut self) {
        self.main.stop_stream();
        self.level_three.stop_stream();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "Requires a real audio device; run with --ignored --test-threads=1"]
    fn real_soundtrack_continuity_pause_resume_and_cinematic_isolation() {
        let audio = RaylibAudio::init_audio_device().unwrap();
        let mut music = Soundtrack::load(
            &audio,
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/minigames/regularshow"),
        )
        .unwrap();
        music.main.set_volume(0.0);
        music.level_three.set_volume(0.0);
        let mut g = Game::new();
        let pump = |music: &mut Soundtrack<'_>, g: &Game| {
            for _ in 0..30 {
                music.update(g, false, false);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        };
        pump(&mut music, &g);
        assert!(music.main.get_time_played() > 0.0);
        let menu_time = music.main.get_time_played();
        g.start();
        pump(&mut music, &g);
        assert!(music.main.get_time_played() > menu_time);
        let first_time = music.main.get_time_played();
        g.load_stage(Stage::Hammer);
        pump(&mut music, &g);
        assert!(music.main.get_time_played() > first_time);
        g.toggle_pause();
        music.update(&g, false, false);
        let paused_time = music.main.get_time_played();
        pump(&mut music, &g);
        assert!(!music.main.is_stream_playing());
        assert!((music.main.get_time_played() - paused_time).abs() < 0.05);
        g.toggle_pause();
        pump(&mut music, &g);
        assert!(music.main.get_time_played() > paused_time);
        music.update(&g, true, false);
        assert!(!music.main.is_stream_playing());
        assert!(!music.level_three.is_stream_playing());
        g.load_stage(Stage::LivingRoom);
        pump(&mut music, &g);
        assert!(!music.main.is_stream_playing());
        assert!(music.level_three.get_time_played() > 0.0);
        music.update(&g, false, true);
        assert!(!music.level_three.is_stream_playing());
        pump(&mut music, &g);
        assert!(music.level_three.is_stream_playing());
        g.phase = Phase::ArcadeClear;
        music.update(&g, true, false);
        assert!(!music.level_three.is_stream_playing());
        g.start();
        pump(&mut music, &g);
        assert!(music.main.is_stream_playing());
        assert!(!music.level_three.is_stream_playing());
    }

    #[test]
    fn complete_sequence_and_replay_do_not_repeat_clips_on_retry() {
        let mut g = Game::new();
        let mut flow = MediaFlow::new(&g);
        assert_eq!(soundtrack(&g, false), Some(Track::Main));
        g.start();
        assert_eq!(flow.update(&g), None);
        g.phase = Phase::StageClear;
        g.advance();
        assert_eq!(flow.update(&g), None);
        assert_eq!(soundtrack(&g, false), Some(Track::Main));
        g.phase = Phase::StageClear;
        g.advance();
        assert_eq!(flow.update(&g), Some(Clip::Transition));
        assert_eq!(g.stage_time, 0.0);
        assert_eq!(soundtrack(&g, true), None);
        assert_eq!(flow.update(&g), None);
        assert_eq!(soundtrack(&g, false), Some(Track::LevelThree));
        g.phase = Phase::Defeat;
        assert_eq!(flow.update(&g), None);
        g.retry();
        assert_eq!(flow.update(&g), None);
        g.phase = Phase::ArcadeClear;
        assert_eq!(flow.update(&g), Some(Clip::Celebration));
        assert_eq!(soundtrack(&g, true), None);
        assert_eq!(flow.update(&g), None);
        assert_eq!(soundtrack(&g, false), None);
        g.start();
        assert_eq!(flow.update(&g), None);
        assert_eq!(soundtrack(&g, false), Some(Track::Main));
        g.load_stage(Stage::Hammer);
        flow.update(&g);
        g.phase = Phase::StageClear;
        g.advance();
        assert_eq!(flow.update(&g), Some(Clip::Transition));
    }
}
