//! Static definition of a mob attack deck.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::game_data::combat_card_def::{CardNumber, CombatCardDef};

/// An attack deck shared by several mobs (behemoth, ravager, zealot).
#[derive(Debug, Clone, Deserialize)]
pub struct AttackDeck {
    /// Card numbers making up the deck.
    pub deck: Vec<CardNumber>,
    /// Card definitions by number.
    pub cards: BTreeMap<CardNumber, CombatCardDef>,
}
