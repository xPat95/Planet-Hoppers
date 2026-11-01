use super::collision::CollisionWorld;
use raylib::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FpsConfig {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub mouse_sensitivity: f32,
    pub pitch_limit_degrees: f32,
    pub radius: f32,
    pub body_height: f32,
    pub fov_degrees: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PlayerInput {
    pub forward: f32,
    pub right: f32,
    pub mouse: Vector2,
    pub sprint: bool,
    pub interact: bool,
    pub toggle_cursor: bool,
}

impl PlayerInput {
    pub fn read(rl: &RaylibHandle) -> Self {
        let down = |key| if rl.is_key_down(key) { 1.0 } else { 0.0 };
        Self {
            forward: down(KeyboardKey::KEY_W) - down(KeyboardKey::KEY_S),
            right: down(KeyboardKey::KEY_D) - down(KeyboardKey::KEY_A),
            mouse: rl.get_mouse_delta(),
            sprint: rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT)
                || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT),
            interact: rl.is_key_pressed(KeyboardKey::KEY_E),
            toggle_cursor: rl.is_key_pressed(KeyboardKey::KEY_ESCAPE),
        }
    }
}

pub struct FpsController {
    pub feet: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub eye_height: f32,
    pub config: FpsConfig,
}

impl FpsController {
    pub fn new(eye: Vector3, forward: Vector3, eye_height: f32, config: FpsConfig) -> Self {
        Self {
            feet: eye - Vector3::up() * eye_height,
            yaw: forward.z.atan2(forward.x),
            pitch: 0.0,
            eye_height,
            config,
        }
    }
    pub fn eye(&self) -> Vector3 {
        self.feet + Vector3::up() * self.eye_height
    }
    pub fn direction(&self) -> Vector3 {
        Vector3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
    }
    pub fn camera(&self) -> Camera3D {
        Camera3D::perspective(
            self.eye(),
            self.eye() + self.direction(),
            Vector3::up(),
            self.config.fov_degrees,
        )
    }
    pub fn update(&mut self, input: PlayerInput, dt: f32, world: &CollisionWorld) {
        // Mouse input is a pixel delta, not a velocity: do not multiply it by dt.
        self.yaw = (self.yaw + input.mouse.x * self.config.mouse_sensitivity)
            .rem_euclid(std::f32::consts::TAU);
        let limit = self.config.pitch_limit_degrees.to_radians();
        self.pitch =
            (self.pitch - input.mouse.y * self.config.mouse_sensitivity).clamp(-limit, limit);
        let forward = Vector3::new(self.yaw.cos(), 0.0, self.yaw.sin());
        let right = Vector3::new(-self.yaw.sin(), 0.0, self.yaw.cos());
        let mut movement = forward * input.forward + right * input.right;
        if movement.dot(movement) > 1.0 {
            movement = movement.normalized();
        }
        let speed = if input.sprint {
            self.config.run_speed
        } else {
            self.config.walk_speed
        };
        let dt = if dt.is_finite() {
            dt.clamp(0.0, 0.1)
        } else {
            0.0
        };
        self.feet = world.move_body(
            self.feet,
            movement * (speed * dt),
            self.config.radius,
            self.config.body_height,
        );
    }
}

/// Cursor state is independent of any particular planet. Ignore the first mouse
/// delta after capture/focus changes so recentering cannot jerk the camera.
pub struct CursorCapture {
    pub captured: bool,
    discard_frames: u8,
}
impl Default for CursorCapture {
    fn default() -> Self {
        Self {
            captured: true,
            discard_frames: 2,
        }
    }
}
impl CursorCapture {
    pub fn apply(&mut self, rl: &mut RaylibHandle) {
        if self.captured {
            rl.disable_cursor();
        } else {
            rl.enable_cursor();
        }
        self.discard_frames = 2;
    }
    pub fn update(
        &mut self,
        rl: &mut RaylibHandle,
        input: &mut PlayerInput,
        focused: bool,
    ) -> bool {
        if input.toggle_cursor || (!focused && self.captured) {
            self.captured = focused && !self.captured;
            self.apply(rl);
        }
        if self.discard_frames > 0 {
            input.mouse = Vector2::zero();
            self.discard_frames -= 1;
        }
        self.captured && focused
    }
}
