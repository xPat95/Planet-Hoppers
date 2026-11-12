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
use raylib::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct SoftVertex {
    pub pos: Vector3,
    pub normal: Vector3,
    pub color: Vector3,
    pub uv: Vector2,
}

#[derive(Clone, Copy, Default)]
pub enum Opacity {
    #[default]
    Opaque,
    Mask(f32),
    Blend,
}

#[derive(Clone, Copy)]
pub struct SoftTriangle {
    pub v: [SoftVertex; 3],
    pub emissive: Vector3,
    /// Sin luz: el color tal cual (los personajes de caricatura).
    pub flat: bool,
    /// De qué material es: al simplificar solo se juntan vértices del mismo (si no, el mar se mezcla con las nubes).
    pub group: u32,
    pub texture: Option<usize>,
    pub alpha: f32,
    pub opacity: Opacity,
}

#[derive(Clone, Default)]
pub struct SoftMesh {
    pub tris: Vec<SoftTriangle>,
    pub images: std::sync::Arc<Vec<gltf::image::Data>>,
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
        let mut best = SoftMesh::default();
        for cells in [
            160.0, 120.0, 90.0, 68.0, 52.0, 40.0, 30.0, 22.0, 16.0, 12.0, 8.0, 6.0, 4.0,
        ] {
            let cell = size / cells;
            let key = |p: Vector3, g: u32| {
                (
                    ((p.x - lo.x) / cell) as i32,
                    ((p.y - lo.y) / cell) as i32,
                    ((p.z - lo.z) / cell) as i32,
                    g,
                )
            };
            let mut sums: HashMap<(i32, i32, i32, u32), (Vector3, f32)> = HashMap::new();
            for t in &self.tris {
                for v in &t.v {
                    let e = sums.entry(key(v.pos, t.group)).or_insert((Vector3::zero(), 0.0));
                    e.0 = e.0 + v.pos;
                    e.1 += 1.0;
                }
            }
            let rep = |k: (i32, i32, i32, u32), v: &SoftVertex| {
                let (p, w) = sums[&k];
                SoftVertex { pos: p / w, ..*v }
            };
            let mut tris = Vec::new();
            for t in &self.tris {
                let k = [
                    key(t.v[0].pos, t.group),
                    key(t.v[1].pos, t.group),
                    key(t.v[2].pos, t.group),
                ];
                if k[0] == k[1] || k[1] == k[2] || k[0] == k[2] {
                    continue;
                }
                tris.push(SoftTriangle {
                    v: [rep(k[0], &t.v[0]), rep(k[1], &t.v[1]), rep(k[2], &t.v[2])],
                    ..*t
                });
            }
            best = SoftMesh {
                tris,
                images: self.images.clone(),
            };
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
        let p = [
            (-1.0, t, 0.0),
            (1.0, t, 0.0),
            (-1.0, -t, 0.0),
            (1.0, -t, 0.0),
            (0.0, -1.0, t),
            (0.0, 1.0, t),
            (0.0, -1.0, -t),
            (0.0, 1.0, -t),
            (t, 0.0, -1.0),
            (t, 0.0, 1.0),
            (-t, 0.0, -1.0),
            (-t, 0.0, 1.0),
        ]
        .map(|(x, y, z)| Vector3::new(x, y, z).normalized());
        let faces = [
            (0, 11, 5),
            (0, 5, 1),
            (0, 1, 7),
            (0, 7, 10),
            (0, 10, 11),
            (1, 5, 9),
            (5, 11, 4),
            (11, 10, 2),
            (10, 7, 6),
            (7, 1, 8),
            (3, 9, 4),
            (3, 4, 2),
            (3, 2, 6),
            (3, 6, 8),
            (3, 8, 9),
            (4, 9, 5),
            (2, 4, 11),
            (6, 2, 10),
            (8, 6, 7),
            (9, 8, 1),
        ];
        let mut tris: Vec<[Vector3; 3]> = faces.iter().map(|&(a, b, c)| [p[a], p[b], p[c]]).collect();
        for _ in 0..levels {
            tris = tris
                .iter()
                .flat_map(|[a, b, c]| {
                    let (ab, bc, ca) = ((*a + *b).normalized(), (*b + *c).normalized(), (*c + *a).normalized());
                    [[*a, ab, ca], [ab, *b, bc], [ca, bc, *c], [ab, bc, ca]]
                })
                .collect();
        }
        let vert = |p: Vector3| SoftVertex {
            pos: p,
            normal: p,
            color,
            uv: Vector2::zero(),
        };
        SoftMesh {
            tris: tris
                .into_iter()
                .map(|[a, b, c]| SoftTriangle {
                    v: [vert(a), vert(b), vert(c)],
                    emissive: Vector3::zero(),
                    flat: false,
                    group: 0,
                    alpha: 1.0,
                    opacity: Opacity::Opaque,
                    texture: None,
                })
                .collect(),
            ..Default::default()
        }
    }
}

/// Runtime geometry shares identical corners, including UV and normal seams.
pub struct IndexedMesh {
    vertices: Vec<SoftVertex>,
    faces: Vec<Face>,
    images: std::sync::Arc<Vec<gltf::image::Data>>,
}
struct Face {
    indices: [u32; 3],
    emissive: Vector3,
    flat: bool,
    texture: Option<usize>,
    alpha: f32,
    opacity: Opacity,
}
impl IndexedMesh {
    pub fn triangle_count(&self) -> usize {
        self.faces.len()
    }
}
impl SoftMesh {
    pub fn indexed(self) -> IndexedMesh {
        let mut vertices = Vec::new();
        let mut corners = HashMap::new();
        let faces = self
            .tris
            .into_iter()
            .map(|t| {
                let indices = t.v.map(|v| {
                    let key = [
                        v.pos.x, v.pos.y, v.pos.z, v.normal.x, v.normal.y, v.normal.z, v.color.x, v.color.y,
                        v.color.z, v.uv.x, v.uv.y,
                    ]
                    .map(f32::to_bits);
                    *corners.entry(key).or_insert_with(|| {
                        let index = vertices.len() as u32;
                        vertices.push(v);
                        index
                    })
                });
                Face {
                    indices,
                    emissive: t.emissive,
                    flat: t.flat,
                    texture: t.texture,
                    alpha: t.alpha,
                    opacity: t.opacity,
                }
            })
            .collect();
        IndexedMesh {
            vertices,
            faces,
            images: self.images,
        }
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
    pub mesh: &'a IndexedMesh,
    pub world: Matrix,
    pub shade: Shade,
}

/// Un triángulo ya en la pantalla: posición (x, y en píxeles; 1/z), y lo que se interpola dividido por z.
#[derive(Clone, Copy)]
struct ScreenTri<'a> {
    bounds: [usize; 4],
    opacity: Opacity,
    uv: [Vector2; 3],
    texture: Option<&'a gltf::image::Data>,
    alpha: f32,
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

#[derive(Clone, Copy)]
pub struct FlightCamera {
    pub position: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub fovy: f32,
}
impl FlightCamera {
    pub fn perspective(position: Vector3, target: Vector3, up: Vector3, fovy: f32) -> Self {
        Self {
            position,
            target,
            up,
            fovy,
        }
    }
    pub fn raylib(self) -> Camera3D {
        Camera3D::perspective(self.position, self.target, self.up, self.fovy)
    }
    pub fn project(self, point: Vector3, width: i32, height: i32) -> Vector2 {
        let forward = (self.target - self.position).normalized();
        let right = forward.cross(self.up).normalized();
        let delta = point - self.position;
        let z = delta.dot(forward);
        let scale =
            height as f32 * 0.5 / ((self.fovy.to_radians() * 0.5).tan() * if z.abs() < 1e-6 { 1e-6 } else { z });
        Vector2::new(
            width as f32 * 0.5 + delta.dot(right) * scale,
            height as f32 * 0.5 - delta.dot(right.cross(forward)) * scale,
        )
    }
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
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        width: i32,
        height: i32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
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
        let stars = (0..2500)
            .map(|_| {
                let (z, a) = (rand() * 2.0 - 1.0, rand() * std::f32::consts::TAU);
                let r = (1.0 - z * z).sqrt();
                (Vector3::new(r * a.cos(), z, r * a.sin()), 0.3 + 0.7 * rand())
            })
            .collect();
        let n = (width * height) as usize;
        Ok(Self {
            width,
            height,
            color: vec![0; n * 4],
            depth: vec![0.0; n],
            texture,
            stars,
            stats: SoftStats::default(),
        })
    }

    /// Dibuja la escena en el buffer propio y lo sube a la textura.
    pub fn render(&mut self, camera: FlightCamera, sun: Vector3, draws: &[Draw], time: f32) {
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
            let (sx, sy) = (
                (dir.dot(r) / (z * tan_x) * 0.5 + 0.5) * w,
                (0.5 - dir.dot(u) / (z * tan_y) * 0.5) * h,
            );
            if sx >= 0.0 && sy >= 0.0 && sx < w && sy < h {
                let i = (sy as usize * self.width as usize + sx as usize) * 4;
                let v = (90.0 + 165.0 * bright) as u8;
                self.color[i..i + 3].copy_from_slice(&[v, v, (v as f32 * 1.05).min(255.0) as u8]);
            }
        }
        // 1-2. Vértices, recorte y proyección
        let mut screen = Vec::new();
        let mut translucent = Vec::new();
        let mut submitted = 0;
        for draw in draws {
            let mut rotation = draw.world;
            rotation.m12 = 0.;
            rotation.m13 = 0.;
            rotation.m14 = 0.;
            let vertices: Vec<_> = draw
                .mesh
                .vertices
                .iter()
                .map(|v| {
                    let wp = v.pos.transform_with(draw.world);
                    let d = wp - eye;
                    (
                        Vector3::new(d.dot(r), d.dot(u), d.dot(f)),
                        wp,
                        v.normal.transform_with(rotation),
                        v.color,
                        v.uv,
                    )
                })
                .collect();
            submitted += draw.mesh.faces.len();
            for t in &draw.mesh.faces {
                let verts = t.indices.map(|i| vertices[i as usize]);
                if verts.iter().all(|v| v.0.z < NEAR) {
                    continue;
                }
                let clipped;
                let polys = if verts.iter().all(|v| v.0.z >= NEAR) {
                    std::slice::from_ref(&verts)
                } else {
                    clipped = clip_near(&verts);
                    &clipped
                };
                for poly in polys {
                    let proj = poly.map(|(v, wp, n, c, uv)| {
                        let iz = 1.0 / v.z;
                        (
                            Vector3::new(
                                (v.x * iz / tan_x * 0.5 + 0.5) * w,
                                (0.5 - v.y * iz / tan_y * 0.5) * h,
                                iz,
                            ),
                            wp * iz,
                            n * iz,
                            c * iz,
                            uv * iz,
                        )
                    });
                    let (minx, maxx) = (
                        proj.iter().map(|p| p.0.x).fold(f32::MAX, f32::min),
                        proj.iter().map(|p| p.0.x).fold(f32::MIN, f32::max),
                    );
                    let (miny, maxy) = (
                        proj.iter().map(|p| p.0.y).fold(f32::MAX, f32::min),
                        proj.iter().map(|p| p.0.y).fold(f32::MIN, f32::max),
                    );
                    // Only pixel centers can be sampled. Most distant triangles cover none.
                    let left = (minx - 0.5).ceil().max(0.);
                    let right = (maxx - 0.5).floor().min(w - 1.);
                    let top = (miny - 0.5).ceil().max(0.);
                    let bottom = (maxy - 0.5).floor().min(h - 1.);
                    if left > right || top > bottom {
                        continue;
                    }
                    let queue = if matches!(t.opacity, Opacity::Blend) {
                        &mut translucent
                    } else {
                        &mut screen
                    };
                    queue.push(ScreenTri {
                        bounds: [left as usize, right as usize, top as usize, bottom as usize],
                        opacity: t.opacity,
                        uv: proj.map(|v| v.4),
                        texture: t.texture.map(|i| &draw.mesh.images[i]),
                        alpha: t.alpha,
                        p: proj.map(|p| p.0),
                        world: proj.map(|p| p.1),
                        normal: proj.map(|p| p.2),
                        color: proj.map(|p| p.3),
                        emissive: t.emissive,
                        flat: t.flat,
                        shade: draw.shade,
                    });
                }
            }
        }
        translucent.sort_unstable_by(|a, b| {
            let depth = |t: &ScreenTri| t.p[0].z + t.p[1].z + t.p[2].z;
            depth(a).total_cmp(&depth(b))
        });
        screen.append(&mut translucent);
        self.stats = SoftStats {
            submitted,
            rasterized: screen.len(),
            ms: 0.0,
        };
        // 3-4. Rasterizado y fragmentos, en franjas horizontales en paralelo
        let threads = std::thread::available_parallelism().map_or(4, |n| n.get()).clamp(1, 16);
        let rows = (self.height as usize).div_ceil(threads);
        let width = self.width as usize;
        let mut bands = vec![Vec::new(); (self.height as usize).div_ceil(rows)];
        for tri in &screen {
            for band in &mut bands[tri.bounds[2] / rows..=tri.bounds[3] / rows] {
                band.push(tri);
            }
        }
        std::thread::scope(|scope| {
            for (band, (color, depth)) in self
                .color
                .chunks_mut(rows * width * 4)
                .zip(self.depth.chunks_mut(rows * width))
                .enumerate()
            {
                let triangles = &bands[band];
                scope.spawn(move || {
                    let y0 = band * rows;
                    let y1 = y0 + depth.len() / width;
                    for t in triangles {
                        raster(t, color, depth, width, y0, y1, eye, sun, time);
                    }
                });
            }
        });
        self.texture
            .update_texture(&self.color)
            .expect("RGBA framebuffer matches presentation texture");
        self.stats.ms = started.elapsed().as_secs_f32() * 1000.0;
    }

    /// Muestra el resultado llenando la ventana.
    pub fn draw(&self, d: &mut impl RaylibDraw, width: i32, height: i32) {
        d.draw_texture_pro(
            &self.texture,
            Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32),
            Rectangle::new(0.0, 0.0, width as f32, height as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }
}

type ClipVert = (Vector3, Vector3, Vector3, Vector3, Vector2);

/// Recorta un triángulo (en espacio de vista) contra el plano cercano: devuelve 0, 1 o 2 triángulos.
fn clip_near(v: &[ClipVert; 3]) -> Vec<[ClipVert; 3]> {
    let lerp = |a: &ClipVert, b: &ClipVert| {
        let k = (NEAR - a.0.z) / (b.0.z - a.0.z);
        (
            a.0 + (b.0 - a.0) * k,
            a.1 + (b.1 - a.1) * k,
            a.2 + (b.2 - a.2) * k,
            a.3 + (b.3 - a.3) * k,
            a.4 + (b.4 - a.4) * k,
        )
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
fn raster(
    t: &ScreenTri,
    color: &mut [u8],
    depth: &mut [f32],
    width: usize,
    y0: usize,
    y1: usize,
    eye: Vector3,
    sun: Vector3,
    time: f32,
) {
    let [a, b, c] = t.p;
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if area.abs() < 1e-8 {
        return;
    }
    let [minx, maxx, top, bottom] = t.bounds;
    let miny = top.max(y0);
    let maxy = bottom.min(y1 - 1);
    let gamma = gamma();
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

            let z = 1.0 / iz;
            let mix = |v: [Vector3; 3]| (v[0] * w0 + v[1] * w1 + v[2] * w2) * z;
            let mut base = mix(t.color);
            let mut opacity = t.alpha;
            if let Some(image) = t.texture {
                let uv = (t.uv[0] * w0 + t.uv[1] * w1 + t.uv[2] * w2) * z;
                let (texel, alpha) = sample(image, uv);
                base = Vector3::new(base.x * texel.x, base.y * texel.y, base.z * texel.z);
                opacity *= alpha;
            }
            opacity = match t.opacity {
                Opacity::Opaque => 1.,
                Opacity::Mask(cutoff) if opacity >= cutoff => 1.,
                Opacity::Mask(_) => continue,
                Opacity::Blend if opacity <= 0. => continue,
                Opacity::Blend => opacity,
            };
            if opacity >= 0.999 {
                depth[di] = iz;
            }
            let rgb = fragment(t, mix(t.world), mix(t.normal), base, eye, sun, time);
            let ci = di * 4;
            for (i, value) in [rgb.x, rgb.y, rgb.z].into_iter().enumerate() {
                let lit = gamma[(value.clamp(0., 1.) * 4095.) as usize];
                color[ci + i] = if opacity >= 0.999 {
                    lit
                } else {
                    (lit as f32 * opacity + color[ci + i] as f32 * (1. - opacity)) as u8
                };
            }
        }
    }
}

fn gamma() -> &'static [u8; 4096] {
    static LUT: std::sync::OnceLock<[u8; 4096]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| std::array::from_fn(|i| ((i as f32 / 4095.).powf(1. / 2.2) * 255.).round() as u8))
}

fn sample(image: &gltf::image::Data, uv: Vector2) -> (Vector3, f32) {
    use gltf::image::Format::*;
    let channels = match image.format {
        R8 => 1,
        R8G8 => 2,
        R8G8B8 => 3,
        R8G8B8A8 => 4,
        _ => unreachable!(),
    };
    let repeat = |coord: f32, size: u32| (((coord - coord.floor()) * size as f32) as usize).min(size as usize - 1);
    let x = repeat(uv.x, image.width);
    let y = repeat(uv.y, image.height);
    let pixel = &image.pixels[(y * image.width as usize + x) * channels..];
    static LUT: std::sync::OnceLock<[f32; 256]> = std::sync::OnceLock::new();
    let linear = LUT.get_or_init(|| std::array::from_fn(|i| (i as f32 / 255.).powf(2.2)));
    let ch = |i: usize| linear[pixel[i] as usize];
    (
        if channels < 3 {
            Vector3::one() * ch(0)
        } else {
            Vector3::new(ch(0), ch(1), ch(2))
        },
        if channels == 2 || channels == 4 {
            pixel[channels - 1] as f32 / 255.
        } else {
            1.
        },
    )
}

/// Ruido de valor 3D (suave, entre 0 y 1).
fn noise(p: Vector3) -> f32 {
    let hash = |x: f32, y: f32, z: f32| ((x * 127.1 + y * 311.7 + z * 74.7).sin() * 43_758.547).fract().abs();
    let (i, f) = (
        Vector3::new(p.x.floor(), p.y.floor(), p.z.floor()),
        Vector3::new(p.x.fract(), p.y.fract(), p.z.fract()),
    );
    let f = Vector3::new(f.x.rem_euclid(1.0), f.y.rem_euclid(1.0), f.z.rem_euclid(1.0));
    let s = Vector3::new(
        f.x * f.x * (3.0 - 2.0 * f.x),
        f.y * f.y * (3.0 - 2.0 * f.y),
        f.z * f.z * (3.0 - 2.0 * f.z),
    );
    let l = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let c = |dx: f32, dy: f32, dz: f32| hash(i.x + dx, i.y + dy, i.z + dz);
    l(
        l(
            l(c(0., 0., 0.), c(1., 0., 0.), s.x),
            l(c(0., 1., 0.), c(1., 1., 0.), s.x),
            s.y,
        ),
        l(
            l(c(0., 0., 1.), c(1., 0., 1.), s.x),
            l(c(0., 1., 1.), c(1., 1., 1.), s.x),
            s.y,
        ),
        s.z,
    )
}

/// Los "shaders de fragmentos".
fn fragment(
    t: &ScreenTri,
    world: Vector3,
    normal: Vector3,
    color: Vector3,
    eye: Vector3,
    sun: Vector3,
    time: f32,
) -> Vector3 {
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
            let c = warm + (hot - warm) * k.clamp(0.0, 1.0);
            Vector3::new(c.x.powf(2.2), c.y.powf(2.2), c.z.powf(2.2))
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
        z.map(|z| {
            (
                Vector3::new(0.0, 0.0, z),
                Vector3::zero(),
                Vector3::zero(),
                Vector3::zero(),
                Vector2::zero(),
            )
        })
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
    fn texture_coordinates_and_visibility_are_perspective_correct() {
        // At (0.5,0.5) affine interpolation gives U=0.25, perspective gives U=1/13.
        let texture = gltf::image::Data {
            width: 4,
            height: 1,
            format: gltf::image::Format::R8G8B8A8,
            pixels: vec![255, 0, 0, 128, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255],
        };
        let mut triangle = ScreenTri {
            bounds: [0; 4],
            opacity: Opacity::Blend,
            alpha: 1.,
            p: [
                Vector3::new(0., 0., 1.),
                Vector3::new(2., 0., 0.25),
                Vector3::new(0., 2., 1.),
            ],
            uv: [Vector2::zero(), Vector2::new(0.25, 0.), Vector2::zero()],
            texture: Some(&texture),
            world: [Vector3::zero(); 3],
            normal: [Vector3::up(); 3],
            color: [Vector3::one(), Vector3::one() * 0.25, Vector3::one()],
            emissive: Vector3::zero(),
            flat: true,
            shade: Shade::Lit,
        };
        let mut color = [0, 0, 0, 255];
        let mut depth = [0.];
        raster(
            &triangle,
            &mut color,
            &mut depth,
            1,
            0,
            1,
            Vector3::up(),
            Vector3::one(),
            0.,
        );
        assert_eq!(color, [128, 0, 0, 255]);
        assert_eq!(depth, [0.]);
        triangle.opacity = Opacity::Opaque;
        raster(
            &triangle,
            &mut color,
            &mut depth,
            1,
            0,
            1,
            Vector3::up(),
            Vector3::one(),
            0.,
        );
        assert_eq!(color, [255, 0, 0, 255]);
        assert_eq!(depth, [0.8125]);
        triangle.opacity = Opacity::Mask(0.8);
        depth[0] = 0.;
        raster(
            &triangle,
            &mut color,
            &mut depth,
            1,
            0,
            1,
            Vector3::up(),
            Vector3::one(),
            0.,
        );
        assert_eq!(depth, [0.]);
    }

    #[test]
    fn la_esfera_se_subdivide_y_queda_sobre_la_superficie() {
        for levels in 0..4 {
            let s = SoftMesh::sphere(levels, Vector3::one());
            assert_eq!(s.tris.len(), 20 * 4usize.pow(levels));
            assert!(s
                .tris
                .iter()
                .flat_map(|t| t.v)
                .all(|v| (v.pos.length() - 1.0).abs() < 1e-4));
        }
    }

    #[test]
    fn el_nivel_de_detalle_respeta_el_presupuesto() {
        let full = SoftMesh::sphere(5, Vector3::one());
        for budget in [8_000, 2_000, 500] {
            let lite = full.decimate(budget);
            assert!(
                lite.tris.len() <= budget && !lite.tris.is_empty(),
                "{budget}: {}",
                lite.tris.len()
            );
        }
    }
}
