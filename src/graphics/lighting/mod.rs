use raylib::prelude::*;

pub fn winter_shader(rl: &mut RaylibHandle, thread: &RaylibThread) -> Shader {
    rl.load_shader_from_memory(
        thread,
        Some(include_str!("../shaders/winter.vs")),
        Some(include_str!("../shaders/winter.fs")),
    )
}
