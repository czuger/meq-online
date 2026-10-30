//! Static definition of corruption cards.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Corruption card code.
pub type CorruptionCode = u32;

/// Lasting behavior flaw given by a corruption card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Flaw {
    /// Hand limited to 7 cards.
    Distraught,
    /// Favor limited to 3.
    Dispairing,
    /// Discard a random card at the start of each combat.
    Cowardly,
    /// Can't meet characters.
    Isolated,
    /// Resting or defeat advances Sauron's lowest marker 2 spaces.
    Despondent,
    /// At most 3 Travel steps per turn.
    Hopeless,
    /// Sauron may draw a Shadow card after playing one on the hero's turn.
    Greedy,
    /// At most 1 favor gained per turn.
    Mistrusted,
    /// Sauron draws 1 more Peril card on the hero's turn.
    Reckless,
    /// Draw 2 Encounter cards instead of 3.
    Indifferent,
}

/// Stat change given by a corruption card while the hero holds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Modification {
    /// Agility -1.
    Helpless,
    /// Fortitude -1.
    Weary,
    /// Wisdom -1 (spelled "derangd" in the data).
    Derangd,
    /// Strength -1.
    Weak,
    /// Favor capped at 3 when gained.
    Dispairing,
    /// No stat change.
    Distraught,
}

/// Effect resolved immediately instead of keeping the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Immediate {
    /// Lose 2 favor, then shuffle the card and the discard back into the deck.
    Careless,
    /// The hero's turn ends and Sauron may send him to the nearest Haven, then shuffle
    /// the card and the discard back into the deck.
    Wretched,
}

/// A corruption card.
#[derive(Debug, Clone, Deserialize)]
pub struct CorruptionCard {
    /// Display name.
    pub name: String,
    /// Favor a hero must pay to discard it.
    #[serde(default)]
    pub favor_cost: i32,
    /// Lasting flaw.
    #[serde(default)]
    pub flaw: Option<Flaw>,
    /// Stat change.
    #[serde(default)]
    pub modification: Option<Modification>,
    /// Immediate effect.
    #[serde(default)]
    pub immediate: Option<Immediate>,
    /// Rules text.
    #[serde(default)]
    pub desc: Option<String>,
}

/// The corruption deck and card definitions.
#[derive(Debug, Clone, Deserialize)]
pub struct CorruptionCards {
    /// Cards making up the starting deck.
    pub deck: Vec<CorruptionCode>,
    /// Card definitions by code.
    pub cards: BTreeMap<CorruptionCode, CorruptionCard>,
}
