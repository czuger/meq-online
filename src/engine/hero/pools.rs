//! Moving hero cards between hand, life, rest and damage pools.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::engine::engine_error::EngineError;
use crate::game_data::combat_card_def::CardNumber;
use crate::model::hero::Hero;

impl Hero {
    /// Rest: the rest pool is shuffled into the life pool (§ 6.1).
    pub fn rest(&mut self, rng: &mut StdRng) {
        self.life_pool.append(&mut self.rest_pool);
        self.life_pool.shuffle(rng);
    }

    /// Heal: the rest and damage pools are shuffled into the life pool (§ 6.1, § 8.9).
    pub fn heal(&mut self, rng: &mut StdRng) {
        self.life_pool.append(&mut self.damage_pool);
        self.rest(rng);
    }

    /// Moves cards from the hand to the rest pool, one copy per listed card.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when a card isn't in hand; the hand is then unchanged.
    pub fn hand_to_rest(&mut self, cards: &[CardNumber]) -> Result<(), EngineError> {
        self.take_from_hand(cards)?;
        self.rest_pool.extend_from_slice(cards);
        Ok(())
    }

    /// Moves cards from the hand to the damage pool, one copy per listed card.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when a card isn't in hand; the hand is then unchanged.
    pub fn hand_to_damages(&mut self, cards: &[CardNumber]) -> Result<(), EngineError> {
        self.take_from_hand(cards)?;
        self.damage_pool.extend_from_slice(cards);
        Ok(())
    }

    /// Removes one copy of each listed card from the hand.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when a card isn't in hand; the hand is then unchanged.
    pub fn take_from_hand(&mut self, cards: &[CardNumber]) -> Result<(), EngineError> {
        let mut hand = self.hand.clone();
        for card in cards {
            let position =
                hand.iter()
                    .position(|c| c == card)
                    .ok_or(EngineError::CardNotAvailable {
                        card: *card,
                        pile: "hand",
                    })?;
            hand.remove(position);
        }
        self.hand = hand;
        Ok(())
    }

    /// Draws up to `count` cards from the top of the life pool into the hand.
    ///
    /// # Returns
    /// Number of cards actually drawn.
    pub fn draw_cards(&mut self, count: usize) -> usize {
        let count = count.min(self.life_pool.len());
        let drawn: Vec<_> = self.life_pool.drain(..count).collect();
        self.hand.extend_from_slice(&drawn);
        count
    }

    /// Pays the pending combat damage with the `from_hand` cards, then the top of the
    /// life pool (§ 6.3).
    ///
    /// # Errors
    /// [`EngineError::Refused`] when more cards are given than the damage or the life
    /// pool can't pay the rest; [`EngineError::CardNotAvailable`] when a card isn't in
    /// hand. Nothing changes on error.
    pub fn pay_damage(&mut self, from_hand: &[CardNumber]) -> Result<(), EngineError> {
        let damage = usize::try_from(self.temporary_damages).unwrap_or(0);
        let from_life = damage
            .checked_sub(from_hand.len())
            .ok_or_else(|| EngineError::Refused(format!("only {damage} card(s) to lose")))?;
        if from_life > self.life_pool.len() {
            return Err(EngineError::Refused(
                "the life pool can't pay the rest: choose more cards".to_owned(),
            ));
        }
        self.hand_to_damages(from_hand)?;
        let lost: Vec<CardNumber> = self.life_pool.drain(..from_life).collect();
        self.damage_pool.extend(lost);
        self.temporary_damages = 0;
        Ok(())
    }

    /// Hero Draw step: draws up to `count` cards, at most fortitude per step (§ 5.12).
    ///
    /// # Returns
    /// Number of cards actually drawn.
    pub fn draw_step_cards(&mut self, count: usize) -> usize {
        let allowed = usize::try_from(self.fortitude - self.cards_drawn_this_round).unwrap_or(0);
        let drawn = self.draw_cards(count.min(allowed));
        self.cards_drawn_this_round += i32::try_from(drawn).unwrap_or(i32::MAX);
        drawn
    }

    /// Cards the hero may still draw during this Hero Draw step.
    pub fn cards_left_to_draw(&self) -> i32 {
        (self.fortitude - self.cards_drawn_this_round)
            .min(i32::try_from(self.life_pool.len()).unwrap_or(i32::MAX))
            .max(0)
    }

    /// Loses `amount` favor, never going below 0.
    pub fn favor_loss(&mut self, amount: i32) {
        self.favor = (self.favor - amount).max(0);
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::test_support::{hero, rng};

    #[test]
    fn removes_one_copy_per_card() {
        let mut h = hero("argalad");
        h.hand = vec![2, 3, 3, 4];
        h.hand_to_rest(&[2, 3]).unwrap();
        assert_eq!(h.hand, vec![3, 4]);
        assert_eq!(h.rest_pool, vec![2, 3]);
    }

    // § 6.3: each damage is paid from the hand or the life pool, in any combination.
    #[test]
    fn damage_paid_from_hand_and_life_pool() {
        let mut h = hero("argalad");
        h.hand = vec![1, 2];
        h.life_pool = vec![3, 4, 5];
        h.temporary_damages = 3;
        assert!(h.pay_damage(&[1, 2, 1]).is_err());
        h.pay_damage(&[2]).unwrap();
        assert_eq!(h.hand, vec![1]);
        assert_eq!(h.life_pool, vec![5]);
        assert_eq!(h.damage_pool, vec![2, 3, 4]);
        assert_eq!(h.temporary_damages, 0);

        h.temporary_damages = 2;
        assert!(h.pay_damage(&[]).is_err(), "one card left in the life pool");
        h.pay_damage(&[1]).unwrap();
        assert!(h.life_pool.is_empty() && h.hand.is_empty());
    }

    // § 5.12: each hero draws up to his fortitude, once per Hero Draw step.
    #[test]
    fn draw_step_is_capped_by_fortitude() {
        let mut h = hero("argalad");
        h.hand.clear();
        h.fortitude = 3;
        assert_eq!(h.draw_step_cards(2), 2);
        assert_eq!(h.cards_left_to_draw(), 1);
        assert_eq!(h.draw_step_cards(5), 1);
        assert_eq!(h.draw_step_cards(5), 0);
        assert_eq!(h.hand.len(), 3);
    }

    #[test]
    fn refuses_cards_not_in_hand() {
        let mut h = hero("argalad");
        h.hand = vec![2];
        assert!(h.hand_to_damages(&[2, 2]).is_err());
        assert_eq!(h.hand, vec![2]);
    }

    // § 6.1: resting shuffles the rest pool into the life pool, healing the damage pool too.
    #[test]
    fn heal_shuffles_rest_and_damage_pools_into_life_pool() {
        let mut h = hero("argalad");
        h.life_pool = (0..10).collect();
        h.rest_pool = (10..20).collect();
        h.damage_pool = (20..30).collect();
        h.heal(&mut rng());
        let mut sorted = h.life_pool.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..30).collect::<Vec<_>>());
        assert_ne!(h.life_pool, sorted, "the life pool is shuffled");
        assert!(h.rest_pool.is_empty() && h.damage_pool.is_empty());
    }
}
