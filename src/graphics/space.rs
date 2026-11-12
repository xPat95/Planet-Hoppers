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
    transport: i32,
    dissolve: i32,
    roughness: i32,
    studio: i32,
    planet_fill: i32,
    planet_key_position: i32,
    planet_key: i32,
    planet_key_range: i32,
    planet_wet: i32,
    wet_surface: i32,
    planet_toon: i32,
    no_bloom: i32,
    /// Mientras se dibuja un modelo que no tiene que brillar en el bloom: las partes cuyo material contiene alguno de
    /// estos nombres sí brillan (en Smiling Friends, Satanás). `None`: todo brilla como siempre.
    pub bloom_only: Option<&'static [&'static str]>,
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

    /// Con un shader de fragmentos propio (por ejemplo, la luz de una escena de minijuego). Tiene que declarar y usar
    /// `lightPosition`, `cameraPosition`, `emissive`, `ambient`, `outline` y `transportGlow`.
    pub fn load_custom(rl: &mut RaylibHandle, thread: &RaylibThread, fragment: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_fragment(rl, thread, fragment)
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
        let transport = loc("transportGlow");
        let dissolve = loc("transportDissolve");
        if !shader.is_shader_valid() || [light, camera, emissive, ambient, outline, transport].contains(&-1) {
            return Err("El shader del espacio no compiló".into());
        }
        let (roughness, studio, planet_fill) = (loc("roughness"), loc("studio"), loc("planetFill"));   // Solo en space.fs (-1 en el del hangar)
        let (planet_key_position, planet_key, planet_key_range, planet_wet) =
            (loc("planetKeyPosition"), loc("planetKey"), loc("planetKeyRange"), loc("planetWet"));
        let planet_toon = loc("planetToon");
        let wet_surface = loc("wetSurface");
        let no_bloom = loc("noBloom");
        Ok(Self { shader, camera_position: Vector3::zero(), light, camera, emissive, ambient, outline, transport, dissolve,
                  roughness, studio, planet_fill, planet_key_position, planet_key, planet_key_range, planet_wet, wet_surface, planet_toon, no_bloom, bloom_only: None })
    }

    /// Valores que no cambian en todo el cuadro.
    pub fn begin_frame(&mut self, sun: Vector3, camera: Vector3, ambient: f32) {
        self.camera_position = camera;
        self.shader.set_shader_value(self.light, sun);
        self.shader.set_shader_value(self.camera, camera);
        self.shader.set_shader_value(self.ambient, ambient);
        self.shader.set_shader_value(self.transport, 0.0f32);
        if self.dissolve >= 0 { self.shader.set_shader_value(self.dissolve, 0.0f32); }
        self.set_planet_fill(Vector3::zero());
        self.set_planet_key(Vector3::zero(), Vector4::new(0.0, 0.0, 0.0, 0.0), 1.0, false);
        self.set_planet_toon(false);
    }
    pub fn set_transport_glow(&mut self, intensity: f32) {
        self.shader.set_shader_value(self.transport, intensity);
    }
    /// Luces de estudio (clave, contorno y de abajo) para lucir la nave en el hangar; en el espacio, apagadas.
    pub fn set_studio(&mut self, on: bool) {
        if self.studio >= 0 { self.shader.set_shader_value(self.studio, if on { 1.0f32 } else { 0.0 }); }
    }
    pub fn set_planet_fill(&mut self, fill: Vector3) {
        if self.planet_fill >= 0 { self.shader.set_shader_value(self.planet_fill, fill); }
    }
    pub fn set_planet_toon(&mut self, on: bool) {
        if self.planet_toon >= 0 { self.shader.set_shader_value(self.planet_toon, if on { 1.0f32 } else { 0.0 }); }
    }
    pub fn set_planet_key(&mut self, position: Vector3, key: Vector4, range: f32, wet: bool) {
        if self.planet_key_position >= 0 { self.shader.set_shader_value(self.planet_key_position, position); }
        if self.planet_key >= 0 { self.shader.set_shader_value(self.planet_key, key); }
        if self.planet_key_range >= 0 { self.shader.set_shader_value(self.planet_key_range, range); }
        if self.planet_wet >= 0 { self.shader.set_shader_value(self.planet_wet, if wet { 1.0f32 } else { 0.0 }); }
    }
    pub fn set_transport_dissolve(&mut self, amount: f32) {
        if self.dissolve >= 0 { self.shader.set_shader_value(self.dissolve, amount); }
    }
}

/// Shader de la superficie del Sol (sin luz: brilla solo).
// Apariencia solar: no cambia el radio, las colisiones ni la luz de los planetas.
const SUN_EMISSION: f32 = 1.7;
const SUN_DETAIL_SCALE: f32 = 18.0;
const SUN_ACTIVITY_SPEED: f32 = 0.08;
const SUN_CONTRAST: f32 = 0.65;
const SUN_HALO_LAYERS: [(f32, Color); 4] = [
    (1.70, Color::new(255, 85, 24, 12)),
    (1.38, Color::new(255, 110, 32, 48)),
    (1.20, Color::new(255, 145, 48, 125)),
    (1.07, Color::new(255, 205, 115, 165)),
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
    /// Nombre del material en el GLB (para ocultar piezas o cambiar su brillo).
    name: String,
    material: usize,
    emissive: Vector3,
    transparent: bool,
    /// Contorno de caricatura: casco invertido negro (material `*Outline`), se dibuja descartando las caras
    /// de adelante para que solo asome el borde.
    outline: bool,
    /// Personaje de caricatura (material `*_Sticker`): el color de su imagen tal cual, sin luz ni sombra.
    sticker: bool,
    /// Rugosidad del material (glTF `roughnessFactor`, 1 si no tiene): define qué tan chico y fuerte es el brillo.
    roughness: f32,
    wet_surface: f32,
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
        // Blender packages include a prepared near GLB beside their authoring OBJ.
        // Prefer that runtime export: same asset identity/materials, without repeatedly
        // parsing the large text mesh. The normal GLB path stays unchanged.
        if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("obj")) {
            if let Some(stem)=path.file_stem().and_then(|s|s.to_str()) {
                let prepared=path.with_file_name(format!("{stem}_near.glb"));
                if prepared.is_file() {
                    return Self::load(rl,thread,&prepared,shader,rotation_degrees,fit);
                }
            }
        }
        // Small OBJ files without a prepared export retain Raylib's MTL/texture loader.
        let gltf = if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("obj")) {
            serde_json::json!({})
        } else { glb_document(path)? };
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
            let inner_boost = match m["name"].as_str() {
                Some("HW_InnerLava" | "HW_InnerGlowHot") => 1.55,
                Some("HW_InnerGlow") => 1.25,
                _ => 1.0,
            };
            let outline = m["name"].as_str().is_some_and(|n| n.ends_with("Outline"));
            let sticker = m["name"].as_str().is_some_and(|n| n.ends_with("_Sticker"));
            let inner_bounce = match m["name"].as_str() {
                Some("HW_Inner") => Vector3::new(0.075, 0.010, 0.004),
                Some("HW_InnerCrust") => Vector3::new(0.045, 0.007, 0.003),
                _ => Vector3::zero(),
            };
            (Vector3::new(f(0), f(1), f(2)) * strength * inner_boost + inner_bounce,
             m["alphaMode"].as_str() == Some("BLEND"), outline, sticker)
        };
        let roughness = |i: usize| -> f32 {
            i.checked_sub(1).and_then(|k| materials.get(k)).map_or(1.0, |m| {
                m["pbrMetallicRoughness"]["roughnessFactor"].as_f64().unwrap_or(1.0) as f32
            })
        };
        let wet_surface = |i: usize| -> f32 {
            match i.checked_sub(1).and_then(|k| materials.get(k)).and_then(|m| m["name"].as_str()) {
                Some("HW_Blood" | "HW_Lava") => 2.0,
                Some("HW_RustDark") => 1.0,
                Some(name) if name.starts_with("HW_Strata") => 1.0,
                _ => 0.0,
            }
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
            let name = material.checked_sub(1).and_then(|k| materials.get(k))
                .and_then(|m| m["name"].as_str()).unwrap_or_default().to_string();
            parts.push(Part {
                mesh: i,
                name,
                material,
                emissive,
                transparent: blend || alpha < 250,
                outline,
                sticker,
                roughness: roughness(material),
                wet_surface: wet_surface(material),
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

    /// Deja de dibujar las piezas cuyo material empieza con alguno de estos nombres.
    /// Cuántos triángulos manda a dibujar este modelo (para el contador de rendimiento, F4).
    pub fn triangles(&self) -> usize {
        self.parts.iter().map(|p| match &p.rebuilt {
            Some(mesh) => mesh.triangleCount.max(0) as usize,
            None => self.model.meshes()[p.mesh].triangleCount.max(0) as usize,
        }).sum()
    }

    pub fn hide_materials(&mut self, prefixes: &[&str]) {
        self.parts.retain(|p| !prefixes.iter().any(|x| p.name.starts_with(x)));
    }

    /// Vértices y normales de cada malla, en el orden del archivo (para mezclar poses de la misma malla).
    pub fn mesh_vertices(&self) -> Vec<(Vec<Vector3>, Vec<Vector3>)> {
        self.model.meshes().iter().map(|m| (m.vertices().to_vec(), m.normals().to_vec())).collect()
    }

    /// Las mallas de las partes con este material.
    pub fn part_meshes(&self, name: &str) -> Vec<usize> {
        self.parts.iter().filter(|p| p.name == name).map(|p| p.mesh).collect()
    }

    /// Reemplaza vértices y normales de cada malla (con la misma cantidad) y los sube a la GPU.
    pub fn set_mesh_vertices(&mut self, data: &[(Vec<Vector3>, Vec<Vector3>)]) {
        for (mesh, (v, n)) in self.model.meshes_mut().iter_mut().zip(data) {
            mesh.vertices_mut().copy_from_slice(v);
            mesh.normals_mut().copy_from_slice(n);
            unsafe {
                ffi::UpdateMeshBuffer(*mesh.as_ref(), 0, v.as_ptr().cast(), (v.len() * 12) as i32, 0);
                ffi::UpdateMeshBuffer(*mesh.as_ref(), 2, n.as_ptr().cast(), (n.len() * 12) as i32, 0);
            }
        }
    }

    /// Punto donde un rayo (en el mundo) toca el modelo dibujado con `world`.
    pub fn ray_hit(&self, ray: Ray, world: Matrix) -> Option<Vector3> {
        let transform = self.fit * world;
        self.parts.iter()
            .map(|p| unsafe { ffi::GetRayCollisionMesh(ray.into(), *self.model.meshes()[p.mesh].as_ref(), transform.into()) })
            .filter(|c| c.hit)
            .min_by(|a, b| a.distance.total_cmp(&b.distance))
            .map(|c| c.point.into())
    }

    /// Dibuja con `world` (del espacio del planeta o nave al mundo). Sin descartar caras traseras: algunas mallas
    /// vienen con normales invertidas y el shader las corrige.
    pub fn draw(&self, d: &mut impl RaylibDraw3D, shader: &mut SpaceShader, world: Matrix) {
        self.draw_with(d, shader, world, &|_, e| e);
    }

    /// Como `draw`, pero `glow` puede cambiar la emisión de cada material (por su nombre) en este dibujo.
    pub fn draw_with(&self, d: &mut impl RaylibDraw3D, shader: &mut SpaceShader, world: Matrix,
                     glow: &dyn Fn(&str, Vector3) -> Vector3) {
        let transform = self.fit * world;
        unsafe { ffi::rlDisableBackfaceCulling() };
        let mut depth_writes = true;
        for part in &self.parts {
            if part.transparent && depth_writes {
                unsafe { ffi::rlDisableDepthMask() };
                depth_writes = false;
            }
            shader.shader.set_shader_value(shader.emissive, glow(&part.name, part.emissive));
            if shader.roughness >= 0 { shader.shader.set_shader_value(shader.roughness, part.roughness); }
            if shader.wet_surface >= 0 { shader.shader.set_shader_value(shader.wet_surface, part.wet_surface); }
            shader.shader.set_shader_value(shader.outline, if part.sticker { 2.0f32 } else if part.outline { 1.0 } else { 0.0 });
            if shader.no_bloom >= 0 {
                let off = shader.bloom_only.is_some_and(|keep| !keep.iter().any(|k| part.name.contains(k)));
                shader.shader.set_shader_value(shader.no_bloom, if off { 1.0f32 } else { 0.0 });
            }
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
        if shader.no_bloom >= 0 { shader.shader.set_shader_value(shader.no_bloom, 0.0f32); }
        unsafe {
            ffi::rlEnableDepthMask();
            ffi::rlEnableBackfaceCulling();
        }
    }
}

// Parámetros del fondo espacial, independientes de los materiales y efectos del Sol.
pub const SKY_BACKGROUND: Color = Color::new(3, 8, 26, 255);
const SKY_STAR_COUNT: usize = 9000;
const SKY_SEED: u32 = 0x5eed_1234;
const SKY_TINTS: [(u8, u8, u8); 7] = [
    (100, 164, 255), (145, 205, 255), (195, 230, 255), (235, 244, 255),
    (189, 135, 255), (225, 150, 230), (255, 178, 215),
];
// Probabilidad acumulada, radio en píxeles y opacidad.
const SKY_STAR_TIERS: [(f32, (f32, f32), (f32, f32)); 3] = [
    (0.76, (0.45, 0.85), (75.0, 135.0)),
    (0.96, (0.80, 1.25), (140.0, 215.0)),
    (1.00, (1.25, 1.85), (215.0, 255.0)),
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

/// Cielo azul oscuro con estrellas reproducibles y un brillo tenue entre ellas.
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
        let right = forward.cross(camera.up).normalized();
        let up = right.cross(forward).normalized();
        let half_vertical = (camera.fovy.to_radians() * 0.5).tan();
        let half_horizontal = half_vertical * rl.get_screen_width() as f32 / rl.get_screen_height().max(1) as f32;
        let project = |dir: Vector3| {
            let depth = dir.dot(forward);
            // Reject stars outside the viewport before raylib's projection call.
            // A small margin retains the edge glow and avoids popping on turns.
            (depth > 0.05
                && dir.dot(right).abs() <= depth * half_horizontal * 1.08
                && dir.dot(up).abs() <= depth * half_vertical * 1.08)
                .then(|| rl.get_world_to_screen(camera.position + dir * 1000.0, camera))
        };
        SkyFrame {
            stars: self.stars.iter().enumerate().filter_map(|(i, s)| project(s.dir).map(|at| (at, i))).collect(),
        }
    }

    pub fn draw(&self, d: &mut impl RaylibDraw, frame: &SkyFrame, time: f32) {
        for (at, i) in &frame.stars {
            let s = &self.stars[*i];
            let shimmer = 0.82 + 0.18 * (time * (0.7 + (*i % 7) as f32 * 0.17) + *i as f32 * 1.71).sin();
            let alpha = (s.color.a as f32 * shimmer).min(255.0) as u8;
            d.draw_circle_v(*at, s.size, Color::new(s.color.r, s.color.g, s.color.b, alpha));
            if *i % 67 == 0 {
                let glow = Color::new(s.color.r, s.color.g, s.color.b, (alpha as f32 * 0.30) as u8);
                d.draw_circle_gradient(at.x as i32, at.y as i32, 6.0, glow, Color::new(glow.r, glow.g, glow.b, 0));
            }
        }
    }
}

impl Default for Sky {
    fn default() -> Self {
        Self::new()
    }
}

/// Seamless low-frequency color field on the sky sphere. No projected patches
/// can enter or leave the screen as the camera rotates.
pub struct NebulaSky {
    shader: Shader,
    forward: i32,
    right: i32,
    up: i32,
    projection: i32,
    resolution: i32,
}

impl NebulaSky {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self, Box<dyn std::error::Error>> {
        let shader = rl.load_shader_from_memory(thread, None, Some(include_str!("shaders/sky_nebula.fs")));
        let forward = shader.get_shader_location("viewForward");
        let right = shader.get_shader_location("viewRight");
        let up = shader.get_shader_location("viewUp");
        let projection = shader.get_shader_location("projectionScale");
        let resolution = shader.get_shader_location("resolution");
        if !shader.is_shader_valid() || [forward, right, up, projection, resolution].contains(&-1) {
            return Err("No se pudo cargar el fondo continuo de la galaxia".into());
        }
        Ok(Self { shader, forward, right, up, projection, resolution })
    }

    pub fn draw(&mut self, d: &mut impl RaylibDraw, camera: Camera3D, width: i32, height: i32) {
        let forward = (camera.target - camera.position).normalized();
        let right = forward.cross(camera.up).normalized();
        let up = right.cross(forward).normalized();
        self.shader.set_shader_value(self.forward, forward);
        self.shader.set_shader_value(self.right, right);
        self.shader.set_shader_value(self.up, up);
        self.shader.set_shader_value(self.projection, Vector2::new(
            width as f32 / height.max(1) as f32,
            (camera.fovy.to_radians() * 0.5).tan(),
        ));
        self.shader.set_shader_value(self.resolution, Vector2::new(width as f32, height as f32));
        let mut layer = d.begin_shader_mode(&mut self.shader);
        layer.draw_rectangle(0, 0, width, height, Color::WHITE);
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
    for (inner, outer, alpha) in [
        (1.00, 1.025, 104), (1.025, 1.05, 94), (1.05, 1.075, 80),
        (1.075, 1.10, 64), (1.10, 1.135, 46), (1.135, 1.175, 33),
        (1.175, 1.23, 19), (1.23, 1.30, 9),
    ] {
        glow.draw_ring(at, radius * inner, radius * outer, 0.0, 360.0, 96,
            Color::new(255, 118, 35, (alpha as f32 * intensity) as u8));
    }
}
