//! Secret completion shortcuts for minigames and the solar adventure.
use super::host::{MinigameExit, MinigameResult, ResultObserver};
use raylib::prelude::*;

#[derive(Default)]
pub struct PresentationShortcut {
    next: usize,
    last_letter: f64,
}

impl PresentationShortcut {
    /// Hold either Ctrl and type C, N, P, with five seconds between letters.
    pub fn update(&mut self, control: bool, letter: Option<char>, now: f64) -> bool {
        if !control || now - self.last_letter > 5.0 {
            self.next = 0;
        }
        if !control {
            return false;
        }
        let Some(letter) = letter else { return false };
        self.last_letter = now;
        if letter == ['C', 'N', 'P'][self.next] {
            self.next += 1;
        } else {
            self.next = usize::from(letter == 'C');
        }
        if self.next == 3 {
            self.next = 0;
            return true;
        }
        false
    }

    pub fn poll(&mut self, rl: &RaylibHandle) -> bool {
        let control = rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL);
        let mut letters = [
            (KeyboardKey::KEY_C, 'C'),
            (KeyboardKey::KEY_N, 'N'),
            (KeyboardKey::KEY_P, 'P'),
        ]
        .into_iter()
        .filter_map(|(key, letter)| rl.is_key_pressed(key).then_some(letter));
        let letter = letters.next();
        if letters.next().is_some() {
            self.next = 0;
            return false;
        }
        self.update(control, letter, rl.get_time())
    }
}

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
