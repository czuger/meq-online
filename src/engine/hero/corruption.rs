//! Corruption cards gained and lost by heroes.

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::engine::choices::{PASS, plain_option, target, target_option};
use crate::engine::defeat::closest_havens;
use crate::engine::engine_error::EngineError;
use crate::game_data::GameData;
use crate::game_data::corruption_card::{CorruptionCode, Flaw, Immediate, Modification};
use crate::model::actor_ref::ActorRef;
use crate::model::board::Pile;
use crate::model::choice::{ChoiceSource, Chooser, PendingChoice, Resume};
use crate::model::game::Game;
use crate::model::hero::Hero;

/// Option code of Sauron sending a wretched hero to the nearest Haven.
pub const SEND_TO_HAVEN: &str = "send_to_haven";

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
        let code = self
            .draw_corruption_cards(1, rng)
            .pop()
            .ok_or_else(|| EngineError::Refused("corruption deck is empty".to_owned()))?;
        self.give_corruption(hero_id, code, data, rng)?;
        Ok(code)
    }

    /// Draws up to `count` corruption cards, reshuffling the discard pile when needed.
    pub fn draw_corruption_cards(&mut self, count: usize, rng: &mut StdRng) -> Vec<CorruptionCode> {
        let pile = &mut self.board.decks.corruption;
        if pile.draw.len() < count {
            shuffle_discard_in(pile, rng);
        }
        let count = count.min(pile.draw.len());
        pile.draw.drain(..count).collect()
    }

    /// The hero receives a corruption card and resolves it. "Careless" and "Wretched"
    /// resolve at once and go back into the deck with its discard pile (the end of
    /// turn of "Wretched" is [`Game::end_turn_if_wretched`]).
    ///
    /// # Errors
    /// Lookup errors.
    pub fn give_corruption(
        &mut self,
        hero_id: i64,
        code: CorruptionCode,
        data: &GameData,
        rng: &mut StdRng,
    ) -> Result<(), EngineError> {
        let card = data.corruption_card(code)?;
        let hero = self.hero_mut(hero_id)?;
        match card.immediate {
            Some(immediate) => {
                if immediate == Immediate::Careless {
                    hero.favor_loss(2);
                }
                let pile = &mut self.board.decks.corruption;
                pile.draw.push(code);
                shuffle_discard_in(pile, rng);
            }
            None => {
                if let Some(modification) = card.modification {
                    hero.apply_modification(modification);
                }
                hero.corruption_cards.push(code);
            }
        }
        Ok(())
    }

    /// After the hero drew `card` during its turn: when it is "Wretched", his turn ends
    /// and Sauron may first send him to the nearest Haven.
    ///
    /// # Returns
    /// `true` when the turn ends (Sauron's choice is pending).
    ///
    /// # Errors
    /// Lookup and transition errors.
    pub fn end_turn_if_wretched(
        &mut self,
        hero_id: i64,
        card: CorruptionCode,
        data: &GameData,
    ) -> Result<bool, EngineError> {
        if data.corruption_card(card)?.immediate != Some(Immediate::Wretched) {
            return Ok(false);
        }
        let location = self.hero(hero_id)?.location.clone();
        let havens = closest_havens(&location, data)
            .into_iter()
            .filter(|h| *h != location)
            .map(|h| target("location", h))
            .collect();
        self.ask(PendingChoice {
            chooser: Chooser::Sauron,
            source: ChoiceSource::Wretched { hero_id },
            options: vec![
                target_option(SEND_TO_HAVEN, havens, 1, 1),
                plain_option(PASS),
            ],
            resume: Resume::EndTurn,
        })?;
        Ok(true)
    }

    /// Sauron sends the wretched hero to the picked Haven, or leaves him.
    pub(crate) fn apply_wretched_choice(
        &mut self,
        hero_id: i64,
        option: &str,
        targets: &[String],
        data: &GameData,
    ) -> Result<(), EngineError> {
        let haven = targets.first().and_then(|t| t.strip_prefix("location:"));
        let (SEND_TO_HAVEN, Some(haven)) = (option, haven) else {
            return Ok(());
        };
        self.hero_mut(hero_id)?.location = haven.to_owned();
        let params = [("location", json!(data.locations.name(haven)))];
        self.log(ActorRef::Hero(hero_id), "corruption.wretched", &params);
        Ok(())
    }

    /// A Corruption card passes from one hero to another ("The Lidless Eye"), with its
    /// attribute malus.
    ///
    /// # Errors
    /// [`EngineError::CardNotAvailable`] when `from` doesn't hold the card; lookup
    /// errors.
    pub fn move_corruption_card(
        &mut self,
        from: i64,
        to: i64,
        card: CorruptionCode,
        data: &GameData,
    ) -> Result<(), EngineError> {
        let modification = data.corruption_card(card)?.modification;
        let giver = self.hero_mut(from)?;
        let position = giver
            .corruption_cards
            .iter()
            .position(|c| *c == card)
            .ok_or(EngineError::CardNotAvailable {
                card,
                pile: "corruption cards",
            })?;
        giver.corruption_cards.remove(position);
        if let Some(modification) = modification {
            giver.undo_modification(modification);
        }
        let receiver = self.hero_mut(to)?;
        receiver.corruption_cards.push(card);
        if let Some(modification) = modification {
            receiver.apply_modification(modification);
        }
        Ok(())
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
