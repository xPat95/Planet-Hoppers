//! Representación del sistema solar (fases 3 y 4): modelos GLB ajustados en el juego, luz del Sol, superficie y
//! halo del Sol, y cielo de fondo. El movimiento vive en `solar_system` y `ships`.
use super::resources::glb_document;
use raylib::{ffi, prelude::*};
use std::path::Path;

/// Shader de planetas y naves: el Sol del centro es la única luz.
pub struct SpaceShader {
    pub shader: Shader,
    camera_position: Vector3,
    light: i32,
    camera: i32,
    emissive: i32,
    ambient: i32,
    outline: i32,
}

impl SpaceShader {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_fragment(rl, thread, include_str!("shaders/space.fs"))
    }

    /// Local architectural lighting; shared loader, no changes to flight or ship materials.
    pub fn load_hangar(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Box<dyn std::error::Error>> {
        let mut shader = Self::load_fragment(rl, thread, include_str!("shaders/hangar.fs"))?;
        let radiance = shader.shader.get_shader_location("bakedRadiance");
        shader.shader.set_shader_value(radiance, 1.0f32);
        Ok(shader)
    }

    fn load_fragment(rl: &mut RaylibHandle, thread: &RaylibThread, fragment: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let shader = rl.load_shader_from_memory(
            thread,
            Some(include_str!("shaders/space.vs")),
            Some(fragment),
        );
        let loc = |name: &str| shader.get_shader_location(name);
        let (light, camera, emissive, ambient) = (loc("lightPosition"), loc("cameraPosition"), loc("emissive"), loc("ambient"));
        let outline = loc("outline");
        if !shader.is_shader_valid() || [light, camera, emissive, ambient, outline].contains(&-1) {
            return Err("El shader del espacio no compiló".into());
        }
        Ok(Self { shader, camera_position: Vector3::zero(), light, camera, emissive, ambient, outline })
    }

    /// Valores que no cambian en todo el cuadro.
    pub fn begin_frame(&mut self, sun: Vector3, camera: Vector3, ambient: f32) {
        self.camera_position = camera;
        self.shader.set_shader_value(self.light, sun);
        self.shader.set_shader_value(self.camera, camera);
        self.shader.set_shader_value(self.ambient, ambient);
    }
}

/// Shader de la superficie del Sol (sin luz: brilla solo).
// Apariencia solar: no cambia el radio, las colisiones ni la luz de los planetas.
const SUN_EMISSION: f32 = 3.2;
const SUN_DETAIL_SCALE: f32 = 18.0;
const SUN_ACTIVITY_SPEED: f32 = 0.08;
const SUN_CONTRAST: f32 = 0.65;
const SUN_HALO_LAYERS: [(f32, Color); 4] = [
    (5.0, Color::new(255, 165, 70, 12)),
    (2.8, Color::new(255, 185, 90, 32)),
    (1.65, Color::new(255, 215, 140, 100)),
    (1.18, Color::new(255, 245, 210, 210)),
];
const SUN_HALO_SCREEN_LIMIT: f32 = 1.6;

pub struct SunShader {
    pub shader: Shader,
    camera: i32,
    center: i32,
    time: i32,
}

impl SunShader {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Box<dyn std::error::Error>> {
        let mut shader = rl.load_shader_from_memory(
            thread,
            Some(include_str!("shaders/space.vs")),
            Some(include_str!("shaders/sun.fs")),
        );
        let loc = |name: &str| shader.get_shader_location(name);
        let (camera, center, time, surface) = (loc("cameraPosition"), loc("sunCenter"), loc("time"), loc("surfaceSettings"));
        if !shader.is_shader_valid() || [camera, center, time, surface].contains(&-1) {
            return Err("El shader del Sol no compiló".into());
        }
        shader.set_shader_value(surface, Vector4::new(SUN_EMISSION, SUN_DETAIL_SCALE, SUN_ACTIVITY_SPEED, SUN_CONTRAST));
        Ok(Self { shader, camera, center, time })
    }

    pub fn begin_frame(&mut self, center: Vector3, camera: Vector3, time: f32) {
        self.shader.set_shader_value(self.camera, camera);
        self.shader.set_shader_value(self.center, center);
        self.shader.set_shader_value(self.time, time);
    }
}

/// A qué tamaño se ajusta un modelo.
#[derive(Clone, Copy)]
pub enum Fit {
    /// El semieje más grande de la caja queda en este radio (planetas).
    Radius(f32),
    /// La medida más larga queda en este largo (naves).
    Length(f32),
    /// Mismo centro y escala que otro modelo (la versión lejana usa los del modelo cercano, para que no salte).
    Same(Matrix, Vector3),
}

struct Part {
    mesh: usize,
    material: usize,
    emissive: Vector3,
    transparent: bool,
    /// Contorno de caricatura: casco invertido negro (material `*Outline`), se dibuja descartando las caras
    /// de adelante para que solo asome el borde.
    outline: bool,
    /// Personaje de caricatura (material `*_Sticker`): el color de su imagen tal cual, sin luz ni sombra.
    sticker: bool,
    /// Malla rearmada que reemplaza a la de raylib (ver `rebuild_large_meshes`).
    rebuilt: Option<Mesh>,
}

/// Bloque binario (BIN) de un GLB.
fn glb_bin(bytes: &[u8]) -> Option<&[u8]> {
    let json_len = u32::from_le_bytes(bytes.get(12..16)?.try_into().ok()?) as usize;
    let start = 20 + json_len;
    let len = u32::from_le_bytes(bytes.get(start..start + 4)?.try_into().ok()?) as usize;
    (bytes.get(start + 4..start + 8)? == b"BIN\0").then(|| bytes.get(start + 8..start + 8 + len))?
}

/// Índices de una primitiva tal como están en el GLB (u8, u16 o u32).
fn glb_indices(gltf: &serde_json::Value, bin: &[u8], accessor: usize) -> Option<Vec<u32>> {
    let a = &gltf["accessors"][accessor];
    let view = &gltf["bufferViews"][a["bufferView"].as_u64()? as usize];
    let start = view["byteOffset"].as_u64().unwrap_or(0) as usize + a["byteOffset"].as_u64().unwrap_or(0) as usize;
    let count = a["count"].as_u64()? as usize;
    let data = bin.get(start..)?;
    let read = |size: usize, f: fn(&[u8]) -> u32| (0..count).map(|i| data.get(i * size..i * size + size).map(f)).collect();
    match a["componentType"].as_u64()? {
        5121 => read(1, |b| b[0] as u32),
        5123 => read(2, |b| u16::from_le_bytes([b[0], b[1]]) as u32),
        5125 => read(4, |b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])),
        _ => None,
    }
}

/// raylib 5.5 pasa a 16 bits los índices de 32 y desarma las mallas de más de 65 535 vértices (dos del planeta de
/// Star Wars). Esas se rearman sin índices, con los índices verdaderos del GLB y los vértices que ya cargó raylib
/// (con la posición de cada nodo aplicada). El archivo no se toca.
fn rebuild_large_meshes(model: &Model, gltf: &serde_json::Value, bytes: &[u8]) -> Vec<Option<Mesh>> {
    let mut rebuilt: Vec<Option<Mesh>> = (0..model.meshes().len()).map(|_| None).collect();
    let Some(bin) = glb_bin(bytes) else { return rebuilt };
    // raylib recorre los nodos en orden y crea una malla por cada primitiva de su malla
    let primitives: Vec<&serde_json::Value> = gltf["nodes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|node| node["mesh"].as_u64())
        .flat_map(|m| gltf["meshes"][m as usize]["primitives"].as_array().into_iter().flatten())
        .collect();
    if primitives.len() != model.meshes().len() {
        return rebuilt;
    }
    for (i, (mesh, prim)) in model.meshes().iter().zip(primitives).enumerate() {
        let raw = mesh.as_ref();
        if raw.vertexCount <= 65_535 || raw.vertices.is_null() {
            continue;
        }
        let Some(indices) = prim["indices"].as_u64().and_then(|a| glb_indices(gltf, bin, a as usize)) else { continue };
        let n = raw.vertexCount as usize;
        if indices.len() != raw.triangleCount as usize * 3 || indices.iter().any(|&k| k as usize >= n) {
            continue;
        }
        let src = |ptr: *const f32, width: usize| (!ptr.is_null()).then(|| unsafe { std::slice::from_raw_parts(ptr, n * width) });
        let (pos, nor, uv) = (src(raw.vertices, 3), src(raw.normals, 3), src(raw.texcoords, 2));
        let expand = |data: Option<&[f32]>, width: usize| {
            data.map(|d| indices.iter().flat_map(|&k| d[k as usize * width..k as usize * width + width].iter().copied()).collect::<Vec<f32>>())
        };
        rebuilt[i] = upload(&expand(pos, 3).unwrap_or_default(), expand(nor, 3).as_deref(), expand(uv, 2).as_deref());
    }
    rebuilt
}

/// Sube a la GPU una malla sin índices (mismo método que `resources::StaticModel`).
fn upload(positions: &[f32], normals: Option<&[f32]>, uvs: Option<&[f32]>) -> Option<Mesh> {
    let mut mesh = unsafe { Mesh::from_raw(std::mem::zeroed()) };
    mesh.as_mut().vertexCount = (positions.len() / 3) as i32;
    mesh.as_mut().triangleCount = (positions.len() / 9) as i32;
    let copy = |values: &[f32]| -> Option<*mut f32> {
        let ptr = unsafe { ffi::MemAlloc(u32::try_from(std::mem::size_of_val(values)).ok()?) } as *mut f32;
        (!ptr.is_null()).then(|| {
            unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), ptr, values.len()) };
            ptr
        })
    };
    mesh.as_mut().vertices = copy(positions)?;
    if let Some(n) = normals {
        mesh.as_mut().normals = copy(n)?;
    }
    if let Some(t) = uvs {
        mesh.as_mut().texcoords = copy(t)?;
    }
    unsafe { ffi::UploadMesh(mesh.as_mut(), false) };
    (mesh.as_ref().vaoId != 0).then_some(mesh)
}

/// Modelo GLB listo para el sistema solar. El archivo no se toca: el centro (pivote), la orientación y la escala
/// se corrigen con la matriz `fit`, que lleva del espacio del archivo al espacio del planeta o la nave.
pub struct SpaceModel {
    parts: Vec<Part>,
    model: Model,
    pub fit: Matrix,
    /// Semiejes de la caja ya ajustada (sirven como forma de choque).
    pub half_extents: Vector3,
    pub triangles: usize,
}

fn rotation(degrees: [f32; 3]) -> Matrix {
    Matrix::rotate_x(degrees[0].to_radians())
        * Matrix::rotate_y(degrees[1].to_radians())
        * Matrix::rotate_z(degrees[2].to_radians())
}

impl SpaceModel {
    /// Apply filtering explicitly to architectural lightmaps; ships keep their settings.
    pub fn filter_albedo(&mut self, filter: TextureFilter) {
        for material in self.model.materials() {
            let texture = material.maps()[0].texture;
            if texture.id != 0 {
                unsafe { ffi::SetTextureFilter(texture, filter as i32) };
            }
        }
    }

    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &Path,
        shader: &SpaceShader,
        rotation_degrees: [f32; 3],
        fit: Fit,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let gltf = glb_document(path)?;
        let bytes = std::fs::read(path)?;
        let mut model = rl
            .load_model(thread, path.to_str().ok_or("Ruta de modelo inválida")?)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if !model.is_model_valid() {
            return Err(format!("{}: raylib no pudo cargarlo", path.display()).into());
        }
        // Emisión de cada material (raylib no la pasa al shader): sale del JSON del GLB. raylib agrega un material
        // por defecto al principio, así que el material i del archivo es el i + 1 del modelo.
        let materials = gltf["materials"].as_array().cloned().unwrap_or_default();
        let emission = |i: usize| -> (Vector3, bool, bool, bool) {
            let Some(m) = i.checked_sub(1).and_then(|k| materials.get(k)) else {
                return (Vector3::zero(), false, false, false);
            };
            let f = |k: usize| m["emissiveFactor"][k].as_f64().unwrap_or(0.0) as f32;
            let strength = m["extensions"]["KHR_materials_emissive_strength"]["emissiveStrength"].as_f64().unwrap_or(1.0) as f32;
            let outline = m["name"].as_str().is_some_and(|n| n.ends_with("Outline"));
            let sticker = m["name"].as_str().is_some_and(|n| n.ends_with("_Sticker"));
            (Vector3::new(f(0), f(1), f(2)) * strength, m["alphaMode"].as_str() == Some("BLEND"), outline, sticker)
        };
        for material in model.materials_mut() {
            *material.shader_mut() = unsafe { WeakShader::from_raw(*shader.shader.as_ref()) };
        }
        let mut rebuilt = rebuild_large_meshes(&model, &gltf, &bytes);
        let mut parts = Vec::new();
        let mut triangles = 0;
        for i in 0..model.meshes().len() {
            let material = unsafe { *model.as_ref().meshMaterial.add(i) } as usize;
            let (emissive, blend, outline, sticker) = emission(material);
            let alpha = model.materials().get(material).map_or(255, |m| m.maps()[0].color.a);
            triangles += model.meshes()[i].as_ref().triangleCount.max(0) as usize;
            parts.push(Part {
                mesh: i,
                material,
                emissive,
                transparent: blend || alpha < 250,
                outline,
                sticker,
                rebuilt: rebuilt[i].take(),
            });
        }
        // Lo transparente al final, para que se vea lo que hay detrás
        parts.sort_by_key(|p| p.transparent);

        let (fit, half_extents) = match fit {
            Fit::Same(m, h) => (m, h),
            Fit::Radius(_) | Fit::Length(_) => {
                let b = model.get_model_bounding_box();
                let center = (b.min + b.max) * 0.5;
                let rot = rotation(rotation_degrees);
                // Caja del modelo ya centrado y girado
                let mut half = Vector3::zero();
                for k in 0..8 {
                    let corner = Vector3::new(
                        if k & 1 == 0 { b.min.x } else { b.max.x },
                        if k & 2 == 0 { b.min.y } else { b.max.y },
                        if k & 4 == 0 { b.min.z } else { b.max.z },
                    );
                    let c = (corner - center).transform_with(rot);
                    half = Vector3::new(half.x.max(c.x.abs()), half.y.max(c.y.abs()), half.z.max(c.z.abs()));
                }
                let biggest = half.x.max(half.y).max(half.z).max(1e-6);
                let scale = match fit {
                    Fit::Radius(r) => r / biggest,
                    Fit::Length(l) => l / (2.0 * biggest),
                    Fit::Same(..) => unreachable!(),
                };
                let m = Matrix::translate(-center.x, -center.y, -center.z) * rot * Matrix::scale(scale, scale, scale);
                (m, half * scale)
            }
        };
        Ok(Self { parts, model, fit, half_extents, triangles })
    }

    /// Dibuja con `world` (del espacio del planeta o nave al mundo). Sin descartar caras traseras: algunas mallas
    /// vienen con normales invertidas y el shader las corrige.
    pub fn draw(&self, d: &mut impl RaylibDraw3D, shader: &mut SpaceShader, world: Matrix) {
        let transform = self.fit * world;
        unsafe { ffi::rlDisableBackfaceCulling() };
        let mut depth_writes = true;
        for part in &self.parts {
            if part.transparent && depth_writes {
                unsafe { ffi::rlDisableDepthMask() };
                depth_writes = false;
            }
            shader.shader.set_shader_value(shader.emissive, part.emissive);
            shader.shader.set_shader_value(shader.outline, if part.sticker { 2.0f32 } else if part.outline { 1.0 } else { 0.0 });
            if part.outline {
                unsafe { ffi::rlEnableBackfaceCulling() };
            }
            let material = self.model.materials()[part.material].clone();
            match &part.rebuilt {
                Some(mesh) => d.draw_mesh(mesh, material, transform),
                None => d.draw_mesh(&self.model.meshes()[part.mesh], material, transform),
            }
            if part.outline {
                unsafe { ffi::rlDisableBackfaceCulling() };
            }
        }
        shader.shader.set_shader_value(shader.outline, 0.0f32);
        unsafe {
            ffi::rlEnableDepthMask();
            ffi::rlEnableBackfaceCulling();
        }
    }
}

// Parámetros del fondo espacial, independientes de los materiales y efectos del Sol.
pub const SKY_BACKGROUND: Color = Color::new(0, 1, 3, 255);
const SKY_STAR_COUNT: usize = 2200;
const SKY_SEED: u32 = 0x5eed_1234;
const SKY_TINTS: [(u8, u8, u8); 4] = [(155, 180, 215), (180, 200, 225), (205, 215, 235), (220, 225, 235)];
// Probabilidad acumulada, radio en píxeles y opacidad: predominan los puntos apenas visibles.
const SKY_STAR_TIERS: [(f32, (f32, f32), (f32, f32)); 3] = [
    (0.80, (0.35, 0.75), (22.0, 68.0)),
    (0.98, (0.60, 1.00), (70.0, 120.0)),
    (1.00, (1.00, 1.25), (135.0, 175.0)),
];

/// Estrella del fondo. Está "en el infinito": solo importa su dirección, así que no se acerca al mover la cámara
/// y el espacio se siente inmenso.
struct Star {
    dir: Vector3,
    size: f32,
    color: Color,
}

/// Generador simple con semilla: el cielo es siempre el mismo.
fn rng(mut seed: u32) -> impl FnMut() -> f32 {
    move || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// Distribución uniforme sobre la esfera, sin cuadrícula, franjas ni textura repetida.
fn sky_dir(next: &mut impl FnMut() -> f32) -> Vector3 {
    let a = next() * std::f32::consts::TAU;
    let z = next() * 2.0 - 1.0;
    let s = (1.0 - z * z).sqrt();
    Vector3::new(s * a.cos(), z, s * a.sin())
}

/// Vacío frío y oscuro: estrellas pequeñas y estables, sin nubes, bandas ni halos.
pub struct Sky {
    stars: Vec<Star>,
}

/// El cielo ya proyectado en la pantalla para este cuadro.
pub struct SkyFrame {
    stars: Vec<(Vector2, usize)>,
}

impl Sky {
    pub fn new() -> Self {
        let mut next = rng(SKY_SEED);
        let mut stars = Vec::with_capacity(SKY_STAR_COUNT);
        for _ in 0..SKY_STAR_COUNT {
            let dir = sky_dir(&mut next);
            let m = next();
            let (_, (min_size, max_size), (min_alpha, max_alpha)) = SKY_STAR_TIERS.iter().find(|tier| m < tier.0).unwrap();
            let size = min_size + next() * (max_size - min_size);
            let alpha = min_alpha + next() * (max_alpha - min_alpha);
            let (r, g, b) = SKY_TINTS[(next() * SKY_TINTS.len() as f32) as usize];
            stars.push(Star { dir, size, color: Color::new(r, g, b, alpha as u8) });
        }
        Self { stars }
    }

    /// Proyecta el cielo desde la cámara (antes de empezar a dibujar).
    pub fn project(&self, rl: &RaylibHandle, camera: Camera3D) -> SkyFrame {
        // Proyectar desde el origen evita pérdida de precisión al viajar lejos:
        // el fondo depende exclusivamente de la orientación de la cámara.
        let mut camera = camera;
        camera.target -= camera.position;
        camera.position = Vector3::zero();
        let forward = (camera.target - camera.position).normalized();
        let project = |dir: Vector3| {
            (dir.dot(forward) > 0.05).then(|| rl.get_world_to_screen(camera.position + dir * 1000.0, camera))
        };
        SkyFrame {
            stars: self.stars.iter().enumerate().filter_map(|(i, s)| project(s.dir).map(|at| (at, i))).collect(),
        }
    }

    pub fn draw(&self, d: &mut impl RaylibDraw, frame: &SkyFrame) {
        for (at, i) in &frame.stars {
            let s = &self.stars[*i];
            d.draw_circle_v(*at, s.size, s.color);
        }
    }
}

impl Default for Sky {
    fn default() -> Self {
        Self::new()
    }
}

/// Resplandor solar aditivo, detrás de la geometría: los planetas y naves lo tapan.
/// Sustituye un bloom de pantalla completa sin render targets ni pasadas de desenfoque.
pub fn draw_sun_glow(d: &mut impl RaylibDraw, at: Vector2, radius: f32, viewport: Vector2) {
    let (width, height) = (viewport.x, viewport.y);
    // Atenuar y acotar el área al acercarse: no cubrir de naranja toda la navegación.
    let intensity = (width.min(height) / (radius * 3.0).max(1.0)).min(1.0);
    let limit = width.max(height) * SUN_HALO_SCREEN_LIMIT;
    let (x, y) = (at.x as i32, at.y as i32);
    let mut glow = d.begin_blend_mode(BlendMode::BLEND_ADDITIVE);
    for (k, color) in SUN_HALO_LAYERS {
        let extent = (radius * k).min(limit);
        if at.x + extent < 0.0 || at.y + extent < 0.0 || at.x - extent > width || at.y - extent > height {
            continue;
        }
        let inner = Color::new(color.r, color.g, color.b, (color.a as f32 * intensity) as u8);
        glow.draw_circle_gradient(x, y, extent, inner, Color::new(color.r, color.g, color.b, 0));
    }
}
