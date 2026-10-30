//! Static definition of a character.

use serde::Deserialize;

/// Hero attribute raised by a character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CharacterAttribute {
    /// Strength.
    Strength,
    /// Agility.
    Agility,
    /// Wisdom.
    Wisdom,
}

/// What a consulted character gives instead of its favor (§ 6.5 Explore).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CharacterAbility {
    /// Remove influence from the Shadow Pool.
    ShadowPoolLoss {
        /// Influence removed.
        count: i32,
    },
    /// Gain the Horse item.
    Horse,
    /// Gain one level of an attribute.
    Level {
        /// The attribute.
        attribute: CharacterAttribute,
    },
    /// Pay favor to advance the heroes' story marker 1 space.
    AdvanceHeroes {
        /// Favor paid.
        cost: i32,
    },
    /// Receive training several times.
    Training {
        /// Number of trainings.
        times: u8,
    },
    /// Sauron discards random Shadow cards.
    DiscardShadows {
        /// Cards discarded.
        count: usize,
    },
}

/// A character heroes can meet on the map.
#[derive(Debug, Clone, Deserialize)]
pub struct Character {
    /// Display name.
    pub name: String,
    /// Power description.
    pub power: String,
    /// Favor gained when consulted for favor.
    pub favor: i32,
    /// Ability used when consulted for it.
    pub ability: CharacterAbility,
}
