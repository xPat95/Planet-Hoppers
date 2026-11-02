//! Load the original GLB once; batch its static triangles by material in RAM.
//! Unsafe code is confined to raylib-owned buffers (allocated with its allocator).
use super::lighting::winter_shader;
use raylib::{ffi, prelude::*};
use serde::Serialize;
use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Serialize)]
pub struct ModelAudit {
    pub original_primitives: usize,
    pub source_materials: usize,
    pub triangles: usize,
    pub draw_batches: usize,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    pub material_colors_match: bool,
}

struct Batch {
    mesh: Mesh,
    material: usize,
    background: bool,
    group: Option<String>,
    visible: bool,
}
#[derive(Default)]
struct CpuBatch {
    positions: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
}

/// Capture the current back buffer before swapping. raylib 5.5 TakeScreenshot
/// strips directory components and multiplies by desktop DPI even without a
/// HiDPI framebuffer, so use the actual drawing dimensions and explicit export.
pub fn save_frame(path: &Path, width: i32, height: i32) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        ffi::rlDrawRenderBatchActive();
    }
    let raw = ffi::Image {
        data: unsafe { ffi::rlReadScreenPixels(width, height) }.cast(),
        width,
        height,
        mipmaps: 1,
        format: ffi::PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32,
    };
    if raw.data.is_null() {
        return Err("Framebuffer capture failed".into());
    }
    // rlReadScreenPixels allocates with raylib; Image owns and releases it.
    let image = unsafe { Image::from_raw(raw) };
    image.export_image(path.to_str().ok_or("Invalid screenshot path")?);
    if !path.is_file() {
        return Err("Screenshot export failed".into());
    }
    Ok(())
}

pub struct StaticModel {
    // Fields drop in this order, while the window/OpenGL context still exists.
    batches: Vec<Batch>,
    model: Model,
    shader: Shader,
    pub audit: ModelAudit,
    surfaces: Option<Vec<Surface>>,
    underground: bool,
}

struct Surface {
    alpha_cutoff: f32,
    textured: i32,
    double_sided: bool,
    emission: Vector3,
}

pub(crate) fn glb_document(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    if bytes.len() < 20
        || &bytes[..4] != b"glTF"
        || u32::from_le_bytes(bytes[4..8].try_into()?) != 2
        || u32::from_le_bytes(bytes[8..12].try_into()?) as usize != bytes.len()
        || &bytes[16..20] != b"JSON"
    {
        return Err("Invalid GLB 2 container".into());
    }
    let length = u32::from_le_bytes(bytes[12..16].try_into()?) as usize;
    Ok(serde_json::from_slice(
        bytes.get(20..20 + length).ok_or("Truncated GLB JSON")?,
    )?)
}

impl CpuBatch {
    fn upload(self) -> Result<Mesh, Box<dyn std::error::Error>> {
        // Zero initializes absent animation/index buffers. Ownership transfers to
        // Mesh; its Drop calls UnloadMesh using the same C allocator as below.
        let raw: ffi::Mesh = unsafe { std::mem::zeroed() };
        let mut mesh = unsafe { Mesh::from_raw(raw) };
        mesh.as_mut().vertexCount = (self.positions.len() / 3) as i32;
        mesh.as_mut().triangleCount = (self.positions.len() / 9) as i32;
        fn copy(values: &[f32]) -> Result<*mut f32, Box<dyn std::error::Error>> {
            let size = u32::try_from(std::mem::size_of_val(values))?;
            // SAFETY: size is checked; fresh raylib allocation is uniquely owned.
            let ptr = unsafe { ffi::MemAlloc(size) } as *mut f32;
            if ptr.is_null() {
                return Err("Mesh buffer allocation failed".into());
            }
            unsafe {
                std::ptr::copy_nonoverlapping(values.as_ptr(), ptr, values.len());
            }
            Ok(ptr)
        }
        mesh.as_mut().vertices = copy(&self.positions)?;
        mesh.as_mut().normals = copy(&self.normals)?;
        mesh.as_mut().texcoords = copy(&self.uvs)?;
        unsafe {
            ffi::UploadMesh(mesh.as_mut(), false);
        }
        if mesh.as_ref().vaoId == 0 {
            return Err("GPU mesh upload failed".into());
        }
        Ok(mesh)
    }
}

impl StaticModel {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &Path,
        flat_ground_material: Option<&str>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_style(rl, thread, path, flat_ground_material, false, false)
    }
    /// Same GLB loader and geometry batching; opt-in pixel textures and daylight.
    /// Existing South Park callers retain their original shader and render state.
    pub fn load_overworld(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_style(rl, thread, path, None, true, false)
    }
    /// Pixel materials, underground lighting and separate future interactive props.
    pub fn load_mine(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &Path,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_style(rl, thread, path, None, true, true)
    }
    fn load_style(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: &Path,
        flat_ground_material: Option<&str>,
        overworld: bool,
        underground: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let gltf = glb_document(path)?;
        let source_materials = gltf["materials"]
            .as_array()
            .ok_or("Missing GLB materials")?;
        let primitives = gltf["meshes"]
            .as_array()
            .ok_or("Missing GLB meshes")?
            .iter()
            .map(|m| m["primitives"].as_array().map_or(0, Vec::len))
            .sum::<usize>();
        let mut model = rl.load_model(thread, path.to_str().ok_or("Invalid model path")?)?;
        if !model.is_model_valid()
            || model.meshes().len() != primitives
            || model.materials().len() != source_materials.len() + 1
        {
            return Err("raylib did not load every GLB primitive/material".into());
        }
        // raylib 5.5 prepends a default material. Verify every loaded albedo color
        // against the actual source, including quantization to 8-bit channels.
        for (i, source) in source_materials.iter().enumerate() {
            let factor = source["pbrMetallicRoughness"]["baseColorFactor"]
                .as_array()
                .ok_or("Missing flat material color")?;
            let color = model.materials()[i + 1].maps()[0].color;
            for (channel, value) in [color.r, color.g, color.b, color.a].into_iter().zip(factor) {
                let expected = (value.as_f64().ok_or("Invalid material channel")? * 255.0) as u8;
                if channel.abs_diff(expected) > 1 {
                    return Err(format!("GLB material color mismatch: {}", source["name"]).into());
                }
            }
        }
        let bounds = model.get_model_bounding_box();
        let mut surfaces = overworld.then(|| {
            vec![Surface {
                alpha_cutoff: -1.0,
                textured: 0,
                double_sided: false,
                emission: Vector3::zero(),
            }]
        });
        if let Some(surfaces) = &mut surfaces {
            for (i, source) in source_materials.iter().enumerate() {
                let textured = source["pbrMetallicRoughness"]
                    .get("baseColorTexture")
                    .is_some();
                let texture = model.materials()[i + 1].maps()[0].texture;
                if textured && (texture.id == 0 || texture.width != 16 || texture.height != 16) {
                    return Err(
                        format!("Missing Minecraft pixel texture: {}", source["name"]).into(),
                    );
                }
                unsafe {
                    // Texture is owned by Model; do not manufacture another owner.
                    ffi::SetTextureFilter(texture, ffi::TextureFilter::TEXTURE_FILTER_POINT as i32);
                    ffi::SetTextureWrap(texture, ffi::TextureWrap::TEXTURE_WRAP_REPEAT as i32);
                }
                surfaces.push(Surface {
                    alpha_cutoff: if source["alphaMode"] == "MASK" {
                        source["alphaCutoff"].as_f64().unwrap_or(0.5) as f32
                    } else {
                        -1.0
                    },
                    textured: i32::from(textured),
                    double_sided: source["doubleSided"].as_bool().unwrap_or(false),
                    emission: Vector3::new(
                        source["emissiveFactor"][0].as_f64().unwrap_or(0.0) as f32,
                        source["emissiveFactor"][1].as_f64().unwrap_or(0.0) as f32,
                        source["emissiveFactor"][2].as_f64().unwrap_or(0.0) as f32,
                    ),
                });
            }
        }
        let mut shader = if overworld {
            rl.load_shader_from_memory(
                thread,
                Some(include_str!("shaders/winter.vs")),
                Some(if underground {
                    include_str!("shaders/mine.fs")
                } else {
                    include_str!("shaders/overworld.fs")
                }),
            )
        } else {
            winter_shader(rl, thread)
        };
        if !shader.is_shader_valid() || shader.get_shader_location("cameraPosition") < 0 {
            return Err("Scene lighting shader failed to compile".into());
        }
        if overworld
            && ["alphaCutoff", "textureIsSrgb"]
                .iter()
                .any(|name| shader.get_shader_location(name) < 0)
        {
            return Err("Overworld texture/alpha shader uniforms missing".into());
        }
        for material in model.materials_mut() {
            // WeakShader is a non-owning handle; StaticModel owns the Shader.
            *material.shader_mut() = unsafe { WeakShader::from_raw(*shader.as_ref()) };
        }
        let primitive_groups: Vec<Option<String>> = gltf["meshes"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|mesh| {
                let name = mesh["name"].as_str().unwrap_or("");
                let group = (underground
                    && (name.starts_with("ORE_") || name.starts_with("BREAK_")))
                .then(|| name.to_owned());
                std::iter::repeat_n(group, mesh["primitives"].as_array().map_or(0, Vec::len))
            })
            .collect();
        let mut cpu: BTreeMap<(usize, Option<String>), Vec<CpuBatch>> = BTreeMap::new();
        let mut triangles = 0;
        for (i, mesh) in model.meshes().iter().enumerate() {
            // SAFETY: valid loaded Model has one meshMaterial entry per mesh.
            let material = unsafe { *model.as_ref().meshMaterial.add(i) } as usize;
            if material >= model.materials().len() {
                return Err("Invalid mesh material index".into());
            }
            let raw = mesh.as_ref();
            if raw.vertices.is_null() || raw.normals.is_null() || raw.triangleCount <= 0 {
                return Err("GLB mesh is missing triangles/normals".into());
            }
            let count = raw.triangleCount as usize * 3;
            triangles += raw.triangleCount as usize;
            // Do not use raylib-rs 5.5's indicies(): its slice length is vertexCount,
            // whereas a glTF index stream has triangleCount * 3 entries.
            let indices = if raw.indices.is_null() {
                None
            } else {
                Some(unsafe { std::slice::from_raw_parts(raw.indices, count) })
            };
            let uv = if raw.texcoords.is_null() {
                None
            } else {
                Some(unsafe {
                    std::slice::from_raw_parts(raw.texcoords, raw.vertexCount as usize * 2)
                })
            };
            for corner in 0..count {
                let index = indices.map_or(corner, |s| s[corner] as usize);
                if index >= raw.vertexCount as usize {
                    return Err("Invalid GLB triangle index".into());
                }
                let groups = cpu
                    .entry((material, primitive_groups[i].clone()))
                    .or_default();
                if groups
                    .last()
                    .is_none_or(|b| b.positions.len() >= 60_000 * 3)
                {
                    groups.push(CpuBatch::default());
                }
                let batch = groups.last_mut().unwrap();
                let p = mesh.vertices()[index];
                let n = mesh.normals()[index];
                batch.positions.extend([p.x, p.y, p.z]);
                batch.normals.extend([n.x, n.y, n.z]);
                batch
                    .uvs
                    .extend(uv.map_or([0.0, 0.0], |s| [s[index * 2], s[index * 2 + 1]]));
            }
        }
        let mut batches = Vec::new();
        for ((material, group), groups) in cpu {
            for cpu_batch in groups {
                batches.push(Batch {
                    mesh: cpu_batch.upload()?,
                    material,
                    background: material > 0
                        && flat_ground_material.is_some_and(|name| {
                            source_materials[material - 1]["name"].as_str() == Some(name)
                        }),
                    group: group.clone(),
                    visible: true,
                });
            }
        }
        batches.sort_by_key(|b| (!b.background, b.material));
        let audit = ModelAudit {
            original_primitives: primitives,
            source_materials: source_materials.len(),
            triangles,
            draw_batches: batches.len(),
            bounds_min: [bounds.min.x, bounds.min.y, bounds.min.z],
            bounds_max: [bounds.max.x, bounds.max.y, bounds.max.z],
            material_colors_match: true,
        };
        // Force the uniform lookup now, before the interactive loop.
        let location = shader.get_shader_location("cameraPosition");
        shader.set_shader_value(location, Vector3::zero());
        Ok(Self {
            batches,
            model,
            shader,
            audit,
            surfaces,
            underground,
        })
    }
    pub fn draw(&mut self, draw: &mut RaylibMode3D<RaylibDrawHandle>, camera_position: Vector3) {
        self.draw_at(draw, camera_position, Vector3::zero(), 0.0);
    }
    pub fn pixel_material_audit(&self) -> Option<serde_json::Value> {
        self.surfaces.as_ref().map(|surfaces| {
            serde_json::json!({
                "point_filtered_textures": surfaces.iter().filter(|s| s.textured == 1).count(),
                "alpha_mask_materials": surfaces.iter().filter(|s| s.alpha_cutoff >= 0.0).count(),
                "double_sided_materials": surfaces.iter().filter(|s| s.double_sided).count(),
                "color_space": if self.underground { "sRGB textures decoded before point-light shading" } else { "sRGB textures decoded before daylight lighting" },
            })
        })
    }
    pub fn draw_at(
        &mut self,
        draw: &mut RaylibMode3D<RaylibDrawHandle>,
        camera_position: Vector3,
        position: Vector3,
        yaw: f32,
    ) {
        self.draw_pose(draw, camera_position, position, yaw, 0.0, Vector3::one());
    }
    pub fn independent_groups(&self) -> Vec<&str> {
        let mut groups: Vec<_> = self
            .batches
            .iter()
            .filter_map(|b| b.group.as_deref())
            .collect();
        groups.sort_unstable();
        groups.dedup();
        groups
    }
    /// Future prop visibility hook. No mining or collision change is triggered here.
    pub fn set_group_visible(&mut self, id: &str, visible: bool) -> bool {
        let mut found = false;
        for batch in &mut self.batches {
            if batch.group.as_deref() == Some(id) {
                batch.visible = visible;
                found = true;
            }
        }
        found
    }
    pub fn group_visible(&self, id: &str) -> Option<bool> {
        self.batches
            .iter()
            .find(|b| b.group.as_deref() == Some(id))
            .map(|b| b.visible)
    }
    pub fn set_mine_lights(
        &mut self,
        positions: &[Vector3],
        colors: &[Vector3],
        radii: &[f32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.underground
            || positions.is_empty()
            || positions.len() > 48
            || positions.len() != colors.len()
            || positions.len() != radii.len()
            || radii.iter().any(|r| !r.is_finite() || *r <= 0.0)
        {
            return Err("Invalid mine lighting rig".into());
        }
        for name in [
            "lightCount",
            "lightPosition[0]",
            "lightColor[0]",
            "lightRadius[0]",
            "emission",
        ] {
            if self.shader.get_shader_location(name) < 0 {
                return Err(format!("Missing mine uniform: {name}").into());
            }
        }
        let loc = self.shader.get_shader_location("lightCount");
        self.shader.set_shader_value(loc, positions.len() as i32);
        let loc = self.shader.get_shader_location("lightPosition[0]");
        self.shader.set_shader_value_v(loc, positions);
        let loc = self.shader.get_shader_location("lightColor[0]");
        self.shader.set_shader_value_v(loc, colors);
        let loc = self.shader.get_shader_location("lightRadius[0]");
        self.shader.set_shader_value_v(loc, radii);
        Ok(())
    }
    /// Controlled prop animation; model buffers/materials stay resident.
    pub fn draw_pose(
        &mut self,
        draw: &mut RaylibMode3D<RaylibDrawHandle>,
        camera_position: Vector3,
        position: Vector3,
        yaw: f32,
        roll: f32,
        scale: Vector3,
    ) {
        let mut transform = Matrix::scale(scale.x, scale.y, scale.z)
            * Matrix::rotate_z(roll)
            * Matrix::rotate_y(yaw);
        transform.m12 = position.x;
        transform.m13 = position.y;
        transform.m14 = position.z;
        let location = self.shader.get_shader_location("cameraPosition");
        self.shader.set_shader_value(location, camera_position);
        for batch in &self.batches {
            if !batch.visible {
                continue;
            }
            if let Some(surfaces) = &self.surfaces {
                let surface = &surfaces[batch.material];
                let location = self.shader.get_shader_location("alphaCutoff");
                self.shader.set_shader_value(location, surface.alpha_cutoff);
                let location = self.shader.get_shader_location("textureIsSrgb");
                self.shader.set_shader_value(location, surface.textured);
                if self.underground {
                    let location = self.shader.get_shader_location("emission");
                    self.shader.set_shader_value(location, surface.emission);
                }
                if surface.double_sided {
                    unsafe {
                        ffi::rlDisableBackfaceCulling();
                    }
                }
            }
            // A known flat ground is a visual background. Drawing it first
            // without depth writes prevents distant roads 12 mm above it from
            // fighting the terrain's depth, without changing any asset vertices.
            if batch.background {
                unsafe {
                    ffi::rlDisableDepthMask();
                }
            }
            draw.draw_mesh(
                &batch.mesh,
                self.model.materials()[batch.material].clone(),
                transform,
            );
            if self
                .surfaces
                .as_ref()
                .is_some_and(|s| s[batch.material].double_sided)
            {
                unsafe {
                    ffi::rlEnableBackfaceCulling();
                }
            }
            if batch.background {
                unsafe {
                    ffi::rlEnableDepthMask();
                }
            }
        }
    }
}
