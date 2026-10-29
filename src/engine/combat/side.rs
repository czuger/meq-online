//! One side of a combat round being resolved.

use crate::model::played_card::PlayedCard;

/// Cards and running totals of one side while a round is resolved.
#[derive(Debug, Clone)]
pub struct Side {
    /// Card revealed this round.
    pub current: PlayedCard,
    /// Card revealed in the previous round, if any.
    pub previous: Option<PlayedCard>,
    /// Attack bonus accumulated by powers, then the printed attack.
    pub final_attack: i32,
    /// Defense bonus accumulated by powers, then the printed defense.
    pub final_defense: i32,
}

impl Side {
    /// Starts a round with zeroed totals.
    ///
    /// # Arguments
    /// * `current` - card revealed this round.
    /// * `previous` - card revealed in the previous round.
    pub fn new(current: PlayedCard, previous: Option<PlayedCard>) -> Self {
        Self {
            current,
            previous,
            final_attack: 0,
            final_defense: 0,
        }
    }

    /// Cancels the current card and wipes its totals.
    pub fn cancel_current(&mut self) {
        self.final_attack = 0;
        self.final_defense = 0;
        self.current.cancelled = true;
    }
}
