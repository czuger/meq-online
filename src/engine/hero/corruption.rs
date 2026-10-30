//! Corruption cards gained and lost by heroes.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::corruption_card::{CorruptionCode, Flaw, Immediate, Modification};
use crate::model::actor_ref::ActorRef;
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

    fn undo_modification(&mut self, modification: Modification) {
        match modification {
            Modification::Helpless => self.agility += 1,
            Modification::Weary => self.fortitude += 1,
            Modification::Derangd => self.wisdom += 1,
            Modification::Weak => self.strength += 1,
            Modification::Dispairing | Modification::Distraught => {}
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

    /// A resting hero discards one of its corruption cards by spending the favor shown
    /// on it (§ 6.1); the card's attribute malus ends.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when the hero doesn't hold the card,
    /// [`EngineError::Refused`] when it lacks the favor; lookup errors.
    pub fn discard_corruption_card(
        &mut self,
        hero_id: i64,
        card: CorruptionCode,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let cost = data.corruption_card(card)?.favor_cost;
        self.remove_corruption_card(hero_id, card, cost, data)
    }

    /// The hero discards one of its corruption cards for `cost` favor (free when an
    /// Event or Encounter card allows it); the card's attribute malus ends.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when the hero doesn't hold the card,
    /// [`EngineError::Refused`] when it lacks the favor; lookup errors.
    pub fn remove_corruption_card(
        &mut self,
        hero_id: i64,
        card: CorruptionCode,
        cost: i32,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let def = data.corruption_card(card)?;
        let hero = self.hero_mut(hero_id)?;
        let position = hero
            .corruption_cards
            .iter()
            .position(|c| *c == card)
            .ok_or(EngineError::CardNotAvailable {
                card,
                pile: "corruption cards",
            })?;
        if hero.favor < cost {
            return Err(EngineError::Refused(
                "not enough favor to discard this corruption card".to_owned(),
            ));
        }
        hero.favor -= cost;
        hero.corruption_cards.remove(position);
        if let Some(modification) = def.modification {
            hero.undo_modification(modification);
        }
        self.board.decks.corruption.discard.push(card);
        let params = [("corruption", json!(def.name))];
        self.log(ActorRef::Hero(hero_id), "corruption.discard", &params);
        Ok(())
    }
}

fn shuffle_discard_in(pile: &mut Pile, rng: &mut StdRng) {
    pile.draw.append(&mut pile.discard);
    pile.draw.shuffle(rng);
}
