//! Presentation metadata and deferred ceremonies. Progress remains owned by GameState.
use crate::gameplay::{progress::Franchise, rewards::RewardOutcome};
use std::collections::{BTreeSet, VecDeque};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardNotice {
    pub franchise: Franchise,
    pub ship_name: Option<String>,
}

pub const CEREMONY_SECONDS: f32 = 3.6;

#[derive(Default)]
pub struct TokenPresentation {
    pending: VecDeque<RewardNotice>,
    announced: BTreeSet<Franchise>,
    active: Option<RewardNotice>,
    elapsed: f32,
}

impl TokenPresentation {
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
        self.pending.push_back(RewardNotice {
            franchise,
            ship_name: outcome.ship_unlocked.then(|| ship_name.to_owned()),
        });
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
        }
        if self.active.is_some() {
            self.elapsed += if dt.is_finite() { dt.max(0.0) } else { 0.0 };
            if self.elapsed >= CEREMONY_SECONDS {
                self.active = None;
                self.elapsed = 0.0;
            }
        }
    }

    pub fn active(&self) -> Option<&RewardNotice> {
        self.active.as_ref()
    }
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub fn is_busy(&self) -> bool {
        self.active.is_some()
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
