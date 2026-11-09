//! Versioned local adventure snapshot, including unfinished reward sequences.
use super::selection::{GameState, ShipCatalog};
use crate::scenes::token_collection::TokenPresentation;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdventureSave {
    version: u32,
    pub game: GameState,
    pub presentation: TokenPresentation,
}

pub fn path(root: &Path) -> PathBuf { root.join("saves/adventure.json") }

pub fn load(path: &Path, catalog: &ShipCatalog) -> Result<Option<AdventureSave>, Box<dyn std::error::Error>> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let save: AdventureSave = serde_json::from_slice(&bytes)?;
    if save.version != 1 { return Err("Versión de partida desconocida".into()); }
    save.game.validate(catalog)?;
    Ok(Some(save))
}

pub fn store(path: &Path, game: &GameState, presentation: &TokenPresentation) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let bytes = serde_json::to_vec_pretty(&AdventureSave { version: 1, game: game.clone(), presentation: presentation.clone() })?;
    let temporary = path.with_extension("tmp");
    use std::io::Write;
    let mut file = std::fs::File::create(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(temporary, path)?;
    Ok(())
}
