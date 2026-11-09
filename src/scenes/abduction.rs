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

pub fn cover_source(image: [f32; 2], viewport: [f32; 2]) -> Rectangle {
    let scale = (viewport[0] / image[0]).max(viewport[1] / image[1]);
    let (w,h) = (viewport[0]/scale, viewport[1]/scale);
    Rectangle::new((image[0]-w)*0.5,(image[1]-h)*0.5,w,h)
}

fn smooth(t: f32) -> f32 { let t=t.clamp(0.0,1.0); t*t*(3.0-2.0*t) }
fn alpha(c: Color, a: f32) -> Color { Color::new(c.r,c.g,c.b,(255.0*a.clamp(0.0,1.0)) as u8) }

struct Cast { background: Texture2D, models: Vec<Model>, ship: SpaceModel }
pub struct AbductionRenderer {
    cast: BTreeMap<Franchise, Cast>,
    // Models hold weak shader references; release them before the shader.
    _shader: Shader,
    ship_shader: SpaceShader,
}

impl AbductionRenderer {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path,
        franchises: &[Franchise]) -> Result<Self, Box<dyn std::error::Error>> {
        let shader = rl.load_shader_from_memory(thread, Some(include_str!("../graphics/shaders/abduction.vs")), Some(include_str!("../graphics/shaders/abduction.fs")));
        if !shader.is_shader_valid() { return Err("No se pudo cargar la iluminación de abducción".into()); }
        let mut cast = BTreeMap::new();
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
            cast.insert(art.franchise, Cast { background, models, ship });
        }
        Ok(Self { cast, _shader: shader, ship_shader })
    }

    pub fn draw(&mut self, d: &mut RaylibDrawHandle, franchise: Franchise, t: f32, width: i32, height: i32) {
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
