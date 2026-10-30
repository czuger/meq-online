//! A hero seated on a board.

use serde::{Deserialize, Serialize};

use crate::game_data::combat_card_def::CardNumber;
use crate::game_data::corruption_card::CorruptionCode;

/// An Item card a hero may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Item {
    /// Horse.
    Horse,
    /// Boat.
    Boat,
}

/// Items carried by a hero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroItems {
    /// A horse lowers path difficulty by 1.
    #[serde(default)]
    pub horse: bool,
    /// A boat crosses water paths with any one card.
    #[serde(default)]
    pub boat: bool,
}

impl HeroItems {
    /// Items held.
    pub fn held(&self) -> Vec<Item> {
        [(self.horse, Item::Horse), (self.boat, Item::Boat)]
            .into_iter()
            .filter(|(held, _)| *held)
            .map(|(_, item)| item)
            .collect()
    }

    /// Discards an Item.
    pub fn remove(&mut self, item: Item) {
        match item {
            Item::Horse => self.horse = false,
            Item::Boat => self.boat = false,
        }
    }
}

/// Level tokens: how many times each attribute was increased (§ 7).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeroLevels {
    /// Fortitude increases.
    #[serde(default)]
    pub fortitude: u8,
    /// Strength increases.
    #[serde(default)]
    pub strength: u8,
    /// Agility increases.
    #[serde(default)]
    pub agility: u8,
    /// Wisdom increases.
    #[serde(default)]
    pub wisdom: u8,
}

/// What a hero did during its current turn, for once-per-turn rules and per-turn limits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnRecord {
    /// Argalad looked at the surrounding monsters.
    #[serde(default)]
    pub argalad: bool,
    /// Travel steps (moves) taken.
    #[serde(default)]
    pub travels: u8,
    /// Favor gained.
    #[serde(default)]
    pub favor_gained: i32,
    /// The hero chose a Dark Path.
    #[serde(default)]
    pub dark_path: bool,
    /// Eleanor's Noble Bloodline gave its favor.
    #[serde(default)]
    pub noble_bloodline: bool,
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
    /// Starting Quest number.
    pub quest: Option<u32>,
    /// Advanced Quest number, usable once the Starting Quest is completed.
    pub advanced_quest: Option<u32>,
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
    /// Cards drawn during the current Hero Draw step.
    pub cards_drawn_this_round: i32,
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
    /// Level tokens.
    pub levels: HeroLevels,
    /// What the hero did this turn.
    pub turn_record: TurnRecord,
    /// Corruption cards held.
    pub corruption_cards: Vec<CorruptionCode>,
}
