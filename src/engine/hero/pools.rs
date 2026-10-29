//! Moving hero cards between hand, life, rest and damage pools.

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::combat_card_def::CardNumber;
use crate::model::hero::Hero;

impl Hero {
    /// Rest: the rest pool goes back to the bottom of the life pool.
    ///
    /// The life pool is not shuffled, as in the original engine.
    pub fn rest(&mut self) {
        self.life_pool.append(&mut self.rest_pool);
    }

    /// Heal: rest, then the damage pool also goes back to the life pool.
    ///
    /// The life pool is not shuffled, as in the original engine.
    pub fn heal(&mut self) {
        self.rest();
        self.life_pool.append(&mut self.damage_pool);
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

    /// Loses `amount` favor, never going below 0.
    pub fn favor_loss(&mut self, amount: i32) {
        self.favor = (self.favor - amount).max(0);
    }

    /// Moves the hero to the haven of its region.
    ///
    /// # Errors
    /// [`EngineError::GameData`] when the location or its haven is unknown.
    pub fn move_to_regional_haven(&mut self, data: &GameData) -> Result<(), EngineError> {
        let color = data.locations.get(&self.location)?.color_code;
        self.location = data.locations.haven_for_color(color)?.to_owned();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::test_support::hero;

    #[test]
    fn removes_one_copy_per_card() {
        let mut h = hero("argalad");
        h.hand = vec![2, 3, 3, 4];
        h.hand_to_rest(&[2, 3]).unwrap();
        assert_eq!(h.hand, vec![3, 4]);
        assert_eq!(h.rest_pool, vec![2, 3]);
    }

    #[test]
    fn refuses_cards_not_in_hand() {
        let mut h = hero("argalad");
        h.hand = vec![2];
        assert!(h.hand_to_damages(&[2, 2]).is_err());
        assert_eq!(h.hand, vec![2]);
    }

    #[test]
    fn heal_brings_back_rest_and_damage_pools() {
        let mut h = hero("argalad");
        h.life_pool = vec![1];
        h.rest_pool = vec![2];
        h.damage_pool = vec![3];
        h.heal();
        assert_eq!(h.life_pool, vec![1, 2, 3]);
        assert!(h.rest_pool.is_empty() && h.damage_pool.is_empty());
    }
}
