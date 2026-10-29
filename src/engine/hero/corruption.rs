//! Corruption cards gained and lost by heroes.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::corruption_card::{CorruptionCode, Flaw, Immediate, Modification};
use crate::model::board::Pile;
use crate::model::game::Game;
use crate::model::hero::Hero;

/// The Careless card goes back to the deck after resolving.
const CARELESS_CARD: CorruptionCode = 10;

impl Hero {
    /// Tells whether one of the hero's corruption cards carries `flaw`.
    pub fn has_flaw(&self, flaw: Flaw, data: &GameData) -> bool {
        self.corruption_cards
            .iter()
            .filter_map(|code| data.corruption_card(*code).ok())
            .any(|card| card.flaw == Some(flaw))
    }

    /// Corruption cards the hero has enough favor to discard.
    pub fn discardable_corruption_cards(&self, data: &GameData) -> Vec<CorruptionCode> {
        self.corruption_cards
            .iter()
            .copied()
            .filter(|code| {
                data.corruption_card(*code)
                    .is_ok_and(|c| c.favor_cost <= self.favor)
            })
            .collect()
    }

    /// Tells whether the hero may discard a corruption card this turn.
    pub fn can_discard_corruption_cards(&self, data: &GameData) -> bool {
        !self.corruption_discarded_this_turn && !self.discardable_corruption_cards(data).is_empty()
    }

    fn apply_modification(&mut self, modification: Modification) {
        match modification {
            Modification::Helpless => self.agility -= 1,
            Modification::Weary => self.fortitude -= 1,
            Modification::Derangd => self.wisdom -= 1,
            Modification::Weak => self.strength -= 1,
            Modification::Dispairing => self.favor = self.favor.min(3),
            Modification::Distraught => {}
        }
    }
}

impl Game {
    /// Draws a corruption card for a hero and resolves it.
    ///
    /// # Returns
    /// The drawn card.
    ///
    /// # Errors
    /// [`EngineError::Refused`] when the corruption deck and discard are both empty.
    pub fn gain_random_corruption(
        &mut self,
        hero_id: i64,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<CorruptionCode, EngineError> {
        let pile = &mut self.board.decks.corruption;
        if pile.draw.is_empty() {
            shuffle_discard_in(pile, rng);
        }
        if pile.draw.is_empty() {
            return Err(EngineError::Refused("corruption deck is empty".to_owned()));
        }
        let code = pile.draw.remove(0);
        let card = data.corruption_card(code)?;

        let hero = self.hero_mut(hero_id)?;
        if card.immediate == Some(Immediate::Careless) {
            hero.favor_loss(2);
            let pile = &mut self.board.decks.corruption;
            pile.draw.push(CARELESS_CARD);
            shuffle_discard_in(pile, rng);
        } else {
            if let Some(modification) = card.modification {
                hero.apply_modification(modification);
            }
            hero.corruption_cards.push(code);
        }
        Ok(code)
    }

    /// A hero discards one of its corruption cards back into the deck.
    ///
    /// The favor cost isn't paid and the stat change isn't undone, as in the original
    /// engine. Any copy of the card held by another hero is removed too.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when the card is already in the deck.
    pub fn discard_corruption_card(
        &mut self,
        hero_id: i64,
        card: CorruptionCode,
    ) -> Result<(), EngineError> {
        if self.board.decks.corruption.draw.contains(&card) {
            return Err(EngineError::CardNotAvailable {
                card,
                pile: "corruption deck (already there)",
            });
        }
        self.board.decks.corruption.draw.push(card);
        for hero in &mut self.heroes {
            hero.corruption_cards.retain(|c| *c != card);
        }
        self.hero_mut(hero_id)?.corruption_discarded_this_turn = true;
        Ok(())
    }
}

fn shuffle_discard_in(pile: &mut Pile, rng: &mut StdRng) {
    pile.draw.append(&mut pile.discard);
    pile.draw.shuffle(rng);
}
