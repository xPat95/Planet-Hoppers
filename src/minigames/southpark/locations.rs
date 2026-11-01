//! Predefined road sites, admitted only after full layout/access validation.
use super::{
    kenny::KENNY_ID,
    navigation::{event_route, RoadNetwork},
    threats::{ThreatDirector, ThreatKind},
};
use crate::minigames::common::{
    collision::CollisionWorld,
    manifest::{from_blender, SceneManifest},
};
use raylib::prelude::Vector3;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct EventSite {
    pub id: String,
    pub road: String,
    pub offset: [f32; 3],
    pub centre: [f32; 3],
    pub entry: [f32; 3],
    pub compatible: Vec<ThreatKind>,
}
impl EventSite {
    pub fn translation(&self) -> Vector3 {
        Vector3::new(self.offset[0], self.offset[1], self.offset[2])
    }
    pub fn centre_position(&self) -> Vector3 {
        Vector3::new(self.centre[0], self.centre[1], self.centre[2])
    }
}
#[derive(Clone, Serialize)]
pub struct WorldLocations {
    pub sites: Vec<EventSite>,
    pub exploration_spawns: Vec<[f32; 3]>,
    pub rejected: Vec<String>,
}
pub fn random_step(seed: &mut u64) -> u64 {
    if *seed == 0 {
        *seed = 0x9e3779b97f4a7c15;
    }
    *seed ^= *seed >> 12;
    *seed ^= *seed << 25;
    *seed ^= *seed >> 27;
    seed.wrapping_mul(0x2545f4914f6cdd1d)
}
impl WorldLocations {
    pub fn build(
        manifest: &SceneManifest,
        world: &CollisionWorld,
        roads: &RoadNetwork,
    ) -> Result<Self, String> {
        let mut result = Self {
            sites: vec![],
            exploration_spawns: vec![],
            rejected: vec![],
        };
        let mut clean = world.clone();
        clean
            .obstacles
            .retain(|o| o.id != KENNY_ID && !o.id.starts_with("threat:") && o.id != "fps-player");
        let root =
            from_blender(manifest.spawn.position) - Vector3::up() * manifest.spawn.eye_height;
        // Named roads and fixed longitudinal stations, never random coordinates.
        for (road_name, x) in [
            ("North_Avenue", -145.0),
            ("Central_Avenue", -170.0),
            ("Residential_Avenue", -150.0),
            ("South_Neighborhood", 80.0),
            ("Infill_Main", 35.0),
            ("North_Avenue", 85.0),
            ("Central_Avenue", 95.0),
            ("Residential_Avenue", 100.0),
            ("South_Neighborhood", -60.0),
        ] {
            let road = manifest
                .roads
                .iter()
                .find(|r| r.name == road_name)
                .ok_or("Missing named event road")?;
            let p = road
                .points
                .windows(2)
                .map(|p| (from_blender(p[0]), from_blender(p[1])))
                .find(|(a, b)| {
                    (a.z - b.z).abs() < 0.01 && x >= a.x.min(b.x) + 25.0 && x <= a.x.max(b.x) - 55.0
                })
                .map(|(a, _)| Vector3::new(x, clean.ground_y, a.z));
            let Some(p) = p else {
                continue;
            };
            let offset = p - Vector3::new(-28.0, 0.0, -42.9);
            let entry = Vector3::new(-43.2, 0.0, -42.9) + offset;
            if event_route(roads, &clean, root, entry, 0.35, 1.8).is_err() {
                continue;
            }
            let mut compatible = vec![];
            for kind in ThreatKind::ALL {
                if result
                    .sites
                    .iter()
                    .filter(|s| s.compatible.contains(&kind))
                    .count()
                    >= 5
                {
                    continue;
                }
                let mut director = ThreatDirector::default();
                director.offset = offset;
                match director.validate_access(kind, &clean, roads, entry) {
                    Ok(_) => compatible.push(kind),
                    Err(e) => result
                        .rejected
                        .push(format!("{road_name}:{x}:{kind:?}: {e}")),
                }
            }
            if !compatible.is_empty() {
                result.sites.push(EventSite {
                    id: format!("{road_name}:{x}"),
                    road: road_name.replace('_', " "),
                    offset: [offset.x, offset.y, offset.z],
                    centre: [p.x, p.y, p.z],
                    entry: [entry.x, entry.y, entry.z],
                    compatible,
                });
            }
            if clean.can_occupy(p, 0.28, 1.35)
                && event_route(roads, &clean, root, p, 0.28, 1.35).is_ok()
            {
                result.exploration_spawns.push([p.x, p.y, p.z]);
            }
        }
        for kind in ThreatKind::ALL {
            if result
                .sites
                .iter()
                .filter(|s| s.compatible.contains(&kind))
                .count()
                < 2
            {
                return Err(format!(
                    "Fewer than two verified sites for {kind:?}: {:?}",
                    result.rejected
                ));
            }
        }
        Ok(result)
    }
    pub fn choose(
        &self,
        kind: ThreatKind,
        last: Option<&str>,
        seed: &mut u64,
    ) -> Option<EventSite> {
        let choices: Vec<_> = self
            .sites
            .iter()
            .filter(|s| s.compatible.contains(&kind) && Some(s.id.as_str()) != last)
            .collect();
        let choices = if choices.is_empty() {
            self.sites
                .iter()
                .filter(|s| s.compatible.contains(&kind))
                .collect()
        } else {
            choices
        };
        choices
            .get((random_step(seed) as usize) % choices.len().max(1))
            .map(|s| (*s).clone())
    }
}
