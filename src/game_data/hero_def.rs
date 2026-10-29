//! Static definition of a hero.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::game_data::combat_card_def::{CardNumber, CombatCardDef};

/// A hero as printed on its sheet, with its cards.
#[derive(Debug, Clone, Deserialize)]
pub struct HeroDef {
    /// Display name.
    pub name: String,
    /// Starting fortitude (hand size).
    pub fortitude: i32,
    /// Starting strength.
    pub strength: i32,
    /// Starting agility.
    pub agility: i32,
    /// Starting wisdom.
    pub wisdom: i32,
    /// Location code where the hero starts.
    #[serde(rename = "start_location_code_name")]
    pub start_location: String,
    /// Card numbers making up the starting life pool.
    pub starting_deck: Vec<CardNumber>,
    /// Card definitions by number.
    pub cards: BTreeMap<CardNumber, CombatCardDef>,
}
