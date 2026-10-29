//! Definition of a combat card, shared by hero decks, mob attack decks and skills.

use serde::{Deserialize, Serialize};

use crate::game_data::terrain::Terrain;

/// Card number inside a hero deck, an attack deck or the skills list.
pub type CardNumber = u32;

/// Kind of attack printed on a combat card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardType {
    /// Ranged attack.
    Ranged,
    /// Melee attack.
    Melee,
    /// Placeholder card played by an exhausted side.
    Exhausted,
}

/// Special power of a combat card.
///
/// Cards whose power has no rule implementation yet map to [`CardPower::Inactive`]
/// and only count their printed values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CardPower {
    /// Cancels the opponent's printed defense.
    Precision,
    /// Cancels the opponent's printed defense.
    Smash,
    /// Brings the opponent's printed values down to 1.
    DeadlyFinesse,
    /// Deals 2 more damage to yourself if you were hurt this round.
    Reckless,
    /// Cancels an opposing ranged card.
    Charge,
    /// +2 defense against melee.
    Parry,
    /// Cancels an opposing melee card.
    Evade,
    /// Next round: +2 attack and +2 defense.
    Concentrate,
    /// Next round: cancels an opposing melee card.
    FallBack,
    /// Next round: +2 attack if you play a ranged card.
    Volley,
    /// +3 attack if the opponent plays the same type twice in a row.
    AimedShot,
    /// +2 attack against ranged.
    Overdraw,
    /// +2 attack against ranged.
    RangedStrike,
    /// +1 attack against melee.
    Hack,
    /// +5 attack if the opponent has no defense.
    AttackOfOpportunity,
    /// No implemented effect.
    #[default]
    #[serde(other)]
    Inactive,
}

/// Static description of a combat card.
#[derive(Debug, Clone, Deserialize)]
pub struct CombatCardDef {
    /// Card name.
    pub name: String,
    /// Melee or ranged.
    #[serde(rename = "type")]
    pub card_type: CardType,
    /// Printed attack.
    pub attack: i32,
    /// Printed defense.
    pub defense: i32,
    /// Strength spent when the card is played.
    pub strength_cost: i32,
    /// Image path under `static/images/`.
    pub pic_path: String,
    /// Special power.
    #[serde(default)]
    pub power: CardPower,
    /// Terrain the card helps to cross (hero cards and skills only).
    #[serde(default)]
    pub movement_type: Option<Terrain>,
}
