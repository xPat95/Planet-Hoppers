//! Secret completion shortcut shared by all six hosted minigames.
use super::host::{MinigameExit, MinigameResult, ResultObserver};
use raylib::prelude::*;

#[derive(Default)]
pub struct RewardShortcut {
    next: u8,
    last_digit: f64,
}

impl RewardShortcut {
    /// Hold either Ctrl and type 1, 2, 3, 4, with at most five seconds per digit.
    /// Releasing Ctrl, entering a wrong digit or waiting too long resets the code.
    pub fn update(&mut self, control: bool, digit: Option<u8>, now: f64) -> bool {
        if !control || now - self.last_digit > 5.0 {
            self.next = 0;
        }
        if !control {
            return false;
        }
        let Some(digit) = digit else { return false };
        self.last_digit = now;
        if digit == self.next + 1 {
            self.next += 1;
        } else {
            self.next = u8::from(digit == 1);
        }
        if self.next == 4 {
            self.next = 0;
            return true;
        }
        false
    }

    /// Report through the normal host observer so token, unlock and save rules
    /// stay identical to a win. Only minigames poll this: flight cannot grant tokens.
    pub fn poll(
        &mut self,
        rl: &RaylibHandle,
        observer: &mut ResultObserver<'_>,
    ) -> Result<Option<MinigameExit>, Box<dyn std::error::Error>> {
        let control = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL);
        let keys = [
            (KeyboardKey::KEY_ZERO, KeyboardKey::KEY_KP_0),
            (KeyboardKey::KEY_ONE, KeyboardKey::KEY_KP_1),
            (KeyboardKey::KEY_TWO, KeyboardKey::KEY_KP_2),
            (KeyboardKey::KEY_THREE, KeyboardKey::KEY_KP_3),
            (KeyboardKey::KEY_FOUR, KeyboardKey::KEY_KP_4),
            (KeyboardKey::KEY_FIVE, KeyboardKey::KEY_KP_5),
            (KeyboardKey::KEY_SIX, KeyboardKey::KEY_KP_6),
            (KeyboardKey::KEY_SEVEN, KeyboardKey::KEY_KP_7),
            (KeyboardKey::KEY_EIGHT, KeyboardKey::KEY_KP_8),
            (KeyboardKey::KEY_NINE, KeyboardKey::KEY_KP_9),
        ];
        let mut digits = keys
            .iter()
            .enumerate()
            .filter_map(|(digit, &(key, keypad))| {
                (rl.is_key_pressed(key) || rl.is_key_pressed(keypad)).then_some(digit as u8)
            });
        let digit = digits.next();
        // Simultaneous numbers are not an ordered sequence.
        if digits.next().is_some() {
            self.next = 0;
            return Ok(None);
        }
        if self.update(control, digit, rl.get_time()) {
            observer(MinigameResult::Victory)?;
            return Ok(Some(MinigameExit::Returned(MinigameResult::Victory)));
        }
        Ok(None)
    }
}
