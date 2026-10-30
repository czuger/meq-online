//! Common behavior of the two sides of a combat.

use crate::model::hero::Hero;
use crate::model::mob::Mob;

/// Something that can take damage in a combat round.
pub trait Combatant {
    /// Applies `amount` damage (negative amounts count as 0).
    fn deal_damages(&mut self, amount: i32);

    /// Damage taken during the current round.
    fn damages_taken_this_turn(&self) -> i32;
}

impl Combatant for Hero {
    /// Damage is not paid immediately: it accumulates in `temporary_damages` and is
    /// paid with cards at the end of the round.
    #[tracing::instrument(level = "debug", skip_all, fields(amount))]
    fn deal_damages(&mut self, amount: i32) {
        let amount = amount.max(0);
        self.temporary_damages += amount;
        self.damages_taken_this_turn += amount;
    }

    fn damages_taken_this_turn(&self) -> i32 {
        self.damages_taken_this_turn
    }
}

impl Combatant for Mob {
    #[tracing::instrument(level = "debug", skip_all, fields(amount))]
    fn deal_damages(&mut self, amount: i32) {
        let amount = amount.max(0);
        self.life -= amount;
        self.damages_taken_this_turn += amount;
    }

    fn damages_taken_this_turn(&self) -> i32 {
        self.damages_taken_this_turn
    }
}
