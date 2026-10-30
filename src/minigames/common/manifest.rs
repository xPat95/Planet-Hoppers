//! Engine-independent manifest data; convert Z-up authoring coordinates once.
use super::collision::{Aabb, CollisionWorld, Obstacle};
use raylib::prelude::Vector3;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct Spawn {
    pub position: [f32; 3],
    pub forward: [f32; 3],
    pub eye_height: f32,
}
#[derive(Deserialize)]
pub struct Collider {
    pub name: String,
    pub shape: String,
    pub role: String,
    pub min: Vec<f32>,
    pub max: Vec<f32>,
    pub z: Option<f32>,
}
#[derive(Deserialize)]
pub struct Coordinates {
    pub units: String,
    pub up: String,
    pub gltf_up: String,
}
#[derive(Deserialize)]
pub struct SceneManifest {
    pub schema_version: u32,
    pub id: String,
    pub visual_asset: String,
    pub coordinates: Coordinates,
    pub spawn: Spawn,
    pub colliders: Vec<Collider>,
}

pub fn from_blender(p: [f32; 3]) -> Vector3 {
    Vector3::new(p[0], p[2], -p[1])
}
pub fn from_blender_bounds(min: [f32; 3], max: [f32; 3]) -> Aabb {
    // Reorder extrema after negating Y, equivalent to converting all eight corners.
    Aabb::new(
        Vector3::new(min[0], min[2], -max[1]),
        Vector3::new(max[0], max[2], -min[1]),
    )
}

impl SceneManifest {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if data.schema_version != 1
            || data.coordinates.units != "meters"
            || data.coordinates.up != "Z"
            || data.coordinates.gltf_up != "Y"
        {
            return Err("Unsupported scene coordinate system/schema".into());
        }
        if !data
            .spawn
            .position
            .iter()
            .chain(data.spawn.forward.iter())
            .all(|v| v.is_finite())
            || !data.spawn.eye_height.is_finite()
            || data.spawn.eye_height <= 0.0
            || from_blender(data.spawn.forward).length() < 0.5
        {
            return Err("Invalid scene spawn".into());
        }
        Ok(data)
    }
    pub fn visual_path(&self, manifest_path: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let relative = Path::new(&self.visual_asset);
        if relative.is_absolute()
            || relative
                .components()
                .any(|p| !matches!(p, std::path::Component::Normal(_)))
        {
            return Err("visual_asset must be a local relative path".into());
        }
        Ok(manifest_path
            .parent()
            .ok_or("Manifest has no parent directory")?
            .join(relative))
    }
    pub fn collision_world(&self) -> Result<CollisionWorld, Box<dyn std::error::Error>> {
        let mut solids = Vec::new();
        let mut ground = None;
        for c in &self.colliders {
            if !c.min.iter().chain(&c.max).all(|n| n.is_finite())
                || c.min.len() != c.max.len()
                || c.min.iter().zip(&c.max).any(|(a, b)| a > b)
            {
                return Err(format!("Invalid collider {}", c.name).into());
            }
            match (c.role.as_str(), c.shape.as_str()) {
                ("solid", "aabb") if c.min.len() == 3 => solids.push(Obstacle {
                    id: c.name.clone(),
                    bounds: from_blender_bounds(c.min[..].try_into()?, c.max[..].try_into()?),
                }),
                ("walkable", "plane") if c.min.len() == 2 => {
                    let z =
                        c.z.filter(|z| z.is_finite())
                            .ok_or("Missing plane elevation")?;
                    if ground.is_some() {
                        return Err("Only a single flat walkable plane is supported".into());
                    }
                    ground = Some((
                        from_blender_bounds([c.min[0], c.min[1], z], [c.max[0], c.max[1], z]),
                        z,
                    ));
                }
                _ => return Err(format!("Unsupported collider {}", c.name).into()),
            }
        }
        let (bounds, elevation) = ground.ok_or("Scene has no walkable plane")?;
        Ok(CollisionWorld::new(bounds, elevation, solids))
    }
}
