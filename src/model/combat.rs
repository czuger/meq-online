//! The combat in progress on a board.

use crate::game_data::combat_card_def::CardNumber;
use crate::model::played_card::PlayedCard;

/// A combat between the current hero and a mob. A board has at most one.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Combat {
    /// Fighting hero.
    pub hero_id: i64,
    /// Fighting mob.
    pub mob_id: i64,
    /// Started at the Ambush step (otherwise during the Travel step).
    pub ambush: bool,
    /// Strength the hero has for this combat; set during combat setup.
    pub hero_strength: Option<i32>,
    /// Card chosen by the hero for the current round, still hidden.
    pub hero_secret_card: Option<CardNumber>,
    /// Card chosen by Sauron for the current round, still hidden.
    pub mob_secret_card: Option<CardNumber>,
    /// Strength spent by the hero so far.
    pub hero_strength_used: i32,
    /// Strength spent by the mob so far.
    pub mob_strength_used: i32,
    /// The hero can't play cards anymore.
    pub hero_exhausted: bool,
    /// The mob can't play cards anymore.
    pub mob_exhausted: bool,
    /// Cards revealed by the hero, oldest first.
    pub hero_cards: Vec<PlayedCard>,
    /// Cards revealed by the mob, oldest first.
    pub mob_cards: Vec<PlayedCard>,
}
