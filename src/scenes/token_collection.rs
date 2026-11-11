//! Presentation metadata and serialized reward sequences. Progress remains owned by GameState.
use crate::gameplay::{progress::Franchise, rewards::RewardOutcome};
use std::collections::{BTreeSet, VecDeque};
use serde::{Serialize, Deserialize};

pub struct TokenArt {
    pub franchise: Franchise,
    pub name: &'static str,
    pub png: &'static str,
}

pub const TOKENS: [TokenArt; 6] = [
    TokenArt {
        franchise: Franchise::Minecraft,
        name: "Minecraft",
        png: "assets/ui/tokens/minecraft.png",
    },
    TokenArt {
        franchise: Franchise::SouthPark,
        name: "South Park",
        png: "assets/ui/tokens/southpark.png",
    },
    TokenArt {
        franchise: Franchise::SmilingFriends,
        name: "Smiling Friends",
        png: "assets/ui/tokens/smilingfriends.png",
    },
    TokenArt {
        franchise: Franchise::StarWars,
        name: "Star Wars",
        png: "assets/ui/tokens/starwars.png",
    },
    TokenArt {
        franchise: Franchise::Halloween,
        name: "Hexbound",
        png: "assets/ui/tokens/halloween.png",
    },
    TokenArt {
        franchise: Franchise::RegularShow,
        name: "Regular Show",
        png: "assets/ui/tokens/regularshow.png",
    },
];

pub fn slot_index(franchise: Franchise) -> usize {
    TOKENS
        .iter()
        .position(|art| art.franchise == franchise)
        .expect("six canonical tokens")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardNotice {
    pub franchise: Franchise,
    pub ship_name: Option<String>,
}

pub const CEREMONY_SECONDS: f32 = 3.6;

pub const ABDUCTION_SECONDS: f32 = 4.8;
/// La de Smiling Friends es un video (`assets/video/smilingfriends/abduction.sfv`: 311 cuadros a 24 por segundo).
pub const SMILINGFRIENDS_ABDUCTION_SECONDS: f32 = 311.0 / 24.0;

/// La de Hexbound: la carta de invitación y la bruja que se sube a la escoba y se va volando.
pub const HEXBOUND_ABDUCTION_SECONDS: f32 = crate::minigames::halloween::INVITATION;

/// Si al ganar su minijuego (con la nave todavía bloqueada) se ve la abducción al volver al sistema solar. La de
/// Smiling Friends no (pedido de Cami): solo se ve al elegir esa nave en el hangar. La de Hexbound sí, pero con la carta
/// de agradecimiento en vez de la invitación.
pub fn returns_abducted(franchise: Franchise) -> bool {
    franchise != Franchise::SmilingFriends
}

/// La escena de Hexbound al ganar el minijuego: la carta de agradecimiento con corazones (más corta que la invitación,
/// sin la bruja).
pub const HEXBOUND_THANKS_SECONDS: f32 = crate::minigames::halloween::THANKS;

/// Cuánto dura la abducción de una franquicia: la de la nave elegida al empezar (`initial`) o la de ganar su
/// minijuego (para Hexbound es otra escena).
pub fn abduction_length(franchise: Franchise, initial: bool) -> f32 {
    if franchise == Franchise::Halloween && !initial { HEXBOUND_THANKS_SECONDS } else { abduction_seconds(franchise) }
}

/// Cuánto dura la abducción de cada franquicia (la de la nave elegida al empezar).
pub fn abduction_seconds(franchise: Franchise) -> f32 {
    match franchise {
        Franchise::SmilingFriends => SMILINGFRIENDS_ABDUCTION_SECONDS,
        Franchise::Halloween => HEXBOUND_ABDUCTION_SECONDS,
        _ => ABDUCTION_SECONDS,
    }
}
pub const TOAST_SECONDS: f32 = 4.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardPhase { Abduction, Token, Toast }

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Sequence { notice: RewardNotice, initial: bool, phase: RewardPhase }

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct TokenPresentation {
    pending: VecDeque<Sequence>,
    announced: BTreeSet<Franchise>,
    active: Option<Sequence>,
    elapsed: f32,
    initial_scheduled: bool,
    revision: u64,
}

impl TokenPresentation {
    pub fn enqueue_initial(&mut self, franchise: Franchise) -> bool {
        if self.initial_scheduled { return false; }
        self.initial_scheduled = true;
        self.pending.push_back(Sequence {
            notice: RewardNotice { franchise, ship_name: None },
            initial: true, phase: RewardPhase::Abduction,
        });
        self.revision += 1;
        true
    }
    /// La abducción inicial ya se vio en el hangar, al confirmar la nave: el sistema solar no la repite.
    pub fn initial_shown() -> Self {
        Self { initial_scheduled: true, ..Self::default() }
    }

    /// No synthetic progress: only a new-token outcome can schedule a notice.
    /// The defensive set deduplicates redelivery, independently of queue consumption.
    pub fn enqueue(
        &mut self,
        franchise: Franchise,
        outcome: RewardOutcome,
        ship_name: &str,
    ) -> bool {
        if !outcome.token_obtained || !self.announced.insert(franchise) {
            return false;
        }
        self.pending.push_back(Sequence {
            notice: RewardNotice { franchise, ship_name: outcome.ship_unlocked.then(|| ship_name.to_owned()) },
            initial: false,
            phase: if outcome.ship_unlocked && returns_abducted(franchise) { RewardPhase::Abduction } else { RewardPhase::Token },
        });
        self.revision += 1;
        true
    }

    /// Advance only on the solar screen after its fade/intro, never inside a minigame.
    pub fn advance(&mut self, dt: f32, ready: bool) {
        if !ready {
            return;
        }
        if self.active.is_none() {
            self.active = self.pending.pop_front();
            self.elapsed = 0.0;
            if self.active.is_some() { self.revision += 1; }
        }
        if let Some(sequence) = &mut self.active {
            self.elapsed += if dt.is_finite() { dt.max(0.0) } else { 0.0 };
            let duration = match sequence.phase {
                RewardPhase::Abduction => abduction_length(sequence.notice.franchise, sequence.initial),
                RewardPhase::Token => CEREMONY_SECONDS,
                RewardPhase::Toast => TOAST_SECONDS,
            };
            if self.elapsed >= duration {
                match sequence.phase {
                    RewardPhase::Abduction if !sequence.initial => sequence.phase = RewardPhase::Token,
                    RewardPhase::Token if sequence.notice.ship_name.is_some() => sequence.phase = RewardPhase::Toast,
                    _ => self.active = None,
                }
                self.elapsed = 0.0;
                self.revision += 1;
            }
        }
    }

    /// Termina la abducción que se está viendo (el botón SKIP del video); el próximo `advance` pasa a la etapa siguiente.
    pub fn skip_abduction(&mut self) {
        if let Some(franchise) = self.abduction() {
            self.elapsed = self.elapsed.max(abduction_length(franchise, self.abduction_is_initial()));
        }
    }

    pub fn active(&self) -> Option<&RewardNotice> {
        (self.phase() == Some(RewardPhase::Token)).then(|| self.notice()).flatten()
    }
    pub fn phase(&self) -> Option<RewardPhase> { self.active.as_ref().map(|s| s.phase) }
    pub fn notice(&self) -> Option<&RewardNotice> { self.active.as_ref().map(|s| &s.notice) }
    /// Si la abducción que se está viendo es la de la nave elegida al empezar (no la de ganar un minijuego).
    pub fn abduction_is_initial(&self) -> bool {
        self.active.as_ref().is_some_and(|s| s.initial)
    }

    pub fn abduction(&self) -> Option<Franchise> {
        (self.phase() == Some(RewardPhase::Abduction)).then(|| self.notice().unwrap().franchise)
    }
    pub fn toast(&self) -> Option<&RewardNotice> {
        (self.phase() == Some(RewardPhase::Toast)).then(|| self.notice()).flatten()
    }
    pub fn revision(&self) -> u64 { self.revision }
    pub fn required_abductions(&self) -> Vec<Franchise> {
        self.pending.iter().chain(self.active.iter())
            .filter(|s| s.phase == RewardPhase::Abduction)
            .map(|s| s.notice.franchise).collect()
    }
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub fn is_busy(&self) -> bool {
        matches!(self.phase(), Some(RewardPhase::Abduction | RewardPhase::Token))
    }
}

/// Shared responsive geometry: the flying medal lands at exactly its collection slot.
pub struct CollectionLayout {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl CollectionLayout {
    pub fn new(width: f32, height: f32) -> Self {
        let scale = ((width - 32.0) / 900.0)
            .min((height - 32.0) / 620.0)
            .max(0.1);
        Self {
            x: (width - 900.0 * scale) * 0.5,
            y: (height - 620.0 * scale) * 0.5,
            scale,
        }
    }

    pub fn slot(&self, index: usize) -> [f32; 4] {
        [
            self.x + (40.0 + (index % 3) as f32 * 280.0) * self.scale,
            self.y + (126.0 + (index / 3) as f32 * 218.0) * self.scale,
            260.0 * self.scale,
            198.0 * self.scale,
        ]
    }

    pub fn medal_center(&self, index: usize) -> [f32; 2] {
        let [x, y, w, _] = self.slot(index);
        [x + w * 0.5, y + 72.0 * self.scale]
    }
}
