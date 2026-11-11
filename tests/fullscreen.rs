use planet_hoppers::graphics::window;
use raylib::{core::window as monitor, ffi, prelude::*};

fn is_borderless_fullscreen() -> bool {
    unsafe { ffi::IsWindowState(ffi::ConfigFlags::FLAG_BORDERLESS_WINDOWED_MODE as u32) }
}

#[test]
#[ignore = "requires a desktop and OpenGL; run with --ignored --test-threads=1"]
fn scene_recreation_keeps_native_fullscreen_and_diagnostics_stay_windowed() {
    // Exercise closing and reopening the actual graphics context, as happens
    // between the menu, flight, planets and return flight. Include a diagnostic
    // window to catch flags leaking between successive raylib initializations.
    for diagnostic in [false, false, true, false] {
        let (mut rl, thread) = window::build(
            raylib::init()
                .size(960, 540)
                .resizable()
                .title("Planet Hoppers | Fullscreen verification"),
            diagnostic,
        );
        rl.set_exit_key(None);
        assert_eq!(is_borderless_fullscreen(), !diagnostic);
        if diagnostic {
            assert!(rl.is_window_hidden());
            assert_eq!((rl.get_screen_width(), rl.get_screen_height()), (960, 540));
            rl.set_window_size(800, 600);
        } else {
            assert!(!rl.is_window_hidden());
            let index = monitor::get_current_monitor();
            assert_eq!(rl.get_screen_width(), monitor::get_monitor_width(index));
            assert_eq!(rl.get_screen_height(), monitor::get_monitor_height(index));
            assert_eq!(
                rl.get_window_position(),
                monitor::get_monitor_position(index)
            );
        }
        // Draw frames too: verify the fullscreen context survives event polling
        // and buffer swaps, including the first loading frame of a new scene.
        for _ in 0..3 {
            let mut draw = rl.begin_drawing(&thread);
            draw.clear_background(Color::BLACK);
        }
        assert_eq!(is_borderless_fullscreen(), !diagnostic);
    }
}
