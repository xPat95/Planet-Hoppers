//! Pixel backgrounds and animation cels share a point-filtered arcade canvas.
mod room;
use super::simulation::*;
use raylib::prelude::*;

const INK: Color = Color::new(24, 16, 27, 255);
const CREAM: Color = Color::new(255, 239, 184, 255);
const GOLD: Color = Color::new(255, 203, 61, 255);
const CYAN: Color = Color::new(112, 245, 227, 255);
const RED: Color = Color::new(246, 77, 74, 255);
fn c(r: u8, g: u8, b: u8) -> Color {
    Color::new(r, g, b, 255)
}
fn rect(d: &mut impl RaylibDraw, x: f32, y: f32, w: f32, h: f32, color: Color) {
    d.draw_rectangle(
        x.round() as i32,
        y.round() as i32,
        w.round() as i32,
        h.round() as i32,
        color,
    );
}
fn block(d: &mut impl RaylibDraw, x: f32, y: f32, w: f32, h: f32, color: Color) {
    rect(d, x - 2.0, y - 2.0, w + 4.0, h + 4.0, INK);
    rect(d, x, y, w, h, color);
}
fn line(d: &mut impl RaylibDraw, a: (f32, f32), b: (f32, f32), w: f32, color: Color) {
    d.draw_line_ex(Vector2::new(a.0, a.1), Vector2::new(b.0, b.1), w, color);
}
fn text(d: &mut impl RaylibDraw, t: &str, x: i32, y: i32, size: i32, color: Color) {
    d.draw_text(t, x + 2, y + 2, size, INK);
    d.draw_text(t, x, y, size, color);
}
fn center(d: &mut impl RaylibDraw, t: &str, y: i32, size: i32, color: Color) {
    let x = (WIDTH - measure(t, size)) / 2;
    text(d, t, x, y, size, color);
}
fn measure(t: &str, size: i32) -> i32 {
    let t = std::ffi::CString::new(t).expect("UI text without NUL");
    unsafe { raylib::ffi::MeasureText(t.as_ptr(), size) }
}

pub struct Art {
    room: room::RoomArt,
    forest: Texture2D,
    dojo: Texture2D,
    fighters: Texture2D,
}
impl Art {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, String> {
        let mut texture = |bytes: &[u8]| {
            let img = Image::load_image_from_mem(".png", bytes).map_err(|e| e.to_string())?;
            let tex = rl
                .load_texture_from_image(thread, &img)
                .map_err(|e| e.to_string())?;
            tex.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_POINT);
            Ok::<_, String>(tex)
        };
        Ok(Self {
            room: room::RoomArt::load(&mut texture)?,
            forest: texture(include_bytes!(
                "../../../assets/minigames/regularshow/claro.png"
            ))?,
            dojo: texture(include_bytes!(
                "../../../assets/minigames/regularshow/dojo.png"
            ))?,
            fighters: texture(include_bytes!(
                "../../../assets/minigames/regularshow/peleadores.png"
            ))?,
        })
    }
}

pub fn draw(d: &mut impl RaylibDraw, g: &Game, art: &Art) {
    d.clear_background(INK);
    if g.phase == Phase::Title {
        title(d, g, art);
        return;
    }
    let room = g.stage == Stage::LivingRoom;
    if room {
        art.room.background(d, g);
    } else {
        let background = if g.stage == Stage::Tutorial {
            &art.forest
        } else {
            &art.dojo
        };
        d.draw_texture_ex(background, Vector2::zero(), 0.0, 2.0, Color::WHITE);
    }
    for wave in &g.waves {
        for i in 0..64 {
            let a = i as f32 / 64.0 * std::f32::consts::TAU;
            let b = (i + 1) as f32 / 64.0 * std::f32::consts::TAU;
            line(
                d,
                (
                    wave.x + a.cos() * wave.radius,
                    wave.y + a.sin() * wave.radius / 1.8,
                ),
                (
                    wave.x + b.cos() * wave.radius,
                    wave.y + b.sin() * wave.radius / 1.8,
                ),
                5.0,
                GOLD,
            );
        }
    }
    let mut order: Vec<(f32, bool, usize)> = g
        .enemies
        .iter()
        .enumerate()
        .map(|(i, f)| (f.y, false, i))
        .chain(g.players.iter().enumerate().map(|(i, f)| (f.y, true, i)))
        .collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0));
    // Props and fighters share the same depth order.
    let mut pi: Vec<_> = g.props.iter().collect();
    pi.sort_by(|a, b| a.y.total_cmp(&b.y));
    let mut pidx = 0;
    let mut visitor = g.delivery.as_ref();
    let mut draw_visitor_before = |d: &mut _, depth: f32| {
        if let Some(v) = visitor {
            if v.y - 8.0 <= depth {
                art.room.visitor(d, v, g.stage_time);
                visitor = None;
            }
        }
    };
    for (y, player, idx) in order {
        while pidx < pi.len() && pi[pidx].y < y {
            draw_visitor_before(d, pi[pidx].y);
            ground_prop(d, pi[pidx], g);
            pidx += 1;
        }
        let f = if player {
            &g.players[idx]
        } else {
            &g.enemies[idx]
        };
        let held = if player {
            g.props.iter().find(|p| p.state == PropState::Held(idx))
        } else {
            None
        };
        draw_visitor_before(d, y);
        if room && !player && g.intro_left > 0.0 {
            continue;
        }
        fighter(
            d,
            art,
            room,
            f,
            if player {
                idx
            } else if f.kind == Kind::Hammer {
                5
            } else {
                2 + idx % 3
            },
            held,
            g.elapsed,
        );
        if player && g.blocked[idx] > 0.0 {
            center_local(d, "BLOQUEO", f.x as i32, (f.y - 169.0) as i32, 14, CYAN);
        }
        if f.kind == Kind::Hammer && g.immune_flash > 0.0 {
            center_local(d, "INMUNE", f.x as i32, (f.y - 225.0) as i32, 16, GOLD);
        }
    }
    for p in &pi[pidx..] {
        draw_visitor_before(d, p.y);
        ground_prop(d, p, g);
    }
    draw_visitor_before(d, f32::INFINITY);
    for p in &g.particles {
        rect(d, p.x, p.y, 4.0, 4.0, if p.gold { GOLD } else { RED });
    }
    if g.flash > 0.0 {
        rect(
            d,
            0.0,
            0.0,
            960.0,
            540.0,
            Color::new(255, 231, 160, (g.flash * 700.0) as u8),
        );
    }
    if room && g.intro_left > 0.0 {
        art.room.intro(d, g);
    } else if g.phase != Phase::Title {
        hud(d, g, art);
    }
    match g.phase {
        Phase::Title => {}
        Phase::Paused => overlay(
            d,
            "PAUSA",
            "Combate en pausa.",
            &[
                "ENTER / ESC   Continuar",
                "R   Reiniciar nivel",
                "RETROCESO   Volver al sistema solar",
            ],
        ),
        Phase::StageClear if g.stage == Stage::Hammer => overlay(
            d,
            "¡DOJO SUPERADO!",
            "La partida terminó. La tele sigue encendida...",
            &[
                "ENTER   Ver video y pasar al nivel 03",
                "RETROCESO   Volver al sistema solar",
            ],
        ),
        Phase::StageClear => overlay(
            d,
            "¡BOSQUE DESPEJADO!",
            "El Martillo es inmune a tus golpes. Busca un mueble en el dojo.",
            &[
                "ENTER   Entrar al nivel 02",
                "RETROCESO   Volver al sistema solar",
            ],
        ),
        Phase::Defeat => overlay(
            d,
            "GAME OVER",
            "Una revancha y lo tienes.",
            &[
                "ENTER / R   Reintentar nivel",
                "RETROCESO   Volver al sistema solar",
            ],
        ),
        Phase::ArcadeClear => {
            overlay(
                d,
                "¡MARTILLO DERROTADO!",
                "La tele se apagó. ¡El parque está a salvo!",
                &[
                    "03 / FUERA DEL JUEGO   ·   COMPLETADO",
                    "ENTER   Volver al sistema solar   /   R   Jugar de nuevo",
                ],
            );
        }
        _ => {}
    }
}

fn center_local(d: &mut impl RaylibDraw, t: &str, x: i32, y: i32, size: i32, color: Color) {
    let w = measure(t, size);
    text(d, t, x - w / 2, y, size, color);
}
fn furniture(d: &mut impl RaylibDraw, x: f32, y: f32, kind: Furniture) {
    if kind == Furniture::Table {
        for dx in [-27.0, 21.0] {
            block(d, x + dx, y - 30.0, 6.0, 30.0, c(115, 69, 43));
        }
        block(d, x - 34.0, y - 38.0, 68.0, 10.0, c(158, 95, 54));
        rect(d, x - 33.0, y - 38.0, 66.0, 4.0, c(232, 179, 114));
        return;
    }
    let chair = kind == Furniture::Chair;
    let w = if chair { 35.0 } else { 43.0 };
    let wood = c(147, 68, 31);
    for dx in [-w * 0.42, w * 0.28] {
        block(d, x + dx, y - 24.0, 5.0, 24.0, wood);
    }
    if chair {
        block(d, x + 7.0, y - 56.0, 6.0, 32.0, wood);
        block(d, x - 11.0, y - 62.0, 28.0, 24.0, c(173, 82, 34));
        rect(d, x - 9.0, y - 60.0, 24.0, 3.0, c(234, 154, 68));
    } else {
        rect(d, x - w * 0.38, y - 8.0, w * 0.7, 4.0, wood);
    }
    block(d, x - w / 2.0, y - 29.0, w, 7.0, c(198, 103, 44));
    rect(
        d,
        x - w / 2.0 + 1.0,
        y - 28.0,
        w - 2.0,
        2.0,
        c(250, 183, 82),
    );
}
fn ground_prop(d: &mut impl RaylibDraw, p: &Prop, g: &Game) {
    match p.state {
        PropState::Held(_) => {}
        PropState::Broken(_) => {
            for (x, y) in [(-14.0, 0.0), (4.0, 3.0), (-2.0, -3.0)] {
                line(
                    d,
                    (p.x + x, p.y + y),
                    (p.x + x + 9.0, p.y + y - 4.0),
                    4.0,
                    c(120, 63, 32),
                );
            }
        }
        PropState::Ready => {
            furniture(d, p.x, p.y, p.kind);
            for (idx, player) in g.players.iter().enumerate() {
                if player.alive()
                    && p.reachable(player)
                    && !g.props.iter().any(|v| v.state == PropState::Held(idx))
                {
                    let y = (p.y - 81.0) as i32;
                    block(d, p.x - 9.0, y as f32, 17.0, 16.0, INK);
                    center_local(
                        d,
                        if idx == 0 { "E" } else { "P" },
                        p.x as i32,
                        y + 2,
                        12,
                        GOLD,
                    );
                    break;
                }
            }
        }
    }
}
fn fighter(
    d: &mut impl RaylibDraw,
    art: &Art,
    room: bool,
    f: &Fighter,
    row: usize,
    held: Option<&Prop>,
    elapsed: f32,
) {
    if !f.alive() {
        return;
    }
    let boss = f.kind == Kind::Hammer;
    let scale = if f.action == Action::Entering {
        1.55 + 1.1 * ((f.y - 235.0) / 82.0).clamp(0.0, 1.0)
    } else if boss {
        2.65
    } else {
        1.82
    };
    let (x, y) = (f.x.round(), (f.y - f.jump).round());
    d.draw_ellipse(
        x as i32,
        f.y as i32,
        if boss { 45.0 } else { 24.0 },
        5.0,
        Color::new(37, 29, 29, 72),
    );
    let walk = if f.walk == 0.0 {
        0
    } else {
        [1, 0, 2, 0][(f.walk * 1.4) as usize % 4]
    };
    let cel = match f.action {
        Action::Strike {
            attack: Attack::Slam,
            fired: false,
            ..
        } => 13,
        Action::Strike {
            attack: Attack::Furniture(_),
            fired: false,
            ..
        } => 10,
        Action::Strike {
            attack: Attack::Furniture(_),
            fired: true,
            ..
        } => 11,
        Action::Strike {
            attack: Attack::Kick,
            fired: true,
            ..
        } => 6,
        Action::Strike { fired: true, .. } => 5,
        Action::Strike { fired: false, .. } => 4,
        Action::Guard => 7,
        Action::Dodge(_) => 12,
        Action::Hurt(_) => 8,
        _ if held.is_some() => 9,
        _ if f.jump > 0.0 => 2,
        _ => walk,
    };
    // Carrying uses the raised-arm cel; the feet cycle independently below the belt.
    let mirror = f.facing < 0.0;
    let left = x - (if mirror { 56.0 } else { 44.0 }) * scale;
    let top = y - 85.0 * scale;
    let tint = if f.invulnerable > 0.0 && (elapsed * 18.0) as i32 % 2 == 0 {
        Color::new(255, 218, 207, 215)
    } else {
        Color::WHITE
    };
    let mut slice = |frame: usize, sy: f32, height: f32| {
        d.draw_texture_pro(
            &art.fighters,
            Rectangle::new(
                frame as f32 * 100.0,
                row as f32 * 94.0 + sy,
                if mirror { -100.0 } else { 100.0 },
                height,
            ),
            Rectangle::new(left, top + sy * scale, 100.0 * scale, height * scale),
            Vector2::zero(),
            0.0,
            tint,
        );
    };
    if room {
        art.room.fighter(d, f, row, tint);
    } else if cel == 9 {
        slice(9, 0.0, 54.0);
        slice(walk, 54.0, 40.0);
    } else {
        slice(cel, 0.0, 94.0);
    }
    if let Action::Strike {
        attack: Attack::Furniture(kind),
        time,
        ..
    } = f.action
    {
        // An overhead swing traces the same arc in either facing direction.
        let t = (time / windup(Kind::Player, Attack::Furniture(kind), false)).clamp(0.0, 1.0);
        if time < 0.29 {
            let angle = (-105.0 + 110.0 * t * t).to_radians();
            furniture(
                d,
                x + f.facing * (19.0 + 89.0 * angle.cos()),
                y - 62.0 + 72.0 * angle.sin(),
                kind,
            );
        }
    } else if let Some(p) = held {
        furniture(d, x + f.facing * 3.0, y - 121.0, p.kind);
    }
    if let Action::Strike {
        attack,
        time,
        fired: false,
    } = f.action
    {
        if f.kind != Kind::Player {
            let progress = (time / windup(f.kind, attack, f.enraged())).min(1.0);
            if attack == Attack::Slam {
                d.draw_ellipse_lines(x as i32, y as i32, 76.0, 24.0, GOLD);
                d.draw_ellipse_lines(x as i32, y as i32, 76.0 * progress, 24.0 * progress, RED);
                center_local(d, "¡SALTA!", x as i32, (top + 9.0) as i32, 16, GOLD);
            } else {
                let length = if boss { 140.0 } else { 70.0 };
                let end = x + f.facing * length;
                line(d, (x, y + 5.0), (end, y + 5.0), 5.0, INK);
                line(
                    d,
                    (x, y + 5.0),
                    (x + f.facing * length * progress, y + 5.0),
                    3.0,
                    GOLD,
                );
                center_local(d, "!", x as i32, (top + 5.0) as i32, 18, GOLD);
            }
        }
    }
    if f.kind == Kind::Rookie && f.hp < f.max_hp {
        bar(
            d,
            x - 17.0,
            top + 3.0,
            34.0,
            f.hp as f32 / f.max_hp as f32,
            RED,
        );
    }
}
fn bar(d: &mut impl RaylibDraw, x: f32, y: f32, w: f32, value: f32, color: Color) {
    rect(d, x, y, w, 14.0, INK);
    rect(d, x + 2.0, y + 2.0, w - 4.0, 10.0, c(60, 42, 47));
    let filled = (w - 4.0) * value.clamp(0.0, 1.0);
    rect(d, x + 2.0, y + 2.0, filled, 10.0, color);
    rect(
        d,
        x + 2.0,
        y + 2.0,
        filled,
        3.0,
        Color::new(255, 255, 255, 75),
    );
    for i in 1..10 {
        rect(
            d,
            x + (w * i as f32 / 10.0),
            y + 2.0,
            1.0,
            10.0,
            Color::new(24, 16, 27, 65),
        );
    }
}
fn portrait(d: &mut impl RaylibDraw, art: &Art, row: usize, x: i32, y: i32) {
    block(d, x as f32, y as f32, 38.0, 38.0, c(68, 55, 61));
    d.draw_texture_pro(
        &art.fighters,
        Rectangle::new(28.0, row as f32 * 94.0 + 7.0, 32.0, 31.0),
        Rectangle::new(x as f32, y as f32, 38.0, 38.0),
        Vector2::zero(),
        0.0,
        Color::WHITE,
    );
}
fn hud(d: &mut impl RaylibDraw, g: &Game, art: &Art) {
    for (i, p) in g.players.iter().enumerate() {
        let y = 12 + i as i32 * 46;
        if g.stage == Stage::LivingRoom {
            art.room.portrait(d, i, 16, y);
        } else {
            portrait(d, art, i, 16, y);
        }
        text(d, if i == 0 { "P1" } else { "P2" }, 62, y, 17, CREAM);
        bar(
            d,
            62.0,
            (y + 23) as f32,
            150.0,
            p.hp as f32 / p.max_hp as f32,
            c(77, 213, 82),
        );
    }
    center(d, &format!("{:06}", g.score), 15, 20, GOLD);
    if g.stage != Stage::Tutorial {
        let f = &g.enemies[0];
        if g.stage == Stage::LivingRoom {
            art.room.portrait(d, 6, 904, 12);
        } else {
            portrait(d, art, 5, 904, 12);
        }
        text(
            d,
            if f.enraged() {
                "MARTILLO EN FURIA"
            } else {
                "EL MARTILLO"
            },
            727,
            12,
            17,
            CREAM,
        );
        bar(
            d,
            726.0,
            35.0,
            166.0,
            f.hp as f32 / f.max_hp as f32,
            if f.enraged() { RED } else { c(77, 213, 82) },
        );
    } else {
        text(d, "01   EL CLARO", 778, 14, 18, CREAM);
        text(
            d,
            &format!(
                "RIVALES   {}/{}",
                g.enemies.iter().filter(|f| !f.alive()).count(),
                g.enemies.len()
            ),
            810,
            40,
            12,
            GOLD,
        );
    }
    if g.combo > 1 && g.combo_time > 0.0 {
        text(
            d,
            &format!("{} HITS", g.combo),
            16,
            if g.coop { 110 } else { 68 },
            18,
            GOLD,
        );
    }
    rect(d, 0.0, 505.0, 960.0, 35.0, INK);
    let countdown;
    let help = if g.stage == Stage::LivingRoom && g.stage_time < ESCAPE_TIME {
        countdown = format!(
            "¡ESQUIVA!   La ayuda llega en {} s",
            (ESCAPE_TIME - g.stage_time).ceil() as u32
        );
        &countdown
    } else if g.stage != Stage::Tutorial {
        "E  AGARRAR     J  GOLPEAR CON MUEBLE     ESPACIO  SALTAR ONDA"
    } else {
        "WASD  MOVER     J  PUÑO     K  PATADA     F  MANTENER BLOQUEO"
    };
    if g.coop {
        text(d, help, 18, 506, 12, CREAM);
        text(
            d,
            "P2  FLECHAS   U PUÑO   I PATADA   Y BLOQUEO   P AGARRAR   SHIFT DER. SALTAR",
            18,
            524,
            11,
            CYAN,
        );
    } else {
        text(d, help, 18, 516, 14, CREAM);
    }
    text(d, "ESC  PAUSA", 848, 516, 13, GOLD);
}
fn title(d: &mut impl RaylibDraw, g: &Game, art: &Art) {
    let pink = c(245, 91, 172);
    let mint = c(126, 245, 219);
    d.clear_background(c(25, 19, 54));
    for i in 0..36 {
        let x = ((i * 137 + 39) % 960) as f32;
        let y = ((i * 83 + 18) % 390) as f32;
        rect(d, x, y, 3.0, 3.0, c(93, 69, 133));
    }
    for i in 0..17 {
        line(
            d,
            (480.0, 330.0),
            (i as f32 * 100.0 - 320.0, 540.0),
            1.0,
            c(72, 41, 100),
        );
    }
    for y in [420.0, 450.0, 488.0, 535.0] {
        rect(d, 0.0, y, 960.0, 1.0, c(72, 41, 100));
    }
    // A complete arcade cabinet, with its own marquee, CRT and control deck.
    rect(d, 52.0, 23.0, 603.0, 492.0, c(8, 11, 27));
    rect(d, 52.0, 23.0, 8.0, 492.0, pink);
    rect(d, 647.0, 23.0, 8.0, 492.0, c(68, 194, 197));
    rect(d, 68.0, 36.0, 571.0, 53.0, pink);
    center_local(d, "UN SHOW MÁS", 354, 48, 29, c(30, 19, 55));
    rect(d, 77.0, 101.0, 552.0, 272.0, mint);
    rect(d, 82.0, 106.0, 542.0, 262.0, INK);
    d.draw_texture_pro(
        &art.forest,
        Rectangle::new(0.0, 0.0, art.forest.width as f32, art.forest.height as f32),
        Rectangle::new(86.0, 110.0, 534.0, 254.0),
        Vector2::zero(),
        0.0,
        c(120, 105, 152),
    );
    rect(d, 86.0, 110.0, 534.0, 114.0, Color::new(18, 17, 46, 205));
    center_local(d, "EL MARTILLO", 354, 130, 48, CREAM);
    center_local(d, "DEL ARCADE A LA SALA DEL PARQUE", 354, 190, 12, mint);
    art.room.title_cast(d);
    for y in (112..364).step_by(4) {
        rect(d, 86.0, y as f32, 534.0, 1.0, Color::new(0, 0, 0, 35));
    }
    rect(d, 80.0, 389.0, 548.0, 53.0, mint);
    center_local(d, "ENTER  /  EMPEZAR PARTIDA", 354, 406, 22, c(17, 30, 44));
    d.draw_circle(121, 478, 12.0, pink);
    line(d, (121.0, 478.0), (121.0, 455.0), 5.0, CREAM);
    for (x, col) in [(536, pink), (574, mint), (612, CREAM)] {
        d.draw_circle(x, 474, 10.0, col);
    }
    text(d, "WASD + J / K", 157, 467, 16, CREAM);
    text(d, "ELIGE TU EQUIPO", 692, 54, 19, pink);
    text(d, "F2  CAMBIAR MODO", 692, 87, 13, mint);
    for (i, label) in ["1 JUGADOR", "2 JUGADORES"].iter().enumerate() {
        let active = g.coop == (i == 1);
        let y = 122.0 + i as f32 * 49.0;
        rect(
            d,
            689.0,
            y,
            224.0,
            38.0,
            if active { mint } else { c(53, 39, 79) },
        );
        text(
            d,
            label,
            708,
            y as i32 + 12,
            16,
            if active { INK } else { CREAM },
        );
    }
    text(d, "TRES RONDAS.", 692, 257, 19, CREAM);
    text(d, "UN SOLO MARTILLO.", 692, 285, 16, pink);
    for (i, (n, label)) in [
        ("01", "EL CLARO"),
        ("02", "EL DOJO"),
        ("03", "FUERA DEL JUEGO"),
    ]
    .iter()
    .enumerate()
    {
        let y = 336 + i as i32 * 37;
        text(d, n, 692, y, 17, mint);
        text(d, label, 730, y + 2, 13, CREAM);
    }
    text(d, "RETROCESO  /  VOLVER", 690, 480, 12, c(185, 171, 209));
}
fn overlay(d: &mut impl RaylibDraw, heading: &str, body: &str, rows: &[&str]) {
    let mint = c(126, 245, 219);
    let pink = c(245, 91, 172);
    rect(d, 0.0, 0.0, 960.0, 540.0, Color::new(16, 12, 39, 205));
    rect(d, 100.0, 119.0, 760.0, 315.0, c(9, 11, 28));
    rect(d, 100.0, 119.0, 760.0, 6.0, pink);
    rect(d, 119.0, 141.0, 662.0, 249.0, mint);
    rect(d, 125.0, 147.0, 650.0, 237.0, c(28, 25, 56));
    text(d, "UN SHOW MÁS / EL MARTILLO", 140, 160, 12, pink);
    center_local(d, heading, 450, 195, 28, mint);
    center_local(d, body, 450, 242, 13, CREAM);
    for (i, row) in rows.iter().enumerate() {
        center_local(
            d,
            row,
            450,
            286 + i as i32 * 27,
            13,
            if i == 0 { mint } else { CREAM },
        );
    }
    for y in (151..383).step_by(4) {
        rect(d, 128.0, y as f32, 644.0, 1.0, Color::new(0, 0, 0, 22));
    }
    for y in [197, 252] {
        d.draw_circle(820, y, 18.0, c(65, 52, 89));
        line(
            d,
            (815.0, y as f32 - 8.0),
            (824.0, y as f32 + 8.0),
            3.0,
            mint,
        );
    }
    for y in (308..368).step_by(7) {
        rect(d, 803.0, y as f32, 34.0, 2.0, c(73, 57, 94));
    }
    text(d, "PARK VIDEO SYSTEM", 126, 405, 12, c(161, 142, 185));
    d.draw_circle(817, 408, 4.0, pink);
}

pub fn confirm_return(d: &mut impl RaylibDraw) {
    overlay(
        d,
        "¿VOLVER AL SISTEMA SOLAR?",
        "Se perderá el intento actual.",
        &["ENTER   Volver al sistema solar", "ESC   Seguir jugando"],
    );
}
