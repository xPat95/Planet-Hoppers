//! A modal paper theatre: it owns no world, mission, reward or input listener state.
use raylib::prelude::*;
use std::path::Path;

const LINES: [(&str, &str); 23] = [
    ("Hello there, children!", "¡Hola, pequeños!"),
    ("Welcome to South Park!", "¡Bienvenidos a South Park!"),
    (
        "A lovely little town where absolutely nothing ever goes wrong...",
        "Un pueblito encantador donde nunca, jamás, pasa nada malo...",
    ),
    ("...Okay, that's a lie.", "...Bueno, eso es mentira."),
    (
        "Now, let me introduce you to someone.",
        "Ahora quiero presentarles a alguien.",
    ),
    (
        "This little fella right here is Kenny.",
        "Este pequeñín de aquí es Kenny.",
    ),
    ("He's a good kid.", "Es un buen chico."),
    (
        "There's just one tiny problem...",
        "Solo tiene un problemita...",
    ),
    ("HE KEEPS DYING!", "¡SE MUERE A CADA RATO!"),
    (
        "And today, things are looking particularly bad.",
        "Y hoy la cosa pinta especialmente mal.",
    ),
    (
        "There are all kinds of crazy things happening around town.",
        "Están pasando toda clase de locuras por el pueblo.",
    ),
    (
        "Aliens, freezing temperatures, runaway buses...",
        "Extraterrestres, un frío de muerte, autobuses sin frenos...",
    ),
    (
        "And trust me, children, that's just the beginning!",
        "¡Y créanme, pequeños, eso es solo el aperitivo!",
    ),
    (
        "So here's what you're gonna do.",
        "Así que esto es lo que van a hacer.",
    ),
    (
        "Explore South Park and find the kids around town.",
        "Exploren South Park y busquen a los chicos del pueblo.",
    ),
    (
        "They've seen what's going on, and they need your help!",
        "¡Ellos han visto lo que pasa y necesitan su ayuda!",
    ),
    (
        "Listen to what they have to say.",
        "Escuchen lo que tienen que contarles.",
    ),
    (
        "Complete their challenges.",
        "Completen sus cinco desafíos.",
    ),
    ("And most importantly...", "Y lo más importante..."),
    ("KEEP KENNY ALIVE!", "¡MANTENGAN A KENNY CON VIDA!"),
    (
        "Now get out there, children!",
        "¡Ahora salgan ahí, pequeños!",
    ),
    (
        "And try not to get anybody killed!",
        "¡Y procuren que nadie acabe muerto!",
    ),
    ("...Especially Kenny.", "...Sobre todo Kenny."),
];
fn surprised(line: usize) -> bool {
    matches!(line, 3 | 8 | 11 | 19)
}
fn chapter(line: usize) -> usize {
    match line {
        0..=3 => 0,
        4..=8 => 1,
        9..=12 => 2,
        13..=19 => 3,
        _ => 4,
    }
}

#[derive(Default)]
struct Playback {
    line: usize,
    age: f32,
    elapsed: f32,
    closing: Option<f32>,
    previous: bool,
}
impl Playback {
    fn tick(&mut self, dt: f32) {
        if let Some(t) = &mut self.closing {
            *t += dt;
        } else {
            self.age += dt;
            self.elapsed += dt;
        }
    }
    fn advance(&mut self) {
        // Edge-triggered input plus a short debounce: held keys never advance.
        if self.closing.is_some() || self.age < 0.28 {
            return;
        }
        if self.line == LINES.len() - 1 {
            self.skip();
        } else {
            self.previous = surprised(self.line);
            self.line += 1;
            self.age = 0.;
        }
    }
    fn skip(&mut self) {
        if self.closing.is_none() {
            self.closing = Some(0.);
        }
    }
    fn finished(&self) -> bool {
        self.closing.is_some_and(|t| t >= 0.4)
    }
}

struct Art {
    normal: Texture2D,
    surprised: Texture2D,
    kenny: Texture2D,
    title: Font,
    body: Font,
}
impl Art {
    fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut normal = super::encounter_ui::decode_portrait(
            &root.join("assets/minigames/south park/portraits/Chef-normal.webp"),
        )?;
        let mut surprised = Image::load_image(
            &root
                .join("assets/minigames/south park/portraits/Chef-surprised.png")
                .to_string_lossy(),
        )?;
        normal.alpha_crop(0.125);
        surprised.alpha_crop(0.125);
        let normal = rl.load_texture_from_image(thread, &normal)?;
        let surprised = rl.load_texture_from_image(thread, &surprised)?;
        let kenny = rl.load_texture(
            thread,
            &root
                .join("assets/textures/minigames/South Park/KennyMcCormick.png")
                .to_string_lossy(),
        )?;
        let chars: String = (32..=255).filter_map(char::from_u32).collect();
        let title = crate::graphics::resources::load_font_chars(&root.join("assets/fonts/GloriaHallelujah.ttf"), 48, &chars)?;
        let body = crate::graphics::resources::load_font_chars(&root.join("assets/fonts/TitilliumWeb-Bold.ttf"), 36, &chars)?;
        for t in [&normal, &surprised, &kenny] {
            t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        }
        Ok(Self {
            normal,
            surprised,
            kenny,
            title,
            body,
        })
    }
}
const INK: Color = Color::new(73, 56, 45, 255);
const PAPER: Color = Color::new(255, 248, 232, 255);
fn text(d: &mut impl RaylibDraw, f: &Font, s: &str, x: f32, y: f32, size: f32, c: Color) {
    d.draw_text_ex(f, s, Vector2::new(x, y), size, 1., c);
}
fn cutout(d: &mut impl RaylibDraw, t: &Texture2D, x: f32, foot: f32, height: f32, alpha: f32) {
    let w = height * t.width as f32 / t.height as f32;
    let src = Rectangle::new(0., 0., t.width as f32, t.height as f32);
    d.draw_texture_pro(
        t,
        src,
        Rectangle::new(x - w / 2. + 6., foot - height + 6., w, height),
        Vector2::zero(),
        0.,
        Color::new(40, 35, 43, (35. * alpha) as u8),
    );
    d.draw_texture_pro(
        t,
        src,
        Rectangle::new(x - w / 2., foot - height, w, height),
        Vector2::zero(),
        0.,
        Color::WHITE.alpha(alpha),
    );
}
fn skip_rect() -> Rectangle {
    Rectangle::new(900., 24., 345., 54.)
}
fn draw(d: &mut impl RaylibDraw, a: &Art, p: &Playback, es: bool, mouse: Vector2) {
    d.draw_rectangle(0, 0, 1280, 800, Color::new(245, 235, 212, 255));
    d.draw_rectangle_rounded(Rectangle::new(18., 18., 1244., 764.), 0.05, 16, INK);
    d.draw_rectangle_rounded(Rectangle::new(22., 22., 1236., 756.), 0.05, 16, PAPER);
    d.draw_rectangle(36, 88, 1208, 480, Color::new(153, 193, 202, 255));
    d.draw_circle(1080, 172, 55., Color::new(249, 224, 162, 255));
    // Layered mountain silhouettes, snow caps and forest paper cutouts.
    for (x, y, r) in [
        (40_f32, 330_f32, 220_f32),
        (310., 280., 230.),
        (610., 320., 240.),
        (930., 300., 270.),
        (1200., 340., 200.),
    ] {
        d.draw_triangle(
            Vector2::new((x - r).max(36.), 490.),
            Vector2::new((x + r).min(1244.), 490.),
            Vector2::new(x, y - 150.),
            Color::new(107, 143, 151, 255),
        );
        d.draw_triangle(
            Vector2::new((x - r * 0.25).max(36.), y - 75.),
            Vector2::new((x + r * 0.25).min(1244.), y - 75.),
            Vector2::new(x, y - 150.),
            Color::new(239, 243, 230, 255),
        );
    }
    d.draw_rectangle(36, 420, 1208, 148, Color::new(229, 237, 224, 255));
    for (i, x) in [85, 275, 825, 1050].iter().enumerate() {
        let col = [
            Color::new(178, 112, 95, 255),
            Color::new(194, 169, 108, 255),
            Color::new(132, 159, 131, 255),
            Color::new(147, 159, 179, 255),
        ][i];
        d.draw_rectangle(*x, 355, 140, 100, col);
        d.draw_triangle(
            Vector2::new(*x as f32 - 15., 355.),
            Vector2::new(*x as f32 + 155., 355.),
            Vector2::new(*x as f32 + 70., 300.),
            PAPER,
        );
        d.draw_rectangle(*x + 22, 375, 30, 35, PAPER);
        d.draw_rectangle(*x + 26, 379, 22, 27, Color::new(122, 161, 177, 255));
        d.draw_rectangle(*x + 88, 398, 30, 57, INK);
    }
    for x in [68, 240, 795, 1015, 1210] {
        d.draw_rectangle(x - 4, 380, 8, 90, INK);
        d.draw_triangle(
            Vector2::new(x as f32 - 30., 425.),
            Vector2::new(x as f32 + 30., 425.),
            Vector2::new(x as f32, 325.),
            Color::new(78, 116, 107, 255),
        );
    }
    // Snowbank, foot shadows and an entrance with the same grounded baseline.
    d.draw_ellipse(620, 550, 570., 38., Color::new(250, 249, 237, 255));
    d.draw_ellipse(442, 552, 132., 13., Color::new(73, 56, 45, 32));
    let enter = (p.elapsed / 0.55).min(1.);
    let ease = 1. - (1. - enter).powi(3);
    let emphasis = if matches!(p.line, 3 | 8 | 19) {
        (p.age * 13.).sin() * (-p.age * 4.).exp() * 5.
    } else {
        0.
    };
    let x = 442. - 70. * (1. - ease) + (p.elapsed * 1.4).sin() * 2. + emphasis;
    let blend = (p.age / 0.18).min(1.);
    if p.previous != surprised(p.line) && blend < 1. {
        cutout(
            d,
            if p.previous { &a.surprised } else { &a.normal },
            x,
            554.,
            420.,
            1. - blend,
        );
    }
    cutout(
        d,
        if surprised(p.line) {
            &a.surprised
        } else {
            &a.normal
        },
        x,
        554.,
        420.,
        if p.previous != surprised(p.line) {
            blend
        } else {
            1.
        },
    );
    if p.line >= 5 {
        let k = if p.line == 5 {
            (p.age / 0.4).min(1.)
        } else {
            1.
        };
        d.draw_ellipse(715, 551, 70., 10., Color::new(73, 56, 45, 28));
        cutout(
            d,
            &a.kenny,
            715.,
            554. - (p.elapsed * 2.5).sin().abs() * 4.,
            218. * (0.9 + 0.1 * k),
            k,
        );
    }
    if (9..=12).contains(&p.line) {
        // Threat silhouettes only; no puzzle answers.
        d.draw_ellipse(910, 235, 62., 17., INK);
        d.draw_circle(910, 223, 25., Color::new(157, 187, 152, 255));
        text(d, &a.title, "?!", 1000., 200., 46., INK);
        text(
            d,
            &a.body,
            if es {
                "Un día complicado..."
            } else {
                "One of those days..."
            },
            815.,
            285.,
            24.,
            INK,
        );
    }
    if p.line == 19 {
        text(d, &a.title, "SAVE KENNY!", 775., 175. + emphasis, 47., INK);
    }
    let labels = if es {
        [
            "Bienvenidos",
            "Nuestro pequeño amigo",
            "Un día de locos",
            "Cinco desafíos",
            "¡A explorar!",
        ]
    } else {
        [
            "Welcome, children",
            "Our little friend",
            "Trouble in town",
            "Five challenges",
            "Off you go!",
        ]
    };
    text(d, &a.title, labels[chapter(p.line)], 55., 30., 30., INK);
    let skip = skip_rect();
    d.draw_rectangle_rounded(
        skip,
        0.2,
        8,
        if skip.check_collision_point_rec(mouse) {
            Color::new(233, 191, 98, 255)
        } else {
            Color::new(235, 223, 201, 255)
        },
    );
    text(
        d,
        &a.body,
        if es {
            "Saltar introducción"
        } else {
            "Skip intro"
        },
        skip.x + 24.,
        skip.y + 8.,
        27.,
        INK,
    );
    d.draw_rectangle_rounded(
        Rectangle::new(58., 584., 1168., 174.),
        0.12,
        12,
        Color::new(73, 56, 45, 28),
    );
    d.draw_rectangle_rounded(Rectangle::new(52., 578., 1168., 174.), 0.12, 12, PAPER);
    text(
        d,
        &a.title,
        "Chef",
        76.,
        585.,
        29.,
        Color::new(167, 122, 81, 255),
    );
    let line = if es { LINES[p.line].1 } else { LINES[p.line].0 };
    let size = if matches!(p.line, 8 | 19) { 35. } else { 32. };
    let mut row = String::new();
    let mut y = 631.;
    for word in line.split_whitespace() {
        let next = if row.is_empty() {
            word.into()
        } else {
            format!("{row} {word}")
        };
        if a.body.measure_text(&next, size, 1.).x > 1090. {
            text(d, &a.body, &row, 78., y, size, INK);
            y += 39.;
            row = word.into();
        } else {
            row = next;
        }
    }
    text(d, &a.body, &row, 78. + emphasis * 0.3, y, size, INK);
    text(
        d,
        &a.body,
        if es {
            "E / Enter / clic: continuar"
        } else {
            "E / Enter / click: continue"
        },
        865.,
        715.,
        20.,
        INK,
    );
    text(
        d,
        &a.body,
        &format!("{} / 5", chapter(p.line) + 1),
        78.,
        715.,
        20.,
        INK,
    );
    text(
        d,
        &a.body,
        if es { "L: English" } else { "L: Español" },
        650.,
        715.,
        20.,
        INK,
    );
    let fade = if let Some(t) = p.closing {
        (t / 0.4).min(1.)
    } else {
        1. - (p.elapsed / 0.4).min(1.)
    };
    if fade > 0. {
        d.draw_rectangle(0, 0, 1280, 800, Color::BLACK.alpha(fade));
    }
}

/// Returns true only after completion/skip. Window close leaves an unfinished intro unseen.
pub fn play(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    root: &Path,
    device: Option<&RaylibAudio>,
    volume: f32,
    es: &mut bool,
    smoke: u8,
) -> Result<bool, Box<dyn std::error::Error>> {
    let a = Art::load(rl, thread, root)?;
    let mut voice = if let Some(device) = device {
        let mut m = device.new_music(
            &root
                .join("assets/minigames/south park/audio/chef.mp3")
                .to_string_lossy(),
        )?;
        m.looping = false;
        m.set_volume(volume);
        Some(m)
    } else {
        None
    };
    let mut p = Playback::default();
    let mut started = false;
    let mut captured = usize::MAX;
    rl.enable_cursor();
    while !rl.window_should_close() && !p.finished() {
        p.tick(if smoke > 0 {
            0.05
        } else {
            rl.get_frame_time().min(0.1)
        });
        if !started {
            if let Some(v) = &mut voice {
                v.play_stream();
            }
            started = true;
        }
        if let Some(v) = &mut voice {
            v.update_stream();
        }
        let w = rl.get_screen_width();
        let h = rl.get_screen_height();
        let scale = (w as f32 / 1280.).min(h as f32 / 800.);
        let offset = Vector2::new(
            (w as f32 - 1280. * scale) / 2.,
            (h as f32 - 800. * scale) / 2.,
        );
        if smoke > 0 {
            unsafe {
                raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                    frame: 0,
                    type_: if smoke == 1 && p.age > 0.55 { 2 } else { 1 },
                    params: [KeyboardKey::KEY_E as i32, 0, 0, 0],
                });
                if smoke == 2 {
                    raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                        frame: 0,
                        type_: 7,
                        params: [
                            (offset.x + 1050. * scale) as i32,
                            (offset.y + 50. * scale) as i32,
                            0,
                            0,
                        ],
                    });
                    raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                        frame: 0,
                        type_: if p.age > 0.4 { 6 } else { 5 },
                        params: [0, 0, 0, 0],
                    });
                }
            }
        }
        let mouse = (rl.get_mouse_position() - offset) / scale;
        if rl.is_key_pressed(KeyboardKey::KEY_L) {
            *es = !*es;
            std::fs::create_dir_all(root.join("saves"))?;
            std::fs::write(
                root.join("saves/save-kenny-language.txt"),
                if *es { "es" } else { "en" },
            )?;
        }
        let click = rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        // Do not reuse the click that dismissed the preceding original intro.
        if p.elapsed > 0.15 && click && skip_rect().check_collision_point_rec(mouse) {
            p.skip();
        } else if click
            || rl.is_key_pressed(KeyboardKey::KEY_E)
            || rl.is_key_pressed(KeyboardKey::KEY_ENTER)
        {
            p.advance();
        }
        if p.closing.is_some() {
            if let Some(v) = &voice {
                v.stop_stream();
            }
        }
        {
            let mut d = rl.begin_drawing(thread);
            d.clear_background(Color::BLACK);
            let camera = Camera2D {
                offset,
                target: Vector2::zero(),
                rotation: 0.,
                zoom: scale,
            };
            {
                let mut canvas = d.begin_mode2D(camera);
                draw(&mut canvas, &a, &p, *es, mouse);
            }
            if smoke == 1 && p.age > 0.4 && captured != p.line {
                std::fs::create_dir_all(root.join("target/chef-intro-smoke"))?;
                crate::graphics::resources::save_frame(
                    &root.join(format!(
                        "target/chef-intro-smoke/{}-{}.png",
                        if *es { "es" } else { "en" },
                        p.line
                    )),
                    w,
                    h,
                )?;
                captured = p.line;
            }
        }
    }
    if let Some(v) = &voice {
        v.stop_stream();
    }
    if smoke > 0 {
        assert!(voice.as_ref().is_none_or(|v| !v.is_stream_playing()));
        assert!(p.finished());
        assert_eq!(p.line, if smoke == 2 { 0 } else { LINES.len() - 1 });
        unsafe {
            raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                frame: 0,
                type_: 1,
                params: [KeyboardKey::KEY_E as i32, 0, 0, 0],
            });
            raylib::ffi::PlayAutomationEvent(raylib::ffi::AutomationEvent {
                frame: 0,
                type_: 5,
                params: [0, 0, 0, 0],
            });
        }
    }
    Ok(p.finished())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dialogue_debounce_completion_and_skip() {
        let mut p = Playback::default();
        p.advance();
        assert_eq!(p.line, 0);
        for i in 1..LINES.len() {
            p.tick(0.3);
            p.advance();
            assert_eq!(p.line, i);
            p.advance();
            assert_eq!(p.line, i);
        }
        p.tick(0.3);
        p.advance();
        p.tick(0.4);
        assert!(p.finished());
        let mut p = Playback::default();
        p.skip();
        p.tick(0.4);
        assert!(p.finished());
        assert_eq!(p.line, 0);
        assert!(LINES
            .iter()
            .all(|(en, es)| !en.is_empty() && !es.is_empty()));
        assert!([3, 8, 11, 19].iter().all(|&i| surprised(i)));
    }
}
