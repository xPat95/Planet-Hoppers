//! Escena del minijuego de Halloween: oscuridad total; la única luz sale de la bruja, tenue y cálida. Se juega desde
//! sus ojos, bajitos (las criaturas la superan y la rodean): al empezar se la ve bailar sola en su luz y la cámara
//! gira hasta meterse en su mirada. Lejos, las criaturas se ven apenas (y sus ojos brillan); cerca, del todo. Maldecida,
//! se derrite en un charco negro que se hunde en el piso. Si una llega, se tira encima de la cámara (susto). Encima, el HUD: corazones, glifos, trazo, hechizos y menús.
//! Los modelos son los de `assets/models/minigames/halloween/`, con sus poses mezcladas.
use super::{Creature, Glyph, Kind, Phase, Session, State};
use crate::graphics::space::{Fit, SpaceModel, SpaceShader};
use raylib::{ffi, prelude::*};
use std::collections::HashMap;
use std::path::Path;

type Error = Box<dyn std::error::Error>;

/// Alto de la bruja en la escena (su modelo del juego mide ~0.7) y altura de sus ojos.
pub(super) const WITCH_H: f32 = 1.3;
const EYE_H: f32 = 0.85;
/// Pasos de baile de la bruja, en orden, y cuánto dura cada uno.
pub(super) const WITCH_DANCE: [usize; 6] = [0, 4, 1, 3, 2, 5];
/// La pose de la bruja advirtiendo, con el dedo levantado.
const WITCH_WARN: usize = 7;
pub(super) const BEAT: f32 = 0.55;
/// Su luz: donde la lleva (un farol en alto, bien arriba de su cabeza) y a qué distancia cae a la mitad.
const LAMP: Vector3 = Vector3::new(0.0, 3.4, -0.3);
const LAMP_RANGE: f32 = 3.0;
/// Niebla oscura: cuánto se hunde lo lejano (por metro).
const FOG_DENSITY: f32 = 0.16;
/// El fondo: negro del todo.
const FOG_COLOR: Color = Color::BLACK;
/// Lado del mapa de sombras del farol.
const SHADOW_SIZE: i32 = 2048;
/// Trazo de la bruja: blanco dorado.
pub const STROKE: Color = Color::new(255, 236, 170, 255);
/// Cuánto dura el susto cuando una criatura llega.
const SCARE: f32 = 0.55;
/// Ojo de pez a pantalla completa (ver lens.fs): ángulo en las esquinas de la pantalla.
pub(super) const FISH_HALF: f32 = 68.0;
/// Lado de la textura cuadrada de la escena, en altos de pantalla (la lente agranda mucho el centro: así no se pixela).
const SUPERSAMPLE: f32 = 4.5;

/// Las criaturas, mucho más grandes que en sus modelos: diablos y fantasmas de ~2,6 m (el doble que la bruja) y los
/// demonios, los más altos (~2,85 m). Desde sus ojos bajitos se las ve de la panza para arriba.
fn creature_scale(kind: Kind) -> f32 {
    match kind {
        Kind::Devil => 2.4,
        Kind::Ghost => 2.15,
        Kind::Demon => 1.35,
    }
}

/// Dónde queda en la pantalla (w x h) un punto de la escena dibujada en la textura cuadrada de lado `side`: la
/// cuenta inversa de lens.fs.
fn screen_from_scene(p: Vector2, side: f32, w: f32, h: f32) -> Vector2 {
    lens_point(p, side, Rectangle::new(0.0, 0.0, w, h), FISH_HALF)
}

/// Lo mismo, con la lente puesta en el rectángulo `dest` y con `half` grados en sus esquinas.
fn lens_point(p: Vector2, side: f32, dest: Rectangle, half: f32) -> Vector2 {
    let q = Vector2::new(p.x / side * 2.0 - 1.0, p.y / side * 2.0 - 1.0);
    let rho = q.length();
    let center = Vector2::new(dest.x + dest.width / 2.0, dest.y + dest.height / 2.0);
    if rho < 1e-5 {
        return center;
    }
    let half = half.to_radians();
    let theta = (rho * half.tan()).atan();
    let r = (theta / 2.0).sin() / (half / 2.0).sin();
    // r = 1 en las esquinas: medio diagonal del rectángulo
    let corner = (dest.width * dest.width + dest.height * dest.height).sqrt() / 2.0;
    center + q * (r / rho * corner)
}

/// Dibuja una escena con luz: primero la profundidad vista desde el farol (sombras), después la escena en la
/// textura cuadrada y por último la lente de ojo de pez (`half` grados en las esquinas: la cámara tiene que tener el
/// doble de campo de visión) dentro de `dest`. `hit` sacude la lente. Devuelve el lado de la textura (para ubicar
/// cosas encima).
#[allow(clippy::too_many_arguments)]
pub(super) fn render_view(screen: &mut RaylibDrawHandle, thread: &RaylibThread, art: &mut Art, items: &[Item],
                          extras: &[Extra], camera: Camera3D, light: Light, half: f32, dest: Rectangle, t: f32, hit: f32,
                          back: Backdrop) -> (f32, f32) {
    let lamp = light.lamp;
    art.shader.shader.set_shader_value(art.lamp_tint, light.tint);
    // Sombras: la profundidad de la escena vista desde el farol, mirando hacia abajo
    let light_cam = Camera3D::perspective(lamp, lamp - UP, FORWARD, 150.0);
    let mut shadow = art.shadow.take().expect("mapa de sombras");
    unsafe { ffi::rlSetClipPlanes(0.3, 25.0) };          // Más precisión de profundidad cerca del farol
    art.shader.begin_frame(lamp, light_cam.position, 0.0);
    art.shader.shader.set_shader_value(art.shadow_on, 0.0f32);
    let light_vp = {
        let mut sd = screen.begin_texture_mode(thread, &mut shadow);
        sd.clear_background(Color::WHITE);
        let mut d3 = sd.begin_mode3D(light_cam);
        let vp = unsafe { Matrix::from(ffi::rlGetMatrixModelview()) * Matrix::from(ffi::rlGetMatrixProjection()) };
        draw_items(&mut d3, art, items);
        vp
    };
    unsafe { ffi::rlSetClipPlanes(0.01, 1000.0) };
    let shadow_depth = shadow.depth.id;
    art.shadow = Some(shadow);
    // La escena se dibuja en una textura (con el campo de visión abierto) y después pasa por la lente de ojo de pez
    let mut target = art.target.take().expect("Art::prepare antes de dibujar");
    let (tw, th) = (target.texture.width as f32, target.texture.height as f32);
    let mut d = screen.begin_texture_mode(thread, &mut target);
    d.clear_background(if back.see_through { Color::BLANK } else { FOG_COLOR });
    if back.stars && !back.see_through {
        // Estrellas finitas, quietas y algunas titilando
        for i in 0..420 {
            let k = i as f32;
            let p = Vector2::new(noise(k * 1.31 + 0.2) * tw, noise(k * 2.17 + 5.1) * th);
            let a = 0.25 + 0.6 * noise(k * 0.71 + 9.0) * (0.7 + 0.3 * (t * (1.0 + noise(k) * 3.0) + k).sin());
            d.draw_circle_v(p, tw / 1400.0 * (1.0 + 1.5 * noise(k * 3.3)), fade(Color::new(220, 225, 255, 255), a));
        }
    }
    {
        art.shader.begin_frame(lamp, camera.position, light.fill);
        art.shader.shader.set_shader_value(art.rim, light.rim);
        art.shader.shader.set_shader_value(art.lamp_range, light.range);
        art.shader.shader.set_shader_value(art.lamp_power, light.power);
        art.shader.shader.set_shader_value(art.fog_density, light.fog);
        art.shader.shader.set_shader_value(art.shadow_on, 1.0f32);
        unsafe {
            ffi::SetShaderValueMatrix(*art.shader.shader.as_ref(), art.light_vp, light_vp.into());
            ffi::rlActiveTextureSlot(10);
            ffi::rlEnableTexture(shadow_depth);
            ffi::rlActiveTextureSlot(0);
        }
        let mut d3 = d.begin_mode3D(camera);
        let emissive = art.shader.shader.get_shader_location("emissive");
        art.shader.shader.set_shader_value(emissive, Vector3::zero());
        draw_items(&mut d3, art, items);
        for e in extras {
            match *e {
                Extra::Glow(p, r, c) => d3.draw_sphere(p, r, c),

            }
        }
    }
    art.shader.shader.set_shader_value(art.lamp_tint, Vector3::zero());
    art.shader.shader.set_shader_value(art.rim, 1.0f32);
    // El resplandor de su luz, cuando se la ve desde afuera
    if back.halo {
        let p = project_in(lamp, camera, tw, th);
        let mut b = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
        b.draw_circle_gradient(p.x as i32, p.y as i32, 140.0 * tw / 1000.0, Color::new(255, 150, 70, 40), Color::new(255, 150, 70, 0));
        b.draw_circle_gradient(p.x as i32, p.y as i32, 26.0 * tw / 1000.0, Color::new(255, 200, 130, 150), Color::new(255, 170, 90, 0));
    }
    drop(d);
    art.lens.set_shader_value(art.lens_aspect, dest.width / dest.height);
    art.lens.set_shader_value(art.lens_half, half.to_radians());
    art.lens.set_shader_value(art.lens_time, t);
    art.lens.set_shader_value(art.lens_hit, hit);
    art.lens.set_shader_value(art.lens_see, if back.see_through { 1.0f32 } else { 0.0 });
    {
        let mut l = screen.begin_shader_mode(&mut art.lens);
        l.draw_texture_pro(target.texture(), Rectangle::new(0.0, 0.0, tw, -th), dest, Vector2::zero(), 0.0, Color::WHITE);
    }
    art.target = Some(target);
    (tw, th)
}

pub fn glyph_color(g: Glyph) -> Color {
    let (r, gg, b) = g.color();
    Color::new(r, gg, b, 255)
}

pub(super) fn fade(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, (c.a as f32 * a.clamp(0.0, 1.0)) as u8)
}

pub(super) fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Un modelo con varias poses de la misma malla: cada vez que se dibuja, se mezclan dos.
pub struct Posed {
    model: SpaceModel,
    poses: Vec<Vec<(Vec<Vector3>, Vec<Vector3>)>>,
    scratch: Vec<(Vec<Vector3>, Vec<Vector3>)>,
    last: Option<(usize, usize, f32)>,
    /// Lo más bajo y el alto de la primera pose (en el espacio del archivo).
    pub(super) base: f32,
    pub(super) height: f32,
}

impl Posed {
    fn load(rl: &mut RaylibHandle, thread: &RaylibThread, shader: &SpaceShader, dir: &Path, names: &[&str])
        -> Result<Self, Error> {
        let mut model = None;
        let mut poses = Vec::new();
        for name in names {
            let m = SpaceModel::load(rl, thread, &dir.join(format!("{name}.glb")), shader, [0.0; 3],
                                     Fit::Same(Matrix::identity(), Vector3::zero()))?;
            let data = m.mesh_vertices();
            if let Some(first) = poses.first() {
                let shape = |d: &Vec<(Vec<Vector3>, Vec<Vector3>)>| d.iter().map(|(v, _)| v.len()).collect::<Vec<_>>();
                if shape(first) != shape(&data) {
                    return Err(format!("{name}: la pose no tiene la misma malla que las demás").into());
                }
            }
            poses.push(data);
            if model.is_none() {
                model = Some(m);
            }
        }
        let first = &poses[0];
        let ys = first.iter().flat_map(|(v, _)| v.iter().map(|p| p.y));
        let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(a, b), y| (a.min(y), b.max(y)));
        Ok(Self { model: model.unwrap(), scratch: first.clone(), poses, last: None, base: lo, height: hi - lo })
    }

    /// El centro de cada ojo (izquierdo y derecho, en el espacio del archivo) en la última pose dibujada: los vértices
    /// del material `name`, partidos por el medio.
    fn eye_centers(&self, name: &str) -> Option<[Vector3; 2]> {
        let points: Vec<Vector3> = self.model.part_meshes(name).into_iter()
            .flat_map(|m| self.scratch[m].0.iter().copied()).collect();
        let (lo, hi) = points.iter().fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.x), b.max(p.x)));
        let mid = (lo + hi) / 2.0;
        let center = |left: bool| {
            let side: Vec<&Vector3> = points.iter().filter(|p| (p.x < mid) == left).collect();
            (!side.is_empty()).then(|| side.iter().fold(Vector3::zero(), |a, p| a + **p) * (1.0 / side.len() as f32))
        };
        Some([center(true)?, center(false)?])
    }

    /// Mezcla las poses `a` y `b` (t de 0 a 1) en la malla que se dibuja.
    fn pose(&mut self, a: usize, b: usize, t: f32) {
        let key = (a, b, (t * 200.0).round() / 200.0);
        if self.last == Some(key) {
            return;
        }
        self.last = Some(key);
        let t = key.2;
        for (mi, (v, n)) in self.scratch.iter_mut().enumerate() {
            let ((va, na), (vb, nb)) = (&self.poses[a][mi], &self.poses[b][mi]);
            for k in 0..v.len() {
                v[k] = va[k].lerp(vb[k], t);
                n[k] = na[k].lerp(nb[k], t).normalized();
            }
        }
        self.model.set_mesh_vertices(&self.scratch);
    }

    /// Como `pose`, y además derretida `u` (0 a 1): se aplasta hacia el piso, abajo se desparrama en un charco y
    /// chorrea en hilos.
    fn pose_melted(&mut self, a: usize, b: usize, t: f32, u: f32) {
        self.last = None;
        let (base, height) = (self.base, self.height);
        for (mi, (v, n)) in self.scratch.iter_mut().enumerate() {
            let ((va, na), (vb, nb)) = (&self.poses[a][mi], &self.poses[b][mi]);
            for k in 0..v.len() {
                let p = va[k].lerp(vb[k], t);
                let h = ((p.y - base) / height).clamp(0.0, 1.0);
                // Hilos que chorrean: algunas franjas caen más que otras
                let drip = ((p.x * 41.0).sin() * (p.z * 37.0).sin()).max(0.0);
                let fall = (h * (1.0 - 0.95 * u) - drip * 0.12 * u * h).max(0.0);
                let spread = 1.0 + 1.4 * u * (1.0 - fall).powi(3);
                v[k] = Vector3::new(p.x * spread, base + fall * height, p.z * spread);
                n[k] = na[k].lerp(nb[k], t).normalized();
            }
        }
        self.model.set_mesh_vertices(&self.scratch);
    }
}

/// Cuánto se le apagaron los ojos a una maldecida, según hace cuánto se empezó a derretir.
fn melt_dark(x: f32) -> f32 {
    smooth((x - 0.2) / 0.4)
}

/// Un framebuffer con solo profundidad (en una textura que el shader puede leer): el mapa de sombras.
fn load_shadow_map(size: i32) -> RenderTexture2D {
    unsafe {
        let id = ffi::rlLoadFramebuffer();
        ffi::rlEnableFramebuffer(id);
        let depth = ffi::rlLoadTextureDepth(size, size, false);
        ffi::rlFramebufferAttach(id, depth, ffi::rlFramebufferAttachType::RL_ATTACHMENT_DEPTH as i32,
                                 ffi::rlFramebufferAttachTextureType::RL_ATTACHMENT_TEXTURE2D as i32, 0);
        ffi::rlDisableFramebuffer();
        let tex = |id: u32, format: i32| ffi::Texture { id, width: size, height: size, mipmaps: 1, format };
        RenderTexture2D::from_raw(ffi::RenderTexture2D { id, texture: tex(0, 7), depth: tex(depth, 19) })
    }
}

/// El farol en este momento: tiembla apenas y respira como una llama; con una criatura encima, tartamudea y casi se
/// apaga. Devuelve dónde está y su fuerza (1 = normal).
fn lamp_now(t: f32, s: &Session) -> (Vector3, f32) {
    let k = (t * 18.0).floor();
    let mut power = 0.86 + 0.07 * (t * 7.3).sin() + 0.04 * (t * 13.7 + 1.3).sin() + 0.1 * (noise(k) - 0.5);
    let near = s.creatures.iter().filter(|c| c.state == State::Approaching).map(|c| c.distance()).fold(f32::MAX, f32::min);
    if near < 2.3 && noise((t * 9.0).floor() + 4.0) > 0.45 {
        power *= 0.06 + 0.25 * noise((t * 40.0).floor() + 9.0);
    }
    let jitter = Vector3::new(noise(k + 1.7) - 0.5, 0.0, noise(k + 5.3) - 0.5) * 0.03;
    (LAMP + jitter, power)
}

pub(super) enum Who {
    Witch,
    /// La bruja feliz del final (ojos cerrados de felicidad), bailando.
    HappyWitch,
    Creature(Kind),
    /// La bruja en su escoba (la nave) y el planeta calavera: sin poses (para la historieta). La nave en el espacio del
    /// archivo y el papelito, para la invitación.
    Broom,
    Rider,
    Note,
    Planet,
}

pub(super) enum Shape {
    Blend(usize, usize, f32),
    Melt(usize, usize, f32, f32),
    Still,
}

/// Un modelo para dibujar en este cuadro (se dibuja dos veces: desde el farol, para las sombras, y desde la cámara).
pub(super) struct Item {
    pub who: Who,
    pub shape: Shape,
    pub m: Matrix,
    pub gone: f32,
    pub goo: f32,
    pub boost: f32,
    /// 1 = todavía en lo oscuro, sin luz ninguna (ni los ojos); 0 = ya la agarró la luz.
    pub hidden: f32,
}

impl Item {
    /// Un modelo quieto o bailando, sin derretirse ni esconderse.
    pub fn plain(who: Who, shape: Shape, m: Matrix) -> Self {
        // El planeta, a oscuras salvo que se diga otra cosa (`boost`: su brillo propio)
        let boost = if matches!(who, Who::Planet) { 0.0 } else { 1.0 };
        Self { who, shape, m, gone: 0.0, goo: 0.0, boost, hidden: 0.0 }
    }
}

/// La luz de una escena: el farol (dónde, cuánto, de qué color: cero es el cálido de siempre, y a qué distancia cae
/// a la mitad), la niebla, un relleno parejo (0 en el juego: lo que no mira al farol queda negro) y cuánto da la luz
/// fría de atrás (1 en el juego).
pub(super) struct Light {
    pub lamp: Vector3,
    pub power: f32,
    pub tint: Vector3,
    pub range: f32,
    pub fog: f32,
    pub fill: f32,
    pub rim: f32,
}

/// Lo que va atrás de los modelos: estrellas (en el espacio) y el resplandor del farol (cuando se lo ve de afuera).
/// Con `see_through`, lo que no tiene modelos queda transparente: se ve lo que ya estaba dibujado en la pantalla
/// (el logo del título, atrás de la bruja).
pub(super) struct Backdrop {
    pub stars: bool,
    pub halo: bool,
    pub see_through: bool,
}

/// Cosas que brillan solas, sin luz (la bola de espejos del final).
pub(super) enum Extra {
    Glow(Vector3, f32, Color),
}

/// Lo que hay en la escena este cuadro: la bruja (si todavía no estamos en sus ojos), las criaturas y el susto.
fn scene_items(art: &Art, fx: &Fx, s: &Session, view: View, camera: Camera3D) -> Vec<Item> {
    let t = fx.time;
    let mut items = Vec::new();
    let inside = match view {
        View::Pov => true,
        View::Turn(u) => u > 0.82,
        View::Show(_) => false,
    };
    if !inside {
        let beat = t / BEAT;
        let i = beat.floor() as usize;
        items.push(Item {
            who: Who::Witch,
            // Bailando, o levantando el dedo para advertir (`fx.warn`)
            shape: if fx.warn > 0.0 {
                Shape::Blend(WITCH_DANCE[i % 6], WITCH_WARN, smooth(fx.warn))
            } else {
                Shape::Blend(WITCH_DANCE[i % 6], WITCH_DANCE[(i + 1) % 6], smooth(beat.fract() * 1.6 - 0.3))
            },
            m: place(Vector3::zero(), FORWARD, WITCH_H / art.witch.height, art.witch.base),
            gone: 0.0,
            goo: 0.0,
            boost: 1.0,
            hidden: 0.0,
        });
    }
    for c in &s.creatures {
        let calm = fx.calm.get(&c.id).copied();
        // Corriendo hacia ella: pasos rápidos
        let phase = t * 7.5 + c.id as f32 * 1.7;
        // Los ojos del trío final, encendidos
        let boost = if c.boss { 3.5 } else { 1.0 };
        let (x, y) = shown_pos(c, t);
        let mut at = point(x, y);
        let mut front = -at;
        let mut goo = 0.0;
        let shape = match calm {
            // Maldecida: un grito mudo, unos sacudones...
            Some(x) if x < 0.15 => {
                let k = (t * 37.0).floor() + c.id as f32 * 11.0;
                at = at + Vector3::new(noise(k) - 0.5, 0.0, noise(k + 3.1) - 0.5) * 0.08;
                let yaw = (noise(k + 7.7) - 0.5) * 0.4;
                front = Vector3::new(front.x * yaw.cos() - front.z * yaw.sin(), 0.0, front.x * yaw.sin() + front.z * yaw.cos());
                Shape::Blend(2, 3, noise(k + 5.5))
            }
            // ...y se derrite: se aplasta hacia el piso, se vuelve negro y mojado, chorrea y se desparrama, y al
            // final el charco se hunde en el piso
            Some(x) => {
                goo = smooth((x - 0.1) / 0.45);
                at = at - UP * (0.6 * smooth((x - 1.05) / 0.35));
                Shape::Melt(0, 1, 0.5, smooth((x - 0.15) / 0.95))
            }
            // El fantasma y el diablo, como una película rota: se quedan en un cuadro y saltan al siguiente, y en
            // cada salto la cabeza y el cuerpo quedan torcidos para cualquier lado
            None if stutters(c) => {
                let k = stutter_frame(c, t);
                front = stutter_front(front, k);
                Shape::Blend(0, 1, noise(k * 0.77 + 5.1))
            }
            None => Shape::Blend(0, 1, 0.5 + 0.5 * phase.sin()),
        };
        let base = match c.kind {
            Kind::Devil => art.devil.base,
            Kind::Ghost => art.ghost.base,
            Kind::Demon => art.demon.base,
        };
        items.push(Item { who: Who::Creature(c.kind), shape, m: place(at, front, creature_scale(c.kind), base),
                          gone: calm.map_or(0.0, melt_dark), goo, boost, hidden: hidden(s, c) });
    }
    // El susto de mentira: pegada a la cara, mirándola de frente (la cara se tiene que ver bien), con tirones cortos
    // cuadro a cuadro y algún cuadro en que no está
    if let Some((kind, k)) = fx.fright {
        let frame = (k * 14.0).floor();
        let r = |i: f32| noise(frame * 3.17 + i * 1.618 + 50.0);
        if r(1.0) > 0.08 {
            let fwd = (camera.target - camera.position).normalized();
            let side = fwd.cross(UP).normalized();
            // No tan pegada que el gran angular la deforme: la cara se tiene que leer
            let near = 1.0 - 0.25 * (k / FRIGHT) + (r(2.0) - 0.5) * 0.05;
            // La cara, justo donde mira la bruja: con los poses en que tiene la cabeza derecha (el diablo, parado, no
            // doblado acechando) y su altura en esos poses
            let face = camera.position + fwd * near + side * (r(3.0) - 0.5) * 0.06;
            // El demonio camina con la cabeza para atrás: se lo inclina hacia adelante para que la cara mire
            let (poses, face_h, pitch) = match kind {
                Kind::Devil => ((2, 3), 1.05 * creature_scale(kind), 0.0),
                Kind::Ghost => ((0, 1), 1.1 * creature_scale(kind), 0.0),
                Kind::Demon => ((0, 1), 0.86 * art.height(kind), 0.4),
            };
            let at = Vector3::new(face.x, face.y - face_h * (1.0 + (r(4.0) - 0.5) * 0.02), face.z);
            // Mirándola de frente (hacia la cámara), con un temblor chico
            let yaw = (r(5.0) - 0.5) * 0.2;
            let to_cam = camera.position - at;
            let front = Vector3::new(to_cam.x * yaw.cos() - to_cam.z * yaw.sin(), 0.0, to_cam.x * yaw.sin() + to_cam.z * yaw.cos());
            let base = match kind {
                Kind::Devil => art.devil.base,
                Kind::Ghost => art.ghost.base,
                Kind::Demon => art.demon.base,
            };
            // Un tirón distinto en cada cuadro (la cara no se deforma)
            items.push(Item { who: Who::Creature(kind), shape: Shape::Blend(poses.0, poses.1, r(8.0)),
                              m: place_tilted(at, front, creature_scale(kind) * (0.97 + 0.06 * r(9.0)), base, pitch, face_h), gone: 0.0,
                              goo: 0.0, boost: 2.4, hidden: 0.0 });
        }
    }
    // El susto: la que llegó se tira encima de la cámara
    if let Some((kind, k)) = fx.scare {
        let u = (k / SCARE).min(1.0);
        let fwd = (camera.target - camera.position).normalized();
        let height = art.height(kind);
        let face = camera.position + fwd * (1.5 - 1.25 * smooth(u * 2.2));
        let at = Vector3::new(face.x, camera.position.y - height * 0.9, face.z);
        let base = match kind {
            Kind::Devil => art.devil.base,
            Kind::Ghost => art.ghost.base,
            Kind::Demon => art.demon.base,
        };
        items.push(Item { who: Who::Creature(kind), shape: Shape::Blend(2, 3, 0.5 + 0.5 * (t * 9.0).sin()),
                          m: place(at, -fwd, creature_scale(kind), base), gone: 0.0, goo: 0.0, boost: 1.6, hidden: 0.0 });
    }
    items
}

/// Los que se mueven a saltos: el fantasma, el diablo y los tríos finales (el demonio común no).
fn stutters(c: &Creature) -> bool {
    c.boss || matches!(c.kind, Kind::Ghost | Kind::Devil)
}

/// Cuadros por segundo del fantasma y el diablo: se mueven a saltos, como animación cuadro por cuadro rota.
const STUTTER_FPS: f32 = 6.0;

/// El desfase de la película rota de una criatura: cada una el suyo, salvo el trío final, que va en unísono.
fn stutter_offset(c: &Creature) -> f32 {
    if c.boss { 0.0 } else { c.id as f32 * 0.37 }
}

/// En qué cuadro de su película rota está una criatura.
fn stutter_frame(c: &Creature, t: f32) -> f32 {
    (t * STUTTER_FPS + stutter_offset(c)).floor()
}

/// Hacia dónde queda torcida una criatura en un cuadro de su película rota.
fn stutter_front(front: Vector3, k: f32) -> Vector3 {
    let yaw = (noise(k * 1.31 + 2.2) - 0.5) * 0.9;
    Vector3::new(front.x * yaw.cos() - front.z * yaw.sin(), 0.0, front.x * yaw.sin() + front.z * yaw.cos())
}

/// Dónde se ve a una criatura: donde está, salvo los que van a saltos, que se quedan quietos en su cuadro (un poco
/// atrás de donde ya están) y saltan al siguiente; algunos cuadros los saltean de golpe y llegan más adelante.
pub fn shown_pos(c: &Creature, t: f32) -> (f32, f32) {
    if !stutters(c) || c.state != State::Approaching {
        return c.pos;
    }
    let k = stutter_frame(c, t);
    let lag = (t * STUTTER_FPS + stutter_offset(c) - k) / STUTTER_FPS;
    let d = c.distance().max(1e-3);
    let back = c.speed * lag * if noise(k * 2.9 + 0.4) > 0.75 { 0.0 } else { 1.0 };
    // El trío, además, se bambolea de costado a la par, como una sola cosa rara
    let sway = if c.boss { (k * std::f32::consts::PI / 3.0).sin() * 0.4 } else { 0.0 };
    (c.pos.0 * (d + back) / d + sway, c.pos.1 * (d + back) / d)
}

/// Cuánto le falta a una criatura para salir de lo oscuro: vienen sin que se las vea y, al llegar a `reveal_radius`, la
/// luz fría del borde las agarra en un momento (en el último metro y poco).
fn hidden(s: &Session, c: &Creature) -> f32 {
    smooth((c.distance() - s.balance.reveal_radius + 1.2) / 1.2)
}

fn draw_items(d3: &mut impl RaylibDraw3D, art: &mut Art, items: &[Item]) {
    let (goo_loc, hidden_loc) = (art.goo, art.hidden);
    for it in items {
        // Los modelos sin poses: la escoba (con su estela que brilla) y el planeta
        match it.who {
            Who::Broom => {
                art.broom.draw_with(d3, &mut art.shader, it.m, &|_, e| e);
                continue;
            }
            Who::Rider => {
                // La estela brilla más que en la nave del sistema solar (las estrellas, las brasas y los brillitos)
                art.rider.draw_with(d3, &mut art.shader, it.m, &|n, e| {
                    if n.contains("BroomStar") || n.contains("BroomEmber") || n.contains("BroomGlitter") { e * TRAIL_GLOW } else { e }
                });
                continue;
            }
            Who::Note => {
                art.note.draw(d3, &mut art.shader, it.m);
                continue;
            }
            Who::Planet => {
                // El brillo propio del planeta (la fiesta en las cuencas), según `boost`: apagado si es 0
                let glow = it.boost;
                art.planet.draw_with(d3, &mut art.shader, it.m, &|_, e| e * glow);
                continue;
            }
            _ => {}
        }
        let (posed, shader) = match it.who {
            Who::Witch => (&mut art.witch, &mut art.shader),
            Who::HappyWitch => (&mut art.witch_happy, &mut art.shader),
            Who::Creature(k) => art.posed(k),
            Who::Broom | Who::Rider | Who::Note | Who::Planet => unreachable!(),
        };
        match it.shape {
            Shape::Blend(a, b, t) => posed.pose(a, b, t),
            Shape::Melt(a, b, t, u) => posed.pose_melted(a, b, t, u),
            Shape::Still => {}
        }
        shader.shader.set_shader_value(goo_loc, it.goo);
        shader.shader.set_shader_value(hidden_loc, it.hidden);
        let (gone, boost) = (it.gone, it.boost);
        match it.who {
            Who::Witch | Who::HappyWitch =>
                posed.model.draw_with(d3, shader, it.m, &|n, e| if n.contains("Bead") || n.contains("WitchEyes") { e } else { e * 0.3 }),
            _ => posed.model.draw_with(d3, shader, it.m, &|n, e| eyes(n, e, gone) * boost),
        }
        shader.shader.set_shader_value(goo_loc, 0.0f32);
        shader.shader.set_shader_value(hidden_loc, 0.0f32);
    }
}

/// Del piso del juego (la bruja en el origen, +y hacia adelante, adonde mira) al mundo de raylib (Y arriba).
fn point(x: f32, y: f32) -> Vector3 {
    Vector3::new(x, 0.0, -y)
}
pub(super) const UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);
pub(super) const FORWARD: Vector3 = Vector3::new(0.0, 0.0, -1.0);

/// Cuánto más brilla la estela de la escoba en la invitación.
const TRAIL_GLOW: f32 = 2.4;

/// El brazo levantado de la bruja de la nave (en el espacio del archivo): el hombro, el codo y desde dónde empieza
/// (lo que está arriba de `ARM_ABOVE` y más afuera que el hombro; más abajo, del mismo lado, están las piernas).
const SHOULDER: Vector3 = Vector3::new(-0.12, 0.29, 0.0);
const ELBOW: Vector3 = Vector3::new(-0.265, 0.30, 0.0);
const ARM_ABOVE: f32 = 0.26;
/// Cuánto gira para bajar el brazo: desde el hombro y desde el codo (radianes).
const ARM_DOWN: (f32, f32) = (1.1, 0.9);

/// Cómo se mueve un punto del brazo levantado: todo el brazo sube y baja desde el hombro (`lift`), el antebrazo
/// saluda girando desde el codo (`wave`) y, con `down` (de 0 a 1), el brazo se baja al costado (desde el hombro, y el
/// codo se estira), con el doblez suave. `None` si el punto no es del brazo.
pub(super) fn arm_bend(p: Vector3, lift: f32, wave: f32, down: f32) -> Option<Matrix> {
    if p.y < ARM_ABOVE || p.x > SHOULDER.x {
        return None;
    }
    let around = |pivot: Vector3, axis: Vector3, angle: f32| {
        Matrix::translate(-pivot.x, -pivot.y, -pivot.z) * Matrix::rotate(axis, angle) * Matrix::translate(pivot.x, pivot.y, pivot.z)
    };
    let w_arm = smooth((SHOULDER.x - p.x) / 0.07);
    let w_fore = smooth((ELBOW.x + 0.005 - p.x) / 0.03);
    let z = Vector3::new(0.0, 0.0, 1.0);
    Some(around(ELBOW, Vector3::new(1.0, 0.0, 0.0), wave * w_fore) * around(ELBOW, z, ARM_DOWN.1 * down * w_fore)
        * around(SHOULDER, z, (ARM_DOWN.0 * down - lift) * w_arm))
}

/// Ruido barato entre 0 y 1 (para las convulsiones).
pub(super) fn noise(a: f32) -> f32 {
    ((a.sin() * 43_758.547).fract() + 1.0).fract()
}

/// Matriz de un modelo con la base en `at` (en el mundo), mirando hacia `front`.
pub(super) fn place(at: Vector3, front: Vector3, scale: f32, base: f32) -> Matrix {
    let front = Vector3::new(front.x, 0.0, front.z).normalized();
    let right = UP.cross(front);
    let basis = Matrix {
        m0: right.x, m1: right.y, m2: right.z, m3: 0.0,
        m4: UP.x, m5: UP.y, m6: UP.z, m7: 0.0,
        m8: front.x, m9: front.y, m10: front.z, m11: 0.0,
        m12: at.x, m13: at.y, m14: at.z, m15: 1.0,
    };
    Matrix::translate(0.0, -base, 0.0) * Matrix::scale(scale, scale, scale) * basis
}

/// Como `place`, pero inclinada hacia adelante `pitch` radianes alrededor de la altura `pivot` (en el mundo, sobre
/// la base): lo que está a esa altura queda en su lugar.
fn place_tilted(at: Vector3, front: Vector3, scale: f32, base: f32, pitch: f32, pivot: f32) -> Matrix {
    Matrix::translate(0.0, -base, 0.0) * Matrix::scale(scale, scale, scale) * Matrix::translate(0.0, -pivot, 0.0)
        * Matrix::rotate_x(pitch) * Matrix::translate(0.0, pivot, 0.0) * place(at, front, 1.0, 0.0)
}

/// Desde dónde se mira: la bruja bailando (de frente; con el acercamiento de 0 a 1 cuando advierte), el giro hasta
/// sus ojos (0 a 1), o sus ojos.
#[derive(Clone, Copy, PartialEq)]
pub enum View {
    Show(f32),
    Turn(f32),
    Pov,
}

pub struct Art {
    shader: SpaceShader,
    lamp_range: i32,
    goo: i32,
    hidden: i32,
    lamp_power: i32,
    fog_density: i32,
    shadow_on: i32,
    light_vp: i32,
    /// Profundidad de la escena vista desde el farol (para las sombras).
    shadow: Option<RenderTexture2D>,
    pub(super) witch: Posed,
    pub(super) witch_happy: Posed,
    /// La bruja en su escoba (la nave) y el planeta calavera, para la historieta; y, para la invitación, la nave en el
    /// espacio del archivo (el origen en el asiento de la bruja, mirando hacia +z) y el papelito.
    broom: SpaceModel,
    rider: SpaceModel,
    /// Las mallas de la nave como vienen (para moverle el brazo y la falda cada cuadro) y cuáles son de piel y de negro.
    rider_rest: Vec<(Vec<Vector3>, Vec<Vector3>)>,
    rider_skin: Vec<usize>,
    rider_black: Vec<usize>,
    note: SpaceModel,
    planet: SpaceModel,
    lamp_tint: i32,
    rim: i32,
    devil: Posed,
    ghost: Posed,
    demon: Posed,
    pub font: Font,
    /// La misma letra, cargada grande: para el cartel enorme de cada oleada.
    pub(super) font_big: Font,
    lens: Shader,
    lens_aspect: i32,
    lens_half: i32,
    lens_time: i32,
    lens_hit: i32,
    lens_see: i32,
    /// La escena se dibuja acá y después pasa por la lente.
    target: Option<RenderTexture2D>,
    /// La transición de círculo de la invitación: la capa de la carta (del tamaño de la ventana) y el shader que le
    /// abre el agujero.
    pub(super) overlay: Option<RenderTexture2D>,
    pub(super) iris: Shader,
    pub(super) iris_center: i32,
    pub(super) iris_radius: i32,
    /// Las vidas: el sombrero de bruja violeta (la que queda) y gris (la que se perdió), dibujos de Cami.
    life_full: Texture2D,
    life_empty: Texture2D,
    /// El logo de Hexbound (dibujo de Cami), para el título de la historieta.
    pub(super) title: Texture2D,
}

impl Art {
    /// Antes de dibujar: la textura cuadrada de la escena, según el alto de la ventana (ver `SUPERSAMPLE`).
    /// Le da vida a la nave (la bruja en la escoba, que es una sola pose): el brazo levantado se mueve desde el hombro
    /// (`lift`, radianes), saluda desde el codo (`wave`) y se baja (`down`, de 0 a 1), y la parte de atrás de la falda
    /// flamea con el viento.
    pub(super) fn animate_rider(&mut self, lift: f32, wave: f32, down: f32, t: f32) {
        let mut data = self.rider_rest.clone();
        for &m in &self.rider_skin {
            let (v, n) = &mut data[m];
            for k in 0..v.len() {
                if let Some(bend) = arm_bend(v[k], lift, wave, down) {
                    v[k] = v[k].transform_with(bend);
                    n[k] = (n[k].transform_with(bend) - Vector3::zero().transform_with(bend)).normalized();
                }
            }
        }
        for &m in &self.rider_black {
            let (v, _) = &mut data[m];
            for p in v.iter_mut() {
                // Lo de atrás y abajo (la falda larga, no el sombrero): ondas que corren hacia atrás
                let back = ((-p.z - 0.1) / 0.3).clamp(0.0, 1.0);
                if back > 0.0 && p.y < 0.1 {
                    let k = back * back;
                    p.y += 0.018 * k * (p.z * 16.0 + t * 13.0).sin();
                    p.x += 0.012 * k * (p.z * 11.0 + t * 9.0 + 1.3).sin();
                }
            }
        }
        self.rider.set_mesh_vertices(&data);
    }

    pub fn prepare(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<(), Error> {
        let side = ((rl.get_render_height() as f32 * SUPERSAMPLE) as i32).min(4096);   // Tope de la placa de video
        if self.target.as_ref().is_none_or(|t| t.texture.width != side) {
            let t = rl.load_render_texture(thread, side as u32, side as u32)?;
            t.texture().set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            self.target = Some(t);
        }
        let (w, h) = (rl.get_screen_width(), rl.get_screen_height());
        if self.overlay.as_ref().is_none_or(|o| o.texture.width != w || o.texture.height != h) {
            self.overlay = Some(rl.load_render_texture(thread, w as u32, h as u32)?);
        }
        Ok(())
    }

    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, root: &Path) -> Result<Self, Error> {
        let mut shader = SpaceShader::load_custom(rl, thread, include_str!("spot.fs"))?;
        let lamp_range = shader.shader.get_shader_location("lightRange");
        let (goo, hidden) = (shader.shader.get_shader_location("goo"), shader.shader.get_shader_location("hidden"));
        let (lamp_power, fog_density) = (shader.shader.get_shader_location("lampPower"), shader.shader.get_shader_location("fogDensity"));
        let (shadow_on, light_vp) = (shader.shader.get_shader_location("shadowOn"), shader.shader.get_shader_location("lightVP"));
        let shadow_map = shader.shader.get_shader_location("shadowMap");
        shader.shader.set_shader_value(shadow_map, 10i32);       // El mapa de sombras va en la unidad de textura 10
        let dir = root.join("assets/models/minigames/halloween");
        let witch = Posed::load(rl, thread, &shader, &dir,
                                &["witch_solo_a", "witch_solo_b", "witch_solo_c", "witch_hop", "witch_sway", "witch_spin", "witch_cast", "witch_warn", "witch_stand",
])?;
        let witch_happy = Posed::load(rl, thread, &shader, &dir, &["witch_happy_solo_a", "witch_happy_solo_b", "witch_happy_solo_c",
                                                                  "witch_happy_hop", "witch_happy_sway", "witch_happy_spin"])?;
        let broom = SpaceModel::load(rl, thread, &root.join("assets/models/ships/halloween/exports/witch_broom.glb"), &shader,
                                     [0.0; 3], Fit::Length(1.6))?;
        let raw = || Fit::Same(Matrix::identity(), Vector3::zero());
        let rider = SpaceModel::load(rl, thread, &root.join("assets/models/ships/halloween/exports/witch_broom.glb"), &shader,
                                     [0.0; 3], raw())?;
        let note = SpaceModel::load(rl, thread, &dir.join("note.glb"), &shader, [0.0; 3], raw())?;
        let rider_rest = rider.mesh_vertices();
        let (rider_skin, rider_black) = (rider.part_meshes("HW_WitchSkin"), rider.part_meshes("HW_WitchBlack"));
        let planet = SpaceModel::load(rl, thread, &root.join("assets/models/planets/halloween/exports/halloween_planet.glb"),
                                      &shader, [0.0; 3], Fit::Radius(1.0))?;
        let lamp_tint = shader.shader.get_shader_location("lampTint");
        let rim = shader.shader.get_shader_location("rimPower");
        shader.shader.set_shader_value(rim, 1.0f32);
        let devil = Posed::load(rl, thread, &shader, &dir, &["devil_walk_a", "devil_walk_b", "devil_dance_a", "devil_dance_b"])?;
        let ghost = Posed::load(rl, thread, &shader, &dir, &["ghost_float_a", "ghost_float_b", "ghost_dance_a", "ghost_dance_b"])?;
        let demon = Posed::load(rl, thread, &shader, &dir, &["demon_walk_a", "demon_walk_b", "demon_dance_a", "demon_dance_b"])?;
        let font = rl.load_font_ex(thread, &root.join("assets/fonts/JollyLodger-Regular.ttf").to_string_lossy(), 72, None)?;
        unsafe { ffi::SetTextureFilter(font.texture, TextureFilter::TEXTURE_FILTER_BILINEAR as i32) };
        let lens = rl.load_shader_from_memory(thread, None, Some(include_str!("lens.fs")));
        let lens_aspect = lens.get_shader_location("aspect");
        let lens_half = lens.get_shader_location("fovHalf");
        let (lens_time, lens_hit) = (lens.get_shader_location("time"), lens.get_shader_location("hit"));
        let lens_see = lens.get_shader_location("seeThrough");
        let iris = rl.load_shader_from_memory(thread, None, Some(include_str!("iris.fs")));
        let (iris_center, iris_radius) = (iris.get_shader_location("center"), iris.get_shader_location("radius"));
        let shadow = Some(load_shadow_map(SHADOW_SIZE));
        let mut life = |name: &str| -> Result<Texture2D, Error> {
            let t = rl.load_texture(thread, &root.join("assets/ui/halloween").join(name).to_string_lossy())?;
            t.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            Ok(t)
        };
        let (life_full, life_empty) = (life("life_full.png")?, life("life_empty.png")?);
        let title = life("title.png")?;
        let font_big = rl.load_font_ex(thread, &root.join("assets/fonts/JollyLodger-Regular.ttf").to_string_lossy(), 256, None)?;
        font_big.texture().set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        Ok(Self { shader, lamp_range, goo, hidden, lamp_power, fog_density, shadow_on, light_vp, shadow, witch, witch_happy,
                  broom, rider, rider_rest, rider_skin, rider_black, note, planet, lamp_tint, rim, devil, ghost, demon, font, lens, lens_aspect, lens_half,
                  lens_time, lens_hit, lens_see, target: None, overlay: None, iris, iris_center, iris_radius, life_full, life_empty, font_big, title })
    }

    /// La cámara de cada momento. Desde los ojos (a la altura real de los de la bruja, con un campo de visión
    /// natural, así las criaturas se ven de su tamaño), se mece apenas con el baile; `look` es hacia dónde gira la
    /// cabeza el jugador (grados: a los costados y arriba/abajo).
    pub fn camera(&self, view: View, time: f32, look: (f32, f32)) -> Camera3D {
        let eye = UP * EYE_H;
        let sway = Vector3::new((time * std::f32::consts::PI / BEAT).sin() * 0.04, (time * 2.0 * std::f32::consts::PI / BEAT).sin().abs() * 0.02, 0.0);
        let (yaw, pitch) = (look.0.to_radians(), (look.1 + 16.0).to_radians());
        let dir = Vector3::new(-yaw.sin() * pitch.cos(), pitch.sin(), -yaw.cos() * pitch.cos());
        // La textura cubre todo el ángulo del ojo de pez de lado a lado
        let fov = 2.0 * FISH_HALF;
        let pov = Camera3D::perspective(eye + sway, eye + sway + dir * 6.0, UP, fov);
        // De frente a la bruja, que mira hacia adelante, cerca (el ojo de pez la aleja)
        let show_pos = FORWARD * 1.7 + UP * 1.05;
        let show_target = UP * 0.7;
        match view {
            View::Pov => pov,
            // Al advertir se acerca un poco, a la altura de su cara y su dedo levantado, y se corre para que ella quede
            // a la izquierda (a la derecha va el mensaje)
            View::Show(z) => {
                let z = smooth(z);
                let right = FORWARD.cross(UP) * -0.42 * z;
                Camera3D::perspective(show_pos.lerp(FORWARD * 1.05 + UP * 1.0, z) + right,
                                      show_target.lerp(UP * 0.92, z) + right, UP, fov)
            }
            View::Turn(u) => {
                // Gira media vuelta alrededor de ella mientras se acerca, y termina en sus ojos
                let s = smooth(u);
                let a = std::f32::consts::PI * s;
                let off = FORWARD * 1.7;
                let rot = Vector3::new(off.x * a.cos() + off.z * a.sin(), 0.0, -off.x * a.sin() + off.z * a.cos()) * (1.0 - s);
                let pos = rot + UP * (1.05 + (pov.position.y - 1.05) * s);
                Camera3D::perspective(pos, show_target.lerp(pov.target, s), UP, fov)
            }
        }
    }

    fn posed(&mut self, kind: Kind) -> (&mut Posed, &mut SpaceShader) {
        match kind {
            Kind::Devil => (&mut self.devil, &mut self.shader),
            Kind::Ghost => (&mut self.ghost, &mut self.shader),
            Kind::Demon => (&mut self.demon, &mut self.shader),
        }
    }

    /// Alto de una criatura en la escena (ya agrandada).
    fn height(&self, kind: Kind) -> f32 {
        creature_scale(kind) * match kind {
            Kind::Devil => self.devil.height,
            Kind::Ghost => self.ghost.height,
            Kind::Demon => self.demon.height,
        }
    }

    /// Dónde flota el glifo de una criatura y adonde van los hechizos: arriba de la cabeza del diablo y del
    /// fantasma, delante del pecho del demonio.
    pub fn head(&self, kind: Kind, pos: (f32, f32)) -> Vector3 {
        let at = point(pos.0, pos.1);
        let toward = (-at).normalized() * 0.35;
        // El diablo y el fantasma lo llevan arriba de la cabeza; el demonio, delante del pecho
        let up = if kind == Kind::Demon { 0.62 } else { 1.06 };
        at + toward + UP * (self.height(kind) * up)
    }

    /// De dónde salen los hechizos: la mano de la bruja, abajo a la derecha de hacia donde mira.
    pub fn witch_hand(&self, camera: Camera3D) -> Vector3 {
        let fwd = (camera.target - camera.position).normalized();
        let right = fwd.cross(UP).normalized();
        camera.position + fwd * 0.55 + right * 0.22 - UP * 0.33
    }
}

/// Brillo propio en lo oscuro: solo los ojos. `fade` lo apaga (una criatura calmada que se va).
fn eyes(name: &str, e: Vector3, fade: f32) -> Vector3 {
    let k = 1.0 - fade;
    match name {
        "HW_GameDemonEyes" => e * k,
        // La cara tallada del diablo: brasa adentro, como una calabaza mala
        "HW_GameDevilEyes" => Vector3::new(2.3, 0.38, 0.05) * k,
        // El fantasma: los blancos de los ojos y el azul, mirando fijo desde lo oscuro
        "HW_GameGhostSclera" => Vector3::new(1.15, 1.12, 1.05) * k,
        "HW_GameGhostIris" => e * 1.4 * k,
        _ => Vector3::zero(),
    }
}

// --- Efectos -----------------------------------------------------------------------------------------------------

struct Bolt {
    from: Vector3,
    to: Vector3,
    color: Color,
    t: f32,
}

/// Lo que se ve y no cambia las reglas: trazos que se apagan, hechizos en vuelo, sustos y carteles.
#[derive(Default)]
pub struct Fx {
    pub time: f32,
    bolts: Vec<Bolt>,
    /// Trazo terminado que se apaga: puntos, color y tiempo restante.
    fading: Vec<(Vec<Vector2>, Color, f32)>,
    /// Desde cuándo está calmada cada criatura (para apagar sus ojos mientras se va).
    calm: HashMap<u32, f32>,
    hurt: f32,
    scare: Option<(Kind, f32)>,
    lost_heart: Option<(u8, f32)>,
    /// El corazón recuperado y cuánto le queda a su animación.
    gained_heart: Option<(u8, f32)>,
    banner: Option<(String, String, f32)>,
    note: Option<(String, f32)>,
    /// Un susto de mentira (sin daño): la criatura y cuánto hace que apareció.
    fright: Option<(Kind, f32)>,
    /// Cuánto levanta el dedo la bruja en la intro (0 a 1); lo pone la ventana.
    pub warn: f32,
    /// Chispas verdes de un glifo aprendido en la intro (en la pantalla): posición, velocidad y vida que le queda.
    sparks: Vec<(Vector2, Vector2, f32)>,
}

impl Fx {
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.hurt = (self.hurt - dt).max(0.0);
        for b in &mut self.bolts {
            b.t += dt / 0.22;
        }
        self.bolts.retain(|b| b.t < 1.0);
        for f in &mut self.fading {
            f.2 -= dt;
        }
        self.fading.retain(|f| f.2 > 0.0);
        for t in self.calm.values_mut() {
            *t += dt;
        }
        if let Some(s) = &mut self.scare {
            s.1 += dt;
        }
        self.scare = self.scare.take().filter(|s| s.1 < SCARE);
        if let Some(f) = &mut self.fright {
            f.1 += dt;
        }
        self.fright = self.fright.take().filter(|f| f.1 < FRIGHT);
        if let Some(b) = &mut self.banner {
            b.2 -= dt;
        }
        self.banner = self.banner.take().filter(|b| b.2 > 0.0);
        if let Some(n) = &mut self.note {
            n.1 -= dt;
        }
        self.note = self.note.take().filter(|n| n.1 > 0.0);
        if let Some(h) = &mut self.lost_heart {
            h.1 -= dt;
        }
        self.lost_heart = self.lost_heart.take().filter(|h| h.1 > 0.0);
        if let Some(h) = &mut self.gained_heart {
            h.1 -= dt;
        }
        self.gained_heart = self.gained_heart.take().filter(|h| h.1 > 0.0);
        for (p, v, life) in &mut self.sparks {
            *p += *v * dt;
            *v = *v * (1.0 - 2.2 * dt).max(0.0) + Vector2::new(0.0, 60.0 * dt);
            *life -= dt;
        }
        self.sparks.retain(|s| s.2 > 0.0);
    }

    /// Un hechizo reconocido que le pegó a la criatura `id`.
    pub fn spell(&mut self, art: &Art, s: &Session, id: u32, glyph: Glyph, calmed: bool, stroke: Vec<Vector2>,
                 camera: Camera3D) {
        let Some(c) = s.creatures.iter().find(|c| c.id == id) else { return };
        let color = glyph_color(glyph);
        self.bolts.push(Bolt { from: art.witch_hand(camera), to: art.head(c.kind, shown_pos(c, self.time)) - UP * 0.35, color, t: 0.0 });
        self.fading.push((stroke, color, 0.35));
        if calmed {
            self.calm.insert(id, 0.0);
        }
    }

    /// Un trazo que no sirvió: se apaga en gris, sin castigo.
    pub fn fizzle(&mut self, stroke: Vec<Vector2>, recognized: Option<Glyph>) {
        self.miss(stroke, &match recognized {
            Some(g) => format!("Nobody has the {} glyph right now", g.name()),
            None => "That didn't look like a glyph. Try again!".into(),
        });
    }

    /// Un trazo que no sirvió, con su aviso.
    pub fn miss(&mut self, stroke: Vec<Vector2>, note: &str) {
        self.fading.push((stroke, Color::new(150, 140, 150, 255), 0.45));
        self.note = Some((note.into(), 1.4));
    }

    /// Un glifo aprendido en la intro: el trazo se apaga en verde y el glifo se deshace en chispas verdes.
    pub fn learned(&mut self, stroke: Vec<Vector2>, at: Vector2, size: f32) {
        self.fading.push((stroke, LEARNED, 0.35));
        let k = self.time * 31.7;
        for i in 0..44 {
            let a = noise(k + i as f32 * 1.618) * std::f32::consts::TAU;
            let speed = size * (1.2 + 3.2 * noise(k + i as f32 * 2.71 + 9.0));
            let from = at + Vector2::new(a.cos(), a.sin()) * size * 0.5 * noise(k + i as f32 * 0.57 + 3.0);
            self.sparks.push((from, Vector2::new(a.cos(), a.sin()) * speed, 0.5 + 0.7 * noise(k + i as f32 * 3.3 + 1.0)));
        }
    }

    /// Una criatura llegó a la bruja: se tira encima de la cámara. `heart`, si le quitó un corazón.
    pub fn scare(&mut self, kind: Kind, heart: Option<u8>) {
        self.scare = Some((kind, 0.0));
        if let Some(h) = heart {
            self.hurt = 0.7;
            self.lost_heart = Some((h, 0.8));
        }
    }

    /// Se recuperó un corazón (el número `heart`, desde 0): el sombrero salta.
    pub fn gain(&mut self, heart: u8) {
        self.gained_heart = Some((heart, 0.9));
    }

    pub fn scaring(&self) -> bool {
        self.scare.is_some_and(|s| s.1 > 0.12)
    }

    /// Un susto de mentira: una criatura aparece pegada a la cara un instante y se va. No hace daño.
    pub fn fright(&mut self, kind: Kind) {
        self.fright = Some((kind, 0.0));
    }

    pub fn frightening(&self) -> bool {
        self.fright.is_some_and(|f| f.1 > 0.12)
    }

    /// Nada en pantalla que tape un susto de mentira: ni cartel, ni susto, ni otro susto de mentira.
    pub fn quiet(&self) -> bool {
        self.banner.is_none() && self.scare.is_none() && self.fright.is_none()
    }

    /// Calmadas al final por la victoria (sin hechizo): igual se apagan y se van.
    pub fn calm_all(&mut self, s: &Session) {
        for c in &s.creatures {
            self.calm.entry(c.id).or_insert(0.0);
        }
    }

    pub fn banner(&mut self, title: &str, sub: &str) {
        self.banner = Some((title.into(), sub.into(), BANNER));
    }

    pub fn clear(&mut self) {
        *self = Self { time: self.time, ..Self::default() };
    }
}

// --- Dibujo ------------------------------------------------------------------------------------------------------

pub(super) fn text_size(font: &Font, text: &str, size: f32) -> Vector2 {
    font.measure_text(text, size, 1.0)
}

pub fn text_center(d: &mut impl RaylibDraw, font: &Font, text: &str, center: Vector2, size: f32, color: Color) {
    let m = text_size(font, text, size);
    let p = Vector2::new(center.x - m.x / 2.0, center.y - m.y / 2.0);
    d.draw_text_ex(font, text, p + Vector2::new(2.0, 3.0), size, 1.0, fade(Color::BLACK, color.a as f32 / 400.0));
    d.draw_text_ex(font, text, p, size, 1.0, color);
}

/// Un glifo rasguñado en el aire, como una alucinación: sin fondo, con trazos finos a mano que se vuelven a rasguñar
/// varias veces por segundo, titila, a veces se apaga o salta de lugar un cuadro y deja un doble corrido.
fn draw_scratched(d: &mut impl RaylibDraw, g: Glyph, c: Vector2, size: f32, color: Color, alpha: f32, t: f32, seed: f32) {
    let frame = (t * 12.0).floor();
    let r = |i: f32| noise(seed + frame * 7.13 + i * 1.618);
    // Titila; de vez en cuando casi se apaga
    let mut a = alpha * (0.75 + 0.25 * r(1.0));
    if r(2.0) > 0.9 {
        a *= 0.2;
    }
    // Un cuadro suelto salta de lugar
    let c = if r(3.0) > 0.93 { c + Vector2::new((r(4.0) - 0.5) * size * 0.8, (r(5.0) - 0.5) * size * 0.3) } else { c };
    let paths: Vec<Vec<(f32, f32)>> = match g {
        Glyph::Line => vec![vec![(-0.95, 0.04), (0.0, -0.03), (0.95, 0.02)]],
        Glyph::Vee => vec![vec![(-0.8, -0.65), (-0.4, 0.0), (0.0, 0.7), (0.4, 0.05), (0.8, -0.65)]],
        // Un poco más de una vuelta, como hecho a mano
        Glyph::Circle => vec![(0..=18).map(|k| {
            let a = k as f32 / 16.0 * std::f32::consts::TAU - 1.2;
            (a.cos() * 0.75, a.sin() * 0.75)
        }).collect()],
        Glyph::Bolt => vec![vec![(0.3, -0.9), (-0.35, 0.05), (0.35, -0.05), (-0.3, 0.9)]],
    };
    let at = |x: f32, y: f32| Vector2::new(c.x + x * size, c.y + y * size);
    // Un doble corrido, rojo y frío, en los cuadros que fallan
    if r(6.0) > 0.7 {
        let off = Vector2::new(size * 0.09 * (r(7.0) - 0.3), 0.0);
        for path in &paths {
            for w in path.windows(2) {
                d.draw_line_ex(at(w[0].0, w[0].1) + off, at(w[1].0, w[1].1) + off, 1.5, fade(Color::new(255, 40, 60, 255), 0.35 * a));
                d.draw_line_ex(at(w[0].0, w[0].1) - off, at(w[1].0, w[1].1) - off, 1.5, fade(Color::new(80, 160, 255, 255), 0.35 * a));
            }
        }
    }
    // Tres rasguños encimados, cada uno temblando distinto; el primero, el más firme
    for (pass, (thick, keep)) in [(2.6, 1.0), (1.4, 0.7), (1.0, 0.5)].into_iter().enumerate() {
        let jit = size * if pass == 0 { 0.035 } else { 0.08 };
        for (pi, path) in paths.iter().enumerate() {
            let pts: Vec<Vector2> = path.iter().enumerate().map(|(i, &(x, y))| {
                let k = 10.0 + pass as f32 * 31.0 + pi as f32 * 97.0 + i as f32 * 5.0;
                at(x, y) + Vector2::new((r(k) - 0.5) * 2.0 * jit, (r(k + 2.5) - 0.5) * 2.0 * jit)
            }).collect();
            for w in pts.windows(2) {
                d.draw_line_ex(w[0], w[1], thick, fade(color, keep * a));
            }
        }
    }
}

/// Trazo mágico: un halo ancho y un centro brillante.
pub fn draw_stroke(d: &mut impl RaylibDraw, pts: &[Vector2], color: Color, alpha: f32) {
    for (w, a) in [(18.0, 0.16), (10.0, 0.35), (4.5, 1.0)] {
        let c = if w < 5.0 { fade(Color::new(255, 252, 240, 255), alpha) } else { fade(color, a * alpha) };
        for s in pts.windows(2) {
            d.draw_line_ex(s[0], s[1], w, c);
        }
        for p in pts.iter().step_by(2) {
            d.draw_circle_v(*p, w / 2.0, c);
        }
    }
}

/// Punto de la escena en la textura donde se dibuja (antes de la lente).
fn project_in(v: Vector3, camera: Camera3D, w: f32, h: f32) -> Vector2 {
    unsafe { ffi::GetWorldToScreenEx(v.into(), camera.into(), w as i32, h as i32).into() }
}

/// Dibuja la escena y, si `hud`, lo que va encima de la partida (sin menús).
#[allow(clippy::too_many_arguments)]
pub fn draw_game(screen: &mut RaylibDrawHandle, thread: &RaylibThread, art: &mut Art, fx: &Fx, s: &Session,
                 stroke: &[Vector2], view: View, look: (f32, f32), hud: bool) {
    let (w, h) = (screen.get_screen_width() as f32, screen.get_screen_height() as f32);
    screen.clear_background(Color::BLACK);
    let t = fx.time;
    let camera = art.camera(view, t, look);
    let items = scene_items(art, fx, s, view, camera);
    let (lamp, mut power) = lamp_now(t, s);
    // En el susto de mentira el farol se corta y vuelve a los tirones
    if let Some((_, k)) = fx.fright {
        power *= if noise((k * 14.0).floor() * 5.3 + 2.0) > 0.2 { 1.8 } else { 0.1 };
    }
    // El sacudón del susto, y uno más corto cuando el cartel de la oleada cae de golpe
    let slam = fx.banner.as_ref().map_or(0.0, |b| 0.7 * (1.0 - (BANNER - b.2) / 0.3).max(0.0))
        .max(fx.fright.map_or(0.0, |(_, k)| 0.15 + 0.2 * noise((k * 14.0).floor() + 7.0)));
    let hit = fx.scare.map_or(0.0, |(_, k)| 1.0 - (k / SCARE).min(1.0)).max(slam);
    let light = Light { lamp, power, tint: Vector3::zero(), range: LAMP_RANGE, fog: FOG_DENSITY, fill: 0.0, rim: 1.0 };
    let (tw, th) = render_view(screen, thread, art, &items, &[], camera, light, FISH_HALF, Rectangle::new(0.0, 0.0, w, h),
                               t, hit, Backdrop { stars: false, halo: view != View::Pov, see_through: false });
    let d = screen;
    if fx.scare.is_some_and(|(_, k)| k < 0.09) {
        d.draw_rectangle(0, 0, w as i32, h as i32, Color::new(255, 255, 255, 230));
    }
    if !hud {
        return;
    }
    // Lo que va encima se ubica sobre lo que se ve a través de la lente
    let project = |v: Vector3, camera: Camera3D| screen_from_scene(project_in(v, camera, tw, th), tw, w, h);
    let fwd = (camera.target - camera.position).normalized();
    // Los ojos del trío de fantasmas: un brillo chico, pegado a cada ojo, azul y blanco, que late apenas
    for c in s.creatures.iter().filter(|c| c.boss && c.kind == Kind::Ghost && c.state == State::Approaching) {
        let Some(eyes) = art.ghost.eye_centers("HW_GameGhostIris") else { continue };
        let (x, y) = shown_pos(c, t);
        let at = point(x, y);
        let m = place(at, stutter_front(-at, stutter_frame(c, t)), creature_scale(c.kind), art.ghost.base);
        let a = 1.0 - hidden(s, c);
        let radius = (150.0 / c.distance()).clamp(16.0, 80.0) * (1.0 + 0.08 * (t * 6.0).sin());
        let mut add = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
        for eye in eyes {
            let world = eye.transform_with(m);
            if (world - camera.position).dot(fwd) < 0.2 {
                continue;
            }
            let p = project(world, camera);
            add.draw_circle_gradient(p.x as i32, p.y as i32, radius, fade(Color::new(70, 150, 255, 255), 0.7 * a), Color::new(70, 150, 255, 0));
            add.draw_circle_gradient(p.x as i32, p.y as i32, radius * 0.35, fade(Color::WHITE, 0.95 * a), Color::new(255, 255, 255, 0));
        }
    }
    // Glifos: aparecen con la criatura, cuando la agarra la luz
    for c in s.creatures.iter().filter(|c| c.state == State::Approaching) {
        let at = art.head(c.kind, shown_pos(c, fx.time));
        if (at - camera.position).dot(fwd) < 0.2 {
            continue;
        }
        let p = project(at, camera);
        let dist = c.distance();
        let alpha = (1.0 - 0.4 * smooth((dist - 3.0) / 5.0)) * (1.0 - hidden(s, c));
        if alpha <= 0.01 {
            continue;
        }
        let size = (30.0 - dist * 1.4).clamp(15.0, 28.0);
        for (k, g) in c.glyphs.iter().enumerate() {
            let center = Vector2::new(p.x + k as f32 * size * 1.9 - (c.glyphs.len() - 1) as f32 * size * 0.95, p.y - size);
            let first = k == 0;
            let r = if first { size } else { size * 0.8 };
            let a = if first { alpha } else { alpha * 0.5 };
            // El diablo, del naranja de su cara tallada; el fantasma, en rojo; el demonio, del color del glifo
            let color = match c.kind {
                Kind::Devil => Color::new(255, 120, 20, 255),
                Kind::Ghost => Color::new(235, 20, 30, 255),
                Kind::Demon => glyph_color(*g),
            };
            draw_scratched(d, *g, center, r, color, a, fx.time, c.id as f32 * 13.1 + k as f32 * 3.7);
        }
    }
    // Hechizos en vuelo
    for b in &fx.bolts {
        let p = |u: f32| project(b.from.lerp(b.to, u) + UP * (0.5 * (u * std::f32::consts::PI).sin()), camera);
        let head = p(b.t);
        for k in 0..8 {
            let (u0, u1) = ((b.t - 0.05 * (k + 1) as f32).max(0.0), (b.t - 0.05 * k as f32).max(0.0));
            let a = 1.0 - k as f32 / 8.0;
            d.draw_line_ex(p(u0), p(u1), 12.0 * a, fade(b.color, 0.5 * a));
            d.draw_line_ex(p(u0), p(u1), 4.0 * a, fade(Color::WHITE, a));
        }
        d.draw_circle_v(head, 12.0, fade(b.color, 0.45));
        d.draw_circle_v(head, 5.0, Color::WHITE);
    }
    for (pts, color, life) in &fx.fading {
        draw_stroke(d, pts, *color, life / 0.35);
    }
    if stroke.len() > 1 {
        draw_stroke(d, stroke, STROKE, 1.0);
    }
    if fx.hurt > 0.0 {
        let a = fx.hurt / 0.7;
        for k in 0..6 {
            let inset = k as f32 * 9.0;
            d.draw_rectangle_lines_ex(Rectangle::new(inset, inset, w - 2.0 * inset, h - 2.0 * inset), 9.0,
                                      fade(Color::new(200, 0, 20, 255), a * (0.5 - k as f32 * 0.08)));
        }
    }
    draw_hud(d, art, fx, s);
}

fn draw_hud(d: &mut RaylibDrawHandle, art: &Art, fx: &Fx, s: &Session) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    let font = &art.font;
    // Vidas: sombreros de bruja; el que se pierde se infla y se apaga de violeta a gris
    let hat = |d: &mut RaylibDrawHandle, tex: &Texture2D, c: Vector2, size: f32, tint: Color| {
        let src = Rectangle::new(0.0, 0.0, tex.width as f32, tex.height as f32);
        d.draw_texture_pro(tex, src, Rectangle::new(c.x - size / 2.0, c.y - size / 2.0, size, size), Vector2::zero(), 0.0, tint);
    };
    for i in 0..s.balance.health {
        let c = Vector2::new(44.0 + i as f32 * 54.0, 42.0);
        let breaking = fx.lost_heart.filter(|(k, _)| *k == i).map(|(_, t)| t / 0.8);
        let gained = fx.gained_heart.filter(|(k, _)| *k == i).map_or(0.0, |(_, t)| t / 0.9);
        let size = 46.0 * (1.0 + breaking.map_or(0.0, |b| 0.35 * b) + 0.7 * gained);
        if i < s.health {
            let beat = 1.0 + 0.05 * (fx.time * 5.0).sin().max(0.0);
            hat(d, &art.life_full, c, size * beat, Color::WHITE);
        } else {
            hat(d, &art.life_empty, c, size, Color::WHITE);
            if let Some(b) = breaking {
                hat(d, &art.life_full, c, size, fade(Color::WHITE, b));
            }
        }
    }
    let (label, len) = match s.phase {
        Phase::Wave(n) => (if n == 1 { "WAVE 1" } else { "WAVE 2" }, s.balance.wave_time[n as usize - 1]),
        Phase::Victory | Phase::Defeat => ("", 1.0),
    };
    if !label.is_empty() {
        text_center(d, font, label, Vector2::new(w / 2.0, 30.0), 30.0, Color::new(230, 225, 235, 220));
        let bw = 220.0;
        let r = Rectangle::new(w / 2.0 - bw / 2.0, 50.0, bw, 6.0);
        d.draw_rectangle_rounded(r, 1.0, 6, Color::new(40, 40, 46, 200));
        d.draw_rectangle_rounded(Rectangle::new(r.x, r.y, bw * (s.phase_time / len).clamp(0.0, 1.0), 6.0), 1.0, 6,
                                 Color::new(235, 230, 220, 220));
    }
    // Cuántas faltan para recuperar un sombrero, a la izquierda de la pausa
    let meter = format!("{}/{}", s.life_meter, s.balance.life_every);
    let m = text_size(font, &meter, 32.0);
    let at = Vector2::new(w - 84.0 - m.x, 42.0 - m.y / 2.0);
    d.draw_text_ex(font, &meter, at, 32.0, 1.0, Color::new(235, 230, 240, 235));
    hat(d, &art.life_full, Vector2::new(at.x - 24.0, 42.0), 36.0, Color::WHITE);
    draw_pause_icon(d, w);
    // El cartel de la oleada: enorme y rojo sangre, cae de golpe (más grande, achicándose de un saque), y después se
    // hunde despacio en lo oscuro mientras se apaga
    if let Some((title, sub, t)) = &fx.banner {
        let age = BANNER - t;
        let hit = smooth(age / 0.14);
        let sink = smooth((age - 0.8) / (BANNER - 0.8));
        let a = (1.0 - sink).min(hit * 3.0).clamp(0.0, 1.0);
        let size = h * 0.46 * (1.7 - 0.7 * hit);
        let c = Vector2::new(w / 2.0, h * (0.4 + 0.08 * sink));
        let big = &art.font_big;
        let m = text_size(big, title, size);
        let p = Vector2::new(c.x - m.x / 2.0, c.y - m.y / 2.0);
        d.draw_text_ex(big, title, p + Vector2::new(0.0, size * 0.04), size, 2.0, fade(Color::new(40, 0, 2, 255), 0.9 * a));
        d.draw_text_ex(big, title, p, size, 2.0, fade(Color::new(196, 8, 16, 255), a));
        text_center(d, font, sub, Vector2::new(w / 2.0, c.y + size * 0.42), 32.0, fade(Color::new(200, 60, 60, 255), a * hit));
    }
    if let Some((text, t)) = &fx.note {
        text_center(d, font, text, Vector2::new(w / 2.0, h - 40.0), 26.0, fade(Color::new(210, 205, 215, 255), t / 0.4));
    }
}

/// La entrada de los glifos para aprender: el primero empieza a subir a los `LEARN_FIRST` segundos de baile, cada
/// uno `LEARN_STEP` después del anterior, y tarda `LEARN_RISE` en llegar.
const LEARN_FIRST: f32 = 0.8;
const LEARN_STEP: f32 = 0.3;
const LEARN_RISE: f32 = 0.8;

/// Cuánto dura un susto de mentira.
pub const FRIGHT: f32 = 0.5;

/// Cuánto dura el cartel de cada oleada.
const BANNER: f32 = 3.0;

/// Verde de lo aprendido en la intro (trazo y chispas).
const LEARNED: Color = Color::new(90, 255, 120, 255);

/// Dónde está cada glifo para aprender en la intro, parejo a los dos lados de la bruja que baila (los de abajo, más
/// cerca de ella), y su tamaño.
pub fn learn_spot(g: Glyph, w: f32, h: f32) -> (Vector2, f32) {
    let (x, y) = match g {
        Glyph::Line => (0.2, 0.375),
        Glyph::Vee => (0.33, 0.675),
        Glyph::Circle => (0.67, 0.675),
        Glyph::Bolt => (0.8, 0.375),
    };
    (Vector2::new(w * x, h * y), h * 0.085)
}

/// La intro mientras la bruja baila: los glifos que faltan aprender (rasguñados, cada uno con su nombre), las
/// chispas de los aprendidos, el trazo y las instrucciones abajo.
/// `since`: segundos desde que empezó el baile (los glifos entran uno por uno, de izquierda a derecha, subiendo
/// desde lo oscuro); `out`: 1 mientras se aprende, baja a 0 cuando gira la cámara.
pub fn draw_learn(d: &mut RaylibDrawHandle, font: &Font, fx: &Fx, learn: &[Glyph], stroke: &[Vector2], since: f32, out: f32) {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    let alpha = smooth(since / LEARN_FIRST) * out;
    for g in learn {
        // Glyph::ALL ya está de izquierda a derecha
        let k = Glyph::ALL.iter().position(|x| x == g).unwrap_or(0) as f32;
        let (c, size) = learn_spot(*g, w, h);
        let start = LEARN_FIRST + k * LEARN_STEP;
        let (u, v) = (smooth((since - start) / LEARN_RISE), smooth((since - start - 0.15) / LEARN_RISE));
        let bob = Vector2::new(0.0, (fx.time * 1.7 + k * 1.3).sin() * size * 0.06);
        let rise = |u: f32| Vector2::new(0.0, (1.0 - u) * size * 0.6);
        draw_scratched(d, *g, c + bob + rise(u), size, glyph_color(*g), u * out, fx.time, 40.0 + k * 7.7);
        text_center(d, font, g.name(), c + bob + rise(v) + Vector2::new(0.0, size * 1.35), 26.0,
                    fade(Color::new(225, 220, 235, 255), v * out));
    }
    for (p, _, life) in &fx.sparks {
        let a = (life / 0.4).min(1.0);
        d.draw_circle_v(*p, 5.0, fade(LEARNED, 0.25 * a));
        d.draw_line_ex(*p - Vector2::new(4.0, 0.0), *p + Vector2::new(4.0, 0.0), 1.5, fade(LEARNED, a));
        d.draw_line_ex(*p - Vector2::new(0.0, 4.0), *p + Vector2::new(0.0, 4.0), 1.5, fade(LEARNED, a));
        d.draw_circle_v(*p, 1.6, fade(Color::new(220, 255, 225, 255), a));
    }
    for (pts, color, life) in &fx.fading {
        draw_stroke(d, pts, *color, life / 0.35);
    }
    if stroke.len() > 1 {
        draw_stroke(d, stroke, STROKE, 1.0);
    }
    if !learn.is_empty() {
        text_center(d, font, "Learn your spells: draw each glyph", Vector2::new(w / 2.0, h - 78.0), 34.0,
                    fade(Color::new(240, 235, 225, 255), alpha));
        text_center(d, font, "Hold the left mouse button and trace it anywhere on the screen", Vector2::new(w / 2.0, h - 42.0),
                    24.0, fade(Color::new(200, 195, 210, 255), alpha));
    }
    if let Some((text, t)) = &fx.note {
        text_center(d, font, text, Vector2::new(w / 2.0, h - 112.0), 26.0, fade(Color::new(210, 205, 215, 255), t / 0.4));
    }
}

/// La advertencia de la intro, abajo, mientras la bruja levanta el dedo.
pub fn draw_warning(d: &mut RaylibDrawHandle, font: &Font, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    // A la derecha de la bruja, centrado a lo alto
    let x = w * 0.68;
    let red = fade(Color::new(214, 22, 30, 255), alpha);
    text_center(d, font, "Careful...", Vector2::new(x, h * 0.5 - 118.0), 76.0, red);
    for (k, line) in ["Sometimes a devil, a ghost or a demon", "will jump right in your face for a split second.",
                      "It's only a trick: it can't take your hats.", "Don't stop drawing!"].iter().enumerate() {
        text_center(d, font, line, Vector2::new(x, h * 0.5 - 40.0 + k as f32 * 44.0), 38.0, red);
    }
    text_center(d, font, "Click to continue", Vector2::new(x, h * 0.5 + 160.0), 22.0, fade(Color::new(170, 160, 170, 255), alpha * 0.8));
}

/// El botón de pausa, arriba a la derecha.
pub fn draw_pause_icon(d: &mut impl RaylibDraw, w: f32) {
    let pc = pause_button(w);
    d.draw_ring(pc, 20.0, 22.0, 0.0, 360.0, 32, Color::new(220, 215, 230, 160));
    d.draw_rectangle_rec(Rectangle::new(pc.x - 8.0, pc.y - 9.0, 5.0, 18.0), Color::new(230, 225, 240, 220));
    d.draw_rectangle_rec(Rectangle::new(pc.x + 3.0, pc.y - 9.0, 5.0, 18.0), Color::new(230, 225, 240, 220));
}

pub fn pause_button(w: f32) -> Vector2 {
    Vector2::new(w - 44.0, 42.0)
}

/// Un menú centrado, con el estilo del logo de Hexbound: caja rosa con degradé (fucsia arriba, rojo abajo), borde
/// negro grueso y sombra dura; título y texto negros, sin sombras; botones negros simples con texto blanco (el
/// elegido, blanco con texto negro). Devuelve los rectángulos de los botones.
pub fn draw_menu(d: &mut RaylibDrawHandle, font: &Font, title: &str, sub: &str, buttons: &[&str], selected: usize)
    -> Vec<Rectangle> {
    let (w, h) = (d.get_screen_width() as f32, d.get_screen_height() as f32);
    let ink = Color::new(10, 6, 12, 255);
    d.draw_rectangle(0, 0, w as i32, h as i32, Color::new(0, 0, 0, 150));
    let ph = 170.0 + buttons.len() as f32 * 62.0;
    let panel = Rectangle::new(w / 2.0 - 280.0, h / 2.0 - ph / 2.0, 560.0, ph);
    let grow = |r: Rectangle, k: f32| Rectangle::new(r.x - k, r.y - k, r.width + 2.0 * k, r.height + 2.0 * k);
    d.draw_rectangle_rounded(Rectangle::new(panel.x + 12.0, panel.y + 14.0, panel.width, panel.height), 0.1, 10, ink);
    d.draw_rectangle_rounded(grow(panel, 6.0), 0.1, 10, ink);
    // Degradé de arriba a abajo, recortado con las esquinas redondeadas
    let (top, bottom) = (Color::new(255, 0, 200, 255), Color::new(255, 12, 40, 255));
    let round = panel.width.min(panel.height) * 0.05;
    d.draw_rectangle_gradient_v(panel.x as i32, (panel.y + round) as i32, panel.width as i32, (panel.height - 2.0 * round) as i32,
                                top, bottom);
    d.draw_rectangle_rounded(Rectangle::new(panel.x, panel.y, panel.width, round * 3.0), 0.5, 10, top);
    d.draw_rectangle_rounded(Rectangle::new(panel.x, panel.y + panel.height - round * 3.0, panel.width, round * 3.0), 0.5, 10, bottom);
    let plain = |d: &mut RaylibDrawHandle, text: &str, c: Vector2, size: f32, color: Color| {
        let m = text_size(font, text, size);
        d.draw_text_ex(font, text, c - m * 0.5, size, 1.0, color);
    };
    plain(d, title, Vector2::new(w / 2.0, panel.y + 52.0), 56.0, ink);
    plain(d, sub, Vector2::new(w / 2.0, panel.y + 102.0), 26.0, ink);
    let mut rects = Vec::new();
    for (i, b) in buttons.iter().enumerate() {
        let r = Rectangle::new(w / 2.0 - 190.0, panel.y + 140.0 + i as f32 * 62.0, 380.0, 50.0);
        let on = i == selected;
        d.draw_rectangle_rounded(r, 0.45, 8, if on { Color::WHITE } else { ink });
        plain(d, b, Vector2::new(r.x + r.width / 2.0, r.y + r.height / 2.0), 30.0, if on { ink } else { Color::WHITE });
        rects.push(r);
    }
    rects
}
