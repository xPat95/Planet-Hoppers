//! Presentation/input only. Match rules stay in SaveKennySession.
use super::button_style;
use crate::minigames::southpark::{
    session::{SaveKennySession, SessionState},
    threats::{ThreatDirector, ThreatKind},
};
use raylib::prelude::*;
use std::path::Path;
use crate::minigames::common::host::{MinigameResult, ReturnConfirmation};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SessionAction {
    None,
    Continue,
    Replay,
    Explore,
    Pause,
    Return(MinigameResult),
    AskReturn,
    CancelReturn,
    ConfirmReturn,
}
pub fn threat_name(kind: ThreatKind) -> &'static str {
    match kind {
        ThreatKind::Bus => "¡Autobús fuera de control!",
        ThreatKind::Ufo => "¡Abducción alienígena!",
        ThreatKind::Sewer => "¡Alcantarilla mortal!",
        ThreatKind::Piano => "¡El piano del cielo!",
        ThreatKind::Snowman => "¡Muñeco de nieve asesino!",
    }
}
fn buttons(w: i32, h: i32) -> [Rectangle; 2] {
    let width = (w as f32 - 64.0).min(560.0);
    let left = (w as f32 - width) * 0.5;
    [
        Rectangle::new(left, h as f32 * 0.5 + 60.0, (width - 16.0) * 0.5, 44.0),
        Rectangle::new(
            left + (width + 16.0) * 0.5,
            h as f32 * 0.5 + 60.0,
            (width - 16.0) * 0.5,
            44.0,
        ),
    ]
}
pub fn pause_buttons(w: i32, h: i32) -> [Rectangle; 3] {
    let width = (w as f32 - 64.0).min(560.0);
    std::array::from_fn(|i| Rectangle::new((w as f32 - width) * 0.5, h as f32 * 0.5 + 20.0 + i as f32 * 50.0, width, 42.0))
}

fn choice_button(d: &mut RaylibDrawHandle, rect: Rectangle, label: &str, size: i32, enabled: bool) {
    button_style::surface(d, rect, false, enabled);
    button_style::centered_label(
        d,
        rect,
        label,
        size,
        if enabled {
            Color::new(8, 24, 55, 255)
        } else {
            Color::new(72, 86, 108, 255)
        },
    );
}

pub fn input(rl: &RaylibHandle, session: &SaveKennySession, confirmation: &ReturnConfirmation) -> SessionAction {
    use SessionAction::*;
    use SessionState::*;
    let clicked = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
    let p = rl.get_mouse_position();
    let hit = |r: Rectangle| clicked && r.check_collision_point_rec(p);
    let escape = rl.is_key_pressed(KeyboardKey::KEY_ESCAPE);
    let enter = rl.is_key_pressed(KeyboardKey::KEY_ENTER);
    if session.state == Paused {
        if confirmation.pending {
            let [yes, no] = buttons(rl.get_screen_width(), rl.get_screen_height());
            if escape || hit(no) { return CancelReturn; }
            if enter || hit(yes) { return ConfirmReturn; }
        } else {
            let [resume, restart, leave] = pause_buttons(rl.get_screen_width(), rl.get_screen_height());
            if escape || enter || rl.is_key_pressed(KeyboardKey::KEY_P) || hit(resume) { return Continue; }
            if hit(restart) { return Replay; }
            if hit(leave) { return AskReturn; }
        }
        return None;
    }
    if !matches!(session.state, Victory | Defeat | MapOpen)
        && (escape || rl.is_key_pressed(KeyboardKey::KEY_P)) {
        return Pause;
    }
    if session.dynamic()
        && matches!(
            session.state,
            Success | Searching | Interlude | Relocating | MapOpen
        )
    {
        return None;
    }
    if !matches!(
        session.state,
        Briefing | Success | Victory | Defeat | Paused
    ) {
        return None;
    }
    let [a, b] = buttons(rl.get_screen_width(), rl.get_screen_height());
    let contains =
        |r: Rectangle| p.x >= r.x && p.x <= r.x + r.width && p.y >= r.y && p.y <= r.y + r.height;
    if clicked && contains(b) {
        return match session.state {
            Victory => Return(MinigameResult::Victory),
            Defeat => Return(MinigameResult::Defeat),
            _ => Explore,
        };
    }
    if rl.is_key_pressed(KeyboardKey::KEY_ENTER)
        || (clicked && contains(a))
        || (session.state == Paused && rl.is_key_pressed(KeyboardKey::KEY_ESCAPE))
    {
        if matches!(session.state, Victory | Defeat) {
            return Replay;
        }
        return Continue;
    }
    None
}

pub struct SessionUi {
    stan: Option<Texture2D>,
    kyle: Option<Texture2D>,
    pub loaded_portraits: usize,
}
impl SessionUi {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Self {
        let mut load = |name: &str| {
            let path = root.join("assets/textures/minigames/South Park").join(name);
            match rl.load_texture(thread, &path.to_string_lossy()) {
                Ok(t) => {
                    println!("PORTRAIT_LOADED {name} {}x{}", t.width, t.height);
                    Some(t)
                }
                Err(e) => {
                    eprintln!("PORTRAIT_WARNING {name}: {e}");
                    None
                }
            }
        };
        let stan = load("StanMarsh.png");
        let kyle = load("KyleBrofloski.png");
        let loaded_portraits = stan.is_some() as usize + kyle.is_some() as usize;
        Self {
            stan,
            kyle,
            loaded_portraits,
        }
    }
    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        s: &SaveKennySession,
        event: &ThreatDirector,
        w: i32,
        h: i32,
        confirmation: &ReturnConfirmation,
    ) {
        use SessionState::*;
        if let Some(a) = &s.adventure {
            if s.state == DeathDialogue { self.death_dialogue(d,s,w,h); return; }
            if s.state == Exploring { return; }
            let tr = |en, es| a.text(en,es);
            if s.state == Paused {
                d.draw_rectangle(0,0,w,h,Color::new(8,16,26,220));
                wrapped(d,tr("SAVE KENNY - Paused","SAVE KENNY - Pausa"),32,h/2-130,w-64,26,Color::YELLOW);
                wrapped(d,if confirmation.pending { tr("Return to the solar system? This attempt's progress will be lost. No rewards will be granted.","¿Regresar al sistema solar? Se perderá el progreso de este intento. No recibirás recompensas.") } else { tr("All danger is paused. L: English / Español","El peligro está detenido. L: Español / English") },32,h/2-80,w-64,18,Color::WHITE);
                if confirmation.pending {
                    for (r,t) in buttons(w,h).into_iter().zip([tr("Confirm return","Confirmar regreso"),tr("Cancel","Cancelar")]) { choice_button(d,r,t,18,true); }
                } else {
                    for (r,t) in pause_buttons(w,h).into_iter().zip([tr("Resume","Continuar"),tr("Restart minigame","Reiniciar minijuego"),tr("Return to solar system","Regresar al sistema solar")]) { choice_button(d,r,t,18,true); }
                }
                return;
            }
            if s.state == MapOpen { return; }
            d.draw_rectangle(16,44,w-32,114,Color::new(15,30,46,238));
            d.draw_text(&format!("{}: {}   {}: {}/5",tr("Lives","Vidas"),s.lives,tr("Completed","Completadas"),s.saves),28,54,20,Color::WHITE);
            let instruction = match s.current() {
                ThreatKind::Ufo => tr("UFO: use E to switch off all three lights.","OVNI: usa E para apagar las tres luces."),
                ThreatKind::Snowman => tr("Heaters: use E to turn on all three heaters.","Calefactores: usa E para encender los tres."),
                ThreatKind::Bus => tr("Bus: use E on the red panel to brake.","Autobús: usa E en el panel rojo para frenar."),
                ThreatKind::Piano => tr("Piano: use E on the yellow safety lock.","Piano: usa E en el seguro amarillo."),
                ThreatKind::Sewer => tr("Manhole: use E to collect the lid, then cover the hole.","Alcantarilla: usa E para recoger la tapa y cubrir el agujero."),
            };
            let text = match s.state {
                Searching => tr("Follow the danger marker. Take your time; approach Kenny when ready.","Sigue el marcador de peligro. Acércate a Kenny cuando estés listo, sin prisa."),
                Success => tr("Kenny is safe! Find another group to continue.","¡Kenny está a salvo! Busca otro grupo para continuar."),
                DeathAnimation | Restoring => tr("Kenny died! The unfinished mission will remain available.","¡Kenny murió! La misión pendiente seguirá disponible."),
                _ => instruction,
            };
            wrapped(d,text,28,86,w-56,18,Color::YELLOW);
            if matches!(s.state, Preparing | Active) {
                let progress = match s.current() {
                    ThreatKind::Ufo => format!("{}/3",event.lights.iter().filter(|x| !**x).count()),
                    ThreatKind::Snowman => format!("{}/3",event.heaters.iter().filter(|x| **x).count()),
                    _ => String::new(),
                };
                d.draw_text(&progress,28,132,18,Color::WHITE);
            }
            if matches!(s.state,Victory|Defeat) {
                d.draw_rectangle(16,h/2-90,w-32,220,Color::new(19,40,65,252));
                wrapped(d,if s.state==Victory { tr("KENNY SURVIVED! All five missions completed.","¡KENNY SOBREVIVIÓ! Completaste las cinco misiones.") } else { tr("OH MY GOD! THEY KILLED KENNY! Game over.","¡DIOS MÍO! ¡MATARON A KENNY! Fin de la partida.") },32,h/2-65,w-64,24,Color::YELLOW);
                for (r,t) in buttons(w,h).into_iter().zip([tr("Play again","Volver a jugar"),tr("Return to solar system","Regresar al sistema solar")]) { choice_button(d,r,t,16,true); }
            }
            if let Some(error) = &s.error { wrapped(d,error,28,170,w-56,16,Color::RED); }
            return;
        }
        if s.state == Paused {
            draw_pause(d, w, h, confirmation);
            if let Some(error) = &s.error { wrapped(d, error, 32, h - 60, w - 64, 16, Color::ORANGE); }
            return;
        }
        if s.state == Relocating {
            d.draw_rectangle(0, 0, w, h, Color::new(15, 30, 46, 255));
            d.draw_text(
                "Un momento en South Park...",
                (w - 380) / 2,
                h / 2 - 10,
                24,
                Color::WHITE,
            );
            if let Some(error) = &s.error {
                wrapped(d, error, 24, h / 2 + 40, w - 48, 18, Color::ORANGE);
            }
            return;
        }
        if s.state == Exploring {
            return;
        }
        d.draw_rectangle(16, 44, w - 32, 120, Color::new(15, 30, 46, 238));
        d.draw_rectangle_lines_ex(
            Rectangle::new(16.0, 44.0, (w - 32) as f32, 120.0),
            2.0,
            Color::new(210, 225, 230, 255),
        );
        d.draw_text("VIDAS", 28, 52, 14, Color::LIGHTGRAY);
        for i in 0..3 {
            heart(
                d,
                30.0 + i as f32 * 35.0,
                74.0,
                if i < s.lives {
                    Color::RED
                } else {
                    Color::DARKGRAY
                },
            );
        }
        d.draw_text(
            &format!("Kenny salvado: {}/5", s.saves),
            154,
            73,
            22,
            Color::WHITE,
        );
        d.draw_text(threat_name(s.current()), 28, 109, 20, Color::YELLOW);
        let detail = if s.state == Searching {
            format!(
                "{}: {:.0} m | Peligro en {:.0} s",
                if s.objective_distance <= s.config.zone_detection_distance {
                    "Zona detectada"
                } else {
                    "Localizalo"
                },
                s.objective_distance,
                s.search_remaining()
            )
        } else if s.state == Interlude {
            "Explorá el pueblo. La próxima alerta llegará automáticamente.".into()
        } else if s.state == Success && s.dynamic() {
            "¡Kenny está a salvo! La partida continúa automáticamente.".into()
        } else if s.state == Preparing {
            format!("En {:.1} s: {}", event.remaining(), s.current().label())
        } else if s.state == Active {
            match s.current() {
                ThreatKind::Ufo => format!(
                    "Luces desactivadas: {}/3",
                    event.lights.iter().filter(|on| !**on).count()
                ),
                ThreatKind::Sewer => {
                    if event.carrying_lid {
                        "Tapa recogida: cubrí la alcantarilla".into()
                    } else {
                        "Buscá la tapa metálica junto a la calle".into()
                    }
                }
                ThreatKind::Snowman => format!(
                    "Calefactores: {}/3",
                    event.heaters.iter().filter(|on| **on).count()
                ),
                ThreatKind::Bus => {
                    format!("Panel rojo: activar freno | {:.1} s", event.remaining())
                }
                ThreatKind::Piano => {
                    if event.progress > 0.75 {
                        "¡El cable está por romperse! Bloqueá el piano.".into()
                    } else {
                        "Seguro amarillo junto a la acera | E para bloquear".into()
                    }
                }
            }
        } else {
            String::new()
        };
        let detail_size = if d.measure_text(&detail, 16) > w - 56 {
            12
        } else {
            16
        };
        d.draw_text(&detail, 28, 138, detail_size, Color::LIGHTGRAY);
        if s.state == Searching {
            d.draw_text("¡KENNY ESTÁ EN PELIGRO!", 28, 178, 24, Color::RED);
            if let Some(site) = &s.site {
                d.draw_text(
                    &format!("{} | Llega antes de que sea demasiado tarde", site.road),
                    28,
                    208,
                    18,
                    Color::DARKBLUE,
                );
            }
        }
        if s.state == DeathDialogue {
            self.death_dialogue(d, s, w, h);
            return;
        }
        if matches!(s.state, DeathAnimation | Restoring) {
            return;
        }
        if !s.blocks_controls() {
            d.draw_text("M: mapa | Esc/P: pausa", w - 230, 52, 14, Color::LIGHTGRAY);
            return;
        }
        let (title,body,left,right)=match s.state {
            Briefing=>(threat_name(s.current()),"Nos reunimos al inicio de North Avenue.\nConfirmá cuando estés listo: después tendrás unos segundos de preparación.","Comenzar amenaza","Volver a explorar"),
            Success=>("¡Kenny está a salvo!","Amenaza superada. La siguiente no comenzará hasta que estés listo.","Siguiente amenaza","Volver a explorar"),
            Victory=>("¡KENNY SOBREVIVIÓ!","Lograste salvar a Kenny de las cinco amenazas.","Jugar nuevamente","Regresar al sistema solar"),
            Defeat=>("OH MY GOD! THEY KILLED KENNY!","GAME OVER","Reintentar","Regresar al sistema solar"),
            Paused=>("SAVE KENNY! — Pausa","El peligro está detenido. Podés continuar o abandonar la partida.","Continuar","Abandonar"),
            _=>return,
        };
        d.draw_rectangle(0, 168, w, (h - 168).max(0), Color::new(8, 16, 26, 110));
        let width = (w - 32).min(640);
        let x = (w - width) / 2;
        let y = h / 2 - 100;
        d.draw_rectangle(x, y, width, 242, Color::new(19, 40, 65, 252));
        d.draw_rectangle_lines_ex(
            Rectangle::new(x as f32, y as f32, width as f32, 242.0),
            3.0,
            Color::new(230, 220, 180, 255),
        );
        let title_size = if d.measure_text(title, 28) > width - 32 {
            20
        } else {
            28
        };
        d.draw_text(
            title,
            x + (width - d.measure_text(title, title_size)) / 2,
            y + 20,
            title_size,
            Color::new(255, 201, 92, 255),
        );
        wrapped(d, body, x + 24, y + 66, width - 48, 20, Color::WHITE);
        if matches!(s.state, Victory | Defeat) {
            d.draw_text(
                &format!("Salvadas: {}/5   Vidas restantes: {}", s.saves, s.lives),
                x + 24,
                y + 120,
                20,
                Color::YELLOW,
            );
        }
        if let Some(error) = &s.error {
            wrapped(d, error, x + 24, y + 110, width - 48, 16, Color::ORANGE);
        }
        for (i, rect) in buttons(w, h).into_iter().enumerate() {
            let disabled =
                i == 0 && s.state == Success && s.elapsed < s.config.success_hold_seconds;
            let text = if i == 0 { left } else { right };
            let size = if rect.width < 230.0 { 16 } else { 18 };
            choice_button(d, rect, text, size, !disabled);
        }
    }
    fn death_dialogue(&self, d: &mut RaylibDrawHandle, s: &SaveKennySession, w: i32, h: i32) {
        let height = (h as f32 * 0.27).clamp(150.0, 220.0) as i32;
        let y = h - height - 16;
        let portrait_width = (height as f32 * 0.7) as i32;
        d.draw_rectangle(16, y, w - 32, height, Color::new(14, 35, 65, 248));
        d.draw_rectangle_lines_ex(
            Rectangle::new(16.0, y as f32, (w - 32) as f32, height as f32),
            3.0,
            Color::new(235, 225, 184, 255),
        );
        let (name, text, count) = s.dialogue();
        let text = if let Some(a) = &s.adventure {
            if name == "Stan" { a.text(text,"¡Dios mío! ¡Mataron a Kenny!") } else { a.text(text,"¡Malditos bastardos!") }
        } else { text };
        let portrait = if name == "Stan" {
            &self.stan
        } else {
            &self.kyle
        };
        let rect = Rectangle::new(
            30.0,
            (y + 12) as f32,
            portrait_width as f32,
            (height - 24) as f32,
        );
        d.draw_rectangle_rec(rect, Color::new(243, 242, 235, 255));
        if let Some(t) = portrait {
            // Crop only the draw rectangle to emphasize the face; PNGs stay intact.
            let source = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32 * 0.74);
            let ratio = (rect.width / source.width).min(rect.height / source.height);
            let dest = Rectangle::new(
                rect.x + (rect.width - source.width * ratio) * 0.5,
                rect.y + (rect.height - source.height * ratio) * 0.5,
                source.width * ratio,
                source.height * ratio,
            );
            d.draw_texture_pro(t, source, dest, Vector2::zero(), 0.0, Color::WHITE);
        }
        let x = 46 + portrait_width;
        d.draw_text(name, x, y + 20, 26, Color::new(255, 201, 92, 255));
        let revealed: String = text.chars().take(count).collect();
        wrapped(d, &revealed, x, y + 64, w - x - 38, 26, Color::WHITE);
    }
}
fn draw_pause(d: &mut RaylibDrawHandle, w: i32, h: i32, confirmation: &ReturnConfirmation) {
    d.draw_rectangle(0, 0, w, h, Color::new(8, 16, 26, 210));
    let width = (w - 32).min(640);
    let x = (w - width) / 2;
    let y = h / 2 - 140;
    d.draw_rectangle(x, y, width, 330, Color::new(19, 40, 65, 255));
    let title = if confirmation.pending { "¿Regresar al sistema solar?" } else { "SAVE KENNY! — Pausa" };
    wrapped(d, title, x + 24, y + 22, width - 48, 24, Color::YELLOW);
    let body = if confirmation.pending { "Se descartará el progreso de este intento.\nNo se otorgarán recompensas." } else { "La partida y todos sus temporizadores están detenidos." };
    wrapped(d, body, x + 24, y + 70, width - 48, 18, Color::WHITE);
    let choices: Vec<_> = if confirmation.pending {
        buttons(w, h).into_iter().zip(["Confirmar regreso", "Cancelar"]).collect()
    } else {
        pause_buttons(w, h).into_iter().zip(["Continuar partida", "Reiniciar minijuego", "Regresar al sistema solar"]).collect()
    };
    for (rect, label) in choices {
        choice_button(d, rect, label, 18, true);
    }
}

fn heart(d: &mut RaylibDrawHandle, x: f32, y: f32, color: Color) {
    d.draw_circle_v(Vector2::new(x + 7.5, y + 7.5), 7.5, color);
    d.draw_circle_v(Vector2::new(x + 21.5, y + 7.5), 7.5, color);
    d.draw_triangle(
        Vector2::new(x, y + 9.0),
        Vector2::new(x + 14.5, y + 28.0),
        Vector2::new(x + 29.0, y + 9.0),
        color,
    );
}
pub(super) fn wrapped(
    d: &mut RaylibDrawHandle,
    text: &str,
    x: i32,
    mut y: i32,
    width: i32,
    size: i32,
    color: Color,
) {
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let next = if line.is_empty() {
                word.into()
            } else {
                format!("{line} {word}")
            };
            if d.measure_text(&next, size) > width && !line.is_empty() {
                d.draw_text(&line, x, y, size, color);
                y += size + 6;
                line = word.into();
            } else {
                line = next;
            }
        }
        d.draw_text(&line, x, y, size, color);
        y += size + 6;
    }
}
