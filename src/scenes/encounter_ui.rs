use crate::{
    graphics::billboard::SpriteBillboard,
    minigames::southpark::{
        encounters::*,
        session::{SaveKennySession, SessionState},
        SouthParkScene,
    },
};
use raylib::prelude::*;
use std::path::{Path, PathBuf};

pub struct EncounterUi<'a> {
    sprites: Vec<SpriteBillboard>,
    portraits: Vec<Texture2D>,
    voices: Vec<Option<Music<'a>>>,
    spoken: Option<(usize, usize)>,
    preference: PathBuf,
}
impl<'a> EncounterUi<'a> {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        root: &Path,
        device: Option<&'a RaylibAudio>,
        volume: f32,
        a: &mut Adventure,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sprites = vec![];
        let mut portraits = vec![];
        let mut voices = vec![];
        for name in NAMES {
            let path = root.join(format!("assets/minigames/south park/portraits/{name}.webp"));
            let image = decode_portrait(&path)?;
            portraits.push(rl.load_texture_from_image(thread, &image)?);
            sprites.push(SpriteBillboard::from_image(rl, thread, image, 1.35, true)?);
            let voice = if ["Tweek", "Jimmy", "Timmy"].contains(&name) {
                if let Some(device) = device {
                    let mut v = device.new_music(
                        &root
                            .join(format!("assets/minigames/south park/audio/{name}.mp3"))
                            .to_string_lossy(),
                    )?;
                    v.looping = false;
                    v.set_volume(volume);
                    Some(v)
                } else {
                    None
                }
            } else {
                None
            };
            voices.push(voice);
            println!("ENCOUNTER_PORTRAIT_LOADED {name}");
        }
        let preference = root.join("saves/save-kenny-language.txt");
        a.spanish = std::fs::read_to_string(&preference).is_ok_and(|v| v.trim() == "es");
        Ok(Self {
            sprites,
            portraits,
            voices,
            spoken: None,
            preference,
        })
    }
    pub fn stop(&self) {
        for v in self.voices.iter().flatten() {
            v.stop_stream();
        }
    }
    pub fn update(
        &mut self,
        rl: &RaylibHandle,
        scene: &mut SouthParkScene,
        s: &mut SaveKennySession,
    ) {
        let Some(a) = &mut s.adventure else {
            return;
        };
        if rl.is_key_pressed(KeyboardKey::KEY_L) {
            a.spanish = !a.spanish;
            if let Some(parent) = self.preference.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&self.preference, if a.spanish { "es" } else { "en" }) {
                eprintln!("Language preference: {e}");
            }
        }
        let prior = a.dialogue;
        if let Some((group, line)) = prior {
            if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                a.dialogue = None;
            } else if rl.is_key_pressed(KeyboardKey::KEY_E)
                || rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            {
                if line + 1 < script(group).len() {
                    a.dialogue = Some((group, line + 1));
                } else {
                    a.dialogue = None;
                    if let Err(e) = s.begin_encounter(scene, group) {
                        s.error = Some(e);
                    }
                }
            }
        }
        self.sync_audio(s);
    }
    pub fn sync_audio(&mut self, s: &SaveKennySession) {
        let current = s.adventure.as_ref().and_then(|a| a.dialogue);
        if current != self.spoken {
            self.stop();
            if let Some((g, n)) = current {
                let line = &script(g)[n];
                if line.voice {
                    if let Some(v) = &self.voices[line.speaker] {
                        v.play_stream();
                    }
                }
            }
            self.spoken = current;
        }
        for v in self.voices.iter().flatten() {
            v.update_stream();
        }
        debug_assert!(
            self.voices
                .iter()
                .flatten()
                .filter(|v| v.is_stream_playing())
                .count()
                <= 1
        );
    }
    pub fn world(&self, d: &mut RaylibMode3D<RaylibDrawHandle>, camera: Camera3D, a: &Adventure) {
        for (i, g) in a.groups.iter().enumerate() {
            for (j, p) in g.positions.iter().enumerate() {
                let sprite = &self.sprites[MEMBERS[i][j]];
                sprite.draw(d, &sprite.pose(*p, camera.position, 0.0, 0.0));
            }
        }
    }
    pub fn draw(
        &self,
        d: &mut RaylibDrawHandle,
        scene: &SouthParkScene,
        s: &SaveKennySession,
        w: i32,
        h: i32,
    ) {
        let Some(a) = &s.adventure else {
            return;
        };
        d.draw_rectangle(w - 218, 10, 208, 28, Color::new(235, 243, 250, 235));
        d.draw_text(
            a.text("L: English / Español", "L: Español / English"),
            w - 210,
            16,
            16,
            Color::DARKBLUE,
        );
        if s.state == SessionState::Exploring && a.dialogue.is_none() {
            if let Some(error) = &s.error {
                super::session_ui::wrapped(d,error,16,78,w-32,16,Color::RED);
            }
            d.draw_text(
                &format!(
                    "{} {}/5",
                    a.text("Missions completed:", "Misiones completadas:"),
                    s.saves
                ),
                16,
                46,
                18,
                Color::DARKBLUE,
            );
            if let Some(i) = a.nearby(scene) {
                d.draw_text(
                    &format!(
                        "E: {} {}",
                        a.text("Talk to", "Hablar con"),
                        MEMBERS[i]
                            .iter()
                            .map(|n| NAMES[*n])
                            .collect::<Vec<_>>()
                            .join(" & ")
                    ),
                    w / 2 - 180,
                    h / 2 + 45,
                    20,
                    Color::DARKBLUE,
                );
            }
        }
        if !s.blocks_controls() {
            for (i, g) in a.groups.iter().enumerate() {
                let p = g.positions[0] + Vector3::up() * 1.8;
                let delta = p - scene.player.eye();
                if delta.dot(scene.player.direction()) > 0.0
                    && delta.length() < 45.0
                    && scene
                        .world
                        .ray_distance_ignoring(
                            scene.player.eye(),
                            delta.normalized(),
                            delta.length(),
                            None,
                        )
                        .is_none()
                {
                    let screen = unsafe {
                        raylib::ffi::GetWorldToScreen(p.into(), scene.player.camera().into())
                    };
                    if !a.completed[i] {
                        d.draw_text("!", screen.x as i32, screen.y as i32, 28, COLORS[i]);
                    }
                }
            }
        }
        if let Some((g, n)) = a.dialogue {
            let line = &script(g)[n];
            let y = h - 230;
            d.draw_rectangle(12, y, w - 24, 218, Color::new(14, 35, 65, 250));
            let t = &self.portraits[line.speaker];
            let size = if w < 640 { 72.0 } else { 125.0 };
            let scale = (size / t.width as f32).min(175.0 / t.height as f32);
            d.draw_texture_ex(
                t,
                Vector2::new(24.0, (y + 20) as f32),
                0.0,
                scale,
                Color::WHITE,
            );
            let x = size as i32 + 40;
            d.draw_text(NAMES[line.speaker], x, y + 14, 24, Color::YELLOW);
            super::session_ui::wrapped(
                d,
                a.text(line.en, line.es),
                x,
                y + 52,
                w - x - 28,
                20,
                Color::WHITE,
            );
            d.draw_text(
                &format!(
                    "{}  {}/{}",
                    a.text(
                        "E / Click: next | Esc: close",
                        "E / Clic: seguir | Esc: cerrar"
                    ),
                    n + 1,
                    script(g).len()
                ),
                x,
                y + 188,
                14,
                Color::LIGHTGRAY,
            );
        }
    }
}
impl Drop for EncounterUi<'_> {
    fn drop(&mut self) {
        self.stop();
    }
}

fn decode_portrait(path: &Path) -> Result<Image, Box<dyn std::error::Error>> {
    let mut decoder =
        image_webp::WebPDecoder::new(std::io::BufReader::new(std::fs::File::open(path)?))?;
    let (w, h) = decoder.dimensions();
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return Err("Invalid portrait dimensions".into());
    }
    let mut pixels = vec![0; decoder.output_buffer_size().ok_or("Portrait too large")?];
    decoder.read_image(&mut pixels)?;
    let rgba = if decoder.has_alpha() {
        pixels
    } else {
        pixels
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect()
    };
    let image = Image::gen_image_color(w as i32, h as i32, Color::BLANK);
    // raylib owns this RGBA8 allocation; dimensions and byte count are checked.
    if rgba.len() != w as usize * h as usize * 4 {
        return Err("Invalid portrait pixels".into());
    }
    unsafe {
        std::ptr::copy_nonoverlapping(rgba.as_ptr(), image.data() as *mut u8, rgba.len());
    }
    Ok(image)
}
