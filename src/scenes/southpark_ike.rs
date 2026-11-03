use super::southpark_video::VideoTexture;
use crate::minigames::southpark::ike;
use raylib::prelude::*;
use std::{path::Path, time::Instant};

/// Owns only this clip's stream; Drop also handles early scene returns/errors.
pub struct IkePlayback<'a> {
    pub video: VideoTexture,
    audio: Option<Music<'a>>,
    started: Instant,
}
impl<'a> IkePlayback<'a> {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
        device: Option<&'a RaylibAudio>,
        volume: f32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Decode/upload the first real image before hiding Ike or starting audio.
        let video = VideoTexture::load(rl, thread, &root.join(ike::VIDEO))?;
        let audio = if let Some(device) = device {
            let mut music = device.new_music(&root.join(ike::AUDIO).to_string_lossy())?;
            music.looping = false;
            if (music.get_time_length() - video.duration()).abs() > 0.2 {
                return Err("Ike video/audio durations differ; regenerate derived assets".into());
            }
            music.set_volume(volume);
            music.play_stream();
            Some(music)
        } else {
            None
        };
        Ok(Self {
            video,
            audio,
            started: Instant::now(),
        })
    }
    /// Audio is the master clock; wall time is only for no-device/device-loss fallback.
    pub fn update(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let elapsed = self.started.elapsed().as_secs_f32();
        let position = if let Some(audio) = &self.audio {
            audio.update_stream();
            if !audio.is_stream_playing() && elapsed > 0.15 {
                return Ok(true);
            }
            audio.get_time_played()
        } else {
            elapsed
        };
        if position >= self.video.duration() || elapsed > self.video.duration() + 2.0 {
            return Ok(true);
        }
        self.video.update(position)?;
        Ok(false)
    }
}
impl Drop for IkePlayback<'_> {
    fn drop(&mut self) {
        if let Some(audio) = &self.audio {
            audio.stop_stream();
        }
    }
}
