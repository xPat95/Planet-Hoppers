//! Session progress. Each completed franchise represents exactly one token.
use std::collections::BTreeSet;
use serde::{Serialize, Deserialize};

/// Canonical gameplay identities, independent of individual ship model IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Franchise {
    Minecraft,
    SouthPark,
    SmilingFriends,
    StarWars,
    Halloween,
    RegularShow,
}

impl Franchise {
    pub const ALL: [Self; 6] = [
        Self::Minecraft,
        Self::SouthPark,
        Self::SmilingFriends,
        Self::StarWars,
        Self::Halloween,
        Self::RegularShow,
    ];

    /// Uses the existing planet and selection catalog IDs, not model aliases.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minecraft => "minecraft",
            Self::SouthPark => "southpark",
            Self::SmilingFriends => "smilingfriends",
            Self::StarWars => "starwars",
            Self::Halloween => "halloween",
            Self::RegularShow => "regularshow",
        }
    }

    pub fn from_id(id: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|franchise| franchise.as_str() == id)
            .ok_or_else(|| format!("Franquicia desconocida: {id}"))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressState {
    completed: BTreeSet<Franchise>,
}

impl ProgressState {
    pub fn is_completed(&self, franchise: Franchise) -> bool {
        self.completed.contains(&franchise)
    }

    pub fn completed(&self) -> impl ExactSizeIterator<Item = Franchise> + '_ {
        self.completed.iter().copied()
    }

    pub fn token_count(&self) -> usize {
        self.completed.len()
    }

    pub fn is_complete(&self) -> bool {
        self.token_count() == Franchise::ALL.len()
    }

    // Only the gameplay reward service should record completed franchises.
    pub(super) fn complete(&mut self, franchise: Franchise) -> bool {
        self.completed.insert(franchise)
    }
}
