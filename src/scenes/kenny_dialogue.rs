use super::button_style;
use crate::minigames::southpark::KennyDialogue;
use raylib::prelude::*;

#[derive(PartialEq, Eq)]
pub enum DialogueAction {
    None,
    Accept,
    Close,
}
fn buttons(width: i32, height: i32) -> [Rectangle; 2] {
    [
        Rectangle::new(
            width as f32 / 2.0 - 220.0,
            height as f32 / 2.0 + 35.0,
            200.0,
            44.0,
        ),
        Rectangle::new(
            width as f32 / 2.0 + 20.0,
            height as f32 / 2.0 + 35.0,
            200.0,
            44.0,
        ),
    ]
}
fn contains(rect: Rectangle, p: Vector2) -> bool {
    p.x >= rect.x && p.x <= rect.x + rect.width && p.y >= rect.y && p.y <= rect.y + rect.height
}
pub fn input(rl: &RaylibHandle, state: KennyDialogue) -> DialogueAction {
    if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
        return DialogueAction::Close;
    }
    let accept = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
    let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let [left, right] = buttons(rl.get_screen_width(), rl.get_screen_height());
    if state == KennyDialogue::Invitation {
        if accept || (clicked && contains(left, rl.get_mouse_position())) {
            return DialogueAction::Accept;
        }
        if clicked && contains(right, rl.get_mouse_position()) {
            return DialogueAction::Close;
        }
    } else if accept
        || rl.is_key_pressed(KeyboardKey::KEY_E)
        || (clicked && contains(left, rl.get_mouse_position()))
    {
        return DialogueAction::Close;
    }
    DialogueAction::None
}
pub fn draw(d: &mut RaylibDrawHandle, state: KennyDialogue, width: i32, height: i32) {
    d.draw_rectangle(0, 0, width, height, Color::new(10, 20, 30, 170));
    d.draw_rectangle(
        width / 2 - 310,
        height / 2 - 125,
        620,
        240,
        Color::new(245, 248, 250, 250),
    );
    let title = "SAVE KENNY!";
    d.draw_text(
        title,
        (width - d.measure_text(title, 32)) / 2,
        height / 2 - 100,
        32,
        Color::new(185, 75, 15, 255),
    );
    let message = if state == KennyDialogue::Invitation {
        "¿Podrás evitar que maten a Kenny?"
    } else {
        "Desafío aceptado. Preparación lista."
    };
    d.draw_text(
        message,
        (width - d.measure_text(message, 24)) / 2,
        height / 2 - 40,
        24,
        Color::DARKGRAY,
    );
    if state == KennyDialogue::Preparing {
        let text = "Las amenazas todavía no están activadas.";
        d.draw_text(
            text,
            (width - d.measure_text(text, 18)) / 2,
            height / 2 - 7,
            18,
            Color::GRAY,
        );
    }
    for (i, rect) in buttons(width, height).into_iter().enumerate() {
        if i == 1 && state == KennyDialogue::Preparing {
            continue;
        }
        let text = if state == KennyDialogue::Preparing {
            "Volver a explorar"
        } else if i == 0 {
            "Comenzar desafío"
        } else {
            "Cancelar"
        };
        button_style::surface(d, rect, false, true);
        button_style::centered_label(d, rect, text, 20, Color::new(8, 24, 55, 255));
    }
    let keys = "Enter: aceptar / continuar    Esc: cerrar";
    d.draw_text(
        keys,
        (width - d.measure_text(keys, 16)) / 2,
        height / 2 + 90,
        16,
        Color::GRAY,
    );
}
