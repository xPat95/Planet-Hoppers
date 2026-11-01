use planet_hoppers::minigames::{
    common::manifest::SceneManifest,
    southpark::{locations::WorldLocations, threats::ThreatKind, DevConfig, SouthParkScene},
};
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = SceneManifest::load(
        &root.join("assets/models/minigames/southpark/southpark_town.scene.json"),
    )?;
    let mut config = DevConfig::load(&root.join("config/southpark-dev.json"))?;
    config.test_objects = false;
    let scene = SouthParkScene::load(&manifest, config)?;
    let sites = WorldLocations::build(&manifest, &scene.world, &scene.roads)?;
    std::fs::write(
        root.join("target/southpark-locations.json"),
        serde_json::to_string_pretty(&sites)?,
    )?;
    for kind in ThreatKind::ALL {
        println!(
            "{kind:?}: {}",
            sites
                .sites
                .iter()
                .filter(|s| s.compatible.contains(&kind))
                .count()
        );
    }
    println!("Exploration spawns: {}", sites.exploration_spawns.len());
    Ok(())
}
