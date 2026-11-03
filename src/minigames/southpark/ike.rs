//! Optional exploration event. No targets, colliders, score or match rules are changed.
use super::{
    locations::{random_step, WorldLocations},
    navigation::event_route,
    session::SessionState,
    SouthParkScene,
};
use raylib::prelude::Vector3;

pub const SPRITE: &str = "assets/textures/minigames/South Park/IkeBroflovski.png";
pub const VIDEO: &str = "assets/video/south park/kick_the_baby.sfv";
pub const AUDIO: &str = "assets/video/south park/kick_the_baby.ogg";
pub const HEIGHT: f32 = 0.96;

#[derive(Default)]
pub struct IkeEvent {
    pub position: Option<Vector3>,
    pub playing: bool,
    pub available: bool,
    pub spawn_attempted: bool,
}
impl IkeEvent {
    /// Pausing/opening the map cancels playback but preserves this exploration's spawn.
    pub fn sync(&mut self, state: SessionState, intro: bool) {
        self.available = state == SessionState::Exploring && !intro;
        if !self.available {
            self.playing = false;
        }
        if !matches!(
            state,
            SessionState::Exploring | SessionState::Paused | SessionState::MapOpen
        ) {
            self.position = None;
            self.spawn_attempted = false;
        }
    }
    pub fn begin(&mut self) -> bool {
        if !self.available || self.position.is_none() || self.playing {
            return false;
        }
        self.playing = true;
        true
    }
    pub fn finish(&mut self) {
        self.playing = false;
    }
    pub fn visible(&self) -> bool {
        self.available && self.position.is_some() && !self.playing
    }
}

/// Use the existing verified road stations, with clearance for the entire sprite.
/// Ike adds no solid collider, so he cannot block road navigation or Kenny.
pub fn choose_position(
    scene: &SouthParkScene,
    locations: &WorldLocations,
    seed: &mut u64,
) -> Option<Vector3> {
    let choices: Vec<_> = locations
        .exploration_spawns
        .iter()
        .map(|p| Vector3::new(p[0], scene.world.ground_y, p[2]))
        .filter(|p| {
            (*p - scene.kenny.position()).length() > 8.0
                && (*p - scene.player.feet).length() > 4.0
                && scene.world.can_occupy(*p, 0.9, HEIGHT.max(1.8))
                && event_route(
                    &scene.roads,
                    &scene.world,
                    scene.player.feet,
                    *p,
                    scene.player.config.radius,
                    scene.player.config.body_height,
                )
                .is_ok()
        })
        .collect();
    choices
        .get(random_step(seed) as usize % choices.len().max(1))
        .copied()
}
