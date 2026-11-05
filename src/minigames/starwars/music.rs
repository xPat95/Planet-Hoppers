//! Flight soundtrack, independent of the audio owned by each cinematic.
use super::simulation::Phase;
use raylib::prelude::*;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Playback {
    Stopped,
    Playing,
    Paused,
}

pub struct FlightMusic<'a> {
    track: Music<'a>,
    state: Playback,
}

impl<'a> FlightMusic<'a> {
    pub fn load(audio: &'a RaylibAudio, root: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = root.join("audios/cancion_fondo_em.mp3");
        let mut track = audio.new_music(&path.to_string_lossy())?;
        track.looping = true;
        track.set_volume(0.55);
        println!(
            "FLIGHT_MUSIC_LOADED duration={:.2}",
            track.get_time_length()
        );
        Ok(Self {
            track,
            state: Playback::Stopped,
        })
    }

    pub fn update(&mut self, phase: Phase) {
        let next = match phase {
            Phase::Running | Phase::Explosion => Playback::Playing,
            Phase::Paused if self.state != Playback::Stopped => Playback::Paused,
            _ => Playback::Stopped,
        };
        if next != self.state {
            match next {
                Playback::Playing if self.state == Playback::Paused => self.track.resume_stream(),
                Playback::Playing => self.track.play_stream(),
                Playback::Paused => self.track.pause_stream(),
                Playback::Stopped => self.track.stop_stream(),
            }
            println!("FLIGHT_MUSIC {:?} -> {:?}", self.state, next);
            self.state = next;
        }
        if self.state == Playback::Playing {
            self.track.update_stream();
        }
    }
}

impl Drop for FlightMusic<'_> {
    fn drop(&mut self) {
        self.track.stop_stream();
    }
}

/// Explicit hardware/decoder check: real stream, pause, resume, reset and cinematic isolation.
pub fn verify(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let audio = RaylibAudio::init_audio_device()?;
    let mut music = FlightMusic::load(&audio, root)?;
    let advance = |music: &mut FlightMusic<'_>, phase| {
        for _ in 0..35 {
            music.update(phase);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    };
    music.update(Phase::Intro);
    assert!(!music.track.is_stream_playing());
    advance(&mut music, Phase::Running);
    assert!(music.track.is_stream_playing());
    let before = music.track.get_time_played();
    assert!(before > 0.0);
    music.update(Phase::Paused);
    let paused = music.track.get_time_played();
    advance(&mut music, Phase::Paused);
    assert!(!music.track.is_stream_playing());
    assert!((music.track.get_time_played() - paused).abs() < 0.05);
    advance(&mut music, Phase::Running);
    assert!(music.track.get_time_played() > before);
    music.update(Phase::Intro);
    assert!(!music.track.is_stream_playing());
    music.update(Phase::Running);
    assert!(music.track.get_time_played() < 0.05);
    advance(&mut music, Phase::Running);
    assert!(music.track.get_time_played() > 0.0);
    music.update(Phase::Lost);
    assert!(!music.track.is_stream_playing());
    music.update(Phase::Running);
    music.update(Phase::Outro);
    assert!(!music.track.is_stream_playing());
    println!("MUSIC_OK playback=true pause=true resume=true restart=true cinematics_isolated=true");
    Ok(())
}
