//! Resident cinematic assets. Uses the original characters and their packed textures.
use crate::gameplay::progress::Franchise;
use crate::{gameplay::selection::ShipCatalog, ships::ShipsConfig,
    graphics::space::{SpaceModel, SpaceShader, Fit}};
use raylib::prelude::*;
use std::{collections::BTreeMap, path::Path};

pub struct AbductionArt {
    pub franchise: Franchise,
    pub background: &'static str,
    pub characters: &'static [&'static str],
    pub color: Color,
}

pub const ART: [AbductionArt; 6] = [
    AbductionArt { franchise: Franchise::Minecraft, background: "minecraft", characters: &["Steve"], color: Color::new(120,255,190,255) },
    AbductionArt { franchise: Franchise::SouthPark, background: "south_park", characters: &["Stan","Kyle","Kenny","Cartman"], color: Color::new(114,222,255,255) },
    AbductionArt { franchise: Franchise::SmilingFriends, background: "smiling_friends", characters: &["Pim","Charlie"], color: Color::new(255,170,225,255) },
    AbductionArt { franchise: Franchise::StarWars, background: "star_wars", characters: &["Grogu"], color: Color::new(137,210,255,255) },
    AbductionArt { franchise: Franchise::Halloween, background: "halloween", characters: &["Ghost_Face"], color: Color::new(197,143,255,255) },
    AbductionArt { franchise: Franchise::RegularShow, background: "regular_show", characters: &["Mordecai","Rigby"], color: Color::new(255,213,128,255) },
];

/// Manda ya lo que raylib tiene juntado para dibujar (antes de borrar la pantalla en medio de un cuadro).
fn flush_2d() {
    unsafe { raylib::ffi::rlDrawRenderBatchActive() };
}

pub fn cover_source(image: [f32; 2], viewport: [f32; 2]) -> Rectangle {
    let scale = (viewport[0] / image[0]).max(viewport[1] / image[1]);
    let (w,h) = (viewport[0]/scale, viewport[1]/scale);
    Rectangle::new((image[0]-w)*0.5,(image[1]-h)*0.5,w,h)
}

fn smooth(t: f32) -> f32 { let t=t.clamp(0.0,1.0); t*t*(3.0-2.0*t) }
fn alpha(c: Color, a: f32) -> Color { Color::new(c.r,c.g,c.b,(255.0*a.clamp(0.0,1.0)) as u8) }

struct Cast { background: Texture2D, models: Vec<Model>, ship: SpaceModel }

/// La de Hexbound no es una abducción sino la invitación a la fiesta, dibujada con el arte y la luz del minijuego
/// (`minigames::halloween::draw_invitation`), con la carta de Cami (si todavía no está el dibujo, una de papel). Al
/// ganar el minijuego es otra escena: la carta de agradecimiento sola, con corazones (`thanks`, `draw_thanks`).
struct Hexbound { art: crate::minigames::halloween::Art, letter: Option<Texture2D>, thank_you: Option<Texture2D>, thanks: bool }

const HEXBOUND_LETTER: &str = "assets/ui/halloween/invitation.png";
const HEXBOUND_THANKS: &str = "assets/ui/halloween/thanks.png";
pub struct AbductionRenderer {
    cast: BTreeMap<Franchise, Cast>,
    hexbound: Option<Hexbound>,
    // Models hold weak shader references; release them before the shader.
    _shader: Shader,
    ship_shader: SpaceShader,
}

impl AbductionRenderer {
    /// Regreso al mundo original: mismos personajes y fondo, sin la nave ni la abducción.
    pub fn draw_home(&self, d: &mut RaylibDrawHandle, franchise: Franchise, t: f32, width: i32, height: i32) {
        let Some(cast) = self.cast.get(&franchise) else { return; };
        let (w, h) = (width as f32, height as f32);
        d.draw_texture_pro(&cast.background,
            cover_source([cast.background.width as f32, cast.background.height as f32], [w, h]),
            Rectangle::new(0.0, 0.0, w, h), Vector2::zero(), 0.0, Color::WHITE);
        let camera = Camera3D::perspective(Vector3::new(0.0, 1.1, 9.0 / (w / h.max(1.0)).min(1.25)),
            Vector3::new(0.0, 1.1, 0.0), Vector3::up(), 43.0);
        let mut d3 = d.begin_mode3D(camera);
        let landing = smooth(t / 3.0);
        for (i, model) in cast.models.iter().enumerate() {
            let x = (i as f32 - (cast.models.len() - 1) as f32 * 0.5) * 1.35;
            d3.draw_model_ex(model, Vector3::new(x, 0.25 + (1.0 - landing) * 4.0, 0.0),
                Vector3::up(), (1.0 - landing) * 30.0, Vector3::one(), Color::WHITE);
        }
    }

    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path,
        franchises: &[Franchise]) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_cast(rl, thread, root, franchises, false)
    }

    pub(super) fn load_home(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path,
        franchise: Franchise) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_cast(rl, thread, root, &[franchise], true)
    }

    fn load_cast(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path,
        franchises: &[Franchise], home: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let shader = rl.load_shader_from_memory(thread, Some(include_str!("../graphics/shaders/abduction.vs")), Some(include_str!("../graphics/shaders/abduction.fs")));
        if !shader.is_shader_valid() { return Err("No se pudo cargar la iluminación de abducción".into()); }
        let mut cast = BTreeMap::new();
        let mut hexbound = None;
        let ships = ShipsConfig::load(&root.join("config/ships.json"))?;
        let catalog = ShipCatalog::load(&root.join("config/ship-selection.json"), &ships)?;
        let ship_shader = SpaceShader::load(rl,thread)?;
        for art in ART.iter().filter(|a| franchises.contains(&a.franchise)) {
            let background = rl.load_texture(thread, &root.join(format!("assets/images/abductions/{}.png",art.background)).to_string_lossy())?;
            background.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            let mut models = Vec::new();
            for name in art.characters {
                let mut model = rl.load_model(thread, &root.join(format!("assets/models/characters/abductions/{name}.glb")).to_string_lossy())?;
                if !model.is_model_valid() { return Err(format!("Modelo de abducción inválido: {name}").into()); }
                let bounds = model.get_model_bounding_box();
                let size = bounds.max - bounds.min;
                let scale = 2.0 / size.x.max(size.y).max(size.z).max(0.01);
                let center = (bounds.min + bounds.max)*0.5;
                model.transform = (Matrix::translate(-center.x,-center.y,-center.z) * Matrix::scale(scale,scale,scale)).into();
                for material in model.materials_mut() {
                    *material.shader_mut() = unsafe { WeakShader::from_raw(*shader.as_ref()) };
                }
                models.push(model);
            }
            let choice = catalog.choice(art.franchise.as_str()).ok_or("Nave de abducción ausente")?;
            let ship = catalog.model(choice,&ships);
            let ship = SpaceModel::load(rl,thread,&root.join(&ship.model),&ship_shader,
                ship.model_rotation_degrees,Fit::Length(2.0))?;
            if art.franchise == Franchise::Halloween && !home {
                let mut card = |path: &str| -> Result<Option<Texture2D>, Box<dyn std::error::Error>> {
                    let path = root.join(path);
                    if !path.is_file() {
                        return Ok(None);
                    }
                    let t = rl.load_texture(thread, &path.to_string_lossy())?;
                    t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
                    Ok(Some(t))
                };
                let (letter, thank_you) = (card(HEXBOUND_LETTER)?, card(HEXBOUND_THANKS)?);
                let art = crate::minigames::halloween::Art::load(rl, thread, root).map_err(|e| e.to_string())?;
                hexbound = Some(Hexbound { art, letter, thank_you, thanks: false });
            }
            cast.insert(art.franchise, Cast { background, models, ship });
        }
        Ok(Self { cast, hexbound, _shader: shader, ship_shader })
    }

    /// Qué carta lleva la escena de Hexbound: la de agradecimiento (al ganar el minijuego) o la invitación.
    pub fn set_hexbound_thanks(&mut self, thanks: bool) {
        if let Some(hex) = &mut self.hexbound {
            hex.thanks = thanks;
        }
    }

    /// Antes de `begin_drawing`: la escena de Hexbound necesita su textura del tamaño de la ventana.
    pub fn prepare(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(hex) = &mut self.hexbound {
            hex.art.prepare(rl, thread).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, thread: &RaylibThread, franchise: Franchise, t: f32, width: i32, height: i32) {
        // raylib junta lo que se dibuja en 2D y lo manda después: lo que ya dibujó el sistema solar (etiquetas, logos)
        // tiene que salir antes de que la escena borre la pantalla, o aparece encima de ella
        flush_2d();
        if let Some(hex) = self.hexbound.as_mut().filter(|_| franchise == Franchise::Halloween) {
            if hex.thanks {
                crate::minigames::halloween::draw_thanks(d, &hex.art.font, hex.thank_you.as_ref(), t);
            } else {
                crate::minigames::halloween::draw_invitation(d, thread, &mut hex.art, hex.letter.as_ref(), t);
            }
            return;
        }
        let Some(cast) = self.cast.get(&franchise) else { return; };
        let art = ART.iter().find(|a| a.franchise == franchise).unwrap();
        let (w,h) = (width as f32,height as f32);
        let source = cover_source([cast.background.width as f32,cast.background.height as f32],[w,h]);
        d.draw_texture_pro(&cast.background,source,Rectangle::new(0.0,0.0,w,h),Vector2::zero(),0.0,Color::WHITE);
        d.draw_rectangle(0,0,width,height,Color::new(2,8,24,85));
        let beam = smooth((t-0.5)/0.6) * (1.0-smooth((t-3.9)/0.5));
        let bottom = Vector2::new(w*0.5,h*0.82);
        // Layered cone, rising rings and deterministic motes surround the real mesh.
        for layer in (0..8).rev() {
            let radius = w.min(h)*(0.13+layer as f32*0.014+(cast.models.len()-1) as f32*0.06);
            d.draw_triangle(Vector2::new(w*0.5,h*0.23),Vector2::new(bottom.x-radius,bottom.y),
                Vector2::new(bottom.x+radius,bottom.y),alpha(art.color,beam*0.025));
        }
        d.draw_ellipse(bottom.x as i32,bottom.y as i32,w.min(h)*(0.22+(cast.models.len()-1) as f32*0.065),h*0.025,alpha(art.color,beam*0.35));
        for i in 0..40 {
            let k = i as f32;
            let rise = (t*0.46+k*0.073).fract();
            let x = w*0.5+(k*2.399+t).sin()*w.min(h)*0.19*(1.0-rise);
            let y = bottom.y-rise*h*0.7;
            d.draw_circle_v(Vector2::new(x,y),1.0+(k%3.0),alpha(art.color,beam*(1.0-rise)*0.8));
        }
        let aspect = w / h.max(1.0);
        let distance = 9.0 / aspect.min(1.25);
        let camera = Camera3D::perspective(Vector3::new(0.0,1.1,distance),Vector3::new(0.0,1.1,0.0),Vector3::up(),43.0);
        {
            let mut d3 = d.begin_mode3D(camera);
            self.ship_shader.begin_frame(Vector3::new(-3.0,7.0,5.0),camera.position,0.65);
            self.ship_shader.set_studio(true);
            let ship_y = 2.6+(t*1.8).sin()*0.045+smooth((t-3.8)/0.7)*3.0;
            cast.ship.draw(&mut d3,&mut self.ship_shader,
                Matrix::rotate_y(0.55) * Matrix::translate(0.0,ship_y,0.0));
            for (i,model) in cast.models.iter().enumerate() {
                let delay = i as f32*0.12;
                let lift = smooth((t-1.2-delay)/2.45);
                let shrink = 1.0-smooth((t-3.0-delay)/0.65);
                if shrink <= 0.001 { continue; }
                let x = (i as f32-(cast.models.len()-1) as f32*0.5)*1.35;
                let position = Vector3::new(x*(1.0-lift),0.25+lift*3.2+(t*3.0+i as f32).sin()*0.04*beam,0.0);
                // Puffy characters carry their original front artwork on shallow
                // meshes; a restrained turn keeps that artwork readable.
                let yaw = (t*0.55+franchise as u8 as f32*0.1).sin()*12.0+lift*22.0;
                d3.draw_model_ex(model,position,Vector3::up(),yaw,Vector3::one()*shrink,Color::WHITE);
            }
        }
        // Letterbox and fade are responsive; they never alter the image aspect.
        d.draw_rectangle(0,0,width,(h*0.065) as i32,Color::new(1,5,15,245));
        d.draw_rectangle(0,(h*0.935) as i32,width,(h*0.065+1.0) as i32,Color::new(1,5,15,245));
        let fade = (1.0-smooth(t/0.35)).max(smooth((t-4.35)/0.45));
        d.draw_rectangle(0,0,width,height,alpha(Color::BLACK,fade));
    }
}

/// La música de la invitación de Hexbound: "Exploring", la de la intro del minijuego, desde el principio.
const HEXBOUND_MUSIC: (&str, f32) = ("assets/audio/music/Halloween/exploring.mp3", 0.35);

/// La música propia de la abducción de una franquicia (solo Hexbound tiene), lista para tocar.
pub fn abduction_music<'a>(device: Option<&'a RaylibAudio>, root: &Path, franchise: Franchise) -> Option<Music<'a>> {
    if franchise != Franchise::Halloween {
        return None;
    }
    device.and_then(|d| d.new_music(&root.join(HEXBOUND_MUSIC.0).to_string_lossy()).ok()).map(|mut m| {
        m.looping = false;
        m.set_volume(HEXBOUND_MUSIC.1);
        m
    })
}

/// El volumen de esa música en el segundo `t` de una escena de `length` segundos: se apaga con el fundido final.
pub fn abduction_music_volume(t: f32, length: f32) -> f32 {
    HEXBOUND_MUSIC.1 * (1.0 - smooth((t - (length - 0.45)) / 0.45))
}

const SMILINGFRIENDS_VIDEO: &str = "assets/video/smilingfriends/abduction.sfv";
const SMILINGFRIENDS_AUDIO: &str = "assets/video/smilingfriends/abduction.ogg";

/// La abducción de Smiling Friends: un video a pantalla completa, sin marco, con su sonido y el botón SKIP de la
/// intro del minijuego (o Espacio, Enter o Esc).
pub struct AbductionVideo<'a> {
    frames: super::sfv::Frames,
    texture: Texture2D,
    current: usize,
    sound: Option<Music<'a>>,
    playing: bool,
    font: Font,
}

impl<'a> AbductionVideo<'a> {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path, device: Option<&'a RaylibAudio>)
        -> Result<Self, Box<dyn std::error::Error>> {
        let frames = super::sfv::Frames::load(&root.join(SMILINGFRIENDS_VIDEO))?;
        let texture = rl.load_texture_from_image(thread, &frames.image(0)?)?;
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        let font = rl.load_font_ex(thread, &root.join("assets/fonts/Bungee-Regular.ttf").to_string_lossy(), 64, None)?;
        let sound = device.and_then(|d| d.new_music(&root.join(SMILINGFRIENDS_AUDIO).to_string_lossy()).ok()).map(|mut m| {
            m.looping = false;
            m
        });
        Ok(Self { frames, texture, current: 0, sound, playing: false, font })
    }

    /// Si se pidió saltear el video este cuadro.
    pub fn skip_requested(&self, rl: &RaylibHandle) -> bool {
        let (w, h) = (rl.get_screen_width() as f32, rl.get_screen_height() as f32);
        (rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
            && super::smilingfriends_game::skip_button(w, h).check_collision_point_rec(rl.get_mouse_position()))
            || [KeyboardKey::KEY_ESCAPE, KeyboardKey::KEY_SPACE, KeyboardKey::KEY_ENTER].iter().any(|&k| rl.is_key_pressed(k))
    }

    /// Lleva el video al segundo `t` (el sonido arranca con el primer cuadro).
    pub fn update(&mut self, t: f32) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(m) = &mut self.sound {
            if !self.playing {
                m.seek_stream(0.0);
                m.play_stream();
            }
            m.update_stream();
        }
        self.playing = true;
        let frame = ((t * self.frames.fps() as f32) as usize).min(self.frames.len() - 1);
        if frame != self.current {
            super::sfv::update_frame(&mut self.texture, &self.frames, frame)?;
            self.current = frame;
        }
        Ok(())
    }

    /// Corta el sonido; la próxima vez empieza de nuevo.
    pub fn stop(&mut self) {
        if let Some(m) = &self.sound {
            m.stop_stream();
        }
        self.playing = false;
        self.current = usize::MAX;
    }

    /// El cuadro actual llenando toda la ventana (recortado al centro, sin estirar) y el botón SKIP.
    pub fn draw(&self, d: &mut RaylibDrawHandle, width: i32, height: i32) {
        flush_2d();
        let (w, h) = (width as f32, height as f32);
        d.clear_background(Color::BLACK);
        let source = cover_source([self.texture.width as f32, self.texture.height as f32], [w, h]);
        d.draw_texture_pro(&self.texture, source, Rectangle::new(0.0, 0.0, w, h), Vector2::zero(), 0.0, Color::WHITE);
        let mouse = d.get_mouse_position();
        super::smilingfriends_game::draw_skip(d, &self.font, w, h, mouse);
    }
}
