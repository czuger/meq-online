//! A card revealed during a combat round.

use serde::{Deserialize, Serialize};

use crate::game_data::combat_card_def::{CardNumber, CardPower, CardType, CombatCardDef};

/// Snapshot of a card played in a combat round.
///
/// Printed values are copied because card powers can change them for the rest of
/// the combat (Smash sets the opponent's printed defense to 0, ...).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayedCard {
    /// Card number; `None` for the placeholder played by an exhausted side.
    pub card: Option<CardNumber>,
    /// Card name.
    pub name: String,
    /// Image path.
    pub pic_path: String,
    /// Card power.
    pub power: CardPower,
    /// Strength cost.
    pub strength_cost: i32,
    /// Printed attack, possibly modified by powers.
    pub attack: i32,
    /// Printed defense, possibly modified by powers.
    pub defense: i32,
    /// Melee, ranged or exhausted.
    pub card_type: CardType,
    /// The card has no effect this round.
    pub cancelled: bool,
}

impl PlayedCard {
    /// Snapshots a card definition.
    ///
    /// # Arguments
    /// * `card` - number of the played card.
    /// * `def` - its static definition.
    pub fn from_def(card: CardNumber, def: &CombatCardDef) -> Self {
        Self {
            card: Some(card),
            name: def.name.clone(),
            pic_path: def.pic_path.clone(),
            power: def.power,
            strength_cost: def.strength_cost,
            attack: def.attack,
            defense: def.defense,
            card_type: def.card_type,
            cancelled: false,
        }
    }

    /// The placeholder played by a side that is exhausted.
    pub fn exhausted() -> Self {
        Self {
            card: None,
            name: "Exhausted".to_owned(),
            pic_path: "exhausted".to_owned(),
            power: CardPower::Inactive,
            strength_cost: 0,
            attack: 0,
            defense: 0,
            card_type: CardType::Exhausted,
            cancelled: true,
        }
    }

    /// Tells whether the card is a melee attack.
    pub fn is_melee(&self) -> bool {
        self.card_type == CardType::Melee
    }

    /// Tells whether the card is a ranged attack.
    pub fn is_ranged(&self) -> bool {
        self.card_type == CardType::Ranged
    }
}
