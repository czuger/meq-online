//! A hero seated on a board.

use serde::{Deserialize, Serialize};

use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::corruption_card::CorruptionCode;

/// Items carried by a hero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroItems {
    /// A horse lowers path difficulty by 1.
    #[serde(default)]
    pub horse: bool,
}

/// Once-per-turn hero powers already used this turn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsedPowers {
    /// Argalad looked at the surrounding monsters.
    #[serde(default)]
    pub argalad: bool,
}

/// A hero and its game state.
#[derive(Debug, Clone, PartialEq)]
pub struct Hero {
    /// Database id; `None` until persisted.
    pub id: Option<i64>,
    /// Owning user.
    pub user_id: i64,
    /// Hero code in `data/heroes.yaml`.
    pub code: String,
    /// Current location code.
    pub location: String,
    /// Hand size.
    pub fortitude: i32,
    /// Strength.
    pub strength: i32,
    /// Agility.
    pub agility: i32,
    /// Wisdom.
    pub wisdom: i32,
    /// Favor tokens.
    pub favor: i32,
    /// Current quest number.
    pub quest: Option<u32>,
    /// Order of play among heroes (0 first).
    pub playing_order: i32,
    /// 1, or 2 during the second turn of a single-hero game.
    pub turn: i32,
    /// The hero is expected to act.
    pub active: bool,
    /// The hero finished its turn this round.
    pub turn_finished: bool,
    /// Damage taken during the current combat round.
    pub damages_taken_this_turn: i32,
    /// Damage dealt in combat that still has to be paid with cards.
    pub temporary_damages: i32,
    /// A corruption card was already discarded this turn.
    pub corruption_discarded_this_turn: bool,
    /// Draw pile.
    pub life_pool: Vec<CardNumber>,
    /// Cards in hand.
    pub hand: Vec<CardNumber>,
    /// Cards spent, back to the life pool on rest.
    pub rest_pool: Vec<CardNumber>,
    /// Cards lost to damage, back to the life pool on heal.
    pub damage_pool: Vec<CardNumber>,
    /// Items carried.
    pub items: HeroItems,
    /// Powers used this turn.
    pub used_powers: UsedPowers,
    /// Corruption cards held.
    pub corruption_cards: Vec<CorruptionCode>,
}
