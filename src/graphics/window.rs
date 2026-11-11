//! Shared window policy for the menu, flight, hangar and every minigame.
use raylib::{
    core::{window as monitor, RaylibBuilder},
    ffi,
    prelude::*,
};

/// Open interactive scenes in borderless fullscreen at the monitor's resolution.
/// Diagnostic runs retain their requested size so resize checks and captures
/// can run in hidden windows without changing the desktop's display mode.
pub fn build(builder: &mut RaylibBuilder, diagnostic: bool) -> (RaylibHandle, RaylibThread) {
    if !diagnostic {
        // Start without borders at monitor size, then enable borderless mode.
        // This avoids a small window and exclusive video-mode changes each
        // time a scene replaces its graphics context.
        builder.size(0, 0).undecorated();
    }
    // RaylibBuilder does not expose this startup flag. Keep the window hidden
    // until its final size and position are ready, including during loading.
    unsafe { ffi::SetConfigFlags(ffi::ConfigFlags::FLAG_WINDOW_HIDDEN as u32) };
    let (mut rl, thread) = builder.build();
    if diagnostic {
        let size = (rl.get_screen_width(), rl.get_screen_height());
        unsafe { ffi::ClearWindowState(ffi::ConfigFlags::FLAG_BORDERLESS_WINDOWED_MODE as u32) };
        rl.clear_window_state(
            WindowState::default()
                .set_window_undecorated(true)
                .set_window_topmost(true),
        );
        rl.set_window_size(size.0, size.1);
        rl.set_window_state(WindowState::default().set_window_hidden(true));
    } else {
        // raylib keeps its flags after CloseWindow. Set the mode idempotently:
        // toggling would turn fullscreen off on every second scene.
        unsafe { ffi::SetWindowState(ffi::ConfigFlags::FLAG_BORDERLESS_WINDOWED_MODE as u32) };
        let index = monitor::get_current_monitor();
        let position = monitor::get_monitor_position(index);
        rl.set_window_size(
            monitor::get_monitor_width(index),
            monitor::get_monitor_height(index),
        );
        rl.set_window_position(position.x as i32, position.y as i32);
        rl.clear_window_state(WindowState::default().set_window_hidden(true));
    }
    (rl, thread)
}
