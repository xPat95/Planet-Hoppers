//! Adventure state, ship ownership and shared selection rules.
use super::progress::ProgressState;
use crate::ships::{ShipModel, ShipsConfig};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::Path};

pub const INITIAL_SHIP_ORDER: [&str; 6] = [
    "starwars",
    "smilingfriends",
    "minecraft",
    "regularshow",
    "southpark",
    "halloween",
];

#[derive(Clone, Debug, Deserialize)]
pub struct ShipChoice {
    pub id: String,
    pub name: String,
    pub ship_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShipCatalog {
    pub choices: Vec<ShipChoice>,
}
impl ShipCatalog {
    pub fn load(path: &Path, ships: &ShipsConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let catalog: Self = serde_json::from_slice(&std::fs::read(path)?)?;
        if catalog
            .choices
            .iter()
            .map(|c| c.id.as_str())
            .collect::<Vec<_>>()
            != INITIAL_SHIP_ORDER
        {
            return Err("El selector debe contener las seis franquicias en el orden del hangar".into());
        }
        for choice in &catalog.choices {
            if choice.name.is_empty()
                || choice
                    .ship_id
                    .as_ref()
                    .is_some_and(|id| !ships.ships.iter().any(|s| &s.id == id))
            {
                return Err(format!("Opción de nave inválida: {}", choice.id).into());
            }
        }
        Ok(catalog)
    }
    pub fn choice(&self, id: &str) -> Option<&ShipChoice> {
        self.choices.iter().find(|c| c.id == id)
    }
    pub fn model(&self, choice: &ShipChoice, ships: &ShipsConfig) -> ShipModel {
        if let Some(model) = choice
            .ship_id
            .as_ref()
            .and_then(|id| ships.ships.iter().find(|s| &s.id == id))
        {
            let mut model = model.clone();
            model.id = choice.id.clone();
            model
        } else {
            ShipModel {
                id: choice.id.clone(),
                name: format!("Nave de {} (provisional)", choice.name),
                model: String::new(),
                length: 4.0,
                model_rotation_degrees: [0.0; 3],
            }
        }
    }
    /// Resolve identity once at launch; flight dimensions remain in ships.json.
    pub fn configure_active(
        &self,
        game: &GameState,
        ships: &mut ShipsConfig,
    ) -> Result<(), String> {
        let choice = self
            .choice(game.active_ship())
            .ok_or("Nave activa desconocida")?;
        let model = self.model(choice, ships);
        if let Some(existing) = ships.ships.iter_mut().find(|s| s.id == model.id) {
            *existing = model;
        } else {
            ships.ships.push(model);
        }
        ships.active_ship = game.active_ship().to_string();
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    first_ship: String,
    active_ship: String,
    unlocked: BTreeSet<String>,
    pub(super) progress: ProgressState,
}
impl GameState {
    pub fn validate(&self, catalog: &ShipCatalog) -> Result<(), String> {
        if !self.is_unlocked(&self.first_ship) || !self.is_unlocked(&self.active_ship)
            || self.unlocked.iter().any(|id| catalog.choice(id).is_none())
            || self.progress.completed().any(|f| !self.is_unlocked(f.as_str())) {
            return Err("Progreso guardado inválido".into());
        }
        Ok(())
    }
    pub fn new_game(id: &str, catalog: &ShipCatalog) -> Result<Self, String> {
        if catalog.choice(id).is_none() {
            return Err("Nave inicial desconocida".into());
        }
        Ok(Self {
            first_ship: id.into(),
            active_ship: id.into(),
            unlocked: BTreeSet::from([id.to_string()]),
            progress: ProgressState::default(),
        })
    }
    /// Read-only progress; attempt outcomes are applied through rewards::record_result.
    pub fn progress(&self) -> &ProgressState {
        &self.progress
    }
    pub fn first_ship(&self) -> &str {
        &self.first_ship
    }
    pub fn active_ship(&self) -> &str {
        &self.active_ship
    }
    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocked.contains(id)
    }
    pub fn unlocked_count(&self) -> usize {
        self.unlocked.len()
    }
    pub fn activate(&mut self, id: &str) -> Result<(), String> {
        if !self.is_unlocked(id) {
            return Err("La nave está bloqueada".into());
        }
        self.active_ship = id.into();
        Ok(())
    }
    /// Unlock ownership without granting tokens, changing ships or triggering a cinematic.
    pub fn unlock(&mut self, id: &str, catalog: &ShipCatalog) -> Result<bool, String> {
        if catalog.choice(id).is_none() {
            return Err("Nave desconocida".into());
        }
        Ok(self.unlocked.insert(id.into()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    Initial,
    Change,
    /// Station fleet: all ships can be previewed, only owned ships can be activated.
    Docked,
}

/// Explicit transition points for future initial/unlock abduction cinematics.
#[derive(Debug)]
pub enum SelectionTransition {
    StartAdventure(GameState),
    ContinueAdventure(super::save::AdventureSave),
    ChangedShip(GameState),
    Quit,
}

pub struct ShipSelection {
    pub index: usize,
    pub mode: SelectionMode,
}
impl ShipSelection {
    pub fn new(mode: SelectionMode, game: Option<&GameState>, catalog: &ShipCatalog) -> Self {
        let index = game
            .and_then(|g| catalog.choices.iter().position(|c| c.id == g.active_ship()))
            .unwrap_or(0);
        Self { index, mode }
    }
    pub fn navigate(&mut self, delta: i32, catalog: &ShipCatalog) {
        self.index = (self.index as i32 + delta).rem_euclid(catalog.choices.len() as i32) as usize;
    }
    pub fn can_confirm(&self, catalog: &ShipCatalog, game: Option<&GameState>) -> bool {
        self.mode == SelectionMode::Initial
            || game.is_some_and(|g| g.is_unlocked(&catalog.choices[self.index].id))
    }
    pub fn confirm(
        &self,
        catalog: &ShipCatalog,
        game: Option<&GameState>,
    ) -> Result<SelectionTransition, String> {
        let id = &catalog.choices[self.index].id;
        match self.mode {
            SelectionMode::Initial => Ok(SelectionTransition::StartAdventure(GameState::new_game(
                id, catalog,
            )?)),
            SelectionMode::Change | SelectionMode::Docked => {
                let mut next = game.ok_or("No hay partida activa")?.clone();
                next.activate(id)?;
                Ok(SelectionTransition::ChangedShip(next))
            }
        }
    }
}
