//! Read the GLB scene directly into CPU memory. Node transforms are baked once at load time.
use super::software::{Opacity, SoftMesh, SoftTriangle, SoftVertex};
use super::space::Fit;
use gltf::{buffer, image, mesh::Mode, Node};
use raylib::prelude::{Matrix, Vector2, Vector3};
use std::{error::Error, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub struct Asset {
    pub mesh: SoftMesh,
    pub half_extents: Vector3,
    pub fit: Matrix,
}

impl Asset {
    pub fn load(path: &Path, rotation: [f32; 3], fit: Fit) -> Result<Self> {
        let path = if path.extension().is_some_and(|s| s.eq_ignore_ascii_case("obj")) {
            path.with_file_name(format!("{}_near.glb", path.file_stem().unwrap().to_string_lossy()))
        } else {
            path.to_owned()
        };
        let bytes = std::fs::read(&path)?;
        let glb = gltf::binary::Glb::from_slice(&bytes)?;
        let mut json: serde_json::Value = serde_json::from_slice(&glb.json)?;
        // Blender can omit the optional nodes list of an empty auxiliary scene.
        if let Some(scenes) = json["scenes"].as_array_mut() {
            for scene in scenes {
                if scene.get("nodes").is_none() {
                    scene["nodes"] = serde_json::json!([]);
                }
            }
        }
        let doc = gltf::Document::from_json(serde_json::from_value(json)?)?;
        let buffers = gltf::import_buffers(&doc, path.parent(), glb.bin.map(|bin| bin.into_owned()))?;
        let images = gltf::import_images(&doc, path.parent(), &buffers)?;
        let scene = doc
            .default_scene()
            .or_else(|| doc.scenes().next())
            .ok_or("GLB sin escena")?;
        let mut mesh = SoftMesh {
            images: Arc::new(images),
            ..Default::default()
        };
        for image in mesh.images.iter() {
            if !matches!(
                image.format,
                image::Format::R8 | image::Format::R8G8 | image::Format::R8G8B8 | image::Format::R8G8B8A8
            ) {
                return Err(format!(
                    "{}: formato de textura no compatible {:?}",
                    path.display(),
                    image.format
                )
                .into());
            }
        }
        let mut group = 0;
        for node in scene.nodes() {
            read_node(node, Matrix::identity(), &buffers, &mut mesh, &mut group)?;
        }
        if mesh.tris.is_empty() {
            return Err(format!("{}: sin triángulos", path.display()).into());
        }
        let (fit, half_extents) = match fit {
            Fit::Same(m, h) => (m, h),
            _ => {
                let mut lo = Vector3::one() * f32::MAX;
                let mut hi = -lo;
                for v in mesh.tris.iter().flat_map(|t| &t.v) {
                    lo = lo.min(v.pos);
                    hi = hi.max(v.pos);
                }
                let center = (lo + hi) * 0.5;
                let rot = Matrix::rotate_x(rotation[0].to_radians())
                    * Matrix::rotate_y(rotation[1].to_radians())
                    * Matrix::rotate_z(rotation[2].to_radians());
                let h = (hi - lo) * 0.5;
                let mut half = Vector3::zero();
                for x in [-h.x, h.x] {
                    for y in [-h.y, h.y] {
                        for z in [-h.z, h.z] {
                            half = half.max(Vector3::new(x, y, z).transform_with(rot));
                        }
                    }
                }
                let target = match fit {
                    Fit::Radius(r) => r,
                    Fit::Length(l) => l * 0.5,
                    _ => unreachable!(),
                };
                let scale = target / half.x.max(half.y).max(half.z).max(1e-6);
                (
                    Matrix::translate(-center.x, -center.y, -center.z) * rot * Matrix::scale(scale, scale, scale),
                    half * scale,
                )
            }
        };
        let mut normals = fit;
        normals.m12 = 0.;
        normals.m13 = 0.;
        normals.m14 = 0.;
        for v in mesh.tris.iter_mut().flat_map(|t| &mut t.v) {
            v.pos = v.pos.transform_with(fit);
            v.normal = v.normal.transform_with(normals).normalized();
        }
        Ok(Self {
            mesh,
            half_extents,
            fit,
        })
    }
}

// A fully opaque factor and a binary alpha texture can use the depth buffer.
// This keeps the cutouts of the Minecraft leaves without sorting their triangles.
fn cutout_material(alpha: f32, image: Option<&image::Data>) -> bool {
    if alpha != 1.0 {
        return false;
    }
    let Some(image) = image else {
        return true;
    };
    let stride = match image.format {
        image::Format::R8G8 => 2,
        image::Format::R8G8B8A8 => 4,
        _ => return true,
    };
    image
        .pixels
        .chunks_exact(stride)
        .all(|pixel| matches!(pixel[stride - 1], 0 | 255))
}

fn read_node(
    node: Node,
    parent: Matrix,
    buffers: &[buffer::Data],
    mesh: &mut SoftMesh,
    group: &mut u32,
) -> Result<()> {
    let [[m0, m1, m2, m3], [m4, m5, m6, m7], [m8, m9, m10, m11], [m12, m13, m14, m15]] = node.transform().matrix();
    let world = Matrix {
        m0,
        m1,
        m2,
        m3,
        m4,
        m5,
        m6,
        m7,
        m8,
        m9,
        m10,
        m11,
        m12,
        m13,
        m14,
        m15,
    } * parent;
    let mut normal = world.inverted().transposed();
    normal.m12 = 0.;
    normal.m13 = 0.;
    normal.m14 = 0.;
    if let Some(object) = node.mesh() {
        for prim in object.primitives() {
            if prim.mode() != Mode::Triangles {
                continue;
            }
            let mat = prim.material();
            let name = mat.name().unwrap_or("");
            if name.ends_with("Outline") {
                continue;
            }
            let pbr = mat.pbr_metallic_roughness();
            let [r, g, b, alpha] = pbr.base_color_factor();
            let tex = pbr.base_color_texture();
            let opacity = match mat.alpha_mode() {
                gltf::material::AlphaMode::Opaque => Opacity::Opaque,
                gltf::material::AlphaMode::Mask => Opacity::Mask(mat.alpha_cutoff().unwrap_or(0.5)),
                gltf::material::AlphaMode::Blend
                    if cutout_material(alpha, tex.as_ref().map(|t| &mesh.images[t.texture().source().index()])) =>
                {
                    Opacity::Mask(0.5)
                }
                gltf::material::AlphaMode::Blend => Opacity::Blend,
            };
            let reader = prim.reader(|buf| Some(&buffers[buf.index()].0));
            let positions = reader.read_positions().ok_or("Malla sin posiciones")?;
            let mut normals = reader.read_normals();
            let mut colors = reader.read_colors(0).map(|c| c.into_rgba_f32());
            let mut uvs = tex
                .as_ref()
                .and_then(|t| reader.read_tex_coords(t.tex_coord()))
                .map(|v| v.into_f32());
            let vertices: Vec<_> = positions
                .map(|p| {
                    let [x, y, z] = p;
                    let [nx, ny, nz] = normals.as_mut().and_then(|v| v.next()).unwrap_or([0., 1., 0.]);
                    let c = colors.as_mut().and_then(|v| v.next()).unwrap_or([1.; 4]);
                    let uv = uvs.as_mut().and_then(|v| v.next()).unwrap_or([0.; 2]);
                    SoftVertex {
                        pos: Vector3::new(x, y, z).transform_with(world),
                        normal: Vector3::new(nx, ny, nz).transform_with(normal).normalized(),
                        color: Vector3::new(r * c[0], g * c[1], b * c[2]),
                        uv: Vector2::new(uv[0], uv[1]),
                    }
                })
                .collect();
            let indices: Vec<_> = reader
                .read_indices()
                .map(|i| i.into_u32().collect())
                .unwrap_or_else(|| (0..vertices.len() as u32).collect());
            *group += 1;
            let e = mat.emissive_factor();
            for tri in indices.chunks_exact(3) {
                let get = |i: u32| vertices.get(i as usize).copied().ok_or("Índice GLB fuera de rango");
                mesh.tris.push(SoftTriangle {
                    v: [get(tri[0])?, get(tri[1])?, get(tri[2])?],
                    texture: tex.as_ref().map(|t| t.texture().source().index()),
                    alpha,
                    opacity,
                    emissive: Vector3::new(e[0], e[1], e[2]),
                    flat: name.ends_with("_Sticker"),
                    group: *group,
                });
            }
        }
    }
    for child in node.children() {
        read_node(child, world, buffers, mesh, group)?;
    }
    Ok(())
}
