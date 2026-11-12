//! Renderizador por software (F7 en el sistema solar): todo el pipeline 3D en la CPU, sin las funciones 3D ni los
//! shaders de la GPU. raylib solo muestra el resultado (una textura que se actualiza cada cuadro).
//!
//! Etapas, por cuadro:
//! 1. **Vértices:** cada triángulo pasa por la matriz de modelo (`world`), la de vista (los ejes de la cámara) y la
//!    proyección en perspectiva.
//! 2. **Recorte:** contra el plano cercano (Sutherland-Hodgman), y se descarta lo que cae fuera de la pantalla.
//! 3. **Rasterizado:** funciones de borde y coordenadas baricéntricas, con interpolación corregida por perspectiva
//!    (dividiendo por la profundidad) y **prueba de profundidad** en un buffer propio. Se reparte en franjas
//!    horizontales, una por núcleo de la CPU.
//! 4. **Fragmentos:** un "shader" escrito en Rust por píxel: `Shade::Lit` (luz del sol, Lambert más un borde frío) y
//!    `Shade::Sun` (superficie del sol animada con ruido que cambia con el tiempo).
//!
//! La geometría se prepara antes de rasterizar: `SoftMesh::decimate` genera versiones más livianas de cada modelo
//! (agrupando vértices en una grilla) y `SoftMesh::sphere` genera el sol subdividiendo un icosaedro.
use raylib::{ffi, prelude::*};
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct SoftVertex {
    pub pos: Vector3,
    pub normal: Vector3,
    pub color: Vector3,
}

#[derive(Clone, Copy)]
pub struct SoftTriangle {
    pub v: [SoftVertex; 3],
    pub emissive: Vector3,
    /// Sin luz: el color tal cual (los personajes de caricatura).
    pub flat: bool,
}

#[derive(Clone, Default)]
pub struct SoftMesh {
    pub tris: Vec<SoftTriangle>,
}

impl SoftMesh {
    /// Una versión con a lo sumo `budget` triángulos: los vértices se agrupan en una grilla (cada celda queda en el
    /// promedio de los suyos) y se tiran los triángulos que se aplastan. Cada vez más gruesa hasta que entra.
    pub fn decimate(&self, budget: usize) -> SoftMesh {
        if self.tris.len() <= budget {
            return self.clone();
        }
        let (mut lo, mut hi) = (Vector3::one() * f32::MAX, Vector3::one() * f32::MIN);
        for t in &self.tris {
            for v in &t.v {
                lo = Vector3::new(lo.x.min(v.pos.x), lo.y.min(v.pos.y), lo.z.min(v.pos.z));
                hi = Vector3::new(hi.x.max(v.pos.x), hi.y.max(v.pos.y), hi.z.max(v.pos.z));
            }
        }
        let size = (hi - lo).x.max((hi - lo).y).max((hi - lo).z).max(1e-6);
        let mut best = self.clone();
        for cells in [160.0, 120.0, 90.0, 68.0, 52.0, 40.0, 30.0, 22.0, 16.0, 12.0, 8.0, 6.0, 4.0] {
            let cell = size / cells;
            let key = |p: Vector3| (((p.x - lo.x) / cell) as i32, ((p.y - lo.y) / cell) as i32, ((p.z - lo.z) / cell) as i32);
            let mut sums: HashMap<(i32, i32, i32), (Vector3, Vector3, Vector3, f32)> = HashMap::new();
            for t in &self.tris {
                for v in &t.v {
                    let e = sums.entry(key(v.pos)).or_insert((Vector3::zero(), Vector3::zero(), Vector3::zero(), 0.0));
                    e.0 = e.0 + v.pos;
                    e.1 = e.1 + v.normal;
                    e.2 = e.2 + v.color;
                    e.3 += 1.0;
                }
            }
            let rep = |k: (i32, i32, i32), v: &SoftVertex| {
                let (p, n, c, w) = sums[&k];
                SoftVertex { pos: p / w, normal: if n.length() > 1e-6 { n.normalized() } else { v.normal }, color: c / w }
            };
            let mut tris = Vec::new();
            for t in &self.tris {
                let k = [key(t.v[0].pos), key(t.v[1].pos), key(t.v[2].pos)];
                if k[0] == k[1] || k[1] == k[2] || k[0] == k[2] {
                    continue;
                }
                tris.push(SoftTriangle { v: [rep(k[0], &t.v[0]), rep(k[1], &t.v[1]), rep(k[2], &t.v[2])], ..*t });
            }
            best = SoftMesh { tris };
            if best.tris.len() <= budget {
                break;
            }
        }
        best
    }

    /// Una esfera de radio 1 generada a mano: un icosaedro con cada triángulo partido en cuatro `levels` veces y los
    /// vértices llevados a la superficie.
    pub fn sphere(levels: u32, color: Vector3) -> SoftMesh {
        let t = (1.0 + 5f32.sqrt()) / 2.0;
        let p = [(-1.0, t, 0.0), (1.0, t, 0.0), (-1.0, -t, 0.0), (1.0, -t, 0.0), (0.0, -1.0, t), (0.0, 1.0, t),
                 (0.0, -1.0, -t), (0.0, 1.0, -t), (t, 0.0, -1.0), (t, 0.0, 1.0), (-t, 0.0, -1.0), (-t, 0.0, 1.0)]
            .map(|(x, y, z)| Vector3::new(x, y, z).normalized());
        let faces = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2),
                     (10, 7, 6), (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5),
                     (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1)];
        let mut tris: Vec<[Vector3; 3]> = faces.iter().map(|&(a, b, c)| [p[a], p[b], p[c]]).collect();
        for _ in 0..levels {
            tris = tris.iter().flat_map(|[a, b, c]| {
                let (ab, bc, ca) = ((*a + *b).normalized(), (*b + *c).normalized(), (*c + *a).normalized());
                [[*a, ab, ca], [ab, *b, bc], [ca, bc, *c], [ab, bc, ca]]
            }).collect();
        }
        let vert = |p: Vector3| SoftVertex { pos: p, normal: p, color };
        SoftMesh { tris: tris.into_iter().map(|[a, b, c]| SoftTriangle { v: [vert(a), vert(b), vert(c)], emissive: Vector3::zero(), flat: false }).collect() }
    }
}

/// El "shader de fragmentos" de cada objeto.
#[derive(Clone, Copy)]
pub enum Shade {
    /// Luz del sol (en `sun`): Lambert, un resto de luz ambiente y un borde frío; más el brillo propio.
    Lit,
    /// La superficie del sol, animada: ruido que se mueve con el tiempo, más claro en el centro y naranja en el borde.
    Sun { center: Vector3 },
}

pub struct Draw<'a> {
    pub mesh: &'a SoftMesh,
    pub world: Matrix,
    pub shade: Shade,
}

/// Un triángulo ya en la pantalla: posición (x, y en píxeles; 1/z), y lo que se interpola dividido por z.
#[derive(Clone, Copy)]
struct ScreenTri {
    p: [Vector3; 3],
    world: [Vector3; 3],
    normal: [Vector3; 3],
    color: [Vector3; 3],
    emissive: Vector3,
    flat: bool,
    shade: Shade,
}

/// Cuánto trabajó el renderizador en el último cuadro.
#[derive(Clone, Copy, Default)]
pub struct SoftStats {
    pub submitted: usize,
    pub rasterized: usize,
    /// Cuánto tardó el cuadro en la CPU (milisegundos).
    pub ms: f32,
}

pub struct SoftRenderer {
    pub width: i32,
    pub height: i32,
    color: Vec<u8>,
    depth: Vec<f32>,
    texture: Texture2D,
    stars: Vec<(Vector3, f32)>,
    pub stats: SoftStats,
}

const NEAR: f32 = 0.5;
const BACKGROUND: [u8; 3] = [6, 9, 22];

impl SoftRenderer {
    pub fn new(rl: &mut RaylibHandle, thread: &RaylibThread, width: i32, height: i32) -> Result<Self, Box<dyn std::error::Error>> {
        let image = Image::gen_image_color(width, height, Color::BLACK);
        let texture = rl.load_texture_from_image(thread, &image)?;
        texture.set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
        // Estrellas: direcciones al azar con semilla fija (siempre las mismas)
        let mut seed: u32 = 0x5eed_0001;
        let mut rand = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32
        };
        let stars = (0..2500).map(|_| {
            let (z, a) = (rand() * 2.0 - 1.0, rand() * std::f32::consts::TAU);
            let r = (1.0 - z * z).sqrt();
            (Vector3::new(r * a.cos(), z, r * a.sin()), 0.3 + 0.7 * rand())
        }).collect();
        let n = (width * height) as usize;
        Ok(Self { width, height, color: vec![0; n * 4], depth: vec![0.0; n], texture, stars, stats: SoftStats::default() })
    }

    /// Dibuja la escena en el buffer propio y lo sube a la textura.
    pub fn render(&mut self, camera: Camera3D, sun: Vector3, draws: &[Draw], time: f32) {
        let started = std::time::Instant::now();
        let (w, h) = (self.width as f32, self.height as f32);
        let eye = camera.position;
        let f = (camera.target - eye).normalized();
        let r = f.cross(camera.up).normalized();
        let u = r.cross(f);
        let tan_y = (camera.fovy.to_radians() * 0.5).tan();
        let tan_x = tan_y * w / h;
        // Fondo y estrellas (lo de atrás de todo: profundidad 0)
        for px in self.color.chunks_exact_mut(4) {
            px.copy_from_slice(&[BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]);
        }
        self.depth.fill(0.0);
        for (dir, bright) in &self.stars {
            let z = dir.dot(f);
            if z <= 0.0 {
                continue;
            }
            let (sx, sy) = ((dir.dot(r) / (z * tan_x) * 0.5 + 0.5) * w, (0.5 - dir.dot(u) / (z * tan_y) * 0.5) * h);
            if sx >= 0.0 && sy >= 0.0 && sx < w && sy < h {
                let i = (sy as usize * self.width as usize + sx as usize) * 4;
                let v = (90.0 + 165.0 * bright) as u8;
                self.color[i..i + 3].copy_from_slice(&[v, v, (v as f32 * 1.05).min(255.0) as u8]);
            }
        }
        // 1-2. Vértices, recorte y proyección
        let mut screen = Vec::new();
        let mut submitted = 0;
        for draw in draws {
            let dir = |n: Vector3| n.transform_with(draw.world) - Vector3::zero().transform_with(draw.world);
            for t in &draw.mesh.tris {
                submitted += 1;
                let verts = t.v.map(|v| {
                    let wp = v.pos.transform_with(draw.world);
                    let d = wp - eye;
                    (Vector3::new(d.dot(r), d.dot(u), d.dot(f)), wp, dir(v.normal), v.color)
                });
                if verts.iter().all(|v| v.0.z < NEAR) {
                    continue;
                }
                for poly in clip_near(&verts) {
                    let proj = poly.map(|(v, wp, n, c)| {
                        let iz = 1.0 / v.z;
                        (Vector3::new((v.x * iz / tan_x * 0.5 + 0.5) * w, (0.5 - v.y * iz / tan_y * 0.5) * h, iz), wp * iz, n * iz, c * iz)
                    });
                    let (minx, maxx) = (proj.iter().map(|p| p.0.x).fold(f32::MAX, f32::min), proj.iter().map(|p| p.0.x).fold(f32::MIN, f32::max));
                    let (miny, maxy) = (proj.iter().map(|p| p.0.y).fold(f32::MAX, f32::min), proj.iter().map(|p| p.0.y).fold(f32::MIN, f32::max));
                    if maxx < 0.0 || maxy < 0.0 || minx >= w || miny >= h {
                        continue;
                    }
                    screen.push(ScreenTri {
                        p: proj.map(|p| p.0), world: proj.map(|p| p.1), normal: proj.map(|p| p.2), color: proj.map(|p| p.3),
                        emissive: t.emissive, flat: t.flat, shade: draw.shade,
                    });
                }
            }
        }
        self.stats = SoftStats { submitted, rasterized: screen.len(), ms: 0.0 };
        // 3-4. Rasterizado y fragmentos, en franjas horizontales en paralelo
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 16);
        let rows = (self.height as usize).div_ceil(threads);
        let width = self.width as usize;
        let screen = &screen;
        std::thread::scope(|scope| {
            for (band, (color, depth)) in self.color.chunks_mut(rows * width * 4).zip(self.depth.chunks_mut(rows * width)).enumerate() {
                scope.spawn(move || {
                    let y0 = band * rows;
                    let y1 = y0 + depth.len() / width;
                    for t in screen {
                        raster(t, color, depth, width, y0, y1, eye, sun, time);
                    }
                });
            }
        });
        unsafe { ffi::UpdateTexture(*self.texture.as_ref(), self.color.as_ptr() as *const _) };
        self.stats.ms = started.elapsed().as_secs_f32() * 1000.0;
    }

    /// Muestra el resultado llenando la ventana.
    pub fn draw(&self, d: &mut impl RaylibDraw, width: i32, height: i32) {
        d.draw_texture_pro(&self.texture, Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
            Rectangle::new(0.0, 0.0, width as f32, height as f32), Vector2::zero(), 0.0, Color::WHITE);
    }
}

type ClipVert = (Vector3, Vector3, Vector3, Vector3);

/// Recorta un triángulo (en espacio de vista) contra el plano cercano: devuelve 0, 1 o 2 triángulos.
fn clip_near(v: &[ClipVert; 3]) -> Vec<[ClipVert; 3]> {
    let lerp = |a: &ClipVert, b: &ClipVert| {
        let k = (NEAR - a.0.z) / (b.0.z - a.0.z);
        (a.0 + (b.0 - a.0) * k, a.1 + (b.1 - a.1) * k, a.2 + (b.2 - a.2) * k, a.3 + (b.3 - a.3) * k)
    };
    let mut out: Vec<ClipVert> = Vec::with_capacity(4);
    for i in 0..3 {
        let (a, b) = (&v[i], &v[(i + 1) % 3]);
        let (ina, inb) = (a.0.z >= NEAR, b.0.z >= NEAR);
        if ina {
            out.push(*a);
        }
        if ina != inb {
            out.push(lerp(a, b));
        }
    }
    match out.len() {
        3 => vec![[out[0], out[1], out[2]]],
        4 => vec![[out[0], out[1], out[2]], [out[0], out[2], out[3]]],
        _ => vec![],
    }
}

/// Rasteriza un triángulo en las filas `y0..y1` de la franja: baricéntricas con funciones de borde, profundidad 1/z
/// (más grande = más cerca) y atributos corregidos por perspectiva.
#[allow(clippy::too_many_arguments)]
fn raster(t: &ScreenTri, color: &mut [u8], depth: &mut [f32], width: usize, y0: usize, y1: usize, eye: Vector3, sun: Vector3, time: f32) {
    let [a, b, c] = t.p;
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if area.abs() < 1e-8 {
        return;
    }
    let minx = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
    let maxx = (a.x.max(b.x).max(c.x).ceil() as usize).min(width.saturating_sub(1));
    let miny = (a.y.min(b.y).min(c.y).floor().max(0.0) as usize).max(y0);
    let maxy = (a.y.max(b.y).max(c.y).ceil().max(0.0) as usize).min(y1.saturating_sub(1));
    if minx > maxx || miny > maxy {
        return;
    }
    let inv = 1.0 / area;
    for y in miny..=maxy {
        let py = y as f32 + 0.5;
        for x in minx..=maxx {
            let px = x as f32 + 0.5;
            let w0 = ((b.x - px) * (c.y - py) - (b.y - py) * (c.x - px)) * inv;
            let w1 = ((c.x - px) * (a.y - py) - (c.y - py) * (a.x - px)) * inv;
            let w2 = 1.0 - w0 - w1;
            if w0 < 0.0 || w1 < 0.0 || w2 < 0.0 {
                continue;
            }
            let iz = w0 * a.z + w1 * b.z + w2 * c.z;
            let di = (y - y0) * width + x;
            if iz <= depth[di] {
                continue;
            }
            depth[di] = iz;
            let z = 1.0 / iz;
            let mix = |v: [Vector3; 3]| (v[0] * w0 + v[1] * w1 + v[2] * w2) * z;
            let rgb = fragment(t, mix(t.world), mix(t.normal), mix(t.color), eye, sun, time);
            let ci = di * 4;
            color[ci] = (rgb.x.clamp(0.0, 1.0) * 255.0) as u8;
            color[ci + 1] = (rgb.y.clamp(0.0, 1.0) * 255.0) as u8;
            color[ci + 2] = (rgb.z.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
}

/// Ruido de valor 3D (suave, entre 0 y 1).
fn noise(p: Vector3) -> f32 {
    let hash = |x: f32, y: f32, z: f32| ((x * 127.1 + y * 311.7 + z * 74.7).sin() * 43_758.547).fract().abs();
    let (i, f) = (Vector3::new(p.x.floor(), p.y.floor(), p.z.floor()), Vector3::new(p.x.fract(), p.y.fract(), p.z.fract()));
    let f = Vector3::new(f.x.rem_euclid(1.0), f.y.rem_euclid(1.0), f.z.rem_euclid(1.0));
    let s = Vector3::new(f.x * f.x * (3.0 - 2.0 * f.x), f.y * f.y * (3.0 - 2.0 * f.y), f.z * f.z * (3.0 - 2.0 * f.z));
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let c = |dx: f32, dy: f32, dz: f32| hash(i.x + dx, i.y + dy, i.z + dz);
    l(l(l(c(0., 0., 0.), c(1., 0., 0.), s.x), l(c(0., 1., 0.), c(1., 1., 0.), s.x), s.y),
      l(l(c(0., 0., 1.), c(1., 0., 1.), s.x), l(c(0., 1., 1.), c(1., 1., 1.), s.x), s.y), s.z)
}

/// Los "shaders de fragmentos".
fn fragment(t: &ScreenTri, world: Vector3, normal: Vector3, color: Vector3, eye: Vector3, sun: Vector3, time: f32) -> Vector3 {
    let view = (eye - world).normalized();
    match t.shade {
        Shade::Sun { center } => {
            // Granulación que se mueve: dos capas de ruido que se desplazan con el tiempo
            let d = (world - center).normalized();
            let g = noise(d * 6.0 + Vector3::new(time * 0.15, 0.0, time * 0.1)) * 0.6
                + noise(d * 14.0 - Vector3::new(0.0, time * 0.25, 0.0)) * 0.4;
            let edge = d.dot(view).max(0.0);
            let hot = Vector3::new(1.0, 0.97, 0.85);
            let warm = Vector3::new(1.0, 0.62, 0.22);
            let k = (0.35 + 0.65 * edge.powf(0.6)) * (0.75 + 0.35 * g);
            warm + (hot - warm) * k.clamp(0.0, 1.0)
        }
        Shade::Lit => {
            if t.flat {
                return color;
            }
            let mut n = normal.normalized();
            if n.dot(view) < 0.0 {
                n = n * -1.0;
            }
            let l = (sun - world).normalized();
            let diffuse = n.dot(l).max(0.0);
            let rim = (1.0 - n.dot(view).abs()).powi(3) * 0.35;
            color * (0.14 + 0.95 * diffuse) + Vector3::new(0.35, 0.55, 1.0) * rim + t.emissive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tri(z: [f32; 3]) -> [ClipVert; 3] {
        z.map(|z| (Vector3::new(0.0, 0.0, z), Vector3::zero(), Vector3::zero(), Vector3::zero()))
    }

    #[test]
    fn el_recorte_contra_el_plano_cercano() {
        assert_eq!(clip_near(&tri([1.0, 2.0, 3.0])).len(), 1);
        assert_eq!(clip_near(&tri([0.0, 2.0, 3.0])).len(), 2);
        assert_eq!(clip_near(&tri([0.0, 0.1, 3.0])).len(), 1);
        assert!(clip_near(&tri([0.0, 0.1, 0.2])).is_empty());
        for t in clip_near(&tri([0.0, 2.0, 3.0])) {
            assert!(t.iter().all(|v| v.0.z >= NEAR - 1e-5));
        }
    }

    #[test]
    fn la_esfera_se_subdivide_y_queda_sobre_la_superficie() {
        for levels in 0..4 {
            let s = SoftMesh::sphere(levels, Vector3::one());
            assert_eq!(s.tris.len(), 20 * 4usize.pow(levels));
            assert!(s.tris.iter().flat_map(|t| t.v).all(|v| (v.pos.length() - 1.0).abs() < 1e-4));
        }
    }

    #[test]
    fn el_nivel_de_detalle_respeta_el_presupuesto() {
        let full = SoftMesh::sphere(5, Vector3::one());
        for budget in [8_000, 2_000, 500] {
            let lite = full.decimate(budget);
            assert!(lite.tris.len() <= budget && !lite.tris.is_empty(), "{budget}: {}", lite.tris.len());
        }
    }
}
