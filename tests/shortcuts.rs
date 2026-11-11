use planet_hoppers::{
    gameplay::{
        progress::Franchise,
        rewards::record_result,
        selection::{GameState, ShipCatalog},
    },
    graphics::window,
    minigames::common::{
        host::{MinigameExit, MinigameResult},
        shortcuts::RewardShortcut,
    },
    ships::ShipsConfig,
};
use raylib::{ffi, prelude::*};
use std::{path::Path, process::Command};

#[test]
fn secret_requires_control_and_all_four_digits_in_order() {
    let mut code = RewardShortcut::default();
    for digit in 1..=4 {
        assert!(!code.update(false, Some(digit), digit as f64));
    }
    for digit in 1..=4 {
        assert_eq!(code.update(true, Some(digit), digit as f64), digit == 4);
    }
    assert!(!code.update(true, None, 4.1));
    assert!(!code.update(true, Some(4), 4.2));
}

#[test]
fn release_wrong_digits_and_timeout_cancel_partial_codes() {
    for interruption in [(false, None, 1.1), (true, Some(8), 1.1), (true, None, 7.0)] {
        let mut code = RewardShortcut::default();
        assert!(!code.update(true, Some(1), 1.0));
        assert!(!code.update(interruption.0, interruption.1, interruption.2));
        for digit in 2..=4 {
            assert!(!code.update(true, Some(digit), interruption.2 + digit as f64 * 0.1));
        }
    }
}

#[test]
fn code_can_restart_after_a_typo_and_does_not_timeout_during_normal_frames() {
    let mut code = RewardShortcut::default();
    for (digit, now) in [(1, 1.0), (3, 2.0), (1, 3.0), (2, 4.0), (3, 5.0)] {
        assert!(!code.update(true, Some(digit), now));
    }
    assert!(!code.update(true, None, 6.0));
    assert!(code.update(true, Some(4), 7.0));
}

fn key(key: KeyboardKey, down: bool) {
    unsafe {
        ffi::PlayAutomationEvent(ffi::AutomationEvent {
            frame: 0,
            type_: if down { 2 } else { 1 },
            params: [key as i32, 0, 0, 0],
        });
    }
}

#[test]
#[ignore = "requires OpenGL; run with --ignored --test-threads=1"]
fn native_secret_code_awards_each_planet_once_and_keeps_the_active_ship() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ships = ShipsConfig::load(&root.join("config/ships.json")).unwrap();
    let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships).unwrap();
    let mut game = GameState::new_game("minecraft", &catalog).unwrap();
    let (mut rl, thread) = window::build(raylib::init().size(640, 480).title("Shortcut QA"), true);
    rl.set_exit_key(None);
    for franchise in Franchise::ALL {
        for (control, digits) in [
            (
                KeyboardKey::KEY_LEFT_CONTROL,
                [
                    KeyboardKey::KEY_ONE,
                    KeyboardKey::KEY_TWO,
                    KeyboardKey::KEY_THREE,
                    KeyboardKey::KEY_FOUR,
                ],
            ),
            (
                KeyboardKey::KEY_RIGHT_CONTROL,
                [
                    KeyboardKey::KEY_KP_1,
                    KeyboardKey::KEY_KP_2,
                    KeyboardKey::KEY_KP_3,
                    KeyboardKey::KEY_KP_4,
                ],
            ),
        ] {
            let before = game.progress().token_count();
            let mut code = RewardShortcut::default();
            let mut reports = 0;
            key(control, true);
            for (index, digit) in digits.into_iter().enumerate() {
                key(digit, true);
                let result = code
                    .poll(&rl, &mut |result| {
                        reports += 1;
                        record_result(&mut game, franchise.as_str(), result, &catalog)?;
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(
                    result,
                    (index == 3).then_some(MinigameExit::Returned(MinigameResult::Victory))
                );
                key(digit, false);
                window::begin_drawing(&mut rl, &thread).clear_background(Color::BLACK);
            }
            key(control, false);
            assert_eq!(reports, 1);
            assert!(game.progress().is_completed(franchise));
            assert!(game.is_unlocked(franchise.as_str()));
            assert!(game.progress().token_count() <= before + 1);
            assert_eq!(game.active_ship(), "minecraft");
        }
    }
    assert_eq!(game.progress().token_count(), 6);
    assert_eq!(game.unlocked_count(), 6);
}

#[test]
#[ignore = "requires OpenGL subprocesses; run with --ignored --test-threads=1"]
fn ctrl_x_exits_the_process_from_loops_and_drawing_with_either_control_key() {
    for path in ["loop", "drawing"] {
        for control in ["left", "right", "none", "control_only"] {
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--ignored", "--exact", "exit_shortcut_child", "--nocapture"])
                .env("PLANET_HOPPERS_SHORTCUT_TEST", format!("{path}:{control}"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{path}:{control}: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            let should_exit = matches!(control, "left" | "right");
            assert_eq!(
                String::from_utf8_lossy(&output.stdout).contains("SHORTCUT_SURVIVED"),
                !should_exit
            );
        }
    }
}

#[test]
#[ignore = "subprocess entry point for the native Ctrl+X test"]
fn exit_shortcut_child() {
    let Ok(scenario) = std::env::var("PLANET_HOPPERS_SHORTCUT_TEST") else {
        return;
    };
    let (path, control) = scenario.split_once(':').unwrap();
    let (mut rl, thread) = window::build(raylib::init().size(640, 480).title("Exit QA"), true);
    rl.set_exit_key(None);
    if control != "none" {
        key(
            if control == "right" {
                KeyboardKey::KEY_RIGHT_CONTROL
            } else {
                KeyboardKey::KEY_LEFT_CONTROL
            },
            true,
        );
    }
    if control != "control_only" {
        key(KeyboardKey::KEY_X, true);
    }
    if path == "loop" {
        assert!(!window::should_close(&rl));
    } else {
        window::begin_drawing(&mut rl, &thread).clear_background(Color::BLACK);
    }
    assert!(
        !matches!(control, "left" | "right"),
        "Ctrl+X did not terminate the application"
    );
    println!("SHORTCUT_SURVIVED");
}
